use crate::blockchain::archive_canister_manager::ArchiveCanisterManager;
use crate::utils::trace;
use std::collections::VecDeque;

use bity_ic_icrc3_archive_api::types::{
    block_interface::{Block, BlockIndex},
    encoded_blocks::EncodedBlock,
    hash::HashOf,
};
use candid::Principal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, RwLock};

/// The threshold at which blocks are automatically archived
const ARCHIVE_THRESHOLD: usize = 50;
/// The maximum number of transactions to keep in local storage
const MAX_LOCAL_TRANSACTIONS: usize = 100;

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
    /// Hash of the last block in the chain
    pub last_hash: Option<HashOf<EncodedBlock>>,
    /// Timestamp of the last block
    pub last_timestamp: u128,
    /// Number of blocks that have been archived
    pub chain_length: u64,
    /// Local transactions waiting to be archived
    pub local_transactions: VecDeque<EncodedBlock>,
    /// Threshold for triggering archiving
    pub archive_threshold: usize,
}

impl Default for Blockchain {
    /// Creates a new Blockchain with default settings.
    fn default() -> Self {
        Self {
            archive_canister_manager: Arc::new(RwLock::new(ArchiveCanisterManager::default())),
            last_hash: None,
            last_timestamp: 0,
            chain_length: 0,
            local_transactions: VecDeque::new(),
            archive_threshold: ARCHIVE_THRESHOLD,
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
            &self.last_hash,
            &self.last_timestamp,
            &self.chain_length,
            &self.local_transactions,
            &self.archive_threshold,
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
            last_hash,
            last_timestamp,
            chain_length,
            local_transactions,
            archive_threshold,
        ) = <(
            ArchiveCanisterManager,
            Option<HashOf<EncodedBlock>>,
            u128,
            u64,
            VecDeque<EncodedBlock>,
            usize,
        )>::deserialize(deserializer)?;

        Ok(Blockchain {
            archive_canister_manager: Arc::new(RwLock::new(archive_manager)),
            last_hash,
            last_timestamp,
            chain_length,
            local_transactions,
            archive_threshold,
        })
    }
}

impl Blockchain {
    /// Archives blocks from local storage to archive canisters.
    ///
    /// This method:
    /// 1. Takes blocks from the front of the local transactions queue
    /// 2. Attempts to insert them into archive canisters
    /// 3. Updates the chain length on successful insertion
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all blocks were successfully archived
    /// * `Err(String)` if archiving failed
    async fn archive_blocks(&mut self) -> Result<(), String> {
        while let Some(block) = self.local_transactions.pop_front() {
            let block_clone = block.clone();

            match self
                .archive_canister_manager
                .write()
                .map_err(|e| format!("Failed to acquire write lock: {}", e))?
                .insert_block(self.chain_length(), block_clone)
                .await
            {
                Ok(_) => {
                    self.chain_length += 1;
                }
                Err(e) => {
                    self.local_transactions.push_front(block);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

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
    pub async fn add_block<B>(&mut self, block: B) -> Result<BlockIndex, String>
    where
        B: Block + Clone,
    {
        let block_clone = block.clone();
        if block_clone.parent_hash() != self.last_hash {
            trace(&format!(
                "add_block error: Cannot apply block because its parent hash doesn't match."
            ));
            return Err("Cannot apply block because its parent hash doesn't match.".to_string());
        }

        if block_clone.timestamp() < self.last_timestamp {
            trace(&format!(
                "add_block error: Cannot apply block because its timestamp is older than the previous tip."
            ));
            return Err(
                "Cannot apply block because its timestamp is older than the previous tip."
                    .to_owned(),
            );
        }

        if self.local_transactions.len() >= MAX_LOCAL_TRANSACTIONS {
            if let Err(e) = self.archive_blocks().await {
                trace(&format!(
                    "add_block error: Maximum local transactions reached and archiving failed: {}",
                    e
                ));
                return Err(e);
            }
        }

        self.last_timestamp = block_clone.timestamp();
        let encoded_block: EncodedBlock = block_clone.encode();
        self.last_hash = Some(B::block_hash(&encoded_block));

        self.local_transactions.push_back(encoded_block.clone());

        if self.local_transactions.len() >= self.archive_threshold {
            if let Err(e) = self.archive_blocks().await {
                trace(&format!("add_block error: {}", e));
            }
        }

        Ok(self.chain_length() + 1)
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
        if block_id > self.chain_length() + self.local_transactions.len() as u64 {
            return None;
        }

        if block_id > self.chain_length() {
            return Some(
                self.local_transactions[block_id as usize - self.chain_length() as usize - 1]
                    .clone(),
            );
        } else {
            return None;
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
    pub async fn get_block_canister_id(&self, block_id: BlockIndex) -> Result<Principal, String> {
        self.archive_canister_manager
            .read()
            .map_err(|_| "Failed to read archive_canister_manager")?
            .get_canister_id_by_block_id(block_id)
    }

    /// Returns the current length of the blockchain.
    pub fn chain_length(&self) -> u64 {
        self.chain_length
    }
}
