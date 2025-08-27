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
use std::time::Duration;

/// The maximum number of transactions to keep in local storage
const MAX_LOCAL_TRANSACTIONS: u128 = 1000;

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
    pub local_transactions: VecDeque<(u128, EncodedBlock)>, // (timestamp, block)
    /// Time to live for non-archived transactions
    pub ttl_for_non_archived_transactions: Duration,
    /// maximum number of unarchived transactions
    pub max_unarchived_transactions: u128,
}

impl Blockchain {
    pub fn new(
        archive_canister_manager: ArchiveCanisterManager,
        last_hash: Option<HashOf<EncodedBlock>>,
        last_timestamp: u128,
        chain_length: u64,
        ttl_for_non_archived_transactions: Duration,
        max_unarchived_transactions: u128,
    ) -> Self {
        Self {
            archive_canister_manager: Arc::new(RwLock::new(archive_canister_manager)),
            last_hash,
            last_timestamp,
            chain_length,
            local_transactions: VecDeque::new(),
            ttl_for_non_archived_transactions,
            max_unarchived_transactions,
        }
    }
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
            ttl_for_non_archived_transactions: Duration::from_secs(120),
            max_unarchived_transactions: MAX_LOCAL_TRANSACTIONS,
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
            &self.ttl_for_non_archived_transactions,
            &self.max_unarchived_transactions,
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
            ttl_for_non_archived_transactions,
            max_unarchived_transactions,
        ) = <(
            ArchiveCanisterManager,
            Option<HashOf<EncodedBlock>>,
            u128,
            u64,
            VecDeque<(u128, EncodedBlock)>,
            Duration,
            u128,
        )>::deserialize(deserializer)?;

        Ok(Blockchain {
            archive_canister_manager: Arc::new(RwLock::new(archive_manager)),
            last_hash,
            last_timestamp,
            chain_length,
            local_transactions,
            ttl_for_non_archived_transactions,
            max_unarchived_transactions,
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

        if self.local_transactions.len() >= self.max_unarchived_transactions as usize {
            return Err(format!(
                "Local transactions limit reached: {}",
                MAX_LOCAL_TRANSACTIONS
            ));
        }

        self.last_timestamp = block_clone.timestamp();
        let encoded_block: EncodedBlock = block_clone.clone().encode();
        self.last_hash = Some(B::block_hash(&encoded_block));

        self.local_transactions
            .push_back((block_clone.timestamp(), encoded_block.clone()));

        trace(format!(
            "Added block to local transactions: {}",
            self.local_transactions.len()
        ));

        Ok(self.chain_length() + self.local_transactions.len() as u64)
    }

    pub async fn archive_blocks_jobs(&mut self) -> Result<u128, String> {
        trace("archive_blocks_jobs");

        trace(format!(
            "archive_blocks_jobs: local_transactions: {}",
            self.local_transactions.len()
        ));

        if self.local_transactions.is_empty() {
            return Ok(0);
        }

        let mut archived_count = 0;
        let current_time = ic_cdk::api::time() as u128;

        while !self.local_transactions.is_empty() {
            if let Some((oldest_timestamp, oldest_block)) = self.local_transactions.front().cloned()
            {
                trace(format!(
                    "oldest_timestamp: {}, current_time: {}",
                    oldest_timestamp, current_time
                ));

                if oldest_timestamp + self.ttl_for_non_archived_transactions.as_nanos()
                    < current_time
                {
                    trace(format!(
                        "oldest_timestamp + ttl_for_non_archived_transactions: {}",
                        oldest_timestamp + self.ttl_for_non_archived_transactions.as_nanos()
                    ));

                    if let Some(tx) = self.local_transactions.pop_front() {
                        trace(format!(
                            "Archiving transaction from timestamp {}",
                            oldest_timestamp
                        ));

                        match self
                            .archive_canister_manager
                            .write()
                            .map_err(|e| format!("Failed to acquire write lock: {}", e))
                        {
                            Ok(mut archive_manager) => {
                                match archive_manager
                                    .insert_block(self.chain_length(), oldest_block.clone())
                                    .await
                                {
                                    Ok(_) => {
                                        archived_count += 1;
                                        self.chain_length += 1;
                                    }
                                    Err(e) => {
                                        self.local_transactions.push_front(tx);
                                        return Err(e);
                                    }
                                }
                            }
                            Err(e) => {
                                self.local_transactions.push_front(tx);
                                return Err(e);
                            }
                        }
                    } else {
                        trace(format!(
                            "oldest_timestamp + ttl_for_non_archived_transactions: {}",
                            oldest_timestamp + self.ttl_for_non_archived_transactions.as_nanos()
                        ));
                        break;
                    }
                } else {
                    trace(format!(
                        "oldest_timestamp + ttl_for_non_archived_transactions: {}",
                        oldest_timestamp + self.ttl_for_non_archived_transactions.as_nanos()
                    ));
                    break;
                }
            } else {
                trace("archive_blocks_jobs: local_transactions is empty");
                break;
            }
        }

        trace(format!("Archived {} transaction(s)", archived_count));
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
        if block_id > self.chain_length() + self.local_transactions.len() as u64 {
            return None;
        }

        if block_id >= self.chain_length() {
            // This is a non-archived block stored locally
            let local_index = block_id - self.chain_length();
            self.local_transactions
                .get(local_index as usize)
                .map(|(_, block)| block.clone())
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
        if block_id >= self.chain_length() {
            trace(format!(
                "get_block_canister_id: Block id is after the end of the chain: {}, chain_length: {}",
                block_id,
                self.chain_length()
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

    /// Returns the current length of the blockchain.
    pub fn chain_length(&self) -> u64 {
        self.chain_length
    }
}
