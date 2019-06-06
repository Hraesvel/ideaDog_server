use crate::AppState;
use actix_web::http::header::http_percent_encode;
use actix_web::http::Method;
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use actix_web::{AsyncResponder, Path};
use futures::future::Future;
use ideadog::QueryIdea;
use rand::prelude::*;
use rand::Rng;
use serde::Deserialize;

pub fn config(cfg: App<AppState>) -> App<AppState> {
    cfg.scope("/ideas", |scope| {
        scope
            .resource("/", |r| {
                r.method(Method::GET).with(get_ideas);
            })
    })
       .scope("/idea", |scope| {
	       scope.resource("/{id}", |r| {
                r.method(Method::GET).with(get_idea_id);
//                r.method(Method::POST).with(post_idea);
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
	        sort: Sort::ALL,
            id: None,
            owner: None,
            owner_id: None,
            tags: None,
            limit: None
        })
        .from_err()
        .and_then(|res| match res {
            Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
	    .responder()
}

fn get_idea_id((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
    state
        .database
        .send(QueryIdea {
	        sort: Sort::ALL,
            id: Some(path.into_inner()),
            owner: None,
            owner_id: None,
            tags: None,
            limit: None,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}
