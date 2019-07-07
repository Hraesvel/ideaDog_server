use crate::models::{Idea, NewIdea, QueryIdea, CastVote, VoteStatus};
use actix_web::actix::Message;
use r2d2::Error;
use actix::MailboxError;

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
