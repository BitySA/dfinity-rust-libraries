use crate::memory::{get_block_log_data_memory, get_block_log_index_memory, VM};

use bity_ic_icrc3_archive_api::{
    archive_config::ArchiveConfig, types::encoded_blocks::EncodedBlock,
};
use candid::Nat;
use ic_cdk::stable::stable_size;
use ic_cdk::stable::WASM_PAGE_SIZE_IN_BYTES;
use ic_stable_structures::{StableLog, Storable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Archive {
    #[serde(skip, default = "init_archive_map")]
    pub archive: StableLog<EncodedBlock, VM, VM>, // NOTE: block id, block
    pub archive_config: ArchiveConfig,
}

impl Default for Archive {
    fn default() -> Self {
        Self {
            archive: init_archive_map(),
            archive_config: ArchiveConfig::default(),
        }
    }
}

impl Archive {
    pub fn new(archive_config: ArchiveConfig) -> Self {
        Self {
            archive: init_archive_map(),
            archive_config,
        }
    }
}

fn init_archive_map() -> StableLog<EncodedBlock, VM, VM> {
    StableLog::init(get_block_log_index_memory(), get_block_log_data_memory())
        .expect("failed to initialize stable log")
}

impl Archive {
    pub fn get_archive_size_bytes(&self) -> usize {
        let num_pages = stable_size();
        (num_pages as usize) * (WASM_PAGE_SIZE_IN_BYTES as usize)
    }

    pub fn remaining_capacity(&self) -> Nat {
        let current_archive_size: u128 = self
            .archive
            .iter()
            .map(|entry| entry.to_bytes().len() as u128)
            .sum();
        self.archive_config
            .max_memory_size_bytes
            .saturating_sub(current_archive_size)
            .into()
    }

    pub fn get_len(&self) -> u64 {
        self.archive.len()
    }

    pub fn insert_blocks(&mut self, new_blocks: Vec<EncodedBlock>) -> Result<(), String> {
        for block in new_blocks {
            self.archive
                .append(&block)
                .unwrap_or_else(|_| ic_cdk::api::trap("no space left"));
        }

        Ok(())
    }

    pub fn get_blocks_range(&self, start: u64, length: u64) -> Vec<EncodedBlock> {
        let length = length.min(self.archive_config.get_max_blocks_per_response());
        self.archive
            .iter()
            .skip(start as usize)
            .take(length as usize)
            .collect()
    }
}
