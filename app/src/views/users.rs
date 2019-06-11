use crate::{AppState};
use actix_web::http::header::http_percent_encode;
use actix_web::http::{Method, NormalizePath, StatusCode};
use actix_web::{AsyncResponder, Responder, Path, Json, HttpRequest};
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use futures::future::{Future, IntoFuture};
use ideadog::{Sort, User, NewUser, QueryUser};
use serde::Deserialize;
use chrono::Utc;
use crate::midware::AuthMiddleware;
use actix_web::client::head;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.scope("/user", |scope| {
		scope.default_resource(|r| {
			r.h(NormalizePath::new(
				true,
				true,
				StatusCode::TEMPORARY_REDIRECT,
			));
			r.method(Method::POST).with(create_user);
		})
		     .resource("/", |r| {
			     r.middleware(AuthMiddleware);
			     r.method(Method::GET).with(get_user);
		})
//		     .resource("/", |r| {
//			     r.method(Method::POST).with(create_user);
//		     })
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

fn get_user((req, state): (HttpRequest<AppState>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let token = req.headers().get("AUTHORIZATION").map(|value| value.to_str().ok())
	               .ok_or(HttpResponse::Unauthorized().into());

	if token.err() {
		return token.into_future();
	}

	let qufig = QueryUser { token: token.clone() };
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