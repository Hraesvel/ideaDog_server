use crate::models::{QueryTag, Tag};
use actix::Message;
use r2d2::Error;

impl Message for QueryTag {
    type Result = Result<Vec<Tag>, Error>;
}
