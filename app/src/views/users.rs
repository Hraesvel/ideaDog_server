use crate::AppState;

use crate::midware::AuthMiddleware;
use crate::views::auth::{exist_user, login, ttl, challenge_gen, Token};
use actix_web::http::{Method, NormalizePath, StatusCode};
use actix_web::actix::{Message, Handler};
use actix_web::{App, FutureResponse, HttpResponse, Responder, State, Path};
use actix_web::{AsyncResponder, HttpRequest, Json};
use chrono::Utc;
use futures::future::{ok, Future, IntoFuture};
use ideadog::{NewUser, QueryUser, DbExecutor, Idea, Login, Challenge};
use serde::Deserialize;
use r2d2::Error;
use serde::Serialize;
use arangors::AqlQuery;

pub fn config(cfg: App<AppState>) -> App<AppState> {
    cfg.scope("/user", |scope| {
        scope
            .resource("/", |r| {
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
            .resource("/{id}/ideas", |r|{
                r.method(Method::GET).with(get_user_ideas);
            })
        //		     .resource("/", |r| {
        //			     r.method(Method::POST).with(create_user);
        //		     })
    })
}

#[derive(Deserialize, Debug)]
pub(crate) struct SignUp {
    pub username: String,
    pub email: String,
}

fn run_query(qufigs: QueryUser, state: State<AppState>) -> FutureResponse<HttpResponse> {
    state
        .database
        .send(qufigs)
        .from_err()
        .and_then(|res| match res {
            Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

#[derive(Deserialize, Debug)]
struct UIdeas(String);

fn get_user_ideas((path,state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
    state
        .database
        .send(UIdeas(path.into_inner().clone()))
        .from_err()
        .and_then( |res| match res {
            Ok(r) => Ok(HttpResponse::Ok().json(r)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        }).responder()
}

impl Message for UIdeas {
    type Result = Result<Vec<Idea>, Error>;
}

impl Handler<UIdeas> for DbExecutor {
    type Result = Result<Vec<Idea>, Error>;

    fn handle(&mut self, msg: UIdeas, ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();

        let aql = AqlQuery::new(
            "FOR i in 1..1 INBOUND CONCAT('users/', @id ) idea_owner
        return i").bind_var("id", msg.0)
                  .batch_size(25);

        let response: Vec<Idea>  = match conn.aql_query(aql) {
            Ok(r) => r,
            Err(_) => vec![]
        };

        Ok(response)
    }
}


fn get_user((req, state): (HttpRequest<AppState>, State<AppState>)) -> impl Responder {
    let tok = req
        .headers()
        .get("AUTHORIZATION")
        .map(|value| value.to_str().ok())
        .unwrap();
    //    dbg!(tok);

    let mut token = tok
        .unwrap()
        .split(" ")
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .pop()
        .unwrap();

    //    HttpResponse::Ok().finish();

    let qufig = QueryUser { token: Some(token) };

    run_query(qufig, state)
}

pub(crate) fn create_user((json, state): (Json<SignUp>, State<AppState>)) -> HttpResponse {
	if exist_user(json.email.clone(), &state).is_ok() {
        let log = Login {
            email: json.email.clone()
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

	let chall = Token { token: challenge_gen(32) };
	let challenge = Challenge {
		challenge: chall.token.clone(),
		email: json.email.clone(),
		username: None,
		pending: true,
		ttl: ttl(15)
	};

    let response = state
        .database
        .send(new_user)
//        .from_err()
        .and_then(|res| match res {
            Ok(u) => Ok(HttpResponse::Ok().json(u)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        }).map_err(|x| return x)
        .then(|_|
            state
                .database
                .send(challenge)
//                .from_err()
                .and_then(|res| match res {
                    Ok(_) => Ok(HttpResponse::Ok().json(chall)),
                    Err(_) => Ok(HttpResponse::build(StatusCode::BAD_REQUEST).into())
                })
                .map_err(|x| return x)
        ).wait();


//    let c = state
//        .database
//        .send(challenge)
//        .from_err()
//        .and_then(|res| match res {
//            Ok(_) => Ok(HttpResponse::Ok().json(chall).finish()),
//            Err(_) => Ok(HttpResponse::build(StatusCode::BAD_REQUEST).json("badrequest").finish())
//        }).wait();

    response.unwrap()
}
