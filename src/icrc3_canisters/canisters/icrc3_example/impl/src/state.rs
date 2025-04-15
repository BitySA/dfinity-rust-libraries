use crate::utils::trace;

use bity_ic_canister_state_macros::canister_state;
use bity_ic_icrc3::transaction::{Hash, TransactionType};
use bity_ic_types::TimestampSeconds;
use bity_ic_types::{BuildVersion, Cycles, TimestampMillis};
use bity_ic_utils::env::{CanisterEnv, Environment};
use bity_ic_utils::memory::MemorySize;
use candid::Nat;
use candid::{CandidType, Principal};
use icrc3_macros::icrc3_state;
use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

icrc3_state!();
canister_state!(RuntimeState);

#[derive(Serialize, Deserialize)]
pub struct RuntimeState {
    pub env: CanisterEnv,
    pub data: Data,
}

impl RuntimeState {
    pub fn new(env: CanisterEnv, data: Data) -> Self {
        RuntimeState { env, data }
    }

    pub fn is_caller_authorized(&self) -> bool {
        self.data.authorized_principals.contains(&self.env.caller())
    }

    pub fn metrics(&self) -> Metrics {
        Metrics {
            canister_info: CanisterInfo {
                test_mode: self.env.is_test_mode(),
                now: self.env.now(),
                version: self.env.version(),
                commit_hash: self.env.commit_hash().to_string(),
                memory_used: MemorySize::used(),
                cycles_balance: self.env.cycles_balance(),
            },
            authorized_principals: self.data.authorized_principals.iter().cloned().collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub authorized_principals: HashSet<Principal>,
}

impl Data {
    #[allow(clippy::too_many_arguments)]
    pub fn new(authorized_principals: Vec<Principal>) -> Self {
        Self {
            authorized_principals: authorized_principals.clone().into_iter().collect(),
        }
    }

    pub fn add_authorized_principals(&mut self, new_principals: Vec<Principal>) {
        for principal in new_principals {
            self.authorized_principals.insert(principal);
        }
    }

    pub fn remove_authorized_principals(&mut self, principals_to_remove: Vec<Principal>) {
        for principal in principals_to_remove {
            self.authorized_principals.remove(&principal);
        }
    }

    pub fn create_fake_transaction(&self) -> FakeTransaction {
        trace(&format!("create_fake_transaction"));
        FakeTransaction::random()
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct FakeTransaction {
    pub phash: String,
    pub btype: String,
    pub timestamp: u64,
    pub sender: Principal,
    pub recipient: Principal,
}

impl Default for FakeTransaction {
    fn default() -> Self {
        Self {
            phash: "".to_string(),
            btype: "".to_string(),
            timestamp: 0,
            sender: Principal::anonymous(),
            recipient: Principal::anonymous(),
        }
    }
}

impl FakeTransaction {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn random() -> Self {
        trace(&format!("random fake transaction"));
        let now = ic_cdk::api::time();
        Self {
            phash: format!("phash_{}", now),
            btype: format!("btype_{}", now),
            timestamp: now,
            sender: Principal::anonymous(),
            recipient: Principal::anonymous(),
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

    fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(self.phash.as_bytes());
        hasher.update(self.btype.as_bytes());
        hasher.update(self.timestamp.to_le_bytes().as_slice());
        hasher.update(self.sender.as_slice());
        hasher.update(self.recipient.as_slice());
        hasher.finalize().into()
    }
}

impl From<FakeTransaction> for ICRC3Value {
    fn from(tx: FakeTransaction) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "phash".to_string(),
            ICRC3Value::Blob(ByteBuf::from(tx.phash.as_bytes())),
        );
        map.insert("btype".to_string(), ICRC3Value::Text(tx.btype));
        map.insert(
            "timestamp".to_string(),
            ICRC3Value::Nat(Nat::from(tx.timestamp)),
        );
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

#[derive(CandidType, Serialize)]
pub struct Metrics {
    pub canister_info: CanisterInfo,
    pub authorized_principals: Vec<Principal>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct CanisterInfo {
    pub now: TimestampMillis,
    pub test_mode: bool,
    pub version: BuildVersion,
    pub commit_hash: String,
    pub memory_used: MemorySize,
    pub cycles_balance: Cycles,
}
