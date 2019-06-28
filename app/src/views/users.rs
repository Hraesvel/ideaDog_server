use crate::{util::user, AppState};

use crate::midware::AuthMiddleware;
use crate::util::user::extract_token;
use crate::views::auth::{challenge_gen, exist_user, login, ttl, Token};
use actix_web::actix::{Handler, Message};
use actix_web::http::header::q;
use actix_web::http::{Method, NormalizePath, StatusCode};
use actix_web::{App, FutureResponse, HttpResponse, Path, Query, Responder, State};
use actix_web::{AsyncResponder, HttpRequest, Json};
use arangors::AqlQuery;
use chrono::Utc;
use futures::future::{err, ok, Future, IntoFuture};
use ideadog::{Challenge, DbExecutor, Idea, Login, NewUser, QueryUser};
use ideadog::{QUser, QUserParams};
use r2d2::Error;
use serde::Deserialize;
use serde::Serialize;

pub fn config(cfg: App<AppState>) -> App<AppState> {
    cfg.scope("/user", |scope| {
        scope
            .resource("", |r| {
                r.middleware(AuthMiddleware);
                r.method(Method::GET).with(get_user);
            })
            .default_resource(|r| {
                r.h(NormalizePath::new(
                    true,
                    true,
                    StatusCode::TEMPORARY_REDIRECT,
                ));
                r.method(Method::POST).with(create_user);
            })
            .resource("/{id}", |r| {
                r.method(Method::GET).with(get_user_by_id);
            })
            .resource("/{id}/ideas", |r| {
                r.method(Method::GET).with(get_user_ideas);
            })
    })
}

#[derive(Deserialize, Debug)]
pub(crate) struct SignUp {
    pub username: String,
    pub email: String,
}

#[derive(Deserialize, Debug)]
struct UIdeas(String);

fn get_user_ideas((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
    state
        .database
        .send(UIdeas(path.into_inner().clone()))
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().json(r)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

impl Message for UIdeas {
    type Result = Result<Vec<Idea>, Error>;
}

// Get ideas from a specific user via ID
impl Handler<UIdeas> for DbExecutor {
    type Result = Result<Vec<Idea>, Error>;

    fn handle(&mut self, msg: UIdeas, ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();

        let aql = AqlQuery::new(
            "FOR i in 1..1 INBOUND CONCAT('users/', @id ) idea_owner
		SORT i.date DESC
        RETURN i",
        )
        .bind_var("id", msg.0)
        .batch_size(25);

        let response: Vec<Idea> = match conn.aql_query(aql) {
            Ok(r) => r,
            Err(_) => vec![],
        };

        Ok(response)
    }
}

fn run_query(qufigs: QUser, state: State<AppState>) -> FutureResponse<HttpResponse> {
    state
        .database
        .send(qufigs)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::BadRequest().into()),
        })
        .responder()
}

fn get_user_by_id((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let qufig = QUser::ID(path.into_inner());

    run_query(qufig, state)
}

fn get_user(
    (req, state): (HttpRequest<AppState>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    let qufig = if let Some(token) = extract_token(&req) {
        QUser::TOKEN(token)
    } else {
        QUser::TOKEN("None".to_string())
    };

    run_query(qufig, state)
}

pub(crate) fn create_user((json, state): (Json<SignUp>, State<AppState>)) -> HttpResponse {
    if exist_user(json.email.clone(), &state).is_ok() {
        let log = Login {
            email: json.email.clone(),
        };
        return login((Json(log), state));
    };

    let new_user = NewUser {
        username: json.username.clone(),
        email: json.email.clone(),
        created_at: Utc::now().timestamp_millis(),
        active: false,
        ..NewUser::default()
    };

    let chall = Token {
        token: challenge_gen(32),
    };
    let challenge = Challenge {
        challenge: chall.token.clone(),
        email: json.email.clone(),
        username: None,
        pending: true,
        ttl: ttl(15),
    };

    let response = state
        .database
        .send(new_user)
        .and_then(|res| match res {
            Ok(u) => Ok(HttpResponse::Ok().json(u)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .map_err(|x| return x)
        .then(|_| {
            state
                .database
                .send(challenge)
                .and_then(|res| match res {
                    Ok(_) => Ok(HttpResponse::Ok().json(chall)),
                    Err(_) => Ok(HttpResponse::build(StatusCode::BAD_REQUEST).into()),
                })
                .map_err(|x| return x)
        })
        .wait();

    response.unwrap()
}
