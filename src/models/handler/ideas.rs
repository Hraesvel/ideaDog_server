use actix::{Handler, MailboxError};
use arangors;
use arangors::AqlQuery;

use r2d2::Error;

use serde::{Deserialize, Serialize};

use crate::models::{CastVote, Idea, NewIdea, Owner, QueryIdea, Sort};
use crate::{DbExecutor, VoteStatus};

use std::collections::HashMap;

//noinspection RsExternalLinter
// TODO: Design a idiomatic way to generate AQL queries,
// Prototype for generating AQL queries as a stack
//struct ArangoQuery<SORT> {
//    collection: String,
//    filters: Option<Vec<String>>,
//    limit: Option<String>,
//    sort: Option<SORT>,
//    sub_query: Option<Box<ArangoQuery<SORT>>>,
//}

/// Generates a AQL FILTER line to be appended
fn filter_with(data: Vec<String>) -> String {
    let mut q_string = "FILTER ".to_string();
    let s = data
        .iter()
        .map(|x| format!("'{}' IN ele.tags ", x))
        .collect::<Vec<String>>()
        .join(" AND ");

    q_string.push_str(s.as_str());
    q_string
}

fn query_simple(msg: QueryIdea) -> String {
    let mut query = "FOR ele in ideas ".to_string();
    if let Some(tags) = msg.tags {
        query.push_str(filter_with(tags).as_str());
    }

    match &msg.sort {
        Sort::ALL => query.push_str("SORT ele.date DESC "),
        Sort::BRIGHT => query.push_str("SORT (ele.upvotes / (ele.upvotes + ele.downvotes)) DESC "),
    }

    if let Some(page) = msg.pagination {
        if page.count > 0 {
            let page_str = format!(
                " LIMIT {offset} , {count} ",
                offset = page.offset,
                count = page.count
            );
            query.push_str(page_str.as_str());
        }
    }

    query.push_str("RETURN ele");

    query
}

fn query_with_search(msg: QueryIdea) -> String {
    let mut query = format!("FOR ele in idea_search SEARCH ANALYZER(ele.text IN TOKENS('{query}' , 'text_en'), 'text_en') "
                            , query=msg.query.unwrap().clone()
    );

    if let Some(tags) = msg.tags {
        query.push_str(filter_with(tags).as_str());
    }

    match &msg.sort {
        Sort::ALL => query.push_str("SORT ele.date DESC "),
        Sort::BRIGHT => query.push_str("SORT (ele.upvotes / (ele.upvotes + ele.downvotes)) DESC "),
    }

    query.push_str(" SORT TFIDF(ele) DESC ");

    if let Some(page) = msg.pagination {
        if page.count > 0 {
            let page_str = format!(
                " LIMIT {offset} , {count} ",
                offset = page.offset,
                count = page.count
            );
            query.push_str(page_str.as_str());
        }
    }

    query.push_str("RETURN ele");

    query
}

impl Handler<QueryIdea> for DbExecutor {
    type Result = Result<Vec<Idea>, Error>;

    fn handle(&mut self, msg: QueryIdea, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();

        let aql = if let Some(id) = msg.id {
            format!("RETURN DOCUMENT(CONCAT('ideas/', {id} ))", id = id)
        } else if msg.query.is_some() {
            query_with_search(msg)
        } else {
            query_simple(msg)
        };

        let request: Vec<Idea> = match conn.aql_query(AqlQuery::new(&aql).batch_size(50)) {
            Ok(r) => r,
            Err(e) => {
                println!("{}", e);
                vec![]
            }
        };
        Ok(request)
    }
}

impl Handler<NewIdea> for DbExecutor {
    type Result = Result<Idea, Error>;

