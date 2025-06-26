use bity_ic_icrc3::transaction::TransactionType;
use bity_ic_types::TimestampSeconds;

use candid::{CandidType, Nat, Principal};
use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct FakeTransaction {
    pub btype: String,
    pub timestamp: u64,
    pub tx: FakeTransactionData,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct FakeTransactionData {
    pub sender: Principal,
    pub recipient: Principal,
}

impl Default for FakeTransaction {
    fn default() -> Self {
        Self {
            btype: "".to_string(),
            timestamp: 0,
            tx: FakeTransactionData {
                sender: Principal::anonymous(),
                recipient: Principal::anonymous(),
            },
        }
    }
}

impl FakeTransaction {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn random() -> Self {
        let now = ic_cdk::api::time();
        Self {
            btype: "btype_test".to_string(),
            timestamp: now,
            tx: FakeTransactionData {
                sender: Principal::anonymous(),
                recipient: Principal::anonymous(),
            },
        }
    }
}

impl TransactionType for FakeTransaction {
    fn validate_transaction_fields(&self) -> Result<(), String> {
        Ok(())
    }

    fn timestamp(&self) -> Option<TimestampSeconds> {
        Some(self.timestamp)
    }

    fn block_type(&self) -> String {
        self.btype.clone()
    }

    fn tx(&self) -> ICRC3Value {
        self.clone().into()
    }
}

impl From<FakeTransactionData> for ICRC3Value {
    fn from(tx: FakeTransactionData) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "sender".to_string(),
            ICRC3Value::Text(tx.sender.to_string()),
        );
        map.insert(
            "recipient".to_string(),
            ICRC3Value::Text(tx.recipient.to_string()),
        );
        ICRC3Value::Map(map)
    }
}

impl From<FakeTransaction> for ICRC3Value {
    fn from(tx: FakeTransaction) -> Self {
        let mut map = BTreeMap::new();
        map.insert("btype".to_string(), ICRC3Value::Text(tx.btype));
        map.insert(
            "timestamp".to_string(),
            ICRC3Value::Nat(Nat::from(tx.timestamp)),
        );
        map.insert("tx".to_string(), tx.tx.into());
        ICRC3Value::Map(map)
    }
}
