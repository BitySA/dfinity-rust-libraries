use candid::Nat;
use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use icrc_ledger_types::icrc1::account::Account;
use std::str::FromStr;

use bity_ic_icrc3::transaction::{ICRC1Transaction, ICRC1TransactionData};

#[test]
fn test_icrc3_hashing_nat() {
    let value = ICRC3Value::Nat(Nat::from(42u64));
    let hash = value.hash();
    let expected_hash: [u8; 32] =
        hex::decode("684888c0ebb17f374298b65ee2807526c066094c701bcc7ebbe1c1095f494fc1")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(hash, expected_hash);
}

#[test]
fn test_icrc3_hashing_int() {
    let value = ICRC3Value::Int(candid::Int::from(-42i64));
    let hash = value.hash();
    let expected_hash: [u8; 32] =
        hex::decode("de5a6f78116eca62d7fc5ce159d23ae6b889b365a1739ad2cf36f925a140d0cc")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(hash, expected_hash);
}

#[test]
fn test_icrc3_hashing_text() {
    let value = ICRC3Value::Text("Hello, World!".to_string());
    let hash = value.hash();
    let expected_hash: [u8; 32] =
        hex::decode("dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(hash, expected_hash);
}

#[test]
fn test_icrc3_hashing_blob() {
    let value = ICRC3Value::Blob(serde_bytes::ByteBuf::from(vec![0x01, 0x02, 0x03, 0x04]));
    let hash = value.hash();
    let expected_hash: [u8; 32] =
        hex::decode("9f64a747e1b97f131fabb6b447296c9b6f0201e79fb3c5356e6c77e89b6a806a")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(hash, expected_hash);
}

#[test]
fn test_icrc3_hashing_array() {
    let value = ICRC3Value::Array(vec![
        ICRC3Value::Nat(Nat::from(3u64)),
        ICRC3Value::Text("foo".to_string()),
        ICRC3Value::Blob(serde_bytes::ByteBuf::from(vec![0x05, 0x06])),
    ]);
    let hash = value.hash();
    let expected_hash: [u8; 32] =
        hex::decode("514a04011caa503990d446b7dec5d79e19c221ae607fb08b2848c67734d468d6")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(hash, expected_hash);
}

#[test]
fn test_icrc3_hashing_map() {
    let mut map = std::collections::BTreeMap::new();
    map.insert(
        "from".to_string(),
        ICRC3Value::Blob(serde_bytes::ByteBuf::from(vec![
            0x00, 0xab, 0xcd, 0xef, 0x00, 0x12, 0x34, 0x00, 0x56, 0x78, 0x9a, 0x00, 0xbc, 0xde,
            0xf0, 0x00, 0x01, 0x23, 0x45, 0x67, 0x89, 0x00, 0xab, 0xcd, 0xef, 0x01,
        ])),
    );
    map.insert(
        "to".to_string(),
        ICRC3Value::Blob(serde_bytes::ByteBuf::from(vec![
            0x00, 0xab, 0x0d, 0xef, 0x00, 0x12, 0x34, 0x00, 0x56, 0x78, 0x9a, 0x00, 0xbc, 0xde,
            0xf0, 0x00, 0x01, 0x23, 0x45, 0x67, 0x89, 0x00, 0xab, 0xcd, 0xef, 0x01,
        ])),
    );
    map.insert("amount".to_string(), ICRC3Value::Nat(Nat::from(42u64)));
    map.insert(
        "created_at".to_string(),
        ICRC3Value::Nat(Nat::from(1699218263u64)),
    );
    map.insert("memo".to_string(), ICRC3Value::Nat(Nat::from(0u64)));

    let value = ICRC3Value::Map(map);
    let hash = value.hash();
    let expected_hash: [u8; 32] =
        hex::decode("c56ece650e1de4269c5bdeff7875949e3e2033f85b2d193c2ff4f7f78bdcfc75")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(hash, expected_hash);
}

#[test]
fn test_icrc1_mint_block_hash() {
    let mint_tx = ICRC1Transaction::new(
        "1mint".to_string(),
        1699218263,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("mint".to_string()),
            amount: Nat::from(1000000u64),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("2vxsx-fae").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: Some(Nat::from(1699218263u64)),
            fee: Some(Nat::from(1000u64)),
        },
    );

    let block: ICRC3Value = mint_tx.into();
    let hash = block.hash();

    assert_eq!(hash.len(), 32);
}

