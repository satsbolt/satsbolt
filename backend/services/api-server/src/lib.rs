pub mod auth;
pub mod handlers;

use log::info;

pub async fn bootstrap_platform_accounts(
    pool: &sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
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
