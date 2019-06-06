use crate::AppState;
use actix_web::http::header::http_percent_encode;
use actix_web::http::Method;
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use actix_web::{AsyncResponder, Path};
use futures::future::Future;
use ideadog::{QueryIdea, Sort};
use serde::Deserialize;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.resource("/ideas", |r| {
		r.method(Method::GET).with(get_ideas);
	})
	   .resource("/ideas/{sort}/", |r| {
		   r.method(Method::GET).with(get_ideas_sort);
	   })
	   .scope("/idea", |scope| {
		   scope.resource("/{id}/", |r| {
			   r.method(Method::GET).with(get_idea_id);
//                r.method(Method::POST).with(post_idea);
		   })
	   })
}

#[derive(Deserialize)]
pub struct Param {
	id: Option<String>,
	tags: Option<String>,
	tags_array: Option<Vec<String>>,
}

fn get_ideas_sort((path, q_string, state): (Path<String>, Query<Param>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let mut q = QueryIdea {
		sort: Sort::ALL,
		id: None,
		owner: None,
		owner_id: None,
		tags: None,
		limit: None,
	};

	match path.into_inner().to_lowercase().as_str() {
		"bright" => { q.sort = Sort::BRIGHT },
		_ => {}
	}

	run_query(q, state)
}

fn get_ideas((q_string, state): (Query<Param>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let mut q = QueryIdea {
		sort: Sort::ALL,
		id: None,
		owner: None,
		owner_id: None,
		tags: None,
		limit: None,
	};

	run_query(q, state)
}

fn run_query(qufigs: QueryIdea, state: State<AppState>) -> FutureResponse<HttpResponse> {
	state
		.database
		.send(qufigs)
		.from_err()
		.and_then(|res| match res {
			Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
			Err(_) => Ok(HttpResponse::InternalServerError().into()),
		})
		.responder()
}

fn get_idea_id((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let mut q = QueryIdea {
		sort: Sort::ALL,
		id: Some(path.into_inner()),
		owner: None,
		owner_id: None,
		tags: None,
		limit: None,
	};

	state
		.database
		.send(q)
		.from_err()
		.and_then(|res| match res {
			Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
			Err(_) => Ok(HttpResponse::InternalServerError().into()),
		})
		.responder()
}
