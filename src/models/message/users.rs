use actix::Message;
use r2d2::Error;
use crate::models::{User, NewUser};

impl Message for User {
	type Result = Result<Vec<User>, Error>;
}


impl Message for NewUser {
	type Result = Result<Vec<User>, Error>;
}