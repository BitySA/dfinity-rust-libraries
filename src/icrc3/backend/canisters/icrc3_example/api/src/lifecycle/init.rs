use bity_ic_types::BuildVersion;
use candid::{CandidType, Principal};
use icrc3::config::ICRC3Config;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct InitArgs {
    pub test_mode: bool,
    pub version: BuildVersion,
    pub commit_hash: String,
    pub authorized_principals: Vec<Principal>,
    pub icrc3_config: ICRC3Config,
}
