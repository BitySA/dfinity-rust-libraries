use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct ArchiveConfig {
    /// The maximum number of bytes archive can use to store encoded blocks.
    pub max_memory_size_bytes: u128,
    /// The maximum number of transactions returned by [get_transactions].
    pub max_blocks_per_response: u64,
    /// The offset of the first block in the archive.
    pub block_offset: u64,
}

const MAX_MEMORY_SIZE_BYTES: u128 = 1024 * 1024 * 1024; // 1GB
const MAX_BLOCKS_PER_RESPONSE: u64 = 1000;

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            max_memory_size_bytes: MAX_MEMORY_SIZE_BYTES,
            max_blocks_per_response: MAX_BLOCKS_PER_RESPONSE,
            block_offset: 0,
        }
    }
}

impl ArchiveConfig {
    pub fn new(
        max_memory_size_bytes: u128,
        max_blocks_per_response: u64,
        block_offset: u64,
    ) -> Self {
        Self {
            max_memory_size_bytes,
            max_blocks_per_response,
            block_offset,
        }
    }

    pub fn get_max_memory_size_bytes(&self) -> u128 {
        self.max_memory_size_bytes
    }

    pub fn get_max_blocks_per_response(&self) -> u64 {
        self.max_blocks_per_response
    }
}
