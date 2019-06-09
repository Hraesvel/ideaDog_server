use actix::Message;
use r2d2::Error;
use crate::models::{User, NewUser, QueryUser};

impl Message for QueryUser {
	type Result = Result<Vec<User>, Error>;
}


impl Message for NewUser {
	type Result = Result<Vec<User>, Error>;
}