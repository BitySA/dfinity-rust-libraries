use crate::blockchain::archive_canister_manager::ArchiveCanisterManager;
use crate::memory::{get_block_log_data_memory, VM};
use crate::utils::trace;

use bity_ic_icrc3_archive_api::types::{
    block_interface::{Block, BlockIndex},
    encoded_blocks::EncodedBlock,
    hash::HashOf,
};
use candid::Principal;
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// The default maximum size of local stable memory for transactions before archiving.
const DEFAULT_MAX_TX_LOCAL_STABLE_MEMORY_SIZE_BYTES: u128 = 100 * 1024 * 1024 * 1024; // 100GB
const TRESHOLD_FOR_ARCHIVING: usize = 100_000;
const BATCH_SIZE_FOR_ARCHIVING: usize = 25;

fn init_archive_map() -> StableBTreeMap<BlockIndex, EncodedBlock, VM> {
    let memory = get_block_log_data_memory();
    StableBTreeMap::init(memory)
}

/// The core blockchain implementation for ICRC3.
///
/// This struct manages the blockchain state, including:
/// * Local transaction storage
/// * Block archiving
/// * Chain length tracking
/// * Hash management
pub struct Blockchain {
    /// Manager for archive canisters
    pub archive_canister_manager: Arc<RwLock<ArchiveCanisterManager>>,
    /// local archive, used to avoid popping new canisters.
    pub local_archive: StableBTreeMap<BlockIndex, EncodedBlock, VM>,
    /// Hash of the last block in the chain
    pub last_hash: Option<HashOf<EncodedBlock>>,
    /// Size of the local archive
    pub local_archive_size: usize,
    /// Timestamp of the last block
    pub last_timestamp: u128,
    /// Number of blocks that have been archived
    pub archived_chain_length: usize,
    /// Time to live for non-archived transactions
    pub ttl_for_non_archived_transactions: Duration,
    /// Maximum size of local stable memory for transactions before archiving.
    /// If None, transactions are directly archived in a new canister, not stored in the local stable memory.
    pub max_tx_local_stable_memory_size_bytes: Option<u128>,
    /// Threshold for archiving blocks to the external archive canister
    pub threshold_for_archiving_to_external_archive: Option<usize>,
}

impl Blockchain {
    pub fn new(
        archive_canister_manager: ArchiveCanisterManager,
        last_hash: Option<HashOf<EncodedBlock>>,
        last_timestamp: u128,
        ttl_for_non_archived_transactions: Duration,
        max_tx_local_stable_memory_size_bytes: Option<u128>,
        threshold_for_archiving_to_external_archive: Option<usize>,
    ) -> Self {
        Self {
            archive_canister_manager: Arc::new(RwLock::new(archive_canister_manager)),
            local_archive: init_archive_map(),
            local_archive_size: 0,
            last_hash,
            last_timestamp,
            archived_chain_length: 0,
            ttl_for_non_archived_transactions,
            max_tx_local_stable_memory_size_bytes,
            threshold_for_archiving_to_external_archive,
        }
    }
}

impl Default for Blockchain {
    /// Creates a new Blockchain with default settings.
    fn default() -> Self {
        Self {
            archive_canister_manager: Arc::new(RwLock::new(ArchiveCanisterManager::default())),
            local_archive: init_archive_map(),
            local_archive_size: 0usize as usize,
            last_hash: None,
            last_timestamp: 0,
            archived_chain_length: 0,
            ttl_for_non_archived_transactions: Duration::from_secs(120),
            max_tx_local_stable_memory_size_bytes: None,
            threshold_for_archiving_to_external_archive: None,
        }
    }
}

impl Serialize for Blockchain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let archive_manager = self
            .archive_canister_manager
            .read()
            .map_err(|_| serde::ser::Error::custom("Failed to read archive_canister_manager"))?;

        (
            &*archive_manager,
            &self.local_archive_size,
            &self.last_hash,
            &self.last_timestamp,
            &self.archived_chain_length,
            &self.ttl_for_non_archived_transactions,
            &self.max_tx_local_stable_memory_size_bytes,
            &self.threshold_for_archiving_to_external_archive,
        )
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Blockchain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (
            archive_manager,
            local_archive_size,
            last_hash,
            last_timestamp,
            archived_chain_length,
            ttl_for_non_archived_transactions,
            max_tx_local_stable_memory_size_bytes,
            threshold_for_archiving_to_external_archive,
        ) = <(
            ArchiveCanisterManager,
            usize,
            Option<HashOf<EncodedBlock>>,
            u128,
            usize,
            Duration,
            Option<u128>,
            Option<usize>,
        )>::deserialize(deserializer)?;

        Ok(Blockchain {
            archive_canister_manager: Arc::new(RwLock::new(archive_manager)),
            local_archive: init_archive_map(),
            archived_chain_length,
            local_archive_size,
            last_hash,
            last_timestamp,
            ttl_for_non_archived_transactions,
            max_tx_local_stable_memory_size_bytes,
            threshold_for_archiving_to_external_archive,
        })
    }
}

