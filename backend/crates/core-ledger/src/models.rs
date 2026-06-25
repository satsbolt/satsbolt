use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub account_type: String, // 'asset', 'liability', 'equity', 'revenue', 'expense'
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LedgerTransaction {
    pub id: Uuid,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub ledger_transaction_id: Uuid,
    pub account_id: Uuid,
    pub amount: i64,
    pub created_at: DateTime<Utc>,
}
