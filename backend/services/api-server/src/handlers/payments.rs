use actix_web::{web, HttpResponse, Responder};
use log::error;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::auth::ReqUser;
use offramp_swap::{PayoutDestination, Quote, SwapProvider, SwapStatus};

pub struct QuotesCache {
    pub quotes: Mutex<std::collections::HashMap<String, Quote>>,
}

#[derive(Deserialize)]
pub struct QuoteRequest {
    pub amount_sats: u64,
    pub currency: String,
}

#[derive(Deserialize)]
pub struct LightningWithdrawRequest {
    pub invoice: String,
}

#[derive(Deserialize)]
pub struct OfframpWithdrawRequest {
    pub quote_id: String,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: String,
}

#[derive(Deserialize)]
pub struct SettleDepositRequest {
    pub payment_hash: String,
    pub amount_sats: i64,
}

// 1. Get Quote Endpoint
pub async fn get_quote(
    provider: web::Data<Arc<dyn SwapProvider>>,
    cache: web::Data<QuotesCache>,
    _user: ReqUser,
    req: web::Json<QuoteRequest>,
) -> impl Responder {
    if req.amount_sats == 0 {
        return HttpResponse::BadRequest().body("Amount must be greater than 0");
    }

    match provider.get_quote(req.amount_sats, &req.currency) {
        Ok(quote) => {
            cache
                .quotes
                .lock()
                .unwrap()
                .insert(quote.quote_id.clone(), quote.clone());
            HttpResponse::Ok().json(quote)
        }
        Err(e) => {
            error!("Failed to get quote from provider: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Failed to get quote: {}", e))
        }
    }
}

// 2. Lightning Withdrawal Endpoint
pub async fn withdraw_lightning(
    pool: web::Data<PgPool>,
    user: ReqUser,
    req: web::Json<LightningWithdrawRequest>,
) -> impl Responder {
    // Call lightning-node to pay the invoice
    let ldk_node_url =
        std::env::var("LIGHTNING_NODE_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());

    // 1. Fetch user liability account
    let user_account = match sqlx::query!(
        r#"
        SELECT id FROM accounts WHERE user_id = $1 AND account_type = 'liability'
        "#,
        user.id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(acc) => acc.id,
        Err(_) => {
            return HttpResponse::InternalServerError().body("User liability account not found")
        }
    };

    // 2. Fetch platform hot_wallet account
    let hot_wallet_account = match sqlx::query!(
        r#"
        SELECT id FROM accounts WHERE name = 'hot_wallet' AND user_id IS NULL
        "#,
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(acc) => acc.id,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Platform hot_wallet account not found")
        }
    };

    // Parse/decode the invoice to see amount.
    // For simplicity, we can query it via lightning-node, or just send it directly to lightning-node /pay.
    // Since /pay takes the invoice, lets make the /pay request first.
    // Wait! To prevent overdraft, we need to check the balance of the user's account first.
    // Let's get the balance of the user.
    let balance = match core_ledger::ledger::get_account_balance(pool.get_ref(), user_account).await
    {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to get balance: {}", e))
        }
    };

    // Call lightning-node `/pay`
    let client = reqwest::Client::new();
    let ldk_payload = serde_json::json!({
        "invoice": req.invoice.clone(),
        "withdrawal_id": Uuid::new_v4().to_string(),
    });

    let res = match client
        .post(format!("{}/pay", ldk_node_url))
        .json(&ldk_payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to contact lightning-node: {}", e))
        }
    };

    if !res.status().is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return HttpResponse::BadRequest().body(format!("Payment failed: {}", err_body));
    }

    #[derive(Deserialize)]
    struct LdkPayResponse {
        #[allow(dead_code)]
        status: String,
        payment_hash: String,
        fee_msat: u64,
    }

    let ldk_data: LdkPayResponse = match res.json().await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to parse response: {}", e))
        }
    };

    // For simplicity, let's assume the amount is parsed/returned or we can estimate it,
    // wait! The Bolt11Invoice can be parsed on the api-server or we can fetch it.
    // Let's decode the invoice using LDK's Bolt11Invoice or simply parse the invoice amount from the invoice string if it contains amount.
    // Wait, since we are in Rust, `ldk-node` v0.7.0's re-exported `Bolt11Invoice` is in scope or we can use it!
    // Yes! `use ldk_node::lightning_invoice::Bolt11Invoice;` is perfectly available in the cargo dependencies!
    // Let's parse it to get the exact amount in sats:
    use lightning_invoice::Bolt11Invoice;
    use std::str::FromStr;
    let amount_sats = match Bolt11Invoice::from_str(&req.invoice) {
        Ok(inv) => match inv.amount_milli_satoshis() {
            Some(msat) => msat.div_ceil(1000), // round up to nearest sat
            None => {
                return HttpResponse::BadRequest().body("Amountless invoices not supported yet")
            }
        },
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid invoice: {:?}", e)),
    };

    let fee_sats = ldk_data.fee_msat.div_ceil(1000);
    let total_sats = (amount_sats + fee_sats) as i64;

    if balance < total_sats {
        return HttpResponse::BadRequest().body("Insufficient balance including fee");
    }

    // Now, write to double-entry ledger inside a Postgres transaction
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("DB transaction failed: {}", e))
        }
    };

    // Save withdrawal record
    let withdrawal_id = Uuid::new_v4();
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO withdrawals (id, user_id, payment_hash, payment_request, amount_sats, fee_sats, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'succeeded')
        "#,
        withdrawal_id,
        user.id,
        ldk_data.payment_hash,
        req.invoice,
        amount_sats as i64,
        fee_sats as i64
    )
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError().body(format!("Failed to save withdrawal: {}", e));
    }

    // Execute double-entry ledger transaction
    let description = format!("Lightning withdrawal: {}", ldk_data.payment_hash);
    let entries = vec![
        core_ledger::ledger::NewLedgerEntry {
            account_id: user_account,
            amount: -total_sats, // Debit user (negative)
        },
        core_ledger::ledger::NewLedgerEntry {
            account_id: hot_wallet_account,
            amount: total_sats, // Credit hot_wallet (positive)
        },
    ];

    if let Err(e) =
        core_ledger::ledger::execute_transaction_tx(&mut tx, &description, &entries).await
    {
        return HttpResponse::InternalServerError()
            .body(format!("Ledger transaction failed: {}", e));
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().body(format!("Failed to commit: {}", e));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "withdrawal_id": withdrawal_id,
        "payment_hash": ldk_data.payment_hash,
        "amount_sats": amount_sats,
        "fee_sats": fee_sats,
    }))
}

