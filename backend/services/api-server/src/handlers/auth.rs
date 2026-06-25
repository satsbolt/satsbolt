use actix_web::{web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{generate_jwt, hash_password, verify_password, ReqUser};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String, // Can be username or email
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    pub token: String,
    pub refresh_token: String,
}

/// Register a new user and create their double-entry accounting ledger liability account
pub async fn register(pool: web::Data<PgPool>, req: web::Json<RegisterRequest>) -> impl Responder {
    let username = req.username.trim().to_lowercase();
    let email = req.email.trim().to_lowercase();
    let password = req.password.trim();

    if username.is_empty() || email.is_empty() || password.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Username, email, and password cannot be empty"
        }));
    }

    // Check if user already exists
    let user_exists = match sqlx::query!(
        "SELECT 1 as \"exists!\" FROM users WHERE username = $1 OR email = $2",
        username,
        email
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(res) => res.is_some(),
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database query error: {}", e)
            }))
        }
    };

    if user_exists {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "Username or email is already taken"
        }));
    }

    // Hash password
    let password_hash = match hash_password(password) {
        Ok(h) => h,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Password hashing failed: {}", e)
            }))
        }
    };

    // Execute User creation and ledger account allocation inside a SQL Transaction
    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to start database transaction: {}", e)
            }))
        }
    };

    let user_id = Uuid::new_v4();
    let now = Utc::now();

    // 1. Insert User
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        "#,
        user_id,
        username,
        email,
        password_hash,
        now
    )
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create user record: {}", e)
        }));
    }

    // 2. Create User Liability Account in Ledger
    let account_name = format!("User {} Liability Account", username);
    if let Err(e) =
        core_ledger::ledger::create_account_tx(&mut tx, Some(user_id), &account_name, "liability")
            .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create user ledger account: {}", e)
        }));
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to commit transaction: {}", e)
        }));
    }

    // Generate JWT Access Token and Refresh Token
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        "super_secret_unbreakable_ledger_key_change_me_in_production".to_string()
    });
    let jwt_exp = std::env::var("JWT_EXPIRATION_SECS")
        .unwrap_or_else(|_| "86400".to_string())
        .parse::<i64>()
        .unwrap_or(86400);

    let token = match generate_jwt(user_id, &username, &jwt_secret, jwt_exp) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("JWT generation failed: {}", e)
            }))
        }
    };

    let refresh_token = Uuid::new_v4().to_string();
    let refresh_expiry = Utc::now() + Duration::days(30);

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        refresh_token,
        refresh_expiry
    )
    .execute(pool.get_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Session creation failed: {}", e)
        }));
    }

    HttpResponse::Created().json(AuthResponse {
        user: UserResponse {
            id: user_id,
            username,
            email,
            created_at: now,
        },
        token,
        refresh_token,
    })
}

/// Authenticate user credentials and return access/refresh token pair
pub async fn login(pool: web::Data<PgPool>, req: web::Json<LoginRequest>) -> impl Responder {
    let identity = req.username.trim().to_lowercase();
    let password = req.password.trim();

    let user = match sqlx::query!(
        r#"
        SELECT id, username, email, password_hash, created_at
        FROM users
        WHERE username = $1 OR email = $1
        "#,
        identity
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid username or password"
            }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    };

    if !verify_password(&user.password_hash, password) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid username or password"
        }));
    }

    // Generate JWT Access Token and Refresh Token
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        "super_secret_unbreakable_ledger_key_change_me_in_production".to_string()
    });
    let jwt_exp = std::env::var("JWT_EXPIRATION_SECS")
        .unwrap_or_else(|_| "86400".to_string())
        .parse::<i64>()
        .unwrap_or(86400);

    let token = match generate_jwt(user.id, &user.username, &jwt_secret, jwt_exp) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("JWT generation failed: {}", e)
            }))
        }
    };

    let refresh_token = Uuid::new_v4().to_string();
    let refresh_expiry = Utc::now() + Duration::days(30);

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user.id,
        refresh_token,
        refresh_expiry
    )
    .execute(pool.get_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Session creation failed: {}", e)
        }));
    }

    HttpResponse::Ok().json(AuthResponse {
        user: UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        },
        token,
        refresh_token,
    })
}

