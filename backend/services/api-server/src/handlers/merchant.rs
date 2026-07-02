use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::ReqUser;

#[derive(Deserialize)]
pub struct CreateInvoiceRequest {
    pub amount_sats: i64,
    pub memo: Option<String>,
}

#[derive(Serialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub payment_request: String, // The actual lightning invoice (e.g. lnbc...)
    pub amount_sats: i64,
    pub memo: String,
    pub status: String, // "pending", "settled", "expired"
}

/// Generate a new Lightning Invoice for a merchant to receive inbound payments.
pub async fn create_invoice(
    pool: web::Data<PgPool>,
    user: ReqUser, // Validates the user is logged in
    req: web::Json<CreateInvoiceRequest>,
) -> impl Responder {
    if req.amount_sats <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invoice amount must be strictly greater than 0."
        }));
    }

    // Call lightning-node to create the invoice
    let ldk_node_url =
        std::env::var("LIGHTNING_NODE_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    let client = reqwest::Client::new();
    let ldk_payload = serde_json::json!({
        "amount_msat": (req.amount_sats * 1000) as u64,
        "description": req.memo.clone().unwrap_or_else(|| "SatsBolt Merchant Payment".to_string()),
        "expiry_secs": 3600,
        "user_id": user.id.to_string(),
    });

    let res = match client
        .post(format!("{}/invoice", ldk_node_url))
        .json(&ldk_payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to contact lightning-node: {}", e)
            }))
        }
    };

    if !res.status().is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("lightning-node returned error: {}", err_body)
        }));
    }

    #[derive(Deserialize)]
    struct LdkInvoiceResponse {
        invoice: String,
        payment_hash: String,
    }

    let ldk_data: LdkInvoiceResponse = match res.json().await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to parse lightning-node response: {}", e)
            }))
        }
    };

    let invoice_id = Uuid::new_v4();
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO invoices (id, user_id, payment_hash, payment_request, amount_sats, status)
        VALUES ($1, $2, $3, $4, $5, 'pending')
        "#,
        invoice_id,
        user.id,
        ldk_data.payment_hash,
        ldk_data.invoice,
        req.amount_sats
    )
    .execute(pool.get_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to save invoice: {}", e)
        }));
    }

    HttpResponse::Created().json(InvoiceResponse {
        id: invoice_id,
        payment_request: ldk_data.invoice,
        amount_sats: req.amount_sats,
        memo: req
            .memo
            .clone()
            .unwrap_or_else(|| "SatsBolt Merchant Payment".to_string()),
        status: "pending".to_string(),
    })
}

/// Get the status of an existing Lightning Invoice.
pub async fn get_invoice(
    pool: web::Data<PgPool>,
    _user: ReqUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let invoice_id = path.into_inner();

    match sqlx::query!(
        r#"
        SELECT id, payment_request, amount_sats, status
        FROM invoices
        WHERE id = $1
        "#,
        invoice_id
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(inv)) => HttpResponse::Ok().json(InvoiceResponse {
            id: inv.id,
            payment_request: inv.payment_request,
            amount_sats: inv.amount_sats,
            memo: "SatsBolt Merchant Payment".to_string(),
            status: inv.status,
        }),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Invoice not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {}", e)
        })),
    }
}
