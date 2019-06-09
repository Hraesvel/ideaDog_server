use actix::Handler;
use arangors;
use arangors::AqlQuery;
use chrono::Utc;
use r2d2::Error;
use serde::{Serialize, Deserialize};

use crate::{QueryUser, NewUser, User, DbExecutor};


impl Handler<QueryUser> for DbExecutor {
	type Result = Result<Vec<User>, Error>;

	fn handle(&mut self, msg: QueryUser, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();
		dbg!(&msg);

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

		dbg!(&response);
		Ok(response)
	}
}
