use crate::AppState;
use actix_web::http::header::http_percent_encode;
use actix_web::http::Method;
use actix_web::{AsyncResponder, Responder, Path};
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use futures::future::Future;
use ideadog::{Sort, Userm NewUser};
use serde::Deserialize;
use actix_net::service::ServiceExt;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.scope("/users", |scope| {
		scope.resource("/", Method::GET).with()
	})
}

fn get_users(state: State<AppState>) -> FutureResponse<HttpResponse> {}