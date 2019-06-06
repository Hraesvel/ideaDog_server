use crate::models::{Idea, NewIdea, QueryIdea, Sort};
use crate::DbExecutor;
use actix::Handler;
use arangors;
use arangors::AqlQuery;
use r2d2::Error;
use r2d2_arangodb::ArangodbConnectionManager;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

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

//impl Handler<NewIdea> for DbExecutor {
//	type Result = ();
//
//	fn handle(&mut self, msg: NewIdea, ctx: &mut Self::Context) -> Self::Result {
//		unimplemented!()
//	}
//}