    fn handle(&mut self, msg: NewIdea, _ctx: &mut Self::Context) -> Self::Result {
        #[derive(Deserialize, Serialize, Debug, Default)]
        pub struct IdeaMIN {
            // title of the idea
            pub text: String,
            // description of idea
            // Owner's username
            pub owner: Owner,
            // This field is for the votes.
            #[serde(default)]
            pub upvotes: u32,
            #[serde(default)]
            pub downvotes: u32,
            pub tags: Vec<String>,
            //            pub date: i64,
            pub votes: HashMap<String, bool>,
        }

        let conn = self.0.get().unwrap();

        let new_idea = IdeaMIN {
            text: msg.text.clone(),
            tags: msg.tags.clone(),
            owner: Owner::get_owner(msg.owner_id, &conn).expect("Fail to get owner details."),
            ..IdeaMIN::default()
        };

        let data = serde_json::to_value(&new_idea).unwrap();

        let mut query = format!(
            "let tags = (for t in {data}.tags return Document('tags', t))
            INSERT MERGE({data}, {{date: DATE_NOW()}}) INTO {collection} LET idea = NEW
			INSERT {{ _from: idea._id, _to: '{owner}' }} INTO idea_owner
            ",
            data = data,
            collection = "ideas",
            owner = format!("users/{}", new_idea.owner.id)
        );

        if msg.tags.is_empty() {
            query.push_str(" RETURN idea");
        } else {
            query.push_str(
                " RETURN FIRST (FOR tag IN tags
            UPDATE tag WITH {count : tag.count + 1} IN tags
            INSERT { _from: tag._id, _to: idea._id } INTO tag_to_idea
            RETURN idea)",
            );
        }

        let aql = AqlQuery::new(&query).batch_size(1);
        let response: Idea = conn
            .aql_query(aql)
            .map_err(|e| panic!("Error: {}", e))
            .unwrap()
            .pop()
            .unwrap();

        Ok(response)
    }
}

impl Handler<CastVote> for DbExecutor {
    type Result = Result<VoteStatus, MailboxError>;

    fn handle(&mut self, msg: CastVote, _ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().unwrap();

        let aql = AqlQuery::new(
            "LET user = FIRST (for u in 1..1 OUTBOUND DOCUMENT('bearer_tokens', @token) bearer_to_user RETURN u)
                LET idea_id = DOCUMENT('ideas', @id)._id
                UPSERT { _from: idea_id , _to: user._id }
                INSERT { _from: idea_id , _to: user._id, vote: @vote }
                UPDATE { vote: @vote } IN idea_voter LET prev = OLD LET new = NEW
                RETURN {idea_id ,prev: OLD.vote, new: NEW.vote}"
        ).batch_size(1)
         .bind_var("vote", msg.vote)
         .bind_var("token", msg.u_token)
         .bind_var("id", msg.idea_id);

        let response: Result<VoteStatus, MailboxError> = match conn.aql_query(aql) {
            Ok(mut r) => Ok(r.pop().unwrap()),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(MailboxError::Closed);
            }
        };

        let answer = response.unwrap();

        let resp = if answer.has_changed() {
            let mut sub = 1;
            if let Some(a) = &answer.new {
                match a.as_ref() {
                    "upvote" => {
                        if answer.prev.is_none() {
                            sub = 0
                        };
                        let _v : Result<Vec<Idea>, failure::Error> = conn.aql_query(AqlQuery::new(
                            "LET doc = DOCUMENT(@idea_id)
                            UPDATE doc with {upvotes: doc.upvotes + 1, downvotes : doc.downvotes - @sub } in ideas RETURN NEW
                            ")
                            .batch_size(1)
                            .bind_var("sub", sub)
                            .bind_var("idea_id", answer.idea_id));
                        //                        dbg!(v);
                    }
                    "downvote" => {
                        if answer.prev.is_none() {
                            sub = 0
                        };
                        let _v: Result<Vec<Idea>, failure::Error> = conn.aql_query(AqlQuery::new(
                            "LET doc = DOCUMENT(@idea_id)
                            UPDATE doc with {upvotes: doc.upvotes - @sub, downvotes : doc.downvotes + 1 } in ideas RETURN NEW
                            ")
                            .batch_size(1)
                            .bind_var("sub", sub)
                            .bind_var("idea_id", answer.idea_id));
                        //                        dbg!(v);
                    }
                    _ => unreachable!(),
                };
                return Ok(VoteStatus {
                    idea_id: "".to_string(),
                    prev: None,
                    new: None,
                });
            } else {
                Err(MailboxError::Closed)
            }
        } else {
            Err(MailboxError::Closed)
        };

        resp
    }
}
