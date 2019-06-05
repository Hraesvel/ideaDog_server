//! IdeaDog is a Social platform for sharing one's idea with the world.

#[macro_use]
extern crate serde_derive;
mod models;
mod database;

pub use database::DbExecutor;
pub use models::*;
