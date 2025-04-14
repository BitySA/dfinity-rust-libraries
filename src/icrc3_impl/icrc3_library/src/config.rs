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
}

impl Default for ICRC3Properties {
    fn default() -> Self {
        ICRC3Properties {
            tx_window: Duration::from_secs(0),
            max_transactions_in_window: 0_u64.into(),
            max_memory_size_bytes: 0_u64.into(),
            max_blocks_per_response: 0_u64.into(),
            initial_cycles: 0_u64.into(),
            reserved_cycles: 0_u64.into(),
            max_transactions_to_purge: 0_u64.into(),
        }
    }
}