#[test]
fn test_icrc1_transfer_block_hash() {
    let transfer_tx = ICRC1Transaction::new(
        "1xfer".to_string(),
        1699218263,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("xfer".to_string()),
            amount: Nat::from(500000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("2vxsx-fae").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("2vxsx-fae").unwrap(),
                subaccount: None,
            }),
            memo: Some(serde_bytes::ByteBuf::from(vec![0x01, 0x02, 0x03])),
            created_at_time: Some(Nat::from(1699218263u64)),
            fee: Some(Nat::from(1000u64)),
        },
    );

    let block: ICRC3Value = transfer_tx.into();
    let hash = block.hash();

    assert_eq!(hash.len(), 32);
}

#[test]
fn test_icrc1_mint_block_expected_hash() {
    let mint_tx = ICRC1Transaction::new(
        "1mint".to_string(),
        1699218263,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("mint".to_string()),
            amount: Nat::from(1000000u64),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: Some(Nat::from(1699218263u64)),
            fee: Some(Nat::from(1000u64)),
        },
    );

    let block: ICRC3Value = mint_tx.into();
    let hash = block.hash();
    let expected_hash: [u8; 32] =
        hex::decode("206d223fd38d2a3b507d728c2f9a42e122c8d27d06ba1b9d84b60a8d88f58700")
            .unwrap()
            .try_into()
            .unwrap();

    assert_eq!(hash, expected_hash);
}

#[test]
fn test_icrc1_transfer_block_expected_hash() {
    let transfer_tx = ICRC1Transaction::new(
        "1xfer".to_string(),
        1699218263,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("xfer".to_string()),
            amount: Nat::from(500000u64),
            from: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            to: Some(Account {
                owner: candid::Principal::from_str("2vxsx-fae").unwrap(),
                subaccount: None,
            }),
            memo: Some(serde_bytes::ByteBuf::from(vec![0x00, 0x01, 0x02])),
            created_at_time: Some(Nat::from(1699218263u64)),
            fee: Some(Nat::from(1000u64)),
        },
    );

    let block: ICRC3Value = transfer_tx.into();
    let hash = block.hash();
    let expected_hash: [u8; 32] =
        hex::decode("2d846b6a7d5a42a6a8640327925c2fd1c3361f13289565846c8ee72b367dc75b")
            .unwrap()
            .try_into()
            .unwrap();

    assert_eq!(hash, expected_hash);
}

#[test]
fn test_block_hashing_deterministic() {
    let mint_tx = ICRC1Transaction::new(
        "1mint".to_string(),
        1699218263,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("mint".to_string()),
            amount: Nat::from(1000000u64),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: Some(Nat::from(1699218263u64)),
            fee: Some(Nat::from(1000u64)),
        },
    );

    let block1: ICRC3Value = mint_tx.clone().into();
    let block2: ICRC3Value = mint_tx.into();

    let hash1 = block1.hash();
    let hash2 = block2.hash();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_different_blocks_different_hashes() {
    let mint_tx1 = ICRC1Transaction::new(
        "1mint".to_string(),
        1699218263,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("mint".to_string()),
            amount: Nat::from(1000000u64),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: Some(Nat::from(1699218263u64)),
            fee: Some(Nat::from(1000u64)),
        },
    );

    let mint_tx2 = ICRC1Transaction::new(
        "1mint".to_string(),
        1699218264,
        Nat::from(1000u64),
        ICRC1TransactionData {
            op: Some("mint".to_string()),
            amount: Nat::from(1000000u64),
            from: None,
            to: Some(Account {
                owner: candid::Principal::from_str("aaaaa-aa").unwrap(),
                subaccount: None,
            }),
            memo: None,
            created_at_time: Some(Nat::from(1699218264u64)),
            fee: Some(Nat::from(1000u64)),
        },
    );

    let block1: ICRC3Value = mint_tx1.into();
    let block2: ICRC3Value = mint_tx2.into();

    let hash1 = block1.hash();
    let hash2 = block2.hash();

    assert_ne!(hash1, hash2);
}