impl Blockchain {
    /// Adds a new block to the blockchain.
    ///
    /// This method:
    /// 1. Validates the block's parent hash and timestamp
    /// 2. Archives blocks if local storage is full
    /// 3. Adds the block to local storage
    /// 4. Triggers archiving if the threshold is reached
    ///
    /// # Arguments
    ///
    /// * `block` - The block to add
    ///
    /// # Returns
    ///
    /// * `Ok(BlockIndex)` containing the new block's index
    /// * `Err(String)` if the block could not be added
    pub fn add_block<B>(&mut self, block: B) -> Result<BlockIndex, String>
    where
        B: Block + Clone,
    {
        let block_clone = block.clone();
        if block_clone.parent_hash() != self.last_hash {
            trace("add_block error: Cannot apply block because its parent hash doesn't match.");
            return Err("Cannot apply block because its parent hash doesn't match.".to_string());
        }

        if block_clone.timestamp() < self.last_timestamp {
            trace(
                "add_block error: Cannot apply block because its timestamp is older than the previous tip."
            );
            return Err(
                "Cannot apply block because its timestamp is older than the previous tip."
                    .to_owned(),
            );
        }

        let encoded_block: EncodedBlock = block_clone.clone().encode();
        let max_tx_local_stable_memory_size_bytes = self
            .max_tx_local_stable_memory_size_bytes
            .unwrap_or(DEFAULT_MAX_TX_LOCAL_STABLE_MEMORY_SIZE_BYTES);

        if (self.local_archive_size as u128) + (encoded_block.size_bytes() as u128)
            > max_tx_local_stable_memory_size_bytes
        {
            return Err(format!(
                "Local archive size limit reached: {}",
                max_tx_local_stable_memory_size_bytes
            ));
        }

        self.last_timestamp = block_clone.timestamp();
        self.last_hash = Some(B::block_hash(&encoded_block));

        self.local_archive.insert(
            self.archived_chain_length as u64 + self.local_archive.len() as u64,
            encoded_block.clone(),
        );

        self.local_archive_size += encoded_block.size_bytes();

        Ok(self.archived_chain_length as u64 + self.local_archive.len() as u64)
    }

