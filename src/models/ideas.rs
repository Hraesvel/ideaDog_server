use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use crate::models::Tag;

#[derive(Deserialize, Serialize, Debug)]
pub struct Owner {
	// _id field from arangodb
	#[serde(alias = "_id")]
	pub id: String,
	// _key field from arangodb
	#[serde(alias = "_key", default)]
	pub key: String,
	pub name: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Idea {
	// _id field from arangodb
	#[serde(alias = "_id")]
	pub id: String,
	// _key field from arangodb
	#[serde(alias = "_key")]
	pub key: String,
	// title of the idea
	pub text: String,
	// description of idea
	// Owner's username
	pub owner: Owner,
	// This field is for the votes.
	pub upvotes: u32,
	pub downvotes: u32,

	pub tags: Vec<String>,

	pub date: i64
}

pub struct NewIdea {
	// title of the idea
	pub title: String,
	// description of idea
	pub description: Option<String>,
	// Owner's username
	pub owner_id: String,

	pub tags: Vec<Tag>,
}

pub enum Sort {
	ALL,
	BRIGHT
}

pub struct QueryIdea {
	pub sort: Sort,
	//id
	pub id: Option<String>,
	// Owner's username
	pub owner: Option<String>,
	// Owner's string id
	pub owner_id: Option<String>,
	// accept tags for query string
	pub tags: Option<Vec<String>>,

	pub limit: Option<u32>,
}
