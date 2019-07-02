use crate::AppState;
use actix_web::{App, FutureResponse, HttpResponse, Json, State, AsyncResponder};
use actix_web::actix::{Message, Handler, MailboxError};
use ideadog::{Idea, ServiceError, DbExecutor};
use actix_web::http::Method;
use futures::future::Future;
use actix_web::actix::dev::Mailbox;
use arangors::AqlQuery;
use toml::map::VacantEntry;
use serde::Deserialize;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.resource("/search", |r| {
		r.method(Method::POST).with(search_for);
	})
}

#[derive(Deserialize)]
struct Input{
	query: String
}

struct QSearch(String);

fn search_for((input, state): (Json<Input>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let qufig = QSearch(input.query.clone());
	state
		.database
		.send(qufig)
		.from_err()
		.and_then(|res| match res {
			Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
			Err(_) => Err(ServiceError::NotFound.into())
		})
		.responder()
}

impl Message for QSearch {
	type Result = Result<Vec<Idea>, MailboxError>;
}

impl Handler<QSearch> for DbExecutor {
	type Result = Result<Vec<Idea>, MailboxError>;

	fn handle(&mut self, msg: QSearch, ctx: &mut Self::Context) -> Self::Result {
		let conn = self.0.get().unwrap();

		let aql = AqlQuery::new(
"
FOR doc in idea_search
SEARCH ANALYZER(doc.text IN TOKENS(@query, 'text_en'), 'text_en')
SORT TFIDF(doc) DESC
RETURN doc
"
		)
			.bind_var("query", msg.0)
			.batch_size(50);

		let response :Result<Vec<Idea>, MailboxError> = match conn.aql_query(aql) {
			Ok(ideas) => Ok(ideas),
			Err(e) => {
				eprintln!("Error: {}", e);
				Err(MailboxError::Closed)
			}
		};

		response
	}
}