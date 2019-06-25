use actix::Handler;
use arangors;
use arangors::AqlQuery;
use chrono::Utc;
use r2d2::Error;

use serde::{Deserialize, Serialize};

use crate::models::{Idea, NewIdea, Owner, QueryIdea, Sort};
use crate::DbExecutor;
use serde::export::PhantomData;
use std::collections::HashMap;

// TODO: Design a idiomatic way to generate AQL queries,

// Prototype for generating AQL queries as a stack
struct ArangoQuery<SORT> {
	collection: String,
	filters: Option<Vec<String>>,
	limit: Option<String>,
	sort: Option<SORT>,
	sub_query: Option<Box<ArangoQuery<SORT>>>
}

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

impl Handler<QueryIdea> for DbExecutor {
    type Result = Result<Vec<Idea>, Error>;

    fn handle(&mut self, msg: QueryIdea, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();

        let mut query = "FOR ele in ideas ".to_string();

        if let Some(tags) = msg.tags {
            query.push_str(filter_with(tags).as_str());
        }

        // Handles Sort
        match &msg.sort {
            Sort::ALL => query.push_str("SORT ele.date DESC "),
	        Sort::BRIGHT => {
		        query.push_str("SORT (ele.upvotes / (ele.upvotes + ele.downvotes)) DESC ")
	        }
        }

        query.push_str("RETURN ele");

	    let mut aql = AqlQuery::new(&query).batch_size(50);

        if let Some(id) = msg.id {
            aql = AqlQuery::new("RETURN DOCUMENT(CONCAT('ideas/', @id ))")
                .bind_var("id", id)
                .batch_size(1);
        }

        let request: Vec<Idea> = match conn.aql_query(aql) {
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
			pub votes : HashMap<String, bool>
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
		    query.push_str(" RETURN FIRST (FOR tag IN tags
            UPDATE tag WITH {count : tag.count + 1} IN tags
            INSERT { _from: tag._id, _to: idea._id } INTO tag_to_idea
            RETURN idea)");
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
