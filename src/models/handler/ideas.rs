use crate::models::{Idea, NewIdea, QueryIdea, Sort, Owner};
use crate::DbExecutor;
use actix::Handler;
use arangors;
use arangors::AqlQuery;
use r2d2::Error;
use r2d2_arangodb::ArangodbConnectionManager;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::borrow::Cow::Owned;

impl Handler<QueryIdea> for DbExecutor {
	type Result = Result<Vec<Idea>, Error>;

	fn handle(&mut self, msg: QueryIdea, ctx: &mut Self::Context) -> Self::Result {
		let conn = &self.0.get().unwrap();

		let mut query = "FOR ele in ideas ".to_string();
		match &msg.sort {
			Sort::ALL => {},
			Sort::BRIGHT => { query.push_str("SORT ele.date ") }
		}

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
	type Result = Result<Idea, Error>;

	fn handle(&mut self, msg: NewIdea, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();

		let mut obj = HashMap::new();
		let owner = Owner::get_owner(msg.owner_id, &conn).map_err(|x| return x).unwrap();
		let t : Vec<String> = msg.tags.iter().map(|x| x._key).collect();
		obj.insert("text",  msg.text);
		obj.insert("tags", t.serialize());
		obj.insert("owner", owner.serialize() );

		// owner : { _id: 'users/key', _key: 'key', }


		let aql = AqlQuery::new("INSERT @@obj INTO @@col")
			.bind_var("@obj", obj)
			.bind_var("@col", ideas)
			.batch_size(1);

		let request : Idea = match conn.aql_query(aql)
			{
				Ok(r) => r,
				Err(e) => {println!("fail to create idea"); vec![]}
			};


		Ok(request)
	}

}
