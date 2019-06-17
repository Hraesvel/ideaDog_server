//! IdeaDog is a Social platform for sharing one's idea with the world.

extern crate serde_derive;
mod database;
mod models;

pub use database::DbExecutor;
pub use models::*;
