use crate::AppState;

use crate::midware::ServiceError;
use crate::util::user::extract_token;
use crate::AuthMiddleware;
use actix_web::actix::{Handler, Message, MailboxError};
use actix_web::http::{Method, NormalizePath, StatusCode};
use actix_web::{App, FutureResponse, HttpRequest, HttpResponse, Json, Query, State, Error};
use actix_web::{AsyncResponder, Path};
use arangors::AqlQuery;
use futures;
use futures::future::{err, Future, ok};
use ideadog::{DbExecutor, NewIdea, QueryIdea, Sort, Idea};
use serde::Deserialize;
use serde_json::Value;
use std::f32::consts::E;
use toml::map::Values;
use crate::midware::ServiceError::BadRequest;

//use actix_web::ws::Message;

pub fn config(cfg: App<AppState>) -> App<AppState> {
    cfg.resource("/ideas", |r| {
        r.method(Method::GET).with(get_ideas);
    })
    .resource("/ideas/{sort}", |r| {
        r.method(Method::GET).with(get_ideas_sort);
    })
    .scope("/idea", |scope| {
        scope
            .default_resource(|r| {
                r.h(NormalizePath::new(
                    true,
                    true,
                    StatusCode::TEMPORARY_REDIRECT,
                ))
            })
            .resource("", |r| {
                r.middleware(AuthMiddleware);
                r.method(Method::POST).with(create_idea);
            })
            .resource("/{id}", |r| {
	            r.middleware(AuthMiddleware);
                r.method(Method::GET).with(get_idea_id);
                r.method(Method::DELETE).with(delete_idea_id);
            })
	        .resource("/{id}/{vote}", |r| {
		        r.middleware(AuthMiddleware);
		        r.method(Method::POST).with(update_idea_id);
	        })

    })
}

#[derive(Deserialize, Debug)]
struct Param {
    id: Option<String>,
    tags: Option<String>,
}

#[derive(Deserialize)]
struct IdeaForm {
    text: String,
    owner_id: String,
    tags: Vec<String>,
}

