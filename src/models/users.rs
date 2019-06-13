use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
	#[serde(alias = "_key")]
	pub key: String,
	#[serde(alias = "_id")]
	pub id: String,
	#[serde(alias = "name")]
	pub username: String,
	pub email: String,
	pub ideas: Vec<String>,
	pub active: bool,
	pub favorite: String,
	pub upvotes: u32,
	pub downvotes: u32,
	pub created_at: i64,
}

#[derive(Debug)]
pub struct QueryUser {
	// find the token and then get the user it points to.
	pub token: Option<String>,
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
