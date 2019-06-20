use crate::AppState;
use actix_web::actix::{Handler, Message};
use actix_web::middleware::{Middleware, Started};
use actix_web::HttpRequest;
use actix_web::{AsyncResponder, HttpResponse, ResponseError, Result};
use arangors::AqlQuery;
use futures::future::Future;
use ideadog::{DbExecutor, User};
use r2d2;

pub struct AuthMiddleware;

impl Middleware<AppState> for AuthMiddleware {
	fn start(&self, req: &HttpRequest<AppState>) -> Result<Started> {


		if req.method() == "OPTIONS" || req.method()  == "GET" {
			return Ok(Started::Done);
		}

		let state: &AppState = req.state();

		let token = req
			.headers()
			.get("AUTHORIZATION")
			.map(|value| value.to_str().ok())
			.ok_or(ServiceError::Unauthorised)?;

		match token {
			Some(t) => {
				let mut token = t.split(" ").map(|x| x.to_string()).collect::<Vec<String>>();
				verify_token(token.pop().unwrap().to_owned(), state)
			},
			None => {
				Err(ServiceError::Unauthorised.into())},
		}
	}
}

struct Verify(String);

/// Verify token function queries the database to see if the provided token
/// existed in the database
fn verify_token(token: String, state: &AppState) -> Result<Started> {
	let tok = Verify(token);

	let response = state
		.database
		.send(tok)
		.from_err()
		.and_then(|res| match res {
			true =>  {
				Ok(Started::Done)
			},
			false => {
				Err(ServiceError::Unauthorised.into())},
		})
		.wait();

	response
}

#[derive(Debug, Fail)]
pub(crate) enum ServiceError {
	#[fail(display = "Unauthorised")]
	Unauthorised,
	#[fail(display = "Bad Request")]
	BadRequest
}

impl ResponseError for ServiceError {
	fn error_response(&self) -> HttpResponse {
		match self {
			ServiceError::Unauthorised => HttpResponse::Unauthorized().json("Unauthorised"),
			ServiceError::BadRequest => HttpResponse::BadRequest().json("Bad_Request")
		}
	}
}

impl Message for Verify {
	type Result = bool;
}

impl Handler<Verify> for DbExecutor {
	type Result = bool;

	fn handle(&mut self, msg: Verify, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();
		let aql = AqlQuery::new("RETURN IS_DOCUMENT(DOCUMENT('bearer_tokens', @tok))")
			.bind_var("tok", msg.0.clone())
			.batch_size(1);

		let response = conn.aql_query(aql).unwrap();
		response[0]
	}
}
