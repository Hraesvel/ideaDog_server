#[macro_use]
extern crate failure;
use actix_web::actix::{Addr, SyncArbiter};
use actix_web::http::{header, NormalizePath};
use actix_web::middleware::cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{actix, server, App, HttpRequest, Responder};
use env_logger;
use ideadog::DbExecutor;
use r2d2;
use r2d2_arangodb::{ArangodbConnectionManager, ConnectionOptions};
use std::env;

use midware::AuthMiddleware;
use r2d2::Pool;
use std::time::Duration;

//routes
mod midware;
mod util;
mod views;
//mod ideas;

pub struct AppState {
    database: Addr<DbExecutor>,
}

fn greatings(_req: &HttpRequest<AppState>) -> impl Responder {
    format!("Welcome to ideaDog API!")
}

fn main() {
    let _ = dotenv::dotenv();
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let hostname = format!(
        "{host}:{port}",
        host = env::var("HOST").unwrap_or_else(|_| {
            println!("HOST is not set, will Default to localhost.");
            format!("localhost")
        }),
        port = env::var("PORT").unwrap_or_else(|_| {
            println!("PORT is not set, will Default to 5000.");
            5000.to_string()
        })
    );

    //actix System for handling Actors
    let ideadog_system = actix::System::new("ideaDog");

    let pool = connect_with_pools(5, 15).unwrap();

    //create the SyncArbiters for r2d2
    let arbiter_cores = env::var("ARBITER_THREAD")
        .expect("ARBITER_THREAD must be set")
        .parse::<usize>()
        .unwrap_or_else(|_| {
            eprintln!("ARBITER_THREAD must be a valid number defaulting to 1");
            1
        });
    let addr = SyncArbiter::start(arbiter_cores, move || DbExecutor(pool.clone()));

    server::new(move || {
        let cors = Cors::build()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
                header::ORIGIN,
            ])
            .supports_credentials()
            .max_age(3600)
            .finish();

        App::with_state(AppState {
            database: addr.clone(),
        })
        .prefix("/api")
        .default_resource(|r| r.h(NormalizePath::default()))
        .middleware(Logger::default())
        .middleware(Logger::new("%a %{User-agent}i"))
        .middleware(cors)
        .resource("/", |r| r.f(greatings))
        .configure(views::ideas::config)
        .configure(views::tags::config)
        .configure(views::users::config)
        .configure(views::auth::config)
        .configure(views::search::config)
        .finish()
    })
    .bind(hostname.clone())
    .unwrap()
    .workers(
        env::var("WORKER")
            .expect("WORKER must be set")
            .parse::<usize>()
            .unwrap_or_else(|_| {
                eprintln!("Workers must be a valid number defaulting to 1");
                1
            }),
    )
    .start();

    println!("Starting http server: {}", hostname);
    let _ = ideadog_system.run();
}

/// Attempts to establish a connection with a database a x number times with time out y
///
/// #Arguments
///  * 'tries' - number of tries to attempt before server will exit.
///  * 'timeout' - Duration total time in which this operation should run. timeout / tries = intervals between attempts
///
fn connect_with_pools(tries: u8, timeout: u64) -> Option<Pool<ArangodbConnectionManager>> {
    let arango_config = ConnectionOptions::builder()
        .with_auth_basic(
            env::var("DB_ACCOUNT").expect("DB_ACCOUNT must be set."),
            env::var("DB_PASSWORD").expect("DB_PASSWORD must be set."),
        )
        .with_host(
            env::var("DB_HOST").expect("DB_HOST must be set"),
            env::var("DB_PORT")
                .expect("DB_PORT must be set")
                .parse()
                .expect("DB_PORT must be digits"),
        )
        .with_db(env::var("DB_NAME").expect("DB_NAME must be set."))
        .build();

    let manager = ArangodbConnectionManager::new(arango_config);

    let mut await_pool;
    let mut conn_tries = 0;

    loop {
        match r2d2::Pool::builder()
            .connection_timeout(Duration::new(timeout / tries as u64, 0))
            .build(manager.clone())
        {
            Err(e) => println!("Error: {}", e),
            Ok(r) => {
                await_pool = Some(r);
                break;
            }
        }

        if conn_tries > tries {
            println!(
                "Exceeded establish connection attempts: number of tries {}",
                conn_tries
            );
            std::process::exit(111);
        }
        conn_tries += 1;
    }

    await_pool
}
