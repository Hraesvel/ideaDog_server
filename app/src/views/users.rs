use crate::AppState;

use crate::midware::AuthMiddleware;
use crate::views::auth::{exist_user, perform_approve_aip};
use actix_web::http::{Method, NormalizePath, StatusCode};
use actix_web::{App, FutureResponse, HttpResponse, Responder, State};
use actix_web::{AsyncResponder, HttpRequest, Json};
use chrono::Utc;
use futures::future::{ok, Future, IntoFuture};
use ideadog::{NewUser, QueryUser};
use serde::Deserialize;

pub fn config(cfg: App<AppState>) -> App<AppState> {
    cfg.scope("/user", |scope| {
        scope
            .resource("/", |r| {
                r.middleware(AuthMiddleware);
                r.method(Method::GET).with(get_user);
            })
            .default_resource(|r| {
                r.h(NormalizePath::new(
                    true,
                    true,
                    StatusCode::TEMPORARY_REDIRECT,
                ));
                r.method(Method::POST).with(create_user);
            })
        //		     .resource("/", |r| {
        //			     r.method(Method::POST).with(create_user);
        //		     })
    })
}

#[derive(Deserialize, Debug)]
pub(crate) struct SignUp {
    pub username: String,
    pub email: String,
}

fn run_query(qufigs: QueryUser, state: State<AppState>) -> FutureResponse<HttpResponse> {
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

fn get_user((req, state): (HttpRequest<AppState>, State<AppState>)) -> impl Responder {
    let tok = req
        .headers()
        .get("AUTHORIZATION")
        .map(|value| value.to_str().ok())
        .unwrap();
    //    dbg!(tok);

    let mut token = tok
        .unwrap()
        .split(" ")
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .pop()
        .unwrap();

    //    HttpResponse::Ok().finish();

    let qufig = QueryUser { token: Some(token) };

    run_query(qufig, state)
}

pub(crate) fn create_user((json, state): (Json<SignUp>, State<AppState>)) -> impl Responder {
    if exist_user(json.email.clone(), &state) {
        let response = perform_approve_aip(json.email.clone(), state);
        return response;
    };

    let new_user = NewUser {
        username: json.username.clone(),
        email: json.email.clone(),
        created_at: Utc::now().timestamp_millis(),
        active: false,
        ..NewUser::default()
    };

    let response = state
        .database
        .send(new_user)
//        .from_err()
        .and_then(|res| match res {
            Ok(ideas) => Ok(HttpResponse::Ok().json(ideas)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .wait();

    perform_approve_aip(json.email.clone(), state)
}
