use crate::models::{NewUser, QUser, QueryUser, User};
use actix::{MailboxError, Message};
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
