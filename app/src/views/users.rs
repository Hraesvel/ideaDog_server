use crate::{AppState};
use actix_web::http::header::http_percent_encode;
use actix_web::http::{Method, NormalizePath, StatusCode};
use actix_web::{AsyncResponder, Responder, Path, Json};
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use futures::future::Future;
use ideadog::{Sort, User, NewUser, QueryUser};
use serde::Deserialize;
use chrono::Utc;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.scope("/user", |scope| {
		scope.default_resource(|r| {
			r.h(NormalizePath::new(
				true,
				true,
				StatusCode::TEMPORARY_REDIRECT,
			))
		})
		     .resource("/{id}", |r| {
			r.method(Method::GET).with(get_user);
		})
		     .resource("/", |r| {
			     r.method(Method::POST).with(create_user);
		     })
	})
}

#[derive(Deserialize, Debug)]
struct SignUp {
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

fn get_user((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let qufig = QueryUser { id: Some(path.into_inner()), active: None };
	run_query(qufig, state)
}

fn create_user((json, state): (Json<SignUp>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let new_user = NewUser {
		username: json.username.clone(),
		email: json.email.clone(),
		create_at: Utc::now().timestamp_millis(),
		active: false,
		..NewUser::default()
	};

	state
		.database
		.send(new_user)
		.from_err()
		.and_then(|res| match res {
			Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
			Err(_) => Ok(HttpResponse::InternalServerError().into()),
		})
		.responder()
}