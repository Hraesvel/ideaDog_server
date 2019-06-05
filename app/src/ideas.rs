use actix_web::{App, HttpResponse, Query, State, FutureResponse};
use crate::AppState;
use actix_web::http::header::http_percent_encode;
use actix_web::http::Method;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.scope("/ideas", |scope| {
		scope
			.resource("/", |r| {
				r.method(Method::GET).with(get_ideas);
			})
	})
}

#[derive(Deserialize)]
pub struct Param {
	id: Option<String>,
	tags: Option<String>,
	tags_array: Option<Vec<String>>
}

fn get_ideas(
	(q_string, state): (Query<Param>, State<AppState>)
) -> FutureResponse<HttpResponse> {

}