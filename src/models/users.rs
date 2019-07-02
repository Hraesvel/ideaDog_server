use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
	#[serde(alias = "_key")]
	pub key: String,
	#[serde(alias = "_id")]
	pub id: String,
	#[serde(alias = "name")]
	pub username: String,
	pub email: String,
	#[serde(default)]
	pub ideas: HashMap<String, String>,
	pub active: bool,
	pub favorite: String,
	pub upvotes: u32,
	pub downvotes: u32,

    #[serde(default)]
    pub votes: Option<HashMap<String, String>>,
    pub created_at: i64,
}

#[derive(Debug)]
pub struct QueryUser {
    // find the token and then get the user it points to.
    pub token: Option<String>,
    pub id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct QUserParams {
    include_ideas: Option<bool>,
}

pub enum QUser {
    TOKEN(String),
    ID(String),
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub ideas: Vec<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub favorite: String,
    #[serde(default)]
    pub upvotes: u32,
    #[serde(default)]
    pub downvotes: u32,
    pub created_at: i64,
}
