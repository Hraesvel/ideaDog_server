use crate::models::{Idea, NewIdea, QueryIdea};
use actix_web::actix::Message;
use r2d2::Error;
use serde_json::Value;

impl Message for Idea {
	type Result = Result<Idea, Error>;
}

impl Message for NewIdea {
	type Result = Result<(), Error>;
}

impl Message for QueryIdea {
	type Result = Result<Vec<Idea>, Error>;
}