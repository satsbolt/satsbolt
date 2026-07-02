pub mod auth;
pub mod handlers;

use log::{info, error, warn};
use sqlx::PgPool;
use std::sync::Arc;
use offramp_swap::{SwapProvider, SwapStatus};

pub async fn bootstrap_platform_accounts(
    pool: &sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if hot_wallet account exists
    let hot_wallet_exists = sqlx::query!(
        r#"SELECT 1 as "exists!" FROM accounts WHERE name = $1 AND user_id IS NULL"#,
        "hot_wallet"
    )
    .fetch_optional(pool)
    .await?
    .is_some();

    if !hot_wallet_exists {
        info!("Bootstrapping platform hot_wallet asset account...");
        core_ledger::ledger::create_account(pool, None, "hot_wallet", "asset").await?;
    }

    // Check if platform_fees account exists
    let platform_fees_exists = sqlx::query!(
        r#"SELECT 1 as "exists!" FROM accounts WHERE name = $1 AND user_id IS NULL"#,
        "platform_fees"
    )
    .fetch_optional(pool)
    .await?
    .is_some();

    if !platform_fees_exists {
        info!("Bootstrapping platform_fees revenue account...");
        core_ledger::ledger::create_account(pool, None, "platform_fees", "revenue").await?;
    }

    Ok(())
}

pub fn spawn_payout_worker(pool: PgPool, provider: Arc<dyn SwapProvider>) {
    tokio::spawn(async move {
        info!("Starting background off-ramp payout worker loop...");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            if let Err(e) = process_pending_payout_jobs(&pool, provider.as_ref()).await {
                error!("Error processing payout jobs: {:?}", e);
            }
        }
    });
}

pub async fn process_pending_payout_jobs(
    pool: &PgPool,
    provider: &dyn SwapProvider,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Select all queued or processing payout jobs
    let jobs = sqlx::query!(
        r#"
        SELECT id, withdrawal_id, status, attempts
        FROM payout_jobs
        WHERE status = 'queued' OR status = 'processing'
        "#
    )
    .fetch_all(pool)
    .await?;

    for job in jobs {
        info!("Processing payout job {} for withdrawal {}", job.id, job.withdrawal_id);

        // Fetch withdrawal details
        let withdrawal = sqlx::query!(
            r#"
            SELECT id, user_id, payment_hash, amount_sats, fee_sats
            FROM withdrawals
            WHERE id = $1
            "#,
            job.withdrawal_id
        )
        .fetch_one(pool)
        .await?;

        // Query the status from the swap provider and map the error to String immediately to avoid holding Box<dyn Error> (non-Send) across await points
        let status_res = provider.get_status(&withdrawal.payment_hash).map_err(|e| format!("{:?}", e));
        match status_res {
            Ok(SwapStatus::Succeeded) => {
                info!("Payout succeeded for swap ID {}", withdrawal.payment_hash);
                
                let mut tx = pool.begin().await?;
                
                sqlx::query!(
                    r#"
                    UPDATE withdrawals
                    SET status = 'succeeded', completed_at = NOW()
                    WHERE id = $1
                    "#,
                    withdrawal.id
                )
                .execute(&mut *tx)
                .await?;

                sqlx::query!(
                    r#"
                    UPDATE payout_jobs
                    SET status = 'completed', updated_at = NOW()
                    WHERE id = $1
                    "#,
                    job.id
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
            }
            Ok(SwapStatus::Failed(err)) => {
                warn!("Payout failed for swap ID {}: {}", withdrawal.payment_hash, err);
                
                let mut tx = pool.begin().await?;

                // Refund the user liability account
                let user_account = sqlx::query!(
                    r#"SELECT id FROM accounts WHERE user_id = $1 AND account_type = 'liability'"#,
                    withdrawal.user_id
                )
                .fetch_one(&mut *tx)
                .await?
                .id;

                let hot_wallet_account = sqlx::query!(
                    r#"SELECT id FROM accounts WHERE name = 'hot_wallet' AND user_id IS NULL"#,
                )
                .fetch_one(&mut *tx)
                .await?
                .id;

                let platform_fees_account = sqlx::query!(
                    r#"SELECT id FROM accounts WHERE name = 'platform_fees' AND user_id IS NULL"#,
                )
                .fetch_one(&mut *tx)
                .await?
                .id;

                let total_sats = withdrawal.amount_sats + withdrawal.fee_sats;
                let description = format!("Refund offramp payout failure for withdrawal {}", withdrawal.id);
                
                let entries = vec![
                    core_ledger::ledger::NewLedgerEntry {
                        account_id: user_account,
                        amount: total_sats, // Credit user (positive refund)
                    },
                    core_ledger::ledger::NewLedgerEntry {
                        account_id: hot_wallet_account,
                        amount: -withdrawal.amount_sats, // Debit hot_wallet (negative)
                    },
                    core_ledger::ledger::NewLedgerEntry {
                        account_id: platform_fees_account,
                        amount: -withdrawal.fee_sats, // Debit fees (negative)
                    },
                ];

                core_ledger::ledger::execute_transaction_tx(&mut tx, &description, &entries).await?;

                sqlx::query!(
                    r#"
                    UPDATE withdrawals
                    SET status = 'failed', completed_at = NOW()
                    WHERE id = $1
                    "#,
                    withdrawal.id
                )
                .execute(&mut *tx)
                .await?;

                sqlx::query!(
                    r#"
                    UPDATE payout_jobs
                    SET status = 'failed', last_error = $1, updated_at = NOW()
                    WHERE id = $2
                    "#,
                    err,
                    job.id
                )
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
            }
            Ok(SwapStatus::Pending) => {
                info!("Payout status still pending for swap ID {}", withdrawal.payment_hash);
                // Increment attempts, set status to processing if it was queued
                sqlx::query!(
                    r#"
                    UPDATE payout_jobs
                    SET status = 'processing', attempts = attempts + 1, updated_at = NOW()
                    WHERE id = $1
                    "#,
                    job.id
                )
                .execute(pool)
                .await?;
            }
            Err(err_str) => {
                error!("Failed to fetch status from provider for swap ID {}: {}", withdrawal.payment_hash, err_str);
                sqlx::query!(
                    r#"
                    UPDATE payout_jobs
                    SET attempts = attempts + 1, last_error = $1, updated_at = NOW()
                    WHERE id = $2
                    "#,
                    err_str,
                    job.id
                )
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}
