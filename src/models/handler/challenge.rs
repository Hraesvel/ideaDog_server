use crate::models::challenge::*;
use crate::DbExecutor;
use actix_web::actix::Handler;
use arangors;
use arangors::AqlQuery;
use failure::Error;

impl Handler<Login> for DbExecutor {
	type Result = Result<Vec<bool>, Error>;

	fn handle(&mut self, msg: Login, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();
		let aql = AqlQuery::new(
			"for u in users
		filter u.email == @email
		return IS_DOCUMENT(u)
			",
		)
			.bind_var("email", msg.email.clone())
			.batch_size(1);

		let res = conn.aql_query(aql).unwrap();
		dbg!(&res);
		Ok(res)
	}
}

impl Handler<Challenge> for DbExecutor {
	type Result = Result<String, Error>;

	fn handle(&mut self, msg: Challenge, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();
		let data = serde_json::to_value(msg).unwrap();

		let s = format!(
			"INSERT {data} INTO challenges LET n = NEW RETURN n",
			data = data
		);
		let aql = AqlQuery::new(&s).batch_size(1);

		let response: Option<Challenge> = match conn.aql_query(aql) {
			Ok(mut r) => Some(r.pop().unwrap()),
			Err(e) => None,
		};

		Ok(response.unwrap().challenge)
	}
}
