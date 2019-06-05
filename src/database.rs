use r2d2::Pool;
use r2d2_arangodb::{ArangodbConnectionManager};

use actix::{Actor, SyncContext};

pub struct DbExecutor(pub Pool<ArangodbConnectionManager>);

impl Actor for DbExecutor {
	type Context = SyncContext<Self>;
}