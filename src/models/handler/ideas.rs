use crate::models::{Idea, NewIdea, QueryIdea};
use crate::DbExecutor;
use actix::Handler;
use arangors;
use r2d2_arangodb::ArangodbConnectionManager;
use serde_json::Value;
use arangors::AqlQuery;
use r2d2::Error;

impl Handler<QueryIdea> for DbExecutor {
	type Result = Result<Vec<Value>, Error>;

	fn handle(&mut self, msg: QueryIdea, ctx: &mut Self::Context) -> Self::Result {
		let conn = &self.0.get().unwrap();

		let aql = AqlQuery::new("FOR i in ideas LIMIT 5 RETURN i")
			.batch_size(100)
			.memory_limit(100000);

		let request : Vec<Value> = conn.aql_query(aql).unwrap();


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