// 3. Offramp Withdrawal Endpoint
pub async fn withdraw_offramp(
    pool: web::Data<PgPool>,
    provider: web::Data<Arc<dyn SwapProvider>>,
    cache: web::Data<QuotesCache>,
    user: ReqUser,
    req: web::Json<OfframpWithdrawRequest>,
) -> impl Responder {
    // 1. Fetch quote from cache
    let quote = {
        let quotes_map = cache.quotes.lock().unwrap();
        match quotes_map.get(&req.quote_id) {
            Some(q) => q.clone(),
            None => return HttpResponse::BadRequest().body("Quote not found or expired"),
        }
    };

    if quote.expires_at < chrono::Utc::now() {
        return HttpResponse::BadRequest().body("Quote expired");
    }

    // 2. Fetch accounts
    let user_account = match sqlx::query!(
        r#"
        SELECT id FROM accounts WHERE user_id = $1 AND account_type = 'liability'
        "#,
        user.id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(acc) => acc.id,
        Err(_) => {
            return HttpResponse::InternalServerError().body("User liability account not found")
        }
    };

    let hot_wallet_account = match sqlx::query!(
        r#"
        SELECT id FROM accounts WHERE name = 'hot_wallet' AND user_id IS NULL
        "#,
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(acc) => acc.id,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Platform hot_wallet account not found")
        }
    };

    let platform_fees_account = match sqlx::query!(
        r#"
        SELECT id FROM accounts WHERE name = 'platform_fees' AND user_id IS NULL
        "#,
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(acc) => acc.id,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Platform platform_fees account not found")
        }
    };

    let total_sats = (quote.amount_sats + quote.fee_sats) as i64;

    // Check balance
    let balance = match core_ledger::ledger::get_account_balance(pool.get_ref(), user_account).await
    {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to check balance: {}", e))
        }
    };

    if balance < total_sats {
        return HttpResponse::BadRequest().body("Insufficient balance");
    }

    // Create database transaction
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Transaction failed: {}", e))
        }
    };

    // Save withdrawal record in pending state
    let withdrawal_id = Uuid::new_v4();
    let payment_hash_placeholder = format!("offramp_{}", Uuid::new_v4().simple());
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO withdrawals (id, user_id, payment_hash, payment_request, amount_sats, fee_sats, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending')
        "#,
        withdrawal_id,
        user.id,
        payment_hash_placeholder,
        format!("offramp:{}", quote.quote_id),
        quote.amount_sats as i64,
        quote.fee_sats as i64
    )
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError().body(format!("Failed to save withdrawal: {}", e));
    }

    // Execute double entry transaction (locks funds)
    let description = format!("Offramp withdrawal (pending): {}", withdrawal_id);
    let entries = vec![
        core_ledger::ledger::NewLedgerEntry {
            account_id: user_account,
            amount: -total_sats, // Debit user (negative)
        },
        core_ledger::ledger::NewLedgerEntry {
            account_id: hot_wallet_account,
            amount: quote.amount_sats as i64, // Credit hot_wallet (positive)
        },
        core_ledger::ledger::NewLedgerEntry {
            account_id: platform_fees_account,
            amount: quote.fee_sats as i64, // Credit fees (positive)
        },
    ];

    if let Err(e) =
        core_ledger::ledger::execute_transaction_tx(&mut tx, &description, &entries).await
    {
        return HttpResponse::InternalServerError()
            .body(format!("Ledger entry creation failed: {}", e));
    }

    // Initiate payout with swap provider
    let dest = PayoutDestination {
        bank_code: req.bank_code.clone(),
        account_number: req.account_number.clone(),
        account_name: req.account_name.clone(),
        phone_number: None,
    };

    let payout_result = match provider.initiate_payout(&quote.quote_id, dest) {
        Ok(res) => res,
        Err(e) => {
            error!("Swap provider payout initiation failed: {:?}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Payout initiation failed: {}", e));
        }
    };

    // Update the withdrawal record with the real swap ID
    if let Err(e) = sqlx::query!(
        r#"
        UPDATE withdrawals
        SET payment_hash = $1
        WHERE id = $2
        "#,
        payout_result.swap_id,
        withdrawal_id
    )
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to update payout ID: {}", e));
    }

    // Queue payout job
    let payout_job_id = Uuid::new_v4();
    let job_status = match payout_result.status {
        SwapStatus::Succeeded => "completed",
        SwapStatus::Failed(_) => "failed",
        SwapStatus::Pending => "queued",
    };

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO payout_jobs (id, withdrawal_id, status)
        VALUES ($1, $2, $3)
        "#,
        payout_job_id,
        withdrawal_id,
        job_status
    )
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to save payout job: {}", e));
    }

    // If immediate failure, we can handle refund inside the background worker or commit first and let background worker process it immediately.
    // Comitting is clean and consistent.
    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().body(format!("Failed to commit: {}", e));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "pending",
        "withdrawal_id": withdrawal_id,
        "swap_id": payout_result.swap_id,
        "amount_sats": quote.amount_sats,
        "fee_sats": quote.fee_sats,
    }))
}

