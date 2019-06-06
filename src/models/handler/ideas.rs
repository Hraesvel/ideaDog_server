use crate::models::{NewIdea, QueryIdea, Idea};
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


impl Handler<QueryIdea> for DbExecutor {
	type Result = Result<Vec<Idea>, Error>;

	fn handle(&mut self, msg: QueryIdea, ctx: &mut Self::Context) -> Self::Result {
		let conn = &self.0.get().unwrap();

		let aql = AqlQuery::new("FOR i in ideas RETURN i")
			.batch_size(25);

		if let Some(id) = msg.id {
			aql = AqlQuery::new("RETURN DOCUMENT(CONCAT('ideas/' @id ))")
				.bind_var("id", id)
				.batch_size(1);
		}

		let request: Vec<Idea> = match conn.aql_query(aql) {
			Ok(r) => r,
			Err(_) => vec!(),
		}


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
