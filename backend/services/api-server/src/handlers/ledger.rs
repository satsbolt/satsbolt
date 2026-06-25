use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::ReqUser;

#[derive(Serialize)]
pub struct BalanceResponse {
    pub account_id: Uuid,
    pub balance_sats: i64,
}

/// Retrieve the balance of the current authenticated user's main liability account.
/// The `ReqUser` is automatically extracted from the JWT Bearer token in the headers.
pub async fn get_balance(pool: web::Data<PgPool>, user: ReqUser) -> impl Responder {
    // 1. Fetch the user's liability account ID from the database
    //    We explicitly look for the 'liability' account because that is the user's main ledger balance.
    let account = match sqlx::query!(
        r#"
        SELECT id
        FROM accounts
        WHERE user_id = $1 AND account_type = 'liability'
        "#,
        user.id
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(acc)) => acc,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "No liability account found for this user."
            }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error fetching account: {}", e)
            }))
        }
    };

    // 2. Query the core_ledger crate for the double-entry sum of all entries for this account
    //    This automatically calculates the current balance based on all credits and debits.
    match core_ledger::ledger::get_account_balance(pool.get_ref(), account.id).await {
        Ok(balance) => HttpResponse::Ok().json(BalanceResponse {
            account_id: account.id,
            balance_sats: balance,
        }),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to calculate ledger balance: {}", e)
        })),
    }
}

#[derive(Deserialize)]
pub struct TipRequest {
    pub recipient_username: String,
    pub amount_sats: i64,
    pub memo: Option<String>,
}

#[derive(Serialize)]
pub struct TipResponse {
    pub transaction_id: Uuid,
    pub amount_sats: i64,
    pub recipient: String,
    pub status: String,
}

/// Send a tip from the authenticated user to another user.
pub async fn post_tip(
    pool: web::Data<PgPool>,
    user: ReqUser,
    req: web::Json<TipRequest>,
) -> impl Responder {
    if req.amount_sats <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Tip amount must be strictly greater than 0."
        }));
    }

    let recipient_username = req.recipient_username.trim().to_lowercase();
    if recipient_username == user.username.to_lowercase() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "You cannot tip yourself."
        }));
    }

    // 1. Get Sender's Liability Account
    let sender_account = match sqlx::query!(
        r#"SELECT id FROM accounts WHERE user_id = $1 AND account_type = 'liability'"#,
        user.id
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(acc)) => acc.id,
        Ok(None) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Sender account not found"})),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)})),
    };

    // 2. Get Recipient's User ID and Liability Account
    let recipient_account = match sqlx::query!(
        r#"
        SELECT a.id 
        FROM accounts a
        JOIN users u ON a.user_id = u.id
        WHERE u.username = $1 AND a.account_type = 'liability'
        "#,
        recipient_username
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(acc)) => acc.id,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({"error": "Recipient user not found"})),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)})),
    };

    // 3. Create the double-entry transactions
    // Since these are liability accounts (user balances), a DEBIT (negative) decreases their balance,
    // and a CREDIT (positive) increases their balance.
    let entries = vec![
        core_ledger::ledger::NewLedgerEntry {
            account_id: sender_account,
            amount: -req.amount_sats, // Debit sender
        },
        core_ledger::ledger::NewLedgerEntry {
            account_id: recipient_account,
            amount: req.amount_sats, // Credit recipient
        },
    ];

    let description = req.memo.clone().unwrap_or_else(|| format!("Tip to {}", recipient_username));

    // 4. Execute atomic transaction (this automatically prevents overdrafts!)
    match core_ledger::ledger::execute_transaction(pool.get_ref(), &description, &entries).await {
        Ok(transaction_id) => HttpResponse::Ok().json(TipResponse {
            transaction_id,
            amount_sats: req.amount_sats,
            recipient: req.recipient_username.clone(),
            status: "success".to_string(),
        }),
        Err(core_ledger::errors::LedgerError::InsufficientBalance(_, _, _)) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Insufficient balance to send this tip."
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Ledger transaction failed: {}", e)
        })),
    }
}

