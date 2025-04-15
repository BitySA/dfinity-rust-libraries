use crate::types::archive::Archive;
use bity_ic_canister_state_macros::canister_state;
use bity_ic_icrc3_archive_api::{archive_config::ArchiveConfig, lifecycle::BlockType};
use bity_ic_types::{BuildVersion, TimestampMillis};
use bity_ic_utils::{
    env::{CanisterEnv, Environment},
    memory::MemorySize,
};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

canister_state!(RuntimeState);

#[derive(Default, Serialize, Deserialize)]
pub struct RuntimeState {
    pub env: CanisterEnv,
    pub data: Data,
}

impl RuntimeState {
    pub fn new(env: CanisterEnv, data: Data) -> Self {
        Self { env, data }
    }

    pub fn metrics(&self) -> Metrics {
        Metrics {
            canister_info: CanisterInfo {
                now: self.env.now(),
                test_mode: self.env.is_test_mode(),
                memory_used: MemorySize::used(),
                cycles_balance_in_tc: self.env.cycles_balance_in_tc(),
                version: self.env.version(),
                commit_hash: self.env.commit_hash().to_string(),
            },
        }
    }

    pub fn is_caller_authorized(&self) -> bool {
        let caller = self.env.caller();
        self.data.authorized_principals.contains(&caller)
    }

    pub fn is_caller_main_canister(&self) -> bool {
        let caller = self.env.caller();
        self.data.master_canister_id == caller
    }
}

#[derive(CandidType, Serialize)]
pub struct Metrics {
    pub canister_info: CanisterInfo,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct CanisterInfo {
    pub now: TimestampMillis,
    pub test_mode: bool,
    pub version: BuildVersion,
    pub commit_hash: String,
    pub memory_used: MemorySize,
    pub cycles_balance_in_tc: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub archive: Archive,
    pub authorized_principals: Vec<Principal>,
    pub master_canister_id: Principal,
    pub block_type: BlockType,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            archive: Archive::new(ArchiveConfig::default()),
            authorized_principals: vec![],
            master_canister_id: Principal::anonymous(),
            block_type: BlockType::Default,
        }
    }
}

impl Data {
    pub fn new(
        archive_config: ArchiveConfig,
        authorized_principals: Vec<Principal>,
        master_canister_id: Principal,
        block_type: BlockType,
    ) -> Self {
        Self {
            archive: Archive::new(archive_config),
            authorized_principals,
            master_canister_id,
            block_type,
        }
    }
}
