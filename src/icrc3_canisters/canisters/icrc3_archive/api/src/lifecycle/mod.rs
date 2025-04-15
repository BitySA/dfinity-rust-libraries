use candid::CandidType;
use init::InitArgs;
use post_upgrade::UpgradeArgs;
use serde::{Deserialize, Serialize};

pub mod init;
pub mod post_upgrade;

#[derive(Deserialize, Serialize, CandidType, Debug, Clone)]
pub enum BlockType {
    Default,
    ICRC1,
    // TODO: add other block types
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum Args {
    Init(InitArgs),
    Upgrade(UpgradeArgs),
}
