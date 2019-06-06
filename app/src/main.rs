use actix_web::actix::{Addr, SyncArbiter};
use actix_web::http::{header, NormalizePath, StatusCode};
use actix_web::middleware::cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{actix, server, App, HttpRequest, Responder};
use env_logger;
use ideadog::DbExecutor;
use r2d2;
use r2d2_arangodb::{ArangodbConnectionManager, ConnectionOptions};
use std::env;

//routes
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

    // arangodb connection configurations.
    let arango_config = ConnectionOptions::builder()
        .with_auth_jwt(
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

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    //create the SyncArbiters for r2d2
    let addr = SyncArbiter::start(2, move || DbExecutor(pool.clone()));

    server::new(move || {
        let cors = Cors::build()
            .send_wildcard()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_header(header::CONTENT_TYPE)
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
        .finish()
    })
        .bind(hostname.clone())
        .unwrap()
        .workers(2)
        .start();

    println!("Starting http server: {}", hostname);
    let _ = ideadog_system.run();
}
