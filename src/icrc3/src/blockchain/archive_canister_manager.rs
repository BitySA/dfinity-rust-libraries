use crate::blockchain::archive_canister::ArchiveCanister;
use crate::utils::trace;

use bity_ic_icrc3_archive_api::{
    archive_config::ArchiveConfig, lifecycle::BlockType, types::encoded_blocks::EncodedBlock,
};
use bity_ic_subcanister_manager::{Canister, SubCanisterManager};
use bity_ic_types::BuildVersion;
use candid::Principal;
use canfund::manager::options::{CyclesThreshold, FundManagerOptions, FundStrategy};
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const ARCHIVE_WASM: &[u8] = include_bytes!("../../wasm/icrc3_archive_canister.wasm.gz");
const DEFAULT_INITIAL_CYCLES: u128 = 100_000_000_000_000_000;
const DEFAULT_RESERVED_CYCLES: u128 = 100_000_000_000_000_000;
const DEFAULT_MIN_CYCLES: u128 = 1_000_000_000_000;
const DEFAULT_FUND_CYCLES: u128 = 2_000_000_000_000;
const DEFAULT_INTERVAL_SECS: u64 = 60;

/// Manages multiple archive canisters for storing blockchain data.
///
/// This struct handles the creation, management, and coordination of multiple
/// archive canisters, including block distribution and space management.
#[derive(Serialize, Deserialize, Clone)]
pub struct ArchiveCanisterManager {
    /// The sub-canister manager for handling canister lifecycle
    pub sub_canister_manager: SubCanisterManager<ArchiveCanister>,
    /// Arguments used for initializing new canisters
    pub init_args: bity_ic_icrc3_archive_api::init::InitArgs,
    /// Arguments used for upgrading existing canisters
    pub upgrade_args: bity_ic_icrc3_archive_api::post_upgrade::UpgradeArgs,
    /// Mapping of block IDs to canister IDs
    pub canisters_by_block_id: Vec<(BlockIndex, Principal)>,
}

impl Default for ArchiveCanisterManager {
    /// Creates a default ArchiveCanisterManager with default settings.
    fn default() -> Self {
        let this_canister_id = ic_cdk::api::canister_self();
        let version = bity_ic_icrc3_archive_api::VERSION.to_string();
        let mut hasher = Sha256::new();
        hasher.update(version.as_bytes());
        let commit_hash = format!("{:x}", hasher.finalize());

        Self {
            sub_canister_manager: SubCanisterManager::new(
                ic_cdk::api::canister_self(),
                HashMap::new(),
                vec![ic_cdk::api::canister_self()],
                vec![ic_cdk::api::canister_self()],
                DEFAULT_INITIAL_CYCLES,
                DEFAULT_RESERVED_CYCLES,
                false,
                commit_hash.clone(),
                ARCHIVE_WASM.to_vec(),
                FundManagerOptions::new()
                    .with_interval_secs(DEFAULT_INTERVAL_SECS)
                    .with_strategy(FundStrategy::BelowThreshold(
                        CyclesThreshold::new()
                            .with_min_cycles(DEFAULT_MIN_CYCLES)
                            .with_fund_cycles(DEFAULT_FUND_CYCLES),
                    )),
            ),
            init_args: bity_ic_icrc3_archive_api::init::InitArgs {
                test_mode: false,
                version: bity_ic_icrc3_archive_api::VERSION
                    .parse::<BuildVersion>()
                    .unwrap(),
                commit_hash: commit_hash.clone(),
                authorized_principals: vec![this_canister_id],
                archive_config: ArchiveConfig::default(),
                master_canister_id: this_canister_id,
                block_type: BlockType::Default,
            },
            upgrade_args: bity_ic_icrc3_archive_api::post_upgrade::UpgradeArgs {
                version: bity_ic_icrc3_archive_api::VERSION
                    .parse::<BuildVersion>()
                    .unwrap(),
                commit_hash,
                block_type: BlockType::Default,
            },
            canisters_by_block_id: vec![],
        }
    }
}

