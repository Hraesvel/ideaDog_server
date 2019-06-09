use chrono::Utc;
use chrono::DateTime;

#[derive(Serialize, Deserialize)]
pub struct User {
	#[serde(aliea = "_key")]
	pub key: String,
	#[serde(aliea = "_id")]
	pub id: String,
	#[serde(aliea = "name")]
	pub username: String,
	pub email: String,
	//	pub date: DateTime<Utc>,
	pub ideas: Vec<String>,
	pub active: bool,
}

pub struct QueryUser {
	// get user by id only an active user
	id: Option<String>,
	// if None all users,
	// if Some(true) should only active users and if Some(false) show inactive users
	active: Option<bool>,
}

pub struct NewUser {
	pub username: String,
	pub email: String,
}

