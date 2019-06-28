use crate::AppState;

use actix_web::http::Method;
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use actix_web::{AsyncResponder, Path};
use futures::future::Future;
use ideadog::{QueryTag, Sort, Tag};

pub fn config(cfg: App<AppState>) -> App<AppState> {
    cfg.scope("/tags", |scope| {
        scope
            .resource("/", |resp| resp.method(Method::GET).with(get_tags))
            .resource("/{id}", |resp| resp.method(Method::GET).with(get_one_tag))
            .resource("/{id}/ideas", |resp| {
                resp.method(Method::GET).with(get_associations)
            })
    })
}

fn run_query(qufig: QueryTag, state: State<AppState>) -> FutureResponse<HttpResponse> {
    state
        .database
        .send(qufig)
        .from_err()
        .and_then(|res| match res {
            Ok(value) => Ok(HttpResponse::Ok().json(value)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

fn get_tags(state: State<AppState>) -> FutureResponse<HttpResponse> {
    let q_tag = QueryTag {
        id: None,
        with_ideas: false,
        sort: Sort::ALL,
    };

    run_query(q_tag, state)
}

fn get_one_tag((path, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let q_tags = QueryTag {
        id: Some(vec![path.into_inner()]),
        with_ideas: false,
        sort: Sort::ALL,
    };

    run_query(q_tags, state)
}

fn get_associations(
    (path, state): (Path<String>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    let vec_of_tags: Vec<String> = path
        .into_inner()
        .split(',')
        .map(|x| x.to_string())
        .collect();
    let q_tags = QueryTag {
        id: Some(vec_of_tags),
        with_ideas: true,
        sort: Sort::ALL,
    };
    run_query(q_tags, state)
}
