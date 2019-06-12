use rand;
use approveapi::CreatePromptRequest;
use actix_web::actix::{Handler, Message};
use crate::{DbExecutor, AppState};
use actix_web::{App, Result, Responder, Form, State, HttpResponse, HttpRequest, Json};
use actix_web::http::{StatusCode, Method};
use rand::OsRng;
use rand::RngCore;
use chrono::{Utc};
use std::time::Duration;
use futures::future::Future;
use arangors::AqlQuery;
use std::env;
use serde::{Deserialize, Serialize};
use failure::Error;
use ideadog::{Challenge, Login, Signup};

//#[derive(Deserialize, Debug)]
//struct Login { email: String }
//
//#[derive(Deserialize, Debug)]
//struct Signup {
//	pub email: String,
//	pub username: String
//}
//
//#[derive(Serialize, Clone, Debug)]
//pub struct Challenge {
//	pub _id: String,
//	#[serde(alias = "_key")]
//	pub challenge: String,
//	pub email: String,
//	pub username: Option<String>,
//	pub pending: bool,
//	pub ttl: i64
//}

pub fn config(cgf: App<AppState>) -> App<AppState> {
	cgf.resource("/login", |r| {
		r.method(Method::POST).with(login);
	})
}

fn login((form, state): (Json<Login>, State<AppState>)) -> impl Responder {
	// check if user via email exists.
	if !exist_user(form.email.clone(), &state) {
		return HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
			.finish();
	}

	// create ttl with a 15 min offset
	let ttl = (15 * 60000) + Utc::now().timestamp_millis();
	let c = challenge();
	let challenge = Challenge {
		_id: format!("challenges/{}", c.clone()),
		challenge: c,
		email: form.email.clone(),
		username: None,
		pending: true,
		ttl
	};

	dbg!(&challenge);

//	let r = state
//		.database
//		// added challenge to database
//		.send(challenge.clone())
////		.from_err()
//		.and_then(|res| {
//			match res {
////				Ok(_) => send_magic_link(challenge, state),
//				Ok(_) => Ok("We got a challenger!".to_string()),
//				Err(_) => Ok("bad joo joo!".to_string()),
//			}
//		}).wait();


	HttpResponse::build(StatusCode::OK)
		.content_type("text/html; charset")
		.body("Something is happening!")
}

fn challenge() -> String {
	let mut nonce = vec![0u8; 32];
	OsRng::new().unwrap().fill_bytes(&mut nonce);
	base64::encode_config(&nonce, base64::URL_SAFE)
}

fn exist_user(email: String, state: &State<AppState>) -> bool {
	let response = state
		.database
		.send(Login { email })
		.and_then(|res| match &res {
			Ok(v) if !v.is_empty() => Ok(true),
			_ => Ok(false)
		}).wait();

	response.unwrap()
}

//impl Message for Login {
//	type Result = Result<Vec<bool>, Error>;
//}

//impl Handler<Login> for DbExecutor {
//	type Result = Result<Vec<bool>, Error>;
//
//	fn handle(&mut self, msg: Login, ctx: &mut Self::Context) -> Self::Result {
//		let conn = self.0.get().unwrap();
//		let aql = AqlQuery::new(
//			"for u in users
//		filter u.email == @email
//		return IS_DOCUMENT(u)
//			")
//			.bind_var("email", msg.email.clone())
//			.batch_size(1);
//
//		let res = conn.aql_query(aql).unwrap();
//
//		Ok(res)
//	}
//}


//impl Message for Challenge {
//	type Result = Result<String, Error>;
//}
//
//impl Handler<Challenge> for DbExecutor {
//	type Result = Result<String, Error>;
//
//	fn handle(&mut self, msg: Challenge, ctx: &mut Self::Context) -> Self::Result {
//		let conn = self.0.get().unwrap();
//		let data = serde_json::to_value(msg).unwrap();
//
//		let aql = AqlQuery::new(
//			"INSERT @data INTO challenges
//			").bind_var("data", data)
//		      .batch_size(1);
//
//		let response = conn.aql_query(aql);
//
//
////		response
//
//		Ok("woof".to_string())
//	}
//}



//fn send_magic_link(challenge: Challenge, state: State<AppState>) ->  Result<HttpResponse>{
//	let client =
//		approveapi::create_client(env::var("APPROVEAPI_TEST_KEY")).unwrap();
//
//	let mut prompt_request = CreatePromptRequest::new(
//		challenge.email.clone(),
//		r#"Click the link below to Sign in to your account.
//		This link will expire in 15 mintues."#
//			.to_string()
//	);
//	prompt_request.title = Some("Magic sign-in link".to_string());
//	prompt_request.approve_text = Some("Accept".to_string());
//	prompt_request.reject_text = Some("Reject".to_string());
//	prompt_request.long_poll = Some(true);
//
//	match client.create_prompt(prompt_request).sync() {
//		Ok(prompt) => {
//			if let Some(answer) = prompt.answer {
//				if answer.result {
//					return set_login(challenge.challenge, state);
//				} else {
//					println!("Request Rejected");
//					return Ok(HttpResponse::build(StatusCode::UNAUTHORIZED)
//						.finish())
//				}
//			} else {
//				println!("No response from user");
//				return Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
//					.finish());
//			}
//		},
//		Err(e) => {
//			println!("ApproveAPI->create_prompt error: {:?}", e);
//			return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).finish())
//		}
//	}
//}

//fn set_login(challenge: String, state: State<AppState> ) -> Result<HttpResponse>{
//	state
//		.database
//		.send(Pending(challenge))
//}

//struct Pending(String);
//
//impl Message for Pending {
//	type Result = Result<String, ()>;
//}
//
//impl Handler<Pending> for DbExecutor {
//	type Result = Result<String, ()>;
//
//	fn handle(&mut self, msg: Pending, ctx: &mut Self::Context) -> Self::Result {
//		let conn = self.0.get().unwrap();
//
//		let aql = AqlQuery::new(
//			"FOR c IN challenges \
//			FILTER c._key == @challenge
//			RETURN c
//		").bind_var("challenge", msg.0.clone())
//			.batch_size(1);
//
//		let bearer = challenge();
//
//		let challenge : Challenge = match conn.aql_query(aql) {
//			Ok(r) if !r.is_empty() => r[0].clone(),
//			_ => vec![]
//		};
//
//		if challenge.pending == true && challenge.ttl < Utc::now().timestamp_millis() {
//			let aql = AqlQuery::new("")
//		}
//
//		Ok(format!("woof"))
//	}
//}
