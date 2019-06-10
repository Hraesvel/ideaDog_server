use actix::Handler;
use arangors;
use arangors::AqlQuery;
use chrono::Utc;
use r2d2::Error;
use serde::{Serialize, Deserialize};
use serde_json;

use crate::{QueryUser, NewUser, User, DbExecutor};


impl Handler<QueryUser> for DbExecutor {
	type Result = Result<Vec<User>, Error>;

	fn handle(&mut self, msg: QueryUser, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();

		// TODO handle querying more then one user
		let mut aql = AqlQuery::new("");

		if let Some(key) = msg.id {
			aql = AqlQuery::new("RETURN DOCUMENT('users', @ele)")
				.bind_var("ele", key.clone())
				.batch_size(1);
		};
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

	fn handle(&mut self, msg: NewUser, ctx: &mut Self::Context) -> Self::Result {
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
