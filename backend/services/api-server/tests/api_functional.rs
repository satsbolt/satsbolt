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
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/v1/auth")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/refresh", web::post().to(handlers::auth::refresh))
                    .route("/profile", web::get().to(handlers::auth::get_profile))
                    .route("/profile", web::put().to(handlers::auth::update_profile)),
            )
            .service(
                web::scope("/api/v1/ledger")
                    .route("/balance", web::get().to(handlers::ledger::get_balance)),
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

#[tokio::test]
async fn test_tip_and_balance_flow() {
    let pool = get_test_pool().await;

    // Clean up previous test runs if any
    let _ = sqlx::query!("DELETE FROM users WHERE username IN ('tip_user_x', 'tip_user_y')")
        .execute(&pool)
        .await;

    // Initialize mock Actix application
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/v1/auth")
                    .route("/register", web::post().to(handlers::auth::register)),
            )
            .service(
                web::scope("/api/v1/ledger")
                    .route("/balance", web::get().to(handlers::ledger::get_balance))
                    .route("/tip", web::post().to(handlers::ledger::post_tip)),
            ),
    )
    .await;

    let username_a = format!("tip_user_x_{}", uuid::Uuid::new_v4());
    let email_a = format!("x_{}@example.com", uuid::Uuid::new_v4());
    let username_b = format!("tip_user_y_{}", uuid::Uuid::new_v4());
    let email_b = format!("y_{}@example.com", uuid::Uuid::new_v4());

    // Register User A
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "username": username_a,
            "email": email_a,
            "password": "pass"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body_a: serde_json::Value = test::read_body_json(resp).await;
    let token_a = body_a["token"]
        .as_str()
        .unwrap_or_else(|| panic!("Registration failed: {:?}", body_a))
        .to_string();

    // Query Balance for User A
    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/balance")
        .insert_header(("Authorization", format!("Bearer {}", token_a)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let balance_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(balance_body["balance_sats"], 0);

    // Register User B
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "username": username_b,
            "email": email_b,
            "password": "pass"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body_b: serde_json::Value = test::read_body_json(resp).await;
    let token_b = body_b["token"].as_str().unwrap().to_string();

    // Give User A 1000 sats manually (Simulate inbound Lightning deposit)
    let user_a_account = sqlx::query!(
        "SELECT a.id FROM accounts a JOIN users u ON a.user_id = u.id WHERE u.username = $1 AND a.account_type = 'liability'",
        username_a
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .id;

    let sys_account = match sqlx::query!("SELECT id FROM accounts WHERE name = 'hot_wallet' AND user_id IS NULL")
        .fetch_optional(&pool)
        .await
        .unwrap()
    {
        Some(acc) => acc.id,
        None => sqlx::query!("INSERT INTO accounts (id, name, account_type) VALUES ($1, 'hot_wallet', 'asset') RETURNING id", uuid::Uuid::new_v4())
            .fetch_one(&pool)
            .await
            .unwrap()
            .id,
    };

    let entries = vec![
        core_ledger::ledger::NewLedgerEntry {
            account_id: sys_account,
            amount: -1000,
        },
        core_ledger::ledger::NewLedgerEntry {
            account_id: user_a_account,
            amount: 1000,
        },
    ];
    core_ledger::ledger::execute_transaction(&pool, "Simulated Deposit", &entries)
        .await
        .unwrap();

    // User A Tips User B 500 sats
    let tip_req = test::TestRequest::post()
        .uri("/api/v1/ledger/tip")
        .insert_header(("Authorization", format!("Bearer {}", token_a)))
        .set_json(json!({
            "recipient_username": username_b,
            "amount_sats": 500,
            "memo": "Enjoy the sats!"
        }))
        .to_request();
    let resp = test::call_service(&app, tip_req).await;
    assert!(resp.status().is_success());

    // Check Balance User A (Should be 500)
    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/balance")
        .insert_header(("Authorization", format!("Bearer {}", token_a)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let balance_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(balance_body["balance_sats"], 500);

    // Check Balance User B (Should be 500)
    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/balance")
        .insert_header(("Authorization", format!("Bearer {}", token_b)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let balance_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(balance_body["balance_sats"], 500);

    // Try Overdrafting User A
    let tip_req = test::TestRequest::post()
        .uri("/api/v1/ledger/tip")
        .insert_header(("Authorization", format!("Bearer {}", token_a)))
        .set_json(json!({
            "recipient_username": username_b,
            "amount_sats": 1000, // They only have 500!
            "memo": "Overdraft"
        }))
        .to_request();
    let resp = test::call_service(&app, tip_req).await;
    assert_eq!(resp.status(), 400); // Bad Request (Insufficient Balance)
}

#[tokio::test]
async fn test_lightning_deposit_merchant_and_offramp_flow() {
    use std::sync::Arc;
    let pool = get_test_pool().await;

    // Clean up previous runs
    let _ = sqlx::query!("DELETE FROM users WHERE username IN ('pay_user_x', 'pay_user_y')")
        .execute(&pool)
        .await;

    let swap_provider: Arc<dyn offramp_swap::SwapProvider> =
        offramp_swap::bitnob::get_provider(None, None).into();
    let quotes_cache = web::Data::new(handlers::payments::QuotesCache {
        quotes: std::sync::Mutex::new(std::collections::HashMap::new()),
    });

    // Initialize mock Actix application with ALL routes
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(swap_provider.clone()))
            .app_data(quotes_cache.clone())
            .service(
                web::scope("/api/v1/auth")
                    .route("/register", web::post().to(handlers::auth::register)),
            )
            .service(
                web::scope("/api/v1/ledger")
                    .route("/balance", web::get().to(handlers::ledger::get_balance))
                    .route(
                        "/withdraw/offramp",
                        web::post().to(handlers::payments::withdraw_offramp),
                    ),
            )
            .service(
                web::scope("/api/v1/offramp")
                    .route("/quote", web::post().to(handlers::payments::get_quote)),
            )
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
            .service(web::scope("/api/v1/internal").route(
                "/settle-deposit",
                web::post().to(handlers::payments::settle_deposit),
            )),
    )
    .await;

    // 1. Register a test user
    let pay_username = format!("pay_user_x_{}", uuid::Uuid::new_v4());
    let pay_email = format!("pay_user_x_{}@example.com", uuid::Uuid::new_v4());
    let register_payload = json!({
        "username": pay_username,
        "email": pay_email,
        "password": "securepassword123"
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let access_token = body["token"].as_str().unwrap().to_string();

    // 2. Mock a deposit settlement
    let invoice_id = uuid::Uuid::new_v4();
    let payment_hash = format!("test_hash_{}", uuid::Uuid::new_v4().simple());

    // Ensure platform accounts are bootstrapped
    let _ = api_server::bootstrap_platform_accounts(&pool).await;

    let user_id = sqlx::query!("SELECT id FROM users WHERE username = $1", pay_username)
        .fetch_one(&pool)
        .await
        .unwrap()
        .id;

    sqlx::query!(
        r#"
        INSERT INTO invoices (id, user_id, payment_hash, payment_request, amount_sats, status)
        VALUES ($1, $2, $3, $4, 10000, 'pending')
        "#,
        invoice_id,
        user_id,
        payment_hash,
        "lnbc100u..."
    )
    .execute(&pool)
    .await
    .unwrap();

    // Call settle-deposit endpoint
    let secret = std::env::var("INTERNAL_SERVICE_SECRET")
        .unwrap_or_else(|_| "super-secret-token".to_string());
    let settle_payload = json!({
        "payment_hash": payment_hash,
        "amount_sats": 10000
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/internal/settle-deposit")
        .insert_header(("Authorization", format!("Bearer {}", secret)))
        .set_json(&settle_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify balance is now 10000
    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/balance")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let balance_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(balance_body["balance_sats"], 10000);

    // 3. Get Off-ramp Quote
    let quote_payload = json!({
        "amount_sats": 5000,
        "currency": "USD"
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/offramp/quote")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&quote_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let quote_body: serde_json::Value = test::read_body_json(resp).await;
    let quote_id = quote_body["quote_id"].as_str().unwrap().to_string();

    // 4. Initiate Off-ramp Withdrawal
    let withdraw_payload = json!({
        "quote_id": quote_id,
        "bank_code": "024",
        "account_number": "12345678",
        "account_name": "John Doe"
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/ledger/withdraw/offramp")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&withdraw_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let withdraw_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(withdraw_body["status"], "pending");
    let withdrawal_id = withdraw_body["withdrawal_id"].as_str().unwrap().to_string();

    // Verify User balance locked (10000 - 5000 - 100 = 4900)
    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/balance")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let balance_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(balance_body["balance_sats"], 4900);

    // 5. Test Background Worker process (simulate payout success)
    let jobs = sqlx::query!("SELECT id FROM payout_jobs WHERE status = 'queued'")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(!jobs.is_empty());

    // Run the worker task directly
    api_server::process_pending_payout_jobs(&pool, swap_provider.as_ref())
        .await
        .unwrap();

    // Check withdrawal is now succeeded in DB
    let w_status = sqlx::query!(
        "SELECT status FROM withdrawals WHERE id = $1",
        uuid::Uuid::parse_str(&withdrawal_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .status;
    assert_eq!(w_status, "succeeded");
}
