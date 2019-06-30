//! IdeaDog is a Social platform for sharing one's idea with the world.

#[macro_use]
extern crate failure;
extern crate serde_derive;
mod database;
mod models;
mod error;

pub use database::DbExecutor;
pub use error::service::ServiceError;
pub use models::*;
