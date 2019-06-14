use crate::{AppState, DbExecutor};
use actix_web::actix::{Handler, Message};
use actix_web::http::{Cookie, Method, StatusCode};
use actix_web::{App, Form, HttpRequest, HttpResponse, Json, Responder, Result, State};
use approveapi::*;
use arangors::AqlQuery;
use chrono::Utc;
use failure::Error;
use futures::future::Future;
use ideadog::{Challenge, Login, Signup};
use rand;
use rand::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::time::Duration;
use crate::views::users::{create_user, SignUp};

pub fn config(cgf: App<AppState>) -> App<AppState> {
    cgf.resource("/login", |r| {
        r.method(Method::POST).with(login);
    }).resource("/signup", |r|{
        r.method(Method::POST).with(create_user);
    } )
}

fn signup((form, state): (Json<Signup>, State<AppState>)) -> impl Responder {
    if exist_user(form.email.clone(), &state) {
        let response =  perform_approve_aip(form.email.clone(), state);
        return response.unwrap();
    }

    HttpResponse::BadRequest().finish()
}

fn login((form, state): (Json<Login>, State<AppState>)) -> impl Responder {
    // check if user via email exists.
    if !exist_user(form.email.clone(), &state) {
        return HttpResponse::build(StatusCode::TEMPORARY_REDIRECT).finish();
    }

    perform_approve_aip(form.email.clone(), state).unwrap()

    // create ttl with a 15 min offset
//    let ttl = ttl(15);
//    let c = challenge_gen(32);
//    let challenge = Challenge {
//        _id: format!("challenges/{}", c.clone()),
//        challenge: c,
//        email: form.email.clone(),
//        username: None,
//        pending: true,
//        ttl,
//    };
//
//    let r = state
//        .database
//        // added challenge to database
//        .send(challenge.clone())
//        .from_err()
//        .and_then(|res| match res {
//            Ok(_) => send_magic_link(challenge, state),
//            Err(_) => Ok(HttpResponse::Unauthorized().finish()),
//        })
//        .wait();
//
//    dbg!(r.unwrap())

    //	HttpResponse::build(StatusCode::OK)
    //		.content_type("text/html; charset")
    //		.body("Something is happening!")
}

pub(crate) fn perform_approve_aip(form: String, state: State<AppState>) -> Result<HttpResponse> {

    let ttl = ttl(15);
    let c = challenge_gen(32);
    let challenge = Challenge {
        _id: format!("challenges/{}", c.clone()),
        challenge: c,
        email: form.clone(),
        username: None,
        pending: true,
        ttl,
    };

    let r = state
        .database
        // added challenge to database
        .send(challenge.clone())
        .from_err()
        .and_then(|res| match res {
            Ok(_) => send_magic_link(challenge, state),
            Err(_) => Ok(HttpResponse::Unauthorized().finish()),
        })
        .wait();

    r
}

fn ttl(mins: i64) -> i64 {
    let ttl = (mins * 60000) + Utc::now().timestamp_millis();
    ttl
}

fn challenge_gen(size: usize) -> String {
    let mut nonce = vec![0u8; size];
    OsRng::new().unwrap().fill_bytes(&mut nonce);
    base64::encode_config(&nonce, base64::URL_SAFE)
}

pub(crate) fn exist_user(email: String, state: &State<AppState>) -> bool {
    let response = state
        .database
        .send(Login { email })
        .and_then(|res| match &res {
            Ok(v) if !v.is_empty() => Ok(true),
            _ => Ok(false),
        })
        .wait();

    response.unwrap()
}

fn send_magic_link(challenge: Challenge, state: State<AppState>) -> Result<HttpResponse> {
    let client = approveapi::create_client(
        env::var("APPROVEAPI_TEST_KEY").expect("APPROVEAPI_TEST_KEY must be set!"),
    );

    let mut prompt_request = CreatePromptRequest::new(
        challenge.email.clone(),
        r#"Click the link below to Sign in to your account.
		This link will expire in 15 mintues."#
            .to_string(),
    );
    prompt_request.title = Some("Magic sign-in link".to_string());
    prompt_request.approve_text = Some("Accept".to_string());
    prompt_request.reject_text = Some("Reject".to_string());
//    prompt_request.expires_in = Some(900.0);
    prompt_request.long_poll = Some(true);

    match client.create_prompt(prompt_request).sync() {
        Ok(prompt) => {
            if let Some(answer) = prompt.answer {
                if answer.result {
                    println!("welcome to ideaDog!");
                    //					return Ok(HttpResponse::build(StatusCode::OK).finish());
                    return set_login(challenge.challenge, state);
                } else {
                    println!("Request Rejected");
                    return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED).finish());
                }
            } else {
                println!("No response from user");
                return Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT).finish());
            }
        }
        Err(e) => {
            println!("ApproveAPI->create_prompt error: {:?}", e);
            return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).finish());
        }
    }
}

fn set_login(challenge: String, state: State<AppState>) -> Result<HttpResponse> {
    let res = state
        .database
        .send(Pending(challenge))
        .from_err()
        .and_then(|res| match res {
            Ok(v) => {
                let cookie = Cookie::new("bearer", v.clone().unwrap().token);
                return Ok(HttpResponse::Ok()
                    .cookie(cookie)
                    .json(serde_json::to_value(&v.unwrap()).unwrap())
                    );
            }

            _ => {
                return Ok(HttpResponse::BadRequest()
                    .content_type("text/html; charset=utf-8")
                    .body("bad show my boy"))
            }
        })
        .wait();

    res
}

struct Pending(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bearer {
    #[serde(alias = "_key")]
    #[serde(rename(serialize = "_key"))]
    pub token: String,
    pub ttl: i64,
}

impl Message for Pending {
    type Result = Result<Option<Bearer>, Error>;
}

impl Handler<Pending> for DbExecutor {
    type Result = Result<Option<Bearer>, Error>;

    fn handle(&mut self, msg: Pending, ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();

        let bearer = Bearer {
            token: challenge_gen(64),
            ttl: ttl(43800),
        };

        let aql = AqlQuery::new(
            "FOR c IN challenges \
			FILTER c._key == @challenge
			RETURN c
		",
        )
            .bind_var("challenge", msg.0.clone())
            .batch_size(1);

        let mut challenge: Vec<Challenge> = match &conn.aql_query(aql) {
            Ok(r) if !r.is_empty() => r.clone(),
            _ => vec![],
        };

        let challenge = challenge.pop().unwrap();

        let mut aql = AqlQuery::new("RETURN {_key: 'false', ttl: 0}");
        if challenge.pending == true && challenge.ttl > Utc::now().timestamp_millis() {
            aql = AqlQuery::new(
                "
			LET usr = FIRST (FOR u IN users FILTER u.email == @email RETURN u)
			INSERT @data INTO bearer_tokens let t = NEW
			INSERT {_from: t._id, _to: usr._id} INTO bearer_to_user LET b = NEW
			RETURN t",
            )
                .bind_var("email", challenge.email)
                .bind_var("data", serde_json::to_value(bearer).unwrap())
                .batch_size(1);
        };

        let mut r: Vec<Bearer> = match &conn.aql_query(aql) {
            Ok(r) if !r.is_empty() => r.clone(),
            _ => vec![],
        };

        let response = Some(r.pop().unwrap());

        Ok(response)
    }
}
