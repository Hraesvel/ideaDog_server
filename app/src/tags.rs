use crate::AppState;
use actix_web::http::header::http_percent_encode;
use actix_web::AsyncResponder;
use actix_web::http::Method;
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use futures::future::Future;
use ideadog::Tag;
use serde::Deserialize;
use actix_web::client::get;

//pub fn config(cfg: App<AppState>) -> App<AppState>{
//	cfg.scope("/tags", |scope|
//	scope
//		.resource("/" ,|resp|
//			resp.method(Method::GET).with(get_tags)
//		)
//		.resource("/{id}", |resp|
//		resp.method(Method::GET).with(get_one_tag)
//		)
//		.resource("/{id}/ideas", |resp|
//		resp.method(Method::GET).with(get_relation)
//		)
//	)
//}