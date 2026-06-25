use actix_web::{test, web, App};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::env;

use api_server::handlers;

async fn get_test_pool() -> sqlx::PgPool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://satsbolt:secretpassword@localhost:5432/satsbolt_ledger".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database for API testing");

    // Force satsbolt_schema
    sqlx::query("SET search_path TO satsbolt_schema, public")
        .execute(&pool)
        .await
        .expect("Failed to set search_path");

    pool
}

#[tokio::test]
async fn test_auth_and_profile_endpoints_flow() {
    let pool = get_test_pool().await;

    // Clean up previous test runs if any
    let _ = sqlx::query!("DELETE FROM users WHERE username = 'api_test_user'")
        .execute(&pool)
        .await;

    // Initialize mock Actix application
    let app = test::init_service(
        App::new().app_data(web::Data::new(pool.clone())).service(
            web::scope("/api/v1/auth")
                .route("/register", web::post().to(handlers::auth::register))
                .route("/login", web::post().to(handlers::auth::login))
                .route("/refresh", web::post().to(handlers::auth::refresh))
                .route("/profile", web::get().to(handlers::auth::get_profile))
                .route("/profile", web::put().to(handlers::auth::update_profile)),
        ),
    )
    .await;

    // --- STEP 1: REGISTER ---
    let register_payload = json!({
        "username": "api_test_user",
        "email": "api_test_user@example.com",
        "password": "securepassword123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Registration failed: {:?}",
        resp.status()
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["username"], "api_test_user");
    assert!(body["token"].is_string());

    let initial_refresh_token = body["refresh_token"].as_str().unwrap().to_string();

    // --- STEP 2: LOGIN ---
    let login_payload = json!({
        "username": "api_test_user",
        "password": "securepassword123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Login failed: {:?}",
        resp.status()
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    let access_token = body["token"].as_str().unwrap().to_string();
    let login_refresh_token = body["refresh_token"].as_str().unwrap().to_string();

    // --- STEP 3: GET PROFILE (SECURED) ---
    let req = test::TestRequest::get()
        .uri("/api/v1/auth/profile")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Profile retrieval failed: {:?}",
        resp.status()
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["username"], "api_test_user");
    assert_eq!(body["email"], "api_test_user@example.com");

    // --- STEP 4: UPDATE PROFILE ---
    let update_payload = json!({
        "email": "updated_test_user@example.com"
    });

    let req = test::TestRequest::put()
        .uri("/api/v1/auth/profile")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Profile update failed: {:?}",
        resp.status()
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["email"], "updated_test_user@example.com");

    // --- STEP 5: REFRESH TOKEN (ROTATION) ---
    let refresh_payload = json!({
        "refresh_token": login_refresh_token
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(&refresh_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Token refresh failed: {:?}",
        resp.status()
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert!(body["refresh_token"].is_string());

    let new_refresh_token = body["refresh_token"].as_str().unwrap().to_string();
    assert_ne!(
        new_refresh_token, login_refresh_token,
        "Refresh token should have rotated"
    );

    // --- STEP 6: VERIFY OLD REFRESH TOKEN IS INVALIDATED (SINGLE-USE ROTATION) ---
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(&refresh_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Old refresh token should be rejected");

    // Clean up initial unused refresh token
    let _ = sqlx::query!(
        "DELETE FROM sessions WHERE token = $1",
        initial_refresh_token
    )
    .execute(&pool)
    .await;
}
