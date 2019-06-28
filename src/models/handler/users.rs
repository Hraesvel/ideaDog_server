use actix::{Handler, MailboxError};
use arangors;
use arangors::AqlQuery;

use r2d2::Error;

use serde_json;

use crate::{DbExecutor, NewUser, QUser, QueryUser, ServiceError, User};
use std::collections::HashMap;

impl Handler<QueryUser> for DbExecutor {
    type Result = Result<User, MailboxError>;

    fn handle(&mut self, msg: QueryUser, _ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();

        let mut aql = AqlQuery::new("");

        if let Some(t) = msg.token {
            aql = AqlQuery::new(
"
let u = FIRST (FOR u in 1..1 OUTBOUND DOCUMENT('bearer_tokens', @ele) bearer_to_user RETURN u)
let votes = (FOR v, e in 1..1 INBOUND u._id idea_voter RETURN {[v._key]: e.vote })
return Merge(u, {votes: MERGE(votes)})
",
            )
            .bind_var("ele", t.clone())
            .batch_size(1);
        }

        let response: Result<User, MailboxError> = match conn.aql_query(aql) {
            Ok(mut r) => {
                if !r.is_empty() {
                    Ok(r.pop().unwrap())
                } else {
                    Err(MailboxError::Closed)
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                Err(MailboxError::Closed)
            }
        };

        response
    }
}

impl Handler<QUser> for DbExecutor {
    type Result = Result<User, MailboxError>;

    fn handle(&mut self, msg: QUser, _ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();

        let mut aql = AqlQuery::new("");

        match msg {
            QUser::TOKEN(tok) => {
                aql = AqlQuery::new(
"
let u = FIRST (FOR u in 1..1 OUTBOUND DOCUMENT('bearer_tokens', @ele) bearer_to_user RETURN u)
let votes = (FOR v, e in 1..1 INBOUND u._id idea_voter RETURN {[v._key]: e.vote })
return Merge(u, {votes: MERGE(votes)})
",
                )
                .bind_var("ele", tok.clone())
                .batch_size(1);
            }
            QUser::ID(id) => {
                aql = AqlQuery::new("RETURN DOCUMENT('users', @ele)")
                    .bind_var("ele", id.clone())
                    .batch_size(1);
            }
        }

        let response: Result<User, MailboxError> = match conn.aql_query(aql) {
            Ok(mut r) => {
                let res = if !r.is_empty() {
                    let mut user: User = r.pop().unwrap();
                    if user.votes.is_some() {
                        for v in user.clone().votes.unwrap().values() {
                            match v.as_ref() {
                                "upvote" => user.upvotes += 1,
                                "downvote" => user.downvotes += 1,
                                _ => {}
                            }
                        }
                    }
                    Ok(user)
                }else {
                    Err(MailboxError::Closed)
                };

                res
                },
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(MailboxError::Closed)
            }
        };

        response
    }
}

impl Handler<NewUser> for DbExecutor {
    type Result = Result<Vec<User>, Error>;

    fn handle(&mut self, msg: NewUser, _ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();
        let data = serde_json::to_value(msg.clone()).unwrap();
        let aql = AqlQuery::new("INSERT @data INTO users LET n = NEW RETURN n")
            .bind_var("data", data)
            .batch_size(1);
        let response = match conn.aql_query(aql) {
            Ok(r) => r,
            Err(e) => {
                println!("Error: {}", e);
                vec![]
            }
        };

        Ok(response)
    }
}