impl ArchiveCanisterManager {
    /// Creates a new ArchiveCanisterManager with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `init_args` - Arguments for initializing new canisters
    /// * `upgrade_args` - Arguments for upgrading existing canisters
    /// * `sub_canisters` - Initial set of sub-canisters
    /// * `controllers` - List of controller principals
    /// * `authorized_principal` - List of authorized principals
    /// * `initial_cycles` - Initial number of cycles for new canisters
    /// * `reserved_cycles` - Reserved number of cycles
    /// * `wasm` - The WASM module for the archive canister
    pub fn new(
        init_args: bity_ic_icrc3_archive_api::init::InitArgs,
        upgrade_args: bity_ic_icrc3_archive_api::post_upgrade::UpgradeArgs,
        sub_canisters: HashMap<Principal, Box<ArchiveCanister>>,
        controllers: Vec<Principal>,
        authorized_principal: Vec<Principal>,
        initial_cycles: u128,
        reserved_cycles: u128,
        wasm: Vec<u8>,
        interval_secs: Option<u64>,
        min_cycles: Option<u128>,
        fund_cycles: Option<u128>,
    ) -> Self {
        let interval_secs = interval_secs.unwrap_or(DEFAULT_INTERVAL_SECS);
        let min_cycles = min_cycles.unwrap_or(DEFAULT_MIN_CYCLES);
        let fund_cycles = fund_cycles.unwrap_or(DEFAULT_FUND_CYCLES);

        Self {
            sub_canister_manager: SubCanisterManager::new(
                ic_cdk::api::canister_self(),
                sub_canisters,
                controllers,
                authorized_principal,
                initial_cycles,
                reserved_cycles,
                init_args.test_mode,
                init_args.commit_hash.clone(),
                wasm,
                FundManagerOptions::new()
                    .with_interval_secs(interval_secs)
                    .with_strategy(FundStrategy::BelowThreshold(
                        CyclesThreshold::new()
                            .with_min_cycles(min_cycles)
                            .with_fund_cycles(fund_cycles),
                    )),
            ),
            init_args,
            upgrade_args,
            canisters_by_block_id: vec![],
        }
    }

    /// Inserts a block into an appropriate archive canister.
    ///
    /// This method will:
    /// 1. Try to insert the block into existing canisters
    /// 2. Create a new canister if no existing canister has space
    ///
    /// # Arguments
    ///
    /// * `block_id` - The ID of the block to insert
    /// * `block` - The encoded block to insert
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the block was successfully inserted
    /// * `Err(String)` if the insertion failed
    pub async fn insert_block(
        &mut self,
        block_id: BlockIndex,
        block: EncodedBlock,
    ) -> Result<(), String> {
        trace(format!("Starting to insert single block"));
        trace(format!("insert_block: block_id: {}", block_id));

        for (_, canister) in self.sub_canister_manager.sub_canisters.iter_mut() {
            trace(format!(
                "Checking available space in canister {:?}...",
                canister.canister_id()
            ));

            match canister.insert_blocks(vec![block.clone()]).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    if e.as_str().contains("no space left") {
                        continue;
                    } else {
                        return Err(format!("Failed to insert block into canister: {}", e));
                    }
                }
            }
        }

        let mut init_args = self.init_args.clone();
        init_args.archive_config.block_offset = block_id;

        // If no canister had enough space, create a new canister
        match self
            .sub_canister_manager
            .create_canister(bity_ic_icrc3_archive_api::Args::Init(init_args))
            .await
        {
            Ok(mut new_canister) => {
                trace(format!("Creating new canister to store block."));
                trace(format!(
                    "Creating new canister to store block. block_id: {}",
                    block_id
                ));
                let canister_id = new_canister.canister_id();
                self.canisters_by_block_id.push((block_id, canister_id));

                if let Err(e) = new_canister.insert_blocks(vec![block.clone()]).await {
                    trace(format!("Failed to insert block into new canister: {}", e));
                    return Err(format!("Failed to insert block into new canister: {}", e));
                }

                Ok(())
            }
            Err(e) => {
                trace(format!("Failed to create a new canister: {:?}", e));
                Err(format!("Failed to create a new canister: {:?}", e))
            }
        }
    }

    /// Returns a list of all installed archive canisters.
    pub fn get_subcanisters_installed(&self) -> Vec<ArchiveCanister> {
        self.sub_canister_manager
            .list_canisters()
            .into_iter()
            .filter_map(|canister| {
                if canister.state() == bity_ic_subcanister_manager::CanisterState::Installed {
                    canister.as_any().downcast_ref::<ArchiveCanister>().cloned()
                } else {
                    None
                }
            })
            .collect()
    }

    /// Gets the canister ID for a specific block ID.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The ID of the block to look up
    ///
    /// # Returns
    ///
    /// * `Ok(Principal)` containing the canister ID
    /// * `Err(String)` if no canister is found for the block ID
    pub fn get_canister_id_by_block_id(&self, block_id: BlockIndex) -> Result<Principal, String> {
        trace(format!(
            "get_canister_id_by_block_id: block_id: {}, canisters_by_block_id: {:?}",
            block_id, self.canisters_by_block_id
        ));
        trace(format!(
            "get_canister_id_by_block_id: canisters_by_block_id: {:?}",
            self.canisters_by_block_id
        ));

        match self
            .canisters_by_block_id
            .iter()
            .rev()
            .find(|(start_id, _)| *start_id <= block_id)
        {
            Some((_, canister_id)) => Ok(canister_id.clone()),
            None => Err(format!("Canister not found for block id: {}", block_id)),
        }
    }
}
