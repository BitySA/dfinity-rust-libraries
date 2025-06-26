use crate::utils::trace;

use bity_ic_canister_state_macros::canister_state;
use bity_ic_icrc3::transaction::TransactionType;
use bity_ic_icrc3_macros::icrc3_state;
use bity_ic_types::{BuildVersion, Cycles, TimestampMillis};
use bity_ic_utils::env::{CanisterEnv, Environment};
use bity_ic_utils::memory::MemorySize;
use candid::{CandidType, Principal};
use icrc3_example_api::types::FakeTransaction;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
