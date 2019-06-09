use chrono::Utc;
use chrono::DateTime;

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
	//	pub date: DateTime<Utc>,
	pub ideas: Vec<String>,
	pub active: bool,
}

#[derive(Debug)]
pub struct QueryUser {
	// get user by id only an active user
	pub id: Option<String>,
	// if None all users,
	// if Some(true) should only active users and if Some(false) show inactive users
	pub active: Option<bool>,
}

#[derive(Debug)]
pub struct NewUser {
	pub username: String,
	pub email: String,
}

