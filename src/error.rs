pub(crate) mod service {
	extern crate failure;
	use actix_web::{ResponseError, HttpResponse};

	#[derive(Debug, Fail)]
	pub enum ServiceError {
		#[fail(display = "Unauthorised")]
		Unauthorised,
		#[fail(display = "Bad Request")]
		BadRequest,
		#[fail(display = "Not Found")]
		NotFound
	}

	impl ResponseError for ServiceError {
		fn error_response(&self) -> HttpResponse {
			match self {
				ServiceError::Unauthorised => HttpResponse::Unauthorized().json("Unauthorised"),
				ServiceError::BadRequest => HttpResponse::BadRequest().json("Bad_Request"),
				ServiceError::NotFound => HttpResponse::NotFound().json("Not Found")
			}
		}
	}
}