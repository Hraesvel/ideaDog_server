use crate::views::users::{create_user};
use crate::{midware::ServiceError, AppState, DbExecutor};
use actix::MailboxError;
use actix_web::actix;
use actix_web::actix::{Handler, Message};
use actix_web::http::{Cookie, Method, StatusCode};
use actix_web::{
    App, AsyncResponder, Form, FutureResponse, HttpRequest, HttpResponse, Json, Responder, Result,
    State,
};

use arangors::AqlQuery;
use chrono::Utc;

use futures::future::Future;
use ideadog::{Challenge, Login};
use rand;
use rand::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json;



#[derive(Deserialize, Debug, Serialize)]
pub struct Token {
    pub token: String,
}

/// Configs for the auth routes.
pub fn config(cgf: App<AppState>) -> App<AppState> {
    cgf.resource("/login", |r| {
        r.method(Method::POST).with(login);
    })
    .resource("/signup", |r| {
        r.method(Method::POST).with(create_user);
    })
    .resource("/validate_login", |r| {
        r.method(Method::POST).with(set_login);
    })
}

pub(crate) fn login((json, state): (Json<Login>, State<AppState>)) -> HttpResponse {
    // is user doesn't exist
    if exist_user(json.email.clone(), &state).is_err() {
        return HttpResponse::build(StatusCode::TEMPORARY_REDIRECT).finish();
    }

    let chall = Token {
        token: challenge_gen(32),
    };
    let challenge = Challenge {
        challenge: chall.token.clone(),
        email: json.email.clone(),
        username: None,
        pending: true,
        ttl: ttl(15),
    };

    let response = state
        .database
        .send(challenge)
        //        .from_err()
        .and_then(|res| match res {
            Ok(_) => Ok(HttpResponse::Ok().json(chall)),
            Err(_) => Ok(HttpResponse::build(StatusCode::BAD_REQUEST).json("badrequest")),
        })
        .wait();

    response.unwrap()
}

/// Generates a UTC timestamp in milliseconds +/- mins given.
pub(crate) fn ttl(mins: i64) -> i64 {
    let ttl = (mins * 60000) + Utc::now().timestamp_millis();
    ttl
}

/// Generate a `String` token of a given size
///
/// # Note
/// recommendation anything above 32 bytes
///
pub(crate) fn challenge_gen(size: usize) -> String {
    let mut nonce = vec![0u8; size];
    OsRng::new().unwrap().fill_bytes(&mut nonce);
    base64::encode_config(&nonce, base64::URL_SAFE)
}

/// Check is a user (by email) exists in the Database registry
pub(crate) fn exist_user(email: String, state: &State<AppState>) -> Result<(), MailboxError> {
    let response = state
        .database
        .send(Login { email })
        .and_then(|res| match &res {
            Ok(v) if !v.is_empty() => Ok(()),
            _ => Err(MailboxError::Timeout),
        })
        .wait();

    response
}

#[derive(Deserialize, Debug)]
struct Pending {
	token: String,

}

/// This function will take a `Challenge` token from the user and compare to the one stored in the Database.
/// if the `Challenge` match one in storage and is valid (not expired or exists) then a 'Bearer token' will be sent back to the user.
fn set_login((challenge, state): (Json<Pending>, State<AppState>)) -> FutureResponse<HttpResponse> {
    dbg!(&challenge);
    let res = state
        .database
        .send(challenge.into_inner())
        .from_err()
        .and_then(|res| match res {
            Ok(v) => {
                let cookie = Cookie::build("bearer", v.clone().unwrap().token)
                    .http_only(true)
                    .finish();

                return Ok(HttpResponse::Ok()
                    .cookie(cookie)
                    .json(serde_json::to_value(&v.unwrap()).unwrap()));
            }

            _ => {
                return Ok(HttpResponse::BadRequest()
                    .content_type("text/html; charset=utf-8")
                    .body("bad show my boy"))
            }
        })
        .responder();

    res
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bearer {
    #[serde(alias = "_key")]
    #[serde(rename(serialize = "_key"))]
    pub token: String,
    pub ttl: i64,
}

/// Message for Pending
impl Message for Pending {
    type Result = Result<Option<Bearer>, ServiceError>;
}

impl Handler<Pending> for DbExecutor {
    type Result = Result<Option<Bearer>, ServiceError>;

    /// Handles the process for validating a `Challenge` token
    fn handle(&mut self, msg: Pending, _ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();

        println!("handle");

        let bearer = Bearer {
            token: challenge_gen(64),
            ttl: ttl(43800),
        };

        // check if the Challenge token exists and then Delete since it should be a one time use.
        let aql = AqlQuery::new(
            "FOR c IN challenges \
			FILTER c._key == @challenge
            REMOVE c in challenges let ch = OLD
			RETURN ch
		",
        )
        .bind_var("challenge", msg.token.clone())
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
