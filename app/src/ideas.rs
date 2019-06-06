use crate::AppState;

use actix_web::http::Method;
use actix_web::{App, FutureResponse, HttpResponse, Query, State, Json};
use actix_web::{AsyncResponder, Path};
use futures::future::Future;
use ideadog::{QueryIdea, Sort, NewIdea};
use serde::Deserialize;
use futures;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.resource("/ideas", |r| {
		r.method(Method::GET).with(get_ideas);
	})
	   .resource("/ideas/{sort}", |r| {
		   r.method(Method::GET).with(get_ideas_sort);
	   })
	   .scope("/idea", |scope| {
		   scope
			   .resource("/", |r|{ r.method(Method::POST).with(create_idea) })
			   .resource("/{id}/", |r| {
			   r.method(Method::GET).with(get_idea_id);
//                r.method(Method::POST).with(post_idea);
		   })
	   })
}

#[derive(Deserialize,Debug)]
struct Param {
	id: Option<String>,
	tags: Option<String>,
}

#[derive(Deserialize)]
struct IdeaForm {
	text: String,
	owner_id: String,
	tags: Vec<String>
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

fn get_ideas_sort((path, _q_string, state): (Path<String>, Query<Param>, State<AppState>)) -> FutureResponse<HttpResponse> {
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

	if let Some(t) = q_string.tags.clone() {
		let tags : Vec<String> = t.split(",").map(|x| x.to_string()).collect();
		q.tags = Some(tags);

	}

	dbg!(q_string);

	run_query(q, state)
}

fn get_idea_id((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let q = QueryIdea {
		sort: Sort::ALL,
		id: Some(path.into_inner()),
		owner: None,
		owner_id: None,
		tags: None,
		limit: None,
	};

	run_query(q, state)
}

fn create_idea((idea, state): (Json<IdeaForm>, State<AppState>)) -> FutureResponse<HttpResponse> {
	println!("new time");
	let new = NewIdea {
		text: idea.text.clone(),
		owner_id: idea.owner_id.clone(),
		tags: idea.tags.clone()
	};
	state
		.database
		.send(new)
		.from_err()
		.and_then(|res| match res {
			Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
			Err(_) => Ok(HttpResponse::InternalServerError().into()),
		})
		.responder()
}