// 4. Internal Settle Deposit Handler
pub async fn settle_deposit(
    pool: web::Data<PgPool>,
    req: web::Json<SettleDepositRequest>,
    req_headers: actix_web::HttpRequest,
) -> impl Responder {
    // 1. Auth check
    let auth_header = match req_headers.headers().get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(s) => s,
            Err(_) => return HttpResponse::Unauthorized().body("Invalid authorization header"),
        },
        None => return HttpResponse::Unauthorized().body("Missing authorization header"),
    };

    let expected_secret = std::env::var("INTERNAL_SERVICE_SECRET")
        .unwrap_or_else(|_| "super-secret-token".to_string());
    if auth_header != format!("Bearer {}", expected_secret) {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    // 2. Database transaction
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("DB transaction begin failed: {}", e))
        }
    };

    // 3. Find pending invoice
    let invoice = match sqlx::query!(
        r#"
        SELECT id, user_id, status, amount_sats
        FROM invoices
        WHERE payment_hash = $1
        FOR UPDATE
        "#,
        req.payment_hash
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(inv)) => inv,
        Ok(None) => return HttpResponse::NotFound().body("Invoice not found"),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to query invoice: {}", e))
        }
    };

    if invoice.status == "settled" {
        return HttpResponse::Ok().body("Invoice already settled");
    }

    // 4. Update invoice status
    if let Err(e) = sqlx::query!(
        r#"
        UPDATE invoices
        SET status = 'settled', settled_at = NOW()
        WHERE id = $1
        "#,
        invoice.id
    )
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to update invoice: {}", e));
    }

    // 5. Fetch user liability account
    let user_account = match sqlx::query!(
        r#"
        SELECT id FROM accounts WHERE user_id = $1 AND account_type = 'liability'
        "#,
        invoice.user_id
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(acc) => acc.id,
        Err(_) => {
            return HttpResponse::InternalServerError().body("User liability account not found")
        }
    };

    // 6. Fetch platform hot_wallet account
    let hot_wallet_account = match sqlx::query!(
        r#"
        SELECT id FROM accounts WHERE name = 'hot_wallet' AND user_id IS NULL
        "#,
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(acc) => acc.id,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Platform hot_wallet account not found")
        }
    };

    // 7. Execute double entry ledger transaction
    let description = format!("Settle deposit for invoice {}", invoice.id);
    let entries = vec![
        core_ledger::ledger::NewLedgerEntry {
            account_id: user_account,
            amount: invoice.amount_sats, // Credit user (positive)
        },
        core_ledger::ledger::NewLedgerEntry {
            account_id: hot_wallet_account,
            amount: -invoice.amount_sats, // Debit hot_wallet (negative)
        },
    ];

    if let Err(e) =
        core_ledger::ledger::execute_transaction_tx(&mut tx, &description, &entries).await
    {
        return HttpResponse::InternalServerError()
            .body(format!("Ledger entry creation failed: {}", e));
    }

    // 8. Commit
    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to commit transaction: {}", e));
    }

    HttpResponse::Ok().body("Invoice settled successfully")
}
