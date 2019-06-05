use crate::models::{NewIdea, QueryIdea, Votes};
use crate::DbExecutor;
use actix::Handler;
use arangors;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;
use r2d2_arangodb::ArangodbConnectionManager;
use serde_json::Value;
use arangors::AqlQuery;
use r2d2::Error;
use rand::prelude::*;
use rand::Rng;

//Todo drop IdeaTmp and use Idea Model
#[derive(Deserialize, Debug)]
pub struct IdeaTmp {
	// _key field from arangodb
//	#[serde(skip_serializing)]
	pub _key: String,
	// _id field from arangodb
	pub _id: String,
	// title of the idea
	pub title: String,

	#[serde(alias="desc")]
	// description of idea
	pub description: Option<String>,
//	// Owner's username
//	pub owner: String,
//	// Owner's string id
//	pub owner_id: String,
//	// This field is for the votes.
//	pub votes: Votes,
}

impl Serialize for IdeaTmp {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok,  S::Error>
		where
		S: Serializer {
		let mut state = serializer.serialize_struct("IdeaTmp", 4)?;
		state.serialize_field("id", &self._id)?;
		state.serialize_field("key", &self._key)?;
		state.serialize_field("title", &self.title)?;
		state.serialize_field("description", &self.description)?;
		state.end()
	}
}

impl Handler<QueryIdea> for DbExecutor {
	type Result = Result<Vec<IdeaTmp>, Error>;

	fn handle(&mut self, msg: QueryIdea, ctx: &mut Self::Context) -> Self::Result {
		let conn = &self.0.get().unwrap();

		let aql = AqlQuery::new("FOR i in ideas LIMIT @limit RETURN i")
			.bind_var("limit", thread_rng().gen_range(1,25))
			.batch_size(100)
			.memory_limit(100000);

		let request : Vec<IdeaTmp> = conn.aql_query(aql).unwrap();


		Ok(request)
	}

}

//impl Handler<NewIdea> for DbExecutor {
//	type Result = ();
//
//	fn handle(&mut self, msg: NewIdea, ctx: &mut Self::Context) -> Self::Result {
//		unimplemented!()
//	}
//}
