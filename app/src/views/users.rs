use crate::{AppState};
use actix_web::http::header::http_percent_encode;
use actix_web::http::Method;
use actix_web::{AsyncResponder, Responder, Path, Json};
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use futures::future::Future;
use ideadog::{Sort, User, NewUser, QueryUser};
use serde::Deserialize;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.scope("/user", |scope| {
		scope.resource("/{id}", |r| {
			r.method(Method::GET).with(get_user);
		})
	})
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

fn get_user((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
	dbg!(&path);
	let qufig = QueryUser { id: Some(path.into_inner()), active: None };
	run_query(qufig, state)
}