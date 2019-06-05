use chrono::Utc;
use chrono::DateTime;

pub struct User {
	pub _key: String,
	pub _id: String,
	pub usersname: String,
	pub email: String,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

