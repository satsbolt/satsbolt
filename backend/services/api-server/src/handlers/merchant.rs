use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
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

/// [STUB] Generate a new Lightning Invoice for a merchant to receive inbound payments.
/// In Sprint 2, this will interface with LDK/Polar to generate a real invoice!
pub async fn create_invoice(
    _user: ReqUser, // Validates the user is logged in
    req: web::Json<CreateInvoiceRequest>,
) -> impl Responder {
    if req.amount_sats <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invoice amount must be strictly greater than 0."
        }));
    }

    let fake_invoice_id = Uuid::new_v4();
    let fake_lnbc = format!(
        "lnbc{}1fakeinvoicestubforflutterdevs{}xyz",
        req.amount_sats,
        fake_invoice_id.simple()
    );

    HttpResponse::Created().json(InvoiceResponse {
        id: fake_invoice_id,
        payment_request: fake_lnbc,
        amount_sats: req.amount_sats,
        memo: req
            .memo
            .clone()
            .unwrap_or_else(|| "SatsBolt Merchant Payment".to_string()),
        status: "pending".to_string(),
    })
}

/// [STUB] Get the status of an existing Lightning Invoice.
/// In Sprint 2, this will check our database/LDK to see if the payment has settled.
pub async fn get_invoice(_user: ReqUser, path: web::Path<Uuid>) -> impl Responder {
    let invoice_id = path.into_inner();

    // Just return a fake "pending" status so the mobile dev can build the loading spinner UI!
    HttpResponse::Ok().json(InvoiceResponse {
        id: invoice_id,
        payment_request: "lnbc...".to_string(),
        amount_sats: 1000,
        memo: "SatsBolt Merchant Payment".to_string(),
        status: "pending".to_string(),
    })
}
