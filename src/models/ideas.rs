use serde::Deserialize;
use serde::Serialize;
use arangors::AqlQuery;
use r2d2::PooledConnection;
use r2d2_arangodb::ArangodbConnectionManager;

type Connection = PooledConnection<ArangodbConnectionManager>;

#[derive(Deserialize, Serialize, Debug)]
pub struct Owner {
    // _id field from arangodb
    #[serde(alias = "_id")]
    pub id: String,
    // _key field from arangodb
    #[serde(alias = "_key", default)]
    pub key: String,
    //Owner's Username
    pub name: String,
}

impl Owner {
	/// This method will fetch Owner (User) from the Database
	///
	/// # Errors
	/// Error occurse if failed to connect to database or Owner (User) doesn't exist
	///
	/// # Example
	/// ```rust,no_run
	/// let owner = Owner::get_owner(msg.owner_id, &conn).map_err(|x| return x);
	/// assert!(owner.is_ok())
	/// ```
    pub fn get_owner(id: String, conn: &Connection) -> Option<Owner> {
        let ident = if id.contains('/') {
            id
        } else {
            format!("users/{}", id)
        };

        let aql = AqlQuery::new("RETURN DOCUMENT(@ident)")
	        .bind_var("ident", ident)
	        .batch_size(1);

	    let owner = match conn.aql_query(aql) {
		    Ok(mut r) => Some(r.pop().unwrap()),
		    Err(e) => {println!("Error: {}",e); None},
	    };

	    owner
    }

}

#[derive(Deserialize, Serialize, Debug)]
pub struct Idea {
    // _id field from arangodb
    #[serde(alias="_id")]
    pub id: String,
    // _key field from arangodb
    #[serde(alias="_key")]
    pub key: String,
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

    pub date: i64,
}

#[derive(Debug, Clone)]
pub struct NewIdea {
    // title of the idea
    pub text: String,
//    #[serde(default="temp_user")]
    // Owner's username
    pub owner_id: String,

    pub tags: Vec<String>,
}

//noinspection RsExternalLinter
fn temp_user() -> String {
    format!("abc")
}

pub enum Sort {
    ALL,
    BRIGHT,
}

pub struct QueryIdea {
    pub sort: Sort,
    //id
    pub id: Option<String>,
    // Owner's username
    pub owner: Option<String>,
    // Owner's string id
    pub owner_id: Option<String>,
    // accept tags for query string
    pub tags: Option<Vec<String>>,

    pub limit: Option<u32>,
}
