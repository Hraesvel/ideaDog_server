use actix::Handler;
use arangors;
use arangors::AqlQuery;

use r2d2::Error;

use serde_json;

use crate::{DbExecutor, NewUser, QueryUser, User};

impl Handler<QueryUser> for DbExecutor {
	type Result = Result<Vec<User>, Error>;

	fn handle(&mut self, msg: QueryUser, _ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();

		let mut aql = AqlQuery::new("");

		if let Some(t) = msg.token {
			aql = AqlQuery::new(
				"FOR u in 1..1 OUTBOUND DOCUMENT('bearer_tokens', @ele) bearer_to_user RETURN u",
			)
				.bind_var("ele", t.clone())
				.batch_size(1);
		}

		let response: Vec<User> = match conn.aql_query(aql) {
			Ok(r) => r,
			Err(e) => {
				println!("Error: {}", e);
				vec![]
			}
		};

		Ok(response)
	}
}

impl Handler<NewUser> for DbExecutor {
	type Result = Result<Vec<User>, Error>;

	fn handle(&mut self, msg: NewUser, _ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();
		let data = serde_json::to_value(msg.clone()).unwrap();
		let aql = AqlQuery::new("INSERT @data INTO users LET n = NEW RETURN n")
			.bind_var("data", data)
			.batch_size(1);
		let response = match conn.aql_query(aql) {
			Ok(r) => r,
			Err(e) => {
				println!("Error: {}", e);
				vec![]
			}
		};

		Ok(response)
	}
}
