use crate::models::challenge::*;
use failure::Error;
use actix_web::actix::Message;
use r2d2;


impl Message for Login {
	type Result = Result<Vec<bool>, Error>;
}

impl Message for Challenge {
	type Result = Result<String, Error>;
}
