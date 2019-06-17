use crate::models::{NewUser, QueryUser, User};
use actix::Message;
use r2d2::Error;

impl Message for QueryUser {
	type Result = Result<Vec<User>, Error>;
}

impl Message for NewUser {
	type Result = Result<Vec<User>, Error>;
}
