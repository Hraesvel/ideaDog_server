use crate::models::{Idea, NewIdea, QueryIdea, Sort, Owner};
use crate::DbExecutor;
use actix::Handler;
use arangors;
use arangors::AqlQuery;
use r2d2::Error;
use r2d2_arangodb::ArangodbConnectionManager;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_derive::*;
use serde_json::Value;
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDateTime};

impl Handler<QueryIdea> for DbExecutor {
	type Result = Result<Vec<Idea>, Error>;

	fn handle(&mut self, msg: QueryIdea, ctx: &mut Self::Context) -> Self::Result {
		let conn = &self.0.get().unwrap();

		let mut query = "FOR ele in ideas ".to_string();

		// Handles Sort
		match &msg.sort {
			Sort::ALL => {},
			Sort::BRIGHT => { query.push_str("SORT ele.date ") }
		}

		// Handles filters
//		match &msg.tags {
//
//		}

		query.push_str("RETURN ele");

		let mut aql = AqlQuery::new(dbg!(&query)).batch_size(25);

		if let Some(id) = msg.id {
			aql = AqlQuery::new("RETURN DOCUMENT(CONCAT('ideas/', @id ))")
				.bind_var("id", id)
				.batch_size(1);
		}

		let request: Vec<Idea> = match conn.aql_query(aql) {
			Ok(r) => r,
			Err(e) => {
				println!("{}", e);
				vec![]
			}
		};
		Ok(request)
	}
}

impl Handler<NewIdea> for DbExecutor {
	type Result = Result<(), Error>;

	fn handle(&mut self, msg: NewIdea, ctx: &mut Self::Context) -> Self::Result {
		#[derive(Deserialize, Serialize, Debug)]
		pub struct IdeaMIN {
			// title of the idea
			pub text: String,
			// description of idea
			// Owner's username
			pub owner: Owner,
			// This field is for the votes.
			#[serde(default)]
			pub upvotes: u32,
			#[serde(default)]
			pub downvotes: u32,
			pub tags: Vec<String>,
			pub date: i64
		}

		let conn = self.0.get().unwrap();

		let new_idea = IdeaMIN {
			text: msg.text.clone(),
			tags: msg.tags.clone(),
			owner: Owner::get_owner(msg.owner_id, &conn).expect("Fail to get owner details."),
			upvotes: 0,
			downvotes: 0,
			date: Utc::now().timestamp_millis()
		};

		let data = serde_json::to_value(&new_idea).unwrap();

		let query = dbg!(format!("INSERT {data} INTO {collection} LET i = NEW RETURN i", data = data, collection = "ideas"));

		let aql = AqlQuery::new(&query)
			.batch_size(1);

		let _ : Vec<Idea> = dbg!(conn.aql_query(aql).map_err(|e| panic!("Error: {}", e)).unwrap());

		Ok(())
	}

}
