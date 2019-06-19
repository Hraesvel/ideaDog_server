use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Login {
	pub email: String,
}

#[derive(Deserialize, Debug)]
pub struct Signup {
	pub email: String,
	pub username: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Challenge {
	#[serde(alias = "_key")]
	#[serde(rename(serialize = "_key"))]
	pub challenge: String,
	pub email: String,
	pub username: Option<String>,
	pub pending: bool,
	pub ttl: i64,
}
