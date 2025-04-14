use crate::blockchain::blockchain::Blockchain;
use crate::config::{ICRC3Config, ICRC3Properties};
use crate::utils::{get_timestamp, last_block_hash_tree, trace};

use candid::Nat;
use ic_certification::AsHashTree;
use icrc3_archive_api::types::hash::HashOf;
use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use icrc_ledger_types::icrc3::blocks::{ICRC3DataCertificate, SupportedBlockType};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::VecDeque;
use std::time::Duration;

/// The maximum allowed time drift for transaction timestamps
pub const PERMITTED_DRIFT: Duration = Duration::from_millis(100);

/// The main ICRC3 implementation struct.
///
/// This struct represents the core of the ICRC3 implementation, managing
/// the blockchain, ledger, and configuration.
///
/// # Fields
///
/// * `blockchain` - The blockchain implementation
/// * `ledger` - A queue of recent transactions
/// * `last_index` - The index of the last transaction
/// * `icrc3_config` - Configuration parameters
#[derive(Serialize, Deserialize)]
pub struct ICRC3 {
    pub blockchain: Blockchain,
    pub ledger: VecDeque<ICRC3Value>,
    pub last_index: u64,
    pub icrc3_config: ICRC3Config,
}

unsafe impl Send for ICRC3 {}
unsafe impl Sync for ICRC3 {}

impl ICRC3 {
    /// Creates a new ICRC3 instance with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `icrc3_config` - The configuration for the ICRC3 instance
    ///
    /// # Returns
    ///
    /// A new ICRC3 instance with an empty blockchain and ledger
    pub fn new(icrc3_config: ICRC3Config) -> Self {
        Self {
            blockchain: Blockchain::default(),
            ledger: VecDeque::new(),
            last_index: 0,
            icrc3_config,
        }
    }

    /// Checks if the system should throttle new transactions.
    ///
    /// This method implements a rate limiting mechanism that:
    /// 1. Allows the first half of max_transactions_in_window freely
    /// 2. After that, throttles on a per-second basis
    ///
    /// # Returns
    ///
    /// `true` if the system should throttle new transactions, `false` otherwise
    pub fn is_throttling(&self) -> bool {
        let num_in_window = self.ledger_len();

        if num_in_window >= self.max_transactions_in_window() / 2 {
            let max_rate = (0.5 * self.max_transactions_in_window() as f64
                / self.transaction_window().as_secs_f64())
            .ceil() as u128;

            trace(&format!(
                "is_throttling: num_in_window: {}, max_rate: {}",
                num_in_window, max_rate
            ));

            if self
                .ledger
                .get(num_in_window.saturating_sub(max_rate).try_into().unwrap())
                .map(|tx| get_timestamp(tx).unwrap_or(Nat::from(0_u64)))
                .unwrap_or_else(|| Nat::from(0_u64))
                + Nat::from(1_u64)
                > Duration::from_nanos(ic_cdk::api::time()).as_secs()
            {
                return true;
            }
        }

        false
    }

    /// Purges old transactions from the ledger.
    ///
    /// Removes transactions older than `now - transaction_window` up to
    /// the maximum number of transactions specified in the configuration.
    ///
    /// # Arguments
    ///
    /// * `now` - The current timestamp in nanoseconds
    ///
    /// # Returns
    ///
    /// The number of transactions that were purged
    pub fn purge_old_transactions(&mut self, now: u128) -> u128 {
        let max_tx_to_purge = self.icrc3_config.constants.max_transactions_to_purge;
        let mut num_tx_purged = 0;
        trace(&format!("purge_old_transactions"));

        while let Some(tx_info) = self.ledger.front() {
            let tx_timestamp = get_timestamp(tx_info).unwrap_or(Nat::from(0_u64));

            if u128::try_from(tx_timestamp.clone().0).unwrap()
                + self.transaction_window().as_nanos()
                + PERMITTED_DRIFT.as_nanos()
                >= now
            {
                trace(&format!(
                    "purge_old_transactions: tx_timestamp: {}, now: {}, tx_window: {}, permitted_drift: {}",
                    Duration::from_nanos(u64::try_from(tx_timestamp.clone().0).unwrap()).as_nanos(), now, self.transaction_window().as_nanos(), PERMITTED_DRIFT.as_nanos()
                ));
                // Stop at a sufficiently recent block.
                break;
            }

            self.ledger.pop_front();
            num_tx_purged += 1;

            if num_tx_purged >= max_tx_to_purge {
                break;
            }
        }
        trace(&format!(
            "purge_old_transactions done, num_tx_purged: {}",
            num_tx_purged
        ));

        num_tx_purged
    }

    /// Returns the current size of the blockchain.
    pub fn blockchain_size(&self) -> u64 {
        self.blockchain.chain_length()
    }

    /// Returns the current number of transactions in the ledger.
    pub fn ledger_len(&self) -> u128 {
        self.ledger.len() as u128
    }

    /// Returns the transaction window duration.
    pub fn transaction_window(&self) -> Duration {
        self.icrc3_config.constants.tx_window
    }

    /// Returns the maximum number of transactions allowed in the window.
    pub fn max_transactions_in_window(&self) -> u128 {
        self.icrc3_config.constants.max_transactions_in_window
    }

    /// Generates a hash tree for the current state.
    ///
    /// This is used for data certification in the Internet Computer.
    ///
    /// # Returns
    ///
    /// A vector containing the root hash of the certification tree
    pub fn get_hash_tree(&self) -> Vec<u8> {
        let hash_tree_root = last_block_hash_tree(
            self.last_index,
            self.blockchain
                .last_hash
                .unwrap_or(HashOf::new([0; 32]))
                .into_bytes(),
        )
        .root_hash();
        hash_tree_root.to_vec()
    }
}

use ic_certification::fork;
use ic_certification::hash_tree::leaf;
use ic_certification::Certificate;

impl From<ICRC3> for Certificate {
    /// Converts an ICRC3 instance into a Certificate.
    ///
    /// This implementation creates a certification tree containing:
    /// * The last block index
    /// * The last block hash
    ///
    /// The certificate is then set as the certified data for the canister.
    fn from(val: ICRC3) -> Self {
        let last_block_index = val.last_index;
        let last_block_hash = val.blockchain.last_hash.unwrap_or(HashOf::new([0; 32]));

        let leaf1 = leaf(last_block_index.to_string());
        let leaf2 = leaf(last_block_hash.as_slice());

        let hash_tree = fork(leaf1, leaf2);
        ic_cdk::api::set_certified_data(&hash_tree.digest());
        let certificate = ic_cdk::api::data_certificate().expect("No data certificate available");
        Certificate {
            tree: hash_tree,
            signature: certificate,
            delegation: None,
        }
    }
}
