use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    #[error("Insufficient balance in account {0}. Required: {1} sats, Available: {2} sats")]
    InsufficientBalance(Uuid, i64, i64),

    #[error("Transaction is not balanced. Sum of entries must be 0, but is {0}")]
    TransactionNotBalanced(i64),

    #[error("Invalid transaction: must contain at least two entries")]
    InvalidTransactionEntries,
}
