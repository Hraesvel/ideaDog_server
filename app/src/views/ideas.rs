use crate::AppState;

use actix_web::http::{Method, NormalizePath, StatusCode};
use actix_web::{App, FutureResponse, HttpResponse, Json, Query, State};
use actix_web::{AsyncResponder, Path};
use futures;
use futures::future::Future;
use ideadog::{NewIdea, QueryIdea, Sort};
use crate::AuthMiddleware;
use serde::Deserialize;

pub fn config(cfg: App<AppState>) -> App<AppState> {
	cfg.resource("/ideas", |r| {
		r.method(Method::GET).with(get_ideas);
	})
	   .resource("/ideas/{sort}", |r| {
		   r.method(Method::GET).with(get_ideas_sort);
	   })
	   .scope("/idea", |scope| {
		   scope
			   .default_resource(|r| {
				   r.h(NormalizePath::new(
					   true,
					   true,
					   StatusCode::TEMPORARY_REDIRECT,
				   ))
			   })
			   .resource("/", |r| {
				   r.middleware(AuthMiddleware);
				   r.method(Method::POST).with(create_idea);
			   })
			   .resource("/{id}/", |r| {
				   r.method(Method::GET).with(get_idea_id);
			   })
	   })
}

#[derive(Deserialize, Debug)]
struct Param {
	id: Option<String>,
	tags: Option<String>,
}

#[derive(Deserialize)]
struct IdeaForm {
	text: String,
	owner_id: String,
	tags: Vec<String>,
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

fn get_ideas_sort(
	(path, q_string, state): (Path<String>, Query<Param>, State<AppState>),
) -> FutureResponse<HttpResponse> {
	let mut vec_of_tags = None;
	if let Some(value) = &q_string.tags {
		let v_string: Vec<String> = value.clone().split(',').map(|x| x.to_string()).collect();
		vec_of_tags = Some(v_string);
	};

	let mut q = QueryIdea {
		sort: Sort::ALL,
		id: None,
		owner: None,
		owner_id: None,
		tags: vec_of_tags,
		limit: None,
	};

	match path.into_inner().to_lowercase().as_str() {
		"bright" => q.sort = Sort::BRIGHT,
		_ => {}
	}

	run_query(q, state)
}

fn get_ideas((q_string, state): (Query<Param>, State<AppState>)) -> FutureResponse<HttpResponse> {
	let mut vec_of_tags = None;
	if let Some(value) = &q_string.tags {
		let v_string: Vec<String> = value.clone().split(',').map(|x| x.to_string()).collect();
		vec_of_tags = Some(v_string);
	};

	let mut q = QueryIdea {
		sort: Sort::ALL,
		id: None,
		owner: None,
		owner_id: None,
		tags: vec_of_tags,
		limit: None,
	};

	if let Some(t) = q_string.tags.clone() {
		let tags: Vec<String> = t.split(",").map(|x| x.to_string()).collect();
		q.tags = Some(tags);
	}

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
	let new = NewIdea {
		text: idea.text.clone(),
		owner_id: idea.owner_id.clone(),
		tags: idea.tags.clone(),
	};

	if idea.text.len() > 140 {
		let error = HttpResponse::build(StatusCode::from_u16(422).unwrap())
			.reason("Text length supplied is greater than allowed.")
			.finish();
		return Box::new(futures::future::ok(error.into()));
	}

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

#[cfg(test)]
mod test {
	use crate::views::ideas::{create_idea, get_idea_id, get_ideas, get_ideas_sort};
	use crate::AppState;
	use actix_web::actix::SyncArbiter;
	use actix_web::http::Method;
	use actix_web::test::TestServer;
	use ideadog::DbExecutor;
	use r2d2_arangodb::{ArangodbConnectionManager, ConnectionOptions};

	use std::env;

	fn build_test_server() -> TestServer {
		let _ = dotenv::dotenv();
		let mut srv = TestServer::build_with_state(|| {
			let adder = SyncArbiter::start(3, || {
				// arangodb connection configurations.
				let arango_config = ConnectionOptions::builder()
					.with_auth_jwt(
						env::var("DB_ACCOUNT").expect("DB_ACCOUNT must be set."),
						env::var("DB_PASSWORD").expect("DB_PASSWORD must be set."),
					)
					.with_host(
						env::var("DB_HOST").expect("DB_HOST must be set"),
						env::var("DB_PORT")
							.expect("DB_PORT must be set")
							.parse()
							.expect("DB_PORT must be digits"),
					)
					.with_db(env::var("DB_NAME").expect("DB_NAME must be set."))
					.build();
				let manager = ArangodbConnectionManager::new(arango_config);

				let pool = r2d2::Pool::builder()
					.build(manager)
					.expect("Failed to create pool");

				DbExecutor(pool.clone())
			});
			AppState {
				database: adder.clone(),
			}
		})
			.start(|app| {
				app.resource("/ideas", |r| {
					r.method(Method::GET).with(get_ideas);
				})
				   .resource("/ideas/{sort}", |r| {
					   r.method(Method::GET).with(get_ideas_sort);
				   })
				   .resource("/idea", |r| r.method(Method::POST).with(create_idea))
				   .resource("/idea/{id}/", |r| {
					   r.method(Method::GET).with(get_idea_id);
				   });
			});

		srv
	}

	#[test]
	fn test_get_all_ideas() {
		let mut srv = build_test_server();

		let all_ideas = srv.client(Method::GET, "/ideas").finish().unwrap();
		let response = srv.execute(all_ideas.send()).unwrap();

		dbg!(&response);
		assert!(response.status().is_success());
	}

	#[test]
	fn test_get_sorted_ideas() {
		let mut srv = build_test_server();

		let sort_ideas = srv.client(Method::GET, "/ideas/bright").finish().unwrap();
		let response = srv.execute(sort_ideas.send()).unwrap();
		assert!(response.status().is_success());
	}
}
