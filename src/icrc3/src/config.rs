use candid::CandidType;
use icrc_ledger_types::icrc3::blocks::SupportedBlockType;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the ICRC3 implementation.
///
/// This struct contains all the necessary configuration parameters for the ICRC3 implementation,
/// including supported block types and system constants.
///
/// # Examples
///
/// ```rust
/// use icrc3_library::config::{ICRC3Config, ICRC3Properties};
/// use std::time::Duration;
///
/// let config = ICRC3Config {
///     supported_blocks: vec![],
///     constants: ICRC3Properties::default(),
/// };
/// ```
#[derive(CandidType, Serialize, Deserialize, Debug, Default)]
pub struct ICRC3Config {
    /// List of supported block types and their URLs
    pub supported_blocks: Vec<SupportedBlockType>,
    /// System constants and limits
    pub constants: ICRC3Properties,
}

impl Clone for ICRC3Config {
    fn clone(&self) -> Self {
        ICRC3Config {
            supported_blocks: self
                .supported_blocks
                .iter()
                .map(|b| SupportedBlockType {
                    block_type: b.block_type.clone(),
                    url: b.url.clone(),
                })
                .collect(),
            constants: self.constants.clone(),
        }
    }
}

/// System constants and limits for the ICRC3 implementation.
///
/// This struct defines various system parameters that control the behavior
/// of the ICRC3 implementation, such as transaction windows, memory limits,
/// and cycle management.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ICRC3Properties {
    /// Time window for transaction validation
    pub tx_window: Duration,
    /// Maximum number of transactions allowed in the window
    pub max_transactions_in_window: u128,
    /// Maximum memory size in bytes
    pub max_memory_size_bytes: u128,
    /// Maximum number of blocks per response
    pub max_blocks_per_response: u128,
    /// Initial number of cycles
    pub initial_cycles: u128,
    /// Number of cycles to reserve
    pub reserved_cycles: u128,
    /// Maximum number of transactions to purge at once
    pub max_transactions_to_purge: u128,
    /// Maximum size of local stable memory for transactions before archiving.
    /// If None, transactions are directly archived in a new canister, not stored in the local stable memory.
    pub max_tx_local_stable_memory_size_bytes: Option<u128>,
    /// Threshold for archiving blocks to the external archive canister
    pub threshold_for_archiving_to_external_archive: Option<usize>,
}

impl ICRC3Properties {
    pub fn new(
        tx_window: Duration,
        max_transactions_in_window: u128,
        max_memory_size_bytes: u128,
        max_blocks_per_response: u128,
        initial_cycles: u128,
        reserved_cycles: u128,
        max_transactions_to_purge: u128,
        max_tx_local_stable_memory_size_bytes: Option<u128>,
        threshold_for_archiving_to_external_archive: Option<usize>,
    ) -> Self {
        Self {
            tx_window,
            max_transactions_in_window,
            max_memory_size_bytes,
            max_blocks_per_response,
            initial_cycles,
            reserved_cycles,
            max_transactions_to_purge,
            max_tx_local_stable_memory_size_bytes,
            threshold_for_archiving_to_external_archive,
        }
    }
}

impl Default for ICRC3Properties {
    fn default() -> Self {
        ICRC3Properties {
            tx_window: Duration::from_millis(100),
            max_transactions_in_window: 200_u64.into(),
            max_memory_size_bytes: 1024 * 1024 * 1024_u128, // 1GB
            max_blocks_per_response: 100_u64.into(),
            initial_cycles: 5_000_000_000_000_u128,
            reserved_cycles: 5_000_000_000_000_u128,
            max_transactions_to_purge: 0_u64.into(),
            max_tx_local_stable_memory_size_bytes: None,
            threshold_for_archiving_to_external_archive: None,
        }
    }
}
