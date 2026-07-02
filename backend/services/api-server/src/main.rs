use actix_web::{middleware, web, App, HttpServer};
use dotenvy::dotenv;
use log::info;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;

use api_server::{bootstrap_platform_accounts, handlers, spawn_payout_worker};

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

    // Run database migrations
    info!("Running database migrations...");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    // Run platform account bootstrapping
    info!("Running platform account check and bootstrapping...");
    bootstrap_platform_accounts(&pool)
        .await
        .expect("Failed to bootstrap default platform accounts");

    // Initialize Off-ramp Swap Provider
    let swap_api_key = env::var("SWAP_PROVIDER_API_KEY").ok();
    let swap_base_url = env::var("SWAP_PROVIDER_BASE_URL").ok();
    let swap_provider: Arc<dyn offramp_swap::SwapProvider> =
        offramp_swap::bitnob::get_provider(swap_api_key, swap_base_url).into();

    // Spawn Background Payout Worker
    spawn_payout_worker(pool.clone(), swap_provider.clone());

    // Initialize Quotes Cache
    let quotes_cache = web::Data::new(handlers::payments::QuotesCache {
        quotes: std::sync::Mutex::new(std::collections::HashMap::new()),
    });

    info!("Starting SatsBolt API server on http://{}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(swap_provider.clone()))
            .app_data(quotes_cache.clone())
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
            // Register Ledger routes
            .service(
                web::scope("/api/v1/ledger")
                    .route("/balance", web::get().to(handlers::ledger::get_balance))
                    .route("/tip", web::post().to(handlers::ledger::post_tip))
                    .route(
                        "/withdraw/lightning",
                        web::post().to(handlers::payments::withdraw_lightning),
                    )
                    .route(
                        "/withdraw/offramp",
                        web::post().to(handlers::payments::withdraw_offramp),
                    ),
            )
            // Register Off-ramp Quote routes
            .service(
                web::scope("/api/v1/offramp")
                    .route("/quote", web::post().to(handlers::payments::get_quote)),
            )
            // Register Merchant routes
            .service(
                web::scope("/api/v1/merchant")
                    .route(
                        "/invoice",
                        web::post().to(handlers::merchant::create_invoice),
                    )
                    .route(
                        "/invoice/{id}",
                        web::get().to(handlers::merchant::get_invoice),
                    ),
            )
            // Register Internal routes
            .service(web::scope("/api/v1/internal").route(
                "/settle-deposit",
                web::post().to(handlers::payments::settle_deposit),
            ))
    })
    .bind(&bind_address)?
    .run()
    .await
}
