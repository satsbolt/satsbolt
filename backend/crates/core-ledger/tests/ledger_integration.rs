use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

use core_ledger::errors::LedgerError;
use core_ledger::ledger::{
    create_account, execute_transaction, get_account_balance, NewLedgerEntry,
};

async fn get_test_pool() -> sqlx::PgPool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://satsbolt:secretpassword@localhost:5432/satsbolt_ledger".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database for testing");

    // Set schema search path for the test run
    sqlx::query("SET search_path TO satsbolt_schema, public")
        .execute(&pool)
        .await
        .expect("Failed to set search_path");

    pool
}

#[tokio::test]
async fn test_ledger_operations_and_constraints() {
    let pool = get_test_pool().await;

    // Clean up all tables in reverse dependency order
    sqlx::query("DELETE FROM ledger_entries")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM ledger_transactions")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM accounts")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users")
        .execute(&pool)
        .await
        .unwrap();

    // 1. Create test accounts using Alice and Bob as an example
    let user_alice_id = Uuid::new_v4();
    let user_bob_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash)
        VALUES 
        ($1, 'alice_test', 'alice_test@test.com', 'hash_dummy'),
        ($2, 'bob_test', 'bob_test@test.com', 'hash_dummy')
        "#,
        user_alice_id,
        user_bob_id
    )
    .execute(&pool)
    .await
    .expect("Failed to insert dummy users for testing");

    // Create hot wallet
    let hot_wallet = create_account(&pool, None, "test_hot_wallet", "asset")
        .await
        .expect("Failed to create hot wallet");

    // Create Alice's account
    let alice_account = create_account(&pool, Some(user_alice_id), "alice_balance", "liability")
        .await
        .expect("Failed to create Alice account");

    // Create Bob's account
    let bob_account = create_account(&pool, Some(user_bob_id), "bob_balance", "liability")
        .await
        .expect("Failed to create Bob account");

    // 2.Testing the default balances for all accounts
    assert_eq!(get_account_balance(&pool, hot_wallet).await.unwrap(), 0);
    assert_eq!(get_account_balance(&pool, alice_account).await.unwrap(), 0);
    assert_eq!(get_account_balance(&pool, bob_account).await.unwrap(), 0);

    // 3. Deposit money into Alice's account (debit asset by -1000, credit Alice's liability by +1000)
    let deposit_entries = vec![
        NewLedgerEntry {
            account_id: hot_wallet,
            amount: -1000,
        },
        NewLedgerEntry {
            account_id: alice_account,
            amount: 1000,
        },
    ];

    execute_transaction(&pool, "Alice Deposit 1000 sats", &deposit_entries)
        .await
        .expect("Failed Alice deposit transaction");

    assert_eq!(get_account_balance(&pool, hot_wallet).await.unwrap(), -1000);
    assert_eq!(
        get_account_balance(&pool, alice_account).await.unwrap(),
        1000
    );

    // 4. Alice transfers 300 sats to Bob (debit Alice by -300, credit Bob by +300)
    let transfer_entries = vec![
        NewLedgerEntry {
            account_id: alice_account,
            amount: -300,
        },
        NewLedgerEntry {
            account_id: bob_account,
            amount: 300,
        },
    ];

    execute_transaction(&pool, "Alice transfers 300 sats to Bob", &transfer_entries)
        .await
        .expect("Failed transfer transaction");

    assert_eq!(
        get_account_balance(&pool, alice_account).await.unwrap(),
        700
    );
    assert_eq!(get_account_balance(&pool, bob_account).await.unwrap(), 300);

    // 5. Test overdraft constraint (Alice tries to transfer 800 sats but only has 700)
    let overdraft_entries = vec![
        NewLedgerEntry {
            account_id: alice_account,
            amount: -800,
        },
        NewLedgerEntry {
            account_id: bob_account,
            amount: 800,
        },
    ];

    let overdraft_result =
        execute_transaction(&pool, "Alice overdraft attempt", &overdraft_entries).await;
    //  Testing overdraft error
    assert!(overdraft_result.is_err());

    match overdraft_result.err().unwrap() {
        LedgerError::InsufficientBalance(acct, required, available) => {
            assert_eq!(acct, alice_account);
            assert_eq!(required, 800);
            assert_eq!(available, 700);
        }
        other => panic!("Unexpected error type: {:?}", other),
    }

    // 6. Test unbalanced double-entry constraint (sum != 0)
    let unbalanced_entries = vec![
        NewLedgerEntry {
            account_id: alice_account,
            amount: -100,
        },
        NewLedgerEntry {
            account_id: bob_account,
            amount: 50,
        },
    ];

    let unbalanced_result = execute_transaction(&pool, "Unbalanced txn", &unbalanced_entries).await;

    assert!(unbalanced_result.is_err());

    match unbalanced_result.err().unwrap() {
        LedgerError::TransactionNotBalanced(sum) => {
            assert_eq!(sum, -50);
        }
        other => panic!("Unexpected error type: {:?}", other),
    }

    // 7. Test single-entry transaction validation
    let single_entry = vec![NewLedgerEntry {
        account_id: alice_account,
        amount: 0,
    }];
    let single_result = execute_transaction(&pool, "Single entry txn", &single_entry).await;

    assert!(single_result.is_err());

    match single_result.err().unwrap() {
        LedgerError::InvalidTransactionEntries => {}
        other => panic!("Unexpected error type: {:?}", other),
    }
}
