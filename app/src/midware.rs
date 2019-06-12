use actix_web::actix::{Message, Handler};
use actix_web::middleware::{Middleware, Started};
use actix_web::{Result, ResponseError, AsyncResponder, HttpResponse};
use crate::AppState;
use actix_web::HttpRequest;
use ideadog::{DbExecutor, User};
use futures::future::Future;
use r2d2;
use arangors::AqlQuery;

pub struct AuthMiddleware;

impl Middleware<AppState> for AuthMiddleware {
	fn start(&self, req: &HttpRequest<AppState>) -> Result<Started> {
		if req.method() == "OPTIONS" {
			return Ok(Started::Done);
		}

		let state: &AppState = req.state();

		let token = req.headers()
		               .get("AUTHORIZATION")
		               .map(|value| value.to_str().ok())
		               .ok_or(ServiceError::Unauthorised)?;

		match token {
			Some(t) => {
				verify_token(t.to_owned(), state)
			},
			None => Err(ServiceError::Unauthorised.into())
		}
	}
}

struct Verify(String);

impl Message for Verify {
	type Result = bool;
}

impl Handler<Verify> for DbExecutor {
	type Result = bool;

	fn handle(&mut self, msg: Verify, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();
		let aql = AqlQuery::new("RETURN IS_DOCUMENT(DOCUMENT('tokens', @tok))")
			.bind_var("tok", msg.0.clone())
			.batch_size(1);

		let response = conn.aql_query(aql).unwrap();

		response[0]
	}
}

/// Verify token function queries the database to see if the provided token
/// existed in the database
fn verify_token(token: String, state: &AppState) -> Result<Started> {
	let t = Verify(token);

	let conn = state.database.send(t)
	                .from_err()
	                .and_then(|res| match res {
		                true => Ok(Started::Done),
		                false => Err(ServiceError::Unauthorised.into())
	                }).wait();

	let response = conn;

	response
}


#[derive(Debug, Fail)]
enum ServiceError {
	#[fail(display = "Unauthorised")]
	Unauthorised
}

impl ResponseError for ServiceError {
	fn error_response(&self) -> HttpResponse {
		match self {
			ServiceError::Unauthorised => HttpResponse::Unauthorized().json("Unauthorised")
		}
	}
}