/// Rotate refresh token and yield a new access token
pub async fn refresh(pool: web::Data<PgPool>, req: web::Json<RefreshRequest>) -> impl Responder {
    let session = match sqlx::query!(
        r#"
        SELECT user_id, expires_at
        FROM sessions
        WHERE token = $1
        "#,
        req.refresh_token
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid refresh token"
            }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    };

    if session.expires_at < Utc::now() {
        // Delete expired session
        let _ = sqlx::query!("DELETE FROM sessions WHERE token = $1", req.refresh_token)
            .execute(pool.get_ref())
            .await;

        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Refresh token has expired"
        }));
    }

    // Retrieve username
    let user = match sqlx::query!("SELECT username FROM users WHERE id = $1", session.user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(u) => u,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database user query error: {}", e)
            }))
        }
    };

    // Delete old session for single-use rotation security
    if let Err(e) = sqlx::query!("DELETE FROM sessions WHERE token = $1", req.refresh_token)
        .execute(pool.get_ref())
        .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete rotated session: {}", e)
        }));
    }

    // Generate new Access and Refresh tokens
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        "super_secret_unbreakable_ledger_key_change_me_in_production".to_string()
    });
    let jwt_exp = std::env::var("JWT_EXPIRATION_SECS")
        .unwrap_or_else(|_| "86400".to_string())
        .parse::<i64>()
        .unwrap_or(86400);

    let token = match generate_jwt(session.user_id, &user.username, &jwt_secret, jwt_exp) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("JWT generation failed: {}", e)
            }))
        }
    };

    let new_refresh_token = Uuid::new_v4().to_string();
    let refresh_expiry = Utc::now() + Duration::days(30);

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
        session.user_id,
        new_refresh_token,
        refresh_expiry
    )
    .execute(pool.get_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("New session creation failed: {}", e)
        }));
    }

    HttpResponse::Ok().json(RefreshResponse {
        token,
        refresh_token: new_refresh_token,
    })
}

/// Retrieve the profile of the current authenticated user
pub async fn get_profile(pool: web::Data<PgPool>, user: ReqUser) -> impl Responder {
    match sqlx::query!(
        "SELECT id, username, email, created_at FROM users WHERE id = $1",
        user.id
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(u)) => HttpResponse::Ok().json(UserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
            created_at: u.created_at,
        }),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User profile not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database profile query failed: {}", e)
        })),
    }
}

/// Update user profile attributes
pub async fn update_profile(
    pool: web::Data<PgPool>,
    user: ReqUser,
    req: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    let username = req.username.as_ref().map(|u| u.trim().to_lowercase());
    let email = req.email.as_ref().map(|e| e.trim().to_lowercase());

    // Validate inputs
    if let Some(ref u) = username {
        if u.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Username cannot be empty"
            }));
        }
    }
    if let Some(ref e) = email {
        if e.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Email cannot be empty"
            }));
        }
    }

    // Check if new username or email is already taken by a different user
    if username.is_some() || email.is_some() {
        let is_taken = match sqlx::query!(
            r#"
            SELECT 1 as "taken!"
            FROM users
            WHERE (username = $1 OR email = $2) AND id != $3
            "#,
            username.as_deref().unwrap_or(""),
            email.as_deref().unwrap_or(""),
            user.id
        )
        .fetch_optional(pool.get_ref())
        .await
        {
            Ok(res) => res.is_some(),
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Conflict verification failed: {}", e)
                }))
            }
        };

        if is_taken {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "Username or email is already in use by another user"
            }));
        }
    }

    // Hash password if provided
    let password_hash = if let Some(ref p) = req.password {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Password cannot be empty"
            }));
        }
        match hash_password(trimmed) {
            Ok(h) => Some(h),
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Argon2 password hashing failed: {}", e)
                }))
            }
        }
    } else {
        None
    };

    // Perform database updates
    let now = Utc::now();
    let updated_user = match sqlx::query!(
        r#"
        UPDATE users
        SET 
            username = COALESCE($1, username),
            email = COALESCE($2, email),
            password_hash = COALESCE($3, password_hash),
            updated_at = $4
        WHERE id = $5
        RETURNING id, username, email, created_at
        "#,
        username.as_deref(),
        email.as_deref(),
        password_hash.as_deref(),
        now,
        user.id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(u) => u,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Update database command failed: {}", e)
            }))
        }
    };

    HttpResponse::Ok().json(UserResponse {
        id: updated_user.id,
        username: updated_user.username,
        email: updated_user.email,
        created_at: updated_user.created_at,
    })
}
