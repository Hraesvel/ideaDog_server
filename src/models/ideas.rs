use std::collections::HashMap;
use serde::Deserialize;

use crate::models::Tag;


#[derive(Deserialize)]
pub struct Votes {
	pub up_vote: HashMap<String,u32>,
	pub down_vote: HashMap<String, u32>,
	pub average: u32
}

pub struct Idea {
	// _key field from arangodb
	pub _key: String,
	// _id field from arangodb
	pub _id: String,
	// title of the idea
	pub title: String,
	// description of idea
	pub description: Option<String>,
	// Owner's username
	pub owner: String,
	// Owner's string id
	pub owner_id: String,
	// This field is for the votes.
	pub votes: Votes
}

pub struct NewIdea {
	// title of the idea
	pub title: String,
	// description of idea
	pub description: Option<String>,
	// Owner's username
	pub owner_id: String,

	pub tags : Vec<Tag>
}







