use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::errors::LedgerError;

pub struct NewLedgerEntry {
    pub account_id: Uuid,
    pub amount: i64, // positive for credit, negative for debit
}

/// Create a new ledger account using a database pool
pub async fn create_account(
    pool: &PgPool,
    user_id: Option<Uuid>,
    name: &str,
    account_type: &str,
) -> Result<Uuid, LedgerError> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO accounts (id, user_id, name, account_type)
        VALUES ($1, $2, $3, $4)
        "#,
        id,
        user_id,
        name,
        account_type
    )
    .execute(pool)
    .await?;

    Ok(id)
}

/// Create a new ledger account within an ongoing transaction
pub async fn create_account_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Option<Uuid>,
    name: &str,
    account_type: &str,
) -> Result<Uuid, LedgerError> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO accounts (id, user_id, name, account_type)
        VALUES ($1, $2, $3, $4)
        "#,
        id,
        user_id,
        name,
        account_type
    )
    .execute(&mut **tx)
    .await?;

    Ok(id)
}

/// Query the balance of an account using a database pool
pub async fn get_account_balance(pool: &PgPool, account_id: Uuid) -> Result<i64, LedgerError> {
    // First, verify the account exists
    let account_exists = sqlx::query!(
        r#"SELECT 1 as "exists!" FROM accounts WHERE id = $1"#,
        account_id
    )
    .fetch_optional(pool)
    .await?
    .is_some();

    if !account_exists {
        return Err(LedgerError::AccountNotFound(account_id));
    }

    let row = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(amount), 0)::BIGINT as "balance!"
        FROM ledger_entries
        WHERE account_id = $1
        "#,
        account_id
    )
    .fetch_one(pool)
    .await?;

    Ok(row.balance)
}

/// Query the balance of an account within an ongoing transaction
pub async fn get_account_balance_tx(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<i64, LedgerError> {
    // First, verify the account exists
    let account_exists = sqlx::query!(
        r#"SELECT 1 as "exists!" FROM accounts WHERE id = $1"#,
        account_id
    )
    .fetch_optional(&mut **tx)
    .await?
    .is_some();

    if !account_exists {
        return Err(LedgerError::AccountNotFound(account_id));
    }

    let row = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(amount), 0)::BIGINT as "balance!"
        FROM ledger_entries
        WHERE account_id = $1
        "#,
        account_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.balance)
}

/// Execute a double-entry transaction atomically inside an existing SQL transaction
pub async fn execute_transaction_tx(
    tx: &mut Transaction<'_, Postgres>,
    description: &str,
    entries: &[NewLedgerEntry],
) -> Result<Uuid, LedgerError> {
    // 1. Validate entries length
    if entries.len() < 2 {
        return Err(LedgerError::InvalidTransactionEntries);
    }

    // 2. Validate double-entry balance (sum of all entries must be 0)
    let sum: i64 = entries.iter().map(|e| e.amount).sum();
    if sum != 0 {
        return Err(LedgerError::TransactionNotBalanced(sum));
    }

    // 3. Verify accounts exist and check for overdraft on liability accounts
    for entry in entries {
        let account = sqlx::query!(
            r#"SELECT account_type FROM accounts WHERE id = $1"#,
            entry.account_id
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(LedgerError::AccountNotFound(entry.account_id))?;

        // Liability accounts are user accounts.
        // A credit (positive) increases liability, debit (negative) decreases liability.
        // We must prevent overdrafts (i.e. balance going below 0).
        if account.account_type == "liability" && entry.amount < 0 {
            let balance = get_account_balance_tx(tx, entry.account_id).await?;
            if balance + entry.amount < 0 {
                return Err(LedgerError::InsufficientBalance(
                    entry.account_id,
                    -entry.amount,
                    balance,
                ));
            }
        }
    }

    // 4. Create ledger transaction
    let transaction_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO ledger_transactions (id, description)
        VALUES ($1, $2)
        "#,
        transaction_id,
        description
    )
    .execute(&mut **tx)
    .await?;

    // 5. Insert ledger entries
    for entry in entries {
        sqlx::query!(
            r#"
            INSERT INTO ledger_entries (ledger_transaction_id, account_id, amount)
            VALUES ($1, $2, $3)
            "#,
            transaction_id,
            entry.account_id,
            entry.amount
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(transaction_id)
}

/// Execute a double-entry transaction atomically opening its own SQL transaction
pub async fn execute_transaction(
    pool: &PgPool,
    description: &str,
    entries: &[NewLedgerEntry],
) -> Result<Uuid, LedgerError> {
    let mut tx = pool.begin().await?;
    let transaction_id = execute_transaction_tx(&mut tx, description, entries).await?;
    tx.commit().await?;
    Ok(transaction_id)
}
