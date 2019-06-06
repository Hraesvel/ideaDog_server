use serde::{Serialize, Deserialize};
use serde_derive::*;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tag {
	#[serde(alias = "key")]
	pub _key: String,
	#[serde(alias="id")]
	pub _id: String,
	#[serde(default)]
	pub count: u64
}