    pub async fn archive_blocks_jobs(&mut self) -> Result<u128, String> {
        trace("archive_blocks_jobs");

        trace(format!(
            "archive_blocks_jobs: local_archive: {}",
            self.local_archive.len()
        ));

        let threshold_for_archiving_to_external_archive = self
            .threshold_for_archiving_to_external_archive
            .unwrap_or(TRESHOLD_FOR_ARCHIVING);

        if self.local_archive.len() < threshold_for_archiving_to_external_archive as u64 {
            // no need to archive blocks on external canister
            return Ok(0);
        }

        if self.local_archive.is_empty() {
            // should never happen, here for safety
            return Ok(0);
        }

        // Calculate half of the blocks to archive
        let total_blocks = self.local_archive.len() as usize;
        let num_to_archive = total_blocks / 2;

        if num_to_archive == 0 {
            return Ok(0);
        }

        trace(format!(
            "archive_blocks_jobs: Archiving {} blocks (half of {})",
            num_to_archive, total_blocks
        ));

        let mut archived_count = 0u128;
        let mut total_size_decreased = 0usize;
        let batch_start_block_id: usize = self.archived_chain_length;

        // Archive blocks in batches
        for batch_start in (batch_start_block_id..num_to_archive + batch_start_block_id)
            .step_by(BATCH_SIZE_FOR_ARCHIVING)
        {
            let batch_end =
                (batch_start + BATCH_SIZE_FOR_ARCHIVING).min(num_to_archive + batch_start_block_id);
            let batch_size = batch_end - batch_start;
            let first_block_id = self.archived_chain_length as u64 + batch_start as u64;

            trace(format!(
                "archive_blocks_jobs: Processing batch from {} to {} (block_id: {} to {})",
                batch_start,
                batch_end,
                first_block_id,
                first_block_id + batch_size as u64 - 1
            ));

            // Collect all blocks in the batch
            let mut batch_blocks = Vec::with_capacity(batch_size);
            for local_index in batch_start..batch_end {
                if let Some(block) = self.local_archive.get(&(local_index as u64)) {
                    total_size_decreased += block.size_bytes();
                    batch_blocks.push(block);
                } else {
                    trace(format!(
                        "archive_blocks_jobs: Block at local_index {} not found",
                        local_index
                    ));
                    return Err(format!("Block at local_index {} not found", local_index));
                }
            }

            // we still have transaction in local_archive, and might have it duplicated in archive canister,
            // but it's fine as when we get the blocks we first check the local_archive and then the archive canister.

            // Send the entire batch in one call
            let archive_manager_guard = self.archive_canister_manager.write();

            match archive_manager_guard {
                Ok(mut archive_manager) => {
                    match archive_manager
                        .insert_blocks(batch_blocks, first_block_id)
                        .await
                    {
                        Ok(_) => {
                            archived_count += batch_size as u128;
                            trace(format!(
                                "archive_blocks_jobs: Successfully archived batch of {} blocks (block_id: {} to {})",
                                batch_size, first_block_id, first_block_id + batch_size as u64 - 1
                            ));

                            archive_manager
                                .get_subcanisters_installed()
                                .iter()
                                .for_each(|canister| {
                                    trace(format!("archive_info: {:?}", canister.archive_info));
                                });

                            // remove the batch from local_archive
                            for index in batch_start..batch_end {
                                self.local_archive.remove(&(index as u64));
                            }
                        }
                        Err(e) => {
                            trace(format!(
                                "archive_blocks_jobs: Failed to archive batch (block_id: {} to {}): {}",
                                first_block_id, first_block_id + batch_size as u64 - 1, e
                            ));
                            return Err(format!(
                                "Failed to archive batch (block_id: {} to {}): {}",
                                first_block_id,
                                first_block_id + batch_size as u64 - 1,
                                e
                            ));
                        }
                    }
                }
                Err(e) => {
                    // Should never happen, here for safety. If it happens, it means that the lock is poisoned.
                    // Will need manual intervention to fix it.
                    trace(format!("archive_blocks_jobs: Lock is poisoned: {}", e));
                    return Err(format!("Lock is poisoned: {}", e));
                }
            }
        }

        // Update the blockchain state after successful archiving
        self.archived_chain_length += num_to_archive;
        self.local_archive_size = self.local_archive_size.saturating_sub(total_size_decreased);

        trace(format!(
            "archive_blocks_jobs: Successfully archived {} blocks. Updated archived_chain_length: {}, local_archive_size: {}",
            archived_count, self.archived_chain_length, self.local_archive_size
        ));

        Ok(archived_count)
    }

    /// Retrieves a block by its index.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The index of the block to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(EncodedBlock)` if the block exists
    /// * `None` if the block doesn't exist
    pub fn get_block(&self, block_id: BlockIndex) -> Option<EncodedBlock> {
        if block_id as usize > self.archived_chain_length + self.local_archive.len() as usize {
            return None;
        }

        if block_id as usize >= self.archived_chain_length {
            self.local_archive.get(&(block_id as u64))
        } else {
            // This is an archived block, we don't have it locally
            None
        }
    }

    /// Gets the canister ID that stores a specific block.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The index of the block
    ///
    /// # Returns
    ///
    /// * `Ok(Principal)` containing the canister ID
    /// * `Err(String)` if the operation failed
    pub fn get_block_canister_id(&self, block_id: BlockIndex) -> Result<Principal, String> {
        if block_id as usize > self.archived_chain_length {
            trace(format!(
                "get_block_canister_id: Block id is after the end of the chain: {}, chain_length: {}",
                block_id,
                self.archived_chain_length
            ));
            return Err(format!(
                "Block id is after the end of the chain: {}",
                block_id
            ));
        }

        self.archive_canister_manager
            .read()
            .map_err(|_| "Failed to read archive_canister_manager")?
            .get_canister_id_by_block_id(block_id)
    }
}
