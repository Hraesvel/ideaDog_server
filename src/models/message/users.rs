use crate::models::{NewUser, QueryUser, User, QUser};
use actix::{Message, MailboxError};
use r2d2::Error;

impl Message for QueryUser {
	type Result = Result<User, MailboxError>;
}

impl Message for QUser {
	type Result = Result<User, MailboxError>;
}

impl Message for NewUser {
	type Result = Result<Vec<User>, Error>;
}
