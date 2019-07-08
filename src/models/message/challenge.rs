use crate::models::challenge::*;
use actix_web::actix::Message;
use failure::Error;


impl Message for Login {
    type Result = Result<Vec<bool>, Error>;
}

impl Message for Challenge {
    type Result = Result<String, Error>;
}
