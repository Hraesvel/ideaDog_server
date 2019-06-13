use actix::Handler;
use arangors;
use arangors::AqlQuery;
use r2d2::Error;

use crate::models::{QueryTag, Tag};
use crate::DbExecutor;

impl Handler<QueryTag> for DbExecutor {
	type Result = Result<Vec<Tag>, Error>;

	fn handle(&mut self, msg: QueryTag, _ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();
		let mut aql = AqlQuery::new("FOR tag IN tags RETURN tag").batch_size(1);

		if msg.with_ideas && msg.id.is_some() {
			aql = AqlQuery::new(
				"for tag in @user_input
			let t = DOCUMENT(CONCAT('tags/', tag))
			RETURN {
                _key: t._key,
				_id: t._id,
				count: t.count,
                ideas: (
                    for v in 1..1 OUTBOUND t._id tag_to_idea
                    RETURN v
                    )
				}",
			)
				.bind_var("user_input", msg.id.unwrap().clone())
				.batch_size(50);
		} else {
			if let Some(key) = msg.id {
				aql = AqlQuery::new("RETURN DOCUMENT(CONCAT('tags/', @key))")
					.bind_var("key", key[0].clone())
					.batch_size(1);
			}
		}

		let response: Vec<Tag> = match conn.aql_query(aql) {
			Ok(r) => r,
			Err(e) => {
				println!("{}", e);
				vec![]
			}
		};
		//		dbg!(&response);
		Ok(response)
	}
}
