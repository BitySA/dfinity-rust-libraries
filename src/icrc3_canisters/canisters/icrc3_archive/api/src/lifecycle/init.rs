use crate::archive_config::ArchiveConfig;
use crate::lifecycle::BlockType;

use bity_ic_types::BuildVersion;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, CandidType, Debug, Clone)]
pub struct InitArgs {
    pub test_mode: bool,
    pub version: BuildVersion,
    pub commit_hash: String,
    pub authorized_principals: Vec<Principal>,
    pub archive_config: ArchiveConfig,
    pub master_canister_id: Principal,
    pub block_type: BlockType,
}

impl Default for InitArgs {
    fn default() -> Self {
        Self {
            test_mode: false,
            version: BuildVersion::default(),
            commit_hash: String::new(),
            authorized_principals: Vec::new(),
            archive_config: ArchiveConfig::default(),
            master_canister_id: Principal::anonymous(),
            block_type: BlockType::Default,
        }
    }
}
