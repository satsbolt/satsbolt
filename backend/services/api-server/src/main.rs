use actix_web::{middleware, web, App, HttpServer};
use dotenvy::dotenv;
use log::info;
use sqlx::postgres::PgPoolOptions;
use std::env;

use api_server::{bootstrap_platform_accounts, handlers};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env environment configuration
    let _ = dotenv();

    // Initialize Logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Server address binding configuration
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", host, port);

    // Database connection URL setup
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://satsbolt:secretpassword@localhost:5432/satsbolt_ledger".to_string()
    });
    let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u32>()
        .unwrap_or(10);

    info!("Connecting to PostgreSQL database...");
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to create database connection pool");

    // Run platform account bootstrapping
    info!("Running platform account check and bootstrapping...");
    bootstrap_platform_accounts(&pool)
        .await
        .expect("Failed to bootstrap default platform accounts");

    info!("Starting SatsBolt API server on http://{}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            // Register Authentication routes
            .service(
                web::scope("/api/v1/auth")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/refresh", web::post().to(handlers::auth::refresh))
                    .route("/profile", web::get().to(handlers::auth::get_profile))
                    .route("/profile", web::put().to(handlers::auth::update_profile)),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
