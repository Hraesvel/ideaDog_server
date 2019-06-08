use serde::{Serialize, Deserialize};
use crate::models::{Sort, Idea};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tag {
	//	#[serde(alias = "key")]
	pub _key: String,
	//	#[serde(alias="id")]
	pub _id: String,
	#[serde(default)]
	pub count: u64,
	#[serde(default)]
	pub ideas: Vec<Idea>
}

#[derive(Debug)]
pub struct QueryTag {
	pub id: Option<Vec<String>>,
	pub with_ideas: bool,
	pub sort: Sort
}


