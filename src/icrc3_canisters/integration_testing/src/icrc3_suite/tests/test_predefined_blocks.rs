use candid::Nat;
use icrc_ledger_types::icrc1::account::Account;
use std::str::FromStr;

use bity_ic_icrc3::transaction::{
    ICRC1Transaction, ICRC1TransactionData, ICRC2Transaction, ICRC2TransactionData,
    ICRC37Transaction, ICRC37TransactionData, ICRC7Transaction, ICRC7TransactionData,
    TransactionType,
};
use icrc_ledger_types::icrc::generic_value::ICRC3Value;

#[test]
fn test_icrc1_transaction_validation() {
    // Test mint transaction
    let mint_tx = ICRC1Transaction::new(
        [0u8; 32],
        "1mint".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("mint".to_string()),
            amount: Nat::from(1000u64),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(mint_tx.validate_transaction_fields().is_ok());

    // Test burn transaction
    let burn_tx = ICRC1Transaction::new(
        [0u8; 32],
        "1burn".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("burn".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(burn_tx.validate_transaction_fields().is_ok());

    // Test transfer transaction
    let transfer_tx = ICRC1Transaction::new(
        [0u8; 32],
        "1xfer".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("xfer".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(transfer_tx.validate_transaction_fields().is_ok());
}

#[test]
fn test_icrc2_transaction_validation() {
    // Test transfer transaction
    let transfer_tx = ICRC2Transaction::new(
        [0u8; 32],
        "2xfer".to_string(),
        1234567890,
        Some(Nat::from(1000u64)),
        ICRC2TransactionData {
            op: Some("xfer".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            spender: None,
            memo: None,
            expected_allowance: None,
            expires_at: None,
        },
    );
    assert!(transfer_tx.validate_transaction_fields().is_ok());

    // Test approve transaction
    let approve_tx = ICRC2Transaction::new(
        [0u8; 32],
        "2approve".to_string(),
        1234567890,
        Some(Nat::from(1000u64)),
        ICRC2TransactionData {
            op: Some("approve".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            spender: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            expected_allowance: None,
            expires_at: None,
        },
    );
    assert!(approve_tx.validate_transaction_fields().is_ok());
}

#[test]
fn test_icrc7_transaction_validation() {
    // Test mint transaction
    let mint_tx = ICRC7Transaction::new(
        [0u8; 32],
        "7mint".to_string(),
        1234567890,
        ICRC7TransactionData {
            tid: Some(Nat::from(1u64)),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            meta: None,
            memo: None,
            created_at_time: None,
        },
    );
    assert!(mint_tx.validate_transaction_fields().is_ok());

    // Test burn transaction
    let burn_tx = ICRC7Transaction::new(
        [0u8; 32],
        "7burn".to_string(),
        1234567890,
        ICRC7TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            meta: None,
            memo: None,
            created_at_time: None,
        },
    );
    assert!(burn_tx.validate_transaction_fields().is_ok());

    // Test transfer transaction
    let transfer_tx = ICRC7Transaction::new(
        [0u8; 32],
        "7xfer".to_string(),
        1234567890,
        ICRC7TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            meta: None,
            memo: None,
            created_at_time: None,
        },
    );
    assert!(transfer_tx.validate_transaction_fields().is_ok());
}

#[test]
fn test_icrc37_transaction_validation() {
    // Test token approval transaction
    let approve_tx = ICRC37Transaction::new(
        [0u8; 32],
        "37approve".to_string(),
        1234567890,
        ICRC37TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            memo: None,
            created_at_time: None,
            spender: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            exp: None,
        },
    );
    assert!(approve_tx.validate_transaction_fields().is_ok());

    // Test collection approval transaction
    let approve_coll_tx = ICRC37Transaction::new(
        [0u8; 32],
        "37approve_coll".to_string(),
        1234567890,
        ICRC37TransactionData {
            tid: None,
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            memo: None,
            created_at_time: None,
            spender: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            exp: None,
        },
    );
    assert!(approve_coll_tx.validate_transaction_fields().is_ok());

    // Test token transfer transaction
    let transfer_tx = ICRC37Transaction::new(
        [0u8; 32],
        "37xfer".to_string(),
        1234567890,
        ICRC37TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            spender: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            exp: None,
        },
    );
    assert!(transfer_tx.validate_transaction_fields().is_ok());
}

#[test]
fn test_invalid_transactions() {
    // Test invalid ICRC1 mint transaction (with from field)
    let invalid_mint_tx = ICRC1Transaction::new(
        [0u8; 32],
        "1mint".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("mint".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(invalid_mint_tx.validate_transaction_fields().is_err());

    // Test invalid ICRC1 mint transaction (without to field)
    let invalid_mint_tx_no_to = ICRC1Transaction::new(
        [0u8; 32],
        "1mint".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("mint".to_string()),
            amount: Nat::from(1000u64),
            from: None,
            to: None,
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(invalid_mint_tx_no_to.validate_transaction_fields().is_err());

    // Test invalid ICRC1 burn transaction (with to field)
    let invalid_burn_tx = ICRC1Transaction::new(
        [0u8; 32],
        "1burn".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("burn".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(invalid_burn_tx.validate_transaction_fields().is_err());

    // Test invalid ICRC1 burn transaction (without from field)
    let invalid_burn_tx_no_from = ICRC1Transaction::new(
        [0u8; 32],
        "1burn".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("burn".to_string()),
            amount: Nat::from(1000u64),
            from: None,
            to: None,
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(invalid_burn_tx_no_from
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC1 transfer transaction (without from field)
    let invalid_transfer_tx_no_from = ICRC1Transaction::new(
        [0u8; 32],
        "1xfer".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("xfer".to_string()),
            amount: Nat::from(1000u64),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(invalid_transfer_tx_no_from
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC1 transfer transaction (without to field)
    let invalid_transfer_tx_no_to = ICRC1Transaction::new(
        [0u8; 32],
        "1xfer".to_string(),
        1234567890,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("xfer".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            memo: None,
            created_at_time: None,
            fee: None,
        },
    );
    assert!(invalid_transfer_tx_no_to
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC2 approve transaction (with to field)
    let invalid_approve_tx = ICRC2Transaction::new(
        [0u8; 32],
        "2approve".to_string(),
        1234567890,
        Some(Nat::from(1000u64)),
        ICRC2TransactionData {
            op: Some("approve".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            spender: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            expected_allowance: None,
            expires_at: None,
        },
    );
    assert!(invalid_approve_tx.validate_transaction_fields().is_err());

    // Test invalid ICRC2 approve transaction (without spender)
    let invalid_approve_tx_no_spender = ICRC2Transaction::new(
        [0u8; 32],
        "2approve".to_string(),
        1234567890,
        Some(Nat::from(1000u64)),
        ICRC2TransactionData {
            op: Some("approve".to_string()),
            amount: Nat::from(1000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            spender: None,
            memo: None,
            expected_allowance: None,
            expires_at: None,
        },
    );
    assert!(invalid_approve_tx_no_spender
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC2 transfer transaction (without from field)
    let invalid_transfer_tx_no_from = ICRC2Transaction::new(
        [0u8; 32],
        "2xfer".to_string(),
        1234567890,
        Some(Nat::from(1000u64)),
        ICRC2TransactionData {
            op: Some("xfer".to_string()),
            amount: Nat::from(1000u64),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            spender: None,
            memo: None,
            expected_allowance: None,
            expires_at: None,
        },
    );
    assert!(invalid_transfer_tx_no_from
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC7 mint transaction (with meta field)
    let invalid_mint_tx = ICRC7Transaction::new(
        [0u8; 32],
        "7mint".to_string(),
        1234567890,
        ICRC7TransactionData {
            tid: Some(Nat::from(1u64)),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            meta: Some(ICRC3Value::Text("meta".to_string())),
            memo: None,
            created_at_time: None,
        },
    );
    assert!(invalid_mint_tx.validate_transaction_fields().is_err());

    // Test invalid ICRC7 mint transaction (without tid)
    let invalid_mint_tx_no_tid = ICRC7Transaction::new(
        [0u8; 32],
        "7mint".to_string(),
        1234567890,
        ICRC7TransactionData {
            tid: None,
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            meta: None,
            memo: None,
            created_at_time: None,
        },
    );
    assert!(invalid_mint_tx_no_tid
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC7 burn transaction (with to field)
    let invalid_burn_tx = ICRC7Transaction::new(
        [0u8; 32],
        "7burn".to_string(),
        1234567890,
        ICRC7TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            meta: None,
            memo: None,
            created_at_time: None,
        },
    );
    assert!(invalid_burn_tx.validate_transaction_fields().is_err());

    // Test invalid ICRC7 update_token transaction (without meta)
    let invalid_update_tx_no_meta = ICRC7Transaction::new(
        [0u8; 32],
        "7update_token".to_string(),
        1234567890,
        ICRC7TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            meta: None,
            memo: None,
            created_at_time: None,
        },
    );
    assert!(invalid_update_tx_no_meta
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC37 token approval transaction (with to field)
    let invalid_approve_tx = ICRC37Transaction::new(
        [0u8; 32],
        "37approve".to_string(),
        1234567890,
        ICRC37TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            spender: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            exp: None,
        },
    );
    assert!(invalid_approve_tx.validate_transaction_fields().is_err());

    // Test invalid ICRC37 token approval transaction (without spender)
    let invalid_approve_tx_no_spender = ICRC37Transaction::new(
        [0u8; 32],
        "37approve".to_string(),
        1234567890,
        ICRC37TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            memo: None,
            created_at_time: None,
            spender: None,
            exp: None,
        },
    );
    assert!(invalid_approve_tx_no_spender
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC37 collection approval transaction (with tid)
    let invalid_coll_approve_tx = ICRC37Transaction::new(
        [0u8; 32],
        "37approve_coll".to_string(),
        1234567890,
        ICRC37TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: None,
            memo: None,
            created_at_time: None,
            spender: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            exp: None,
        },
    );
    assert!(invalid_coll_approve_tx
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC37 token transfer transaction (without spender)
    let invalid_transfer_tx_no_spender = ICRC37Transaction::new(
        [0u8; 32],
        "37xfer".to_string(),
        1234567890,
        ICRC37TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            spender: None,
            exp: None,
        },
    );
    assert!(invalid_transfer_tx_no_spender
        .validate_transaction_fields()
        .is_err());

    // Test invalid ICRC37 token transfer transaction (with exp)
    let invalid_transfer_tx_with_exp = ICRC37Transaction::new(
        [0u8; 32],
        "37xfer".to_string(),
        1234567890,
        ICRC37TransactionData {
            tid: Some(Nat::from(1u64)),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: None,
            spender: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            exp: Some(Nat::from(1234567890u64)),
        },
    );
    assert!(invalid_transfer_tx_with_exp
        .validate_transaction_fields()
        .is_err());
}
