use crate::models::{CastVote, Idea, NewIdea, QueryIdea, VoteStatus};
use actix::MailboxError;
use actix_web::actix::Message;
use r2d2::Error;

impl Message for Idea {
    type Result = Result<Idea, Error>;
}

impl Message for NewIdea {
    type Result = Result<Idea, Error>;
}

impl Message for QueryIdea {
    type Result = Result<Vec<Idea>, Error>;
}

impl Message for CastVote {
    type Result = Result<VoteStatus, MailboxError>;
}
