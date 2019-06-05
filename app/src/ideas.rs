use crate::AppState;
use actix_web::http::header::http_percent_encode;
use actix_web::AsyncResponder;
use actix_web::http::Method;
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use futures::future::Future;
use ideadog::QueryIdea;
use serde::Deserialize;

pub fn config(cfg: App<AppState>) -> App<AppState> {
    cfg.scope("/ideas", |scope| {
        scope.resource("/", |r| {
            r.method(Method::GET).with(get_ideas);
        })
    })
}

#[derive(Deserialize)]
pub struct Param {
    id: Option<String>,
    tags: Option<String>,
    tags_array: Option<Vec<String>>,
}

fn get_ideas((q_string, state): (Query<Param>, State<AppState>)) -> FutureResponse<HttpResponse> {
    state
        .database
        .send(QueryIdea {
            _key: None,
            _id: None,
            owner: None,
            owner_id: None,
            tags: None,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
	    .responder()
}