fn run_query(qufigs: QueryIdea, state: State<AppState>) -> FutureResponse<HttpResponse> {
    state
        .database
        .send(qufigs)
        .from_err()
        .and_then(|res| match res {
            Ok(ideas) => Ok(HttpResponse::Ok().chunked().json(ideas)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

fn fail_query() -> FutureResponse<HttpResponse> {
    err::<HttpResponse, actix_web::Error>(actix_web::error::Error::from(ServiceError::BadRequest))
        .responder()
}

fn get_ideas_sort(
    (path, q_string, state): (Path<String>, Query<Param>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    let mut vec_of_tags = None;
    if let Some(value) = &q_string.tags {
        let v_string: Vec<String> = value.clone().split(',').map(|x| x.to_string()).collect();
        vec_of_tags = Some(v_string);
    };

    let mut q = QueryIdea {
        sort: Sort::ALL,
        id: None,
        owner: None,
        owner_id: None,
        tags: vec_of_tags,
        limit: None,
    };

    match path.into_inner().to_lowercase().as_str() {
        "bright" => q.sort = Sort::BRIGHT,
        _ => {}
    }

    run_query(q, state)
}

fn get_ideas((q_string, state): (Query<Param>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let mut vec_of_tags = None;
    if let Some(value) = &q_string.tags {
        let v_string: Vec<String> = value.clone().split(',').map(|x| x.to_string()).collect();
        vec_of_tags = Some(v_string);
    };

    let mut q = QueryIdea {
        sort: Sort::ALL,
        id: None,
        owner: None,
        owner_id: None,
        tags: vec_of_tags,
        limit: None,
    };

    if let Some(t) = q_string.tags.clone() {
        let tags: Vec<String> = t.split(",").map(|x| x.to_string()).collect();
        q.tags = Some(tags);
    }

    run_query(q, state)
}

fn get_idea_id((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let q = QueryIdea {
        sort: Sort::ALL,
        id: Some(path.into_inner()),
        owner: None,
        owner_id: None,
        tags: None,
        limit: None,
    };

    run_query(q, state)
}

fn create_idea((idea, state): (Json<IdeaForm>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let new = NewIdea {
        text: idea.text.clone(),
        owner_id: idea.owner_id.clone(),
        tags: idea.tags.clone(),
    };

    if idea.text.len() > 140 {
        let error = HttpResponse::build(StatusCode::from_u16(422).unwrap())
            .reason("Text length supplied is greater than allowed.")
            .finish();
        return Box::new(futures::future::ok(error.into()));
    }

    state
        .database
        .send(new)
        .from_err()
        .and_then(|res| match res {
            Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}


#[derive(Clone)]
struct UserVote{
    idea_id: String,
    u_token: String,
    vote: String
}

fn update_idea_id((req, path, state): (HttpRequest<AppState> ,Path<(String,String)>, State<AppState>)) -> FutureResponse<HttpResponse>{
    let current_user = dbg!(extract_token(&req));
	let (id, vote) = path.into_inner();

//
//	match vote.as_ref() {
//		"upvote" => {},
//		"downvote" => {},
//		_ => {return err::<HttpResponse, actix_web::error::Error>(ServiceError::BadRequest.into());}
//	}


    ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok().finish())
        .and_then( |_| {
            match vote.as_ref() {
                "upvote" => Ok(vote),
                "downvote" => Ok(vote),
                _ => {
                    println!("A");
                    Err(ServiceError::BadRequest.into())
                }
            }
        })
        .and_then(move |vote| {
            let user_vote = if let Some(token) = current_user {
               Ok( UserVote {
                    idea_id: id,
                    u_token: token,
                    vote
                })
            } else {

                Err(ServiceError::BadRequest.into())
            };
            user_vote
        })
        .from_err()
        .and_then(move |qufigs| {
            state
                .database
                .send(qufigs)
                .from_err()
                .and_then(|res| match res {
                    Ok(_) => Ok(HttpResponse::Ok().finish()),
                    Err(_) =>{
                        println!("here?");
                        Err(ServiceError::BadRequest.into())
                    }
                })
        }
        ).responder()
}

impl Message for UserVote{
	type Result = Result<Value, MailboxError>;
}

impl Handler<UserVote> for DbExecutor {
	type Result = Result<Value, MailboxError>;

	fn handle(&mut self, msg: UserVote, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();

        let aql = AqlQuery::new(
            "LET user = FIRST (for u in 1..1 OUTBOUND DOCUMENT('bearer_tokens', @token) bearer_to_user RETURN u)
                LET idea_id = CONCAT('ideas/', @id)
                UPSERT { _from: idea_id , _to: user._id }
                INSERT { _from: idea_id , _to: user._id, vote: @vote }
                UPDATE { vote: @vote } IN idea_voter LET vote = NEW
                RETURN NEW"
        ).batch_size(1)
            .bind_var("vote", msg.vote)
            .bind_var("token", msg.u_token)
            .bind_var("id", msg.idea_id);

        let response: Result<Value, MailboxError> = match conn.aql_query(aql) {
            Ok(mut r) => Ok(r.pop().unwrap()),
            Err(e) => {
                println!("Error: {}", e);
                Err(MailboxError::Closed)}
        };

        dbg!(response)
	}
}

fn delete_idea_id(
    (path, req, state): (Path<String>, HttpRequest<AppState>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    if let Some(t) = extract_token(&req) {

        let qufigs = DeleteIdea {
            token: t,
            idea_id: path.into_inner(),
        };

        state
            .database
            .send(qufigs)
            .from_err()
            .and_then(|res| match res {
                Ok(_) => Ok(HttpResponse::Ok().finish()),
                Err(_) => Err(actix_web::error::Error::from(ServiceError::BadRequest)),
            })
            .responder()
    } else {
        fail_query()
    }
}

struct DeleteIdea {
    token: String,
    idea_id: String,
}

impl Message for DeleteIdea {
    type Result = Result<(), MailboxError>;
}

impl Handler<DeleteIdea> for DbExecutor {
    type Result = Result<(), MailboxError>;

    fn handle(&mut self, msg: DeleteIdea, _ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();
        let aql = AqlQuery::new(
            "
            let idea = FIRST (
            FOR i in 2..2 ANY DOCUMENT('bearer_tokens', @token) bearer_to_user, idea_owner
            FILTER i._key == @idea_key
            return i
            )
            let owner = FIRST (FOR v, e in 1..1 OUTBOUND idea._id idea_owner RETURN e)
            REMOVE owner IN idea_owner let e = OLD
            REMOVE idea IN ideas LET removed = OLD
            return removed
		"
        ).bind_var("token", msg.token.clone())
        .bind_var("idea_key", msg.idea_id)
        .batch_size(1);

        let response : Option<Vec<Idea>>  =  match conn.aql_query(aql) {
            Ok(r) => Some(r),
           _ => None
        };

        if let Some(r) = response {
            return Ok(());
        }else {
            return Err(MailboxError::Closed)
        }
    }
}
