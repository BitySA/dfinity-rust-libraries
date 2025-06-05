use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Error types for the ICRC3 implementation.
///
/// This enum represents all possible error conditions that can occur
/// during ICRC3 operations.
#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub enum Icrc3Error {
    /// The ledger size has exceeded its maximum limit
    LedgerSizeExceeded,
    /// A general ICRC3 error with a message
    Icrc3Error(String),
    /// An error occurred during block creation
    BlockCreationError(String),
    /// A duplicate transaction occurred
    DuplicateTransaction { duplicate_of: u64 },
}

impl std::fmt::Display for Icrc3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Icrc3Error {}

/// Module containing types for the `icrc3_get_properties` endpoint.
pub mod icrc3_get_properties {
    use crate::config::ICRC3Properties;

    /// Arguments for the `icrc3_get_properties` endpoint
    pub type Args = ();
    /// Response type for the `icrc3_get_properties` endpoint
    pub type Response = ICRC3Properties;
}

/// Module containing types for the `icrc3_get_archives` endpoint.
pub mod icrc3_get_archives {
    use icrc_ledger_types::icrc3::archive::{GetArchivesArgs, GetArchivesResult};

    /// Arguments for the `icrc3_get_archives` endpoint
    pub type Args = GetArchivesArgs;
    /// Response type for the `icrc3_get_archives` endpoint
    pub type Response = GetArchivesResult;
}

/// Module containing types for the `icrc3_get_blocks` endpoint.
pub mod icrc3_get_blocks {
    use icrc_ledger_types::icrc3::blocks::{GetBlocksRequest, GetBlocksResult};

    /// Arguments for the `icrc3_get_blocks` endpoint
    pub type Args = Vec<GetBlocksRequest>;
    /// Response type for the `icrc3_get_blocks` endpoint
    pub type Response = GetBlocksResult;
}

/// Module containing types for the `icrc3_get_tip_certificate` endpoint.
pub mod icrc3_get_tip_cerificate {
    use icrc_ledger_types::icrc3::blocks::ICRC3DataCertificate;

    /// Arguments for the `icrc3_get_tip_certificate` endpoint
    pub type Args = ();
    /// Response type for the `icrc3_get_tip_certificate` endpoint
    pub type Response = ICRC3DataCertificate;
}

/// Module containing types for the `icrc3_supported_block_types` endpoint.
pub mod icrc3_supported_block_types {
    use icrc_ledger_types::icrc3::blocks::SupportedBlockType;

    /// Arguments for the `icrc3_supported_block_types` endpoint
    pub type Args = ();
    /// Response type for the `icrc3_supported_block_types` endpoint
    pub type Response = Vec<SupportedBlockType>;
}

/// Module containing types for the `add_transaction` endpoint.
pub mod add_transaction {
    use crate::types::Icrc3Error;
    use candid::Principal;
    use icrc_ledger_types::icrc::generic_value::ICRC3Value;

    /// Arguments for the `add_transaction` endpoint
    pub type Args = ICRC3Value;
    /// Response type for the `add_transaction` endpoint
    ///
    /// Returns a tuple containing:
    /// * The index of the added transaction
    /// * A vector of principals that need to be notified
    pub type Response = Result<(u64, Vec<Principal>), Icrc3Error>;
}

/// Module containing types for the `add_batch_transactions` endpoint.
pub mod add_batch_transactions {
    use candid::Principal;
    use icrc_ledger_types::icrc::generic_value::ICRC3Value;

    /// Arguments for the `add_batch_transactions` endpoint
    pub type Args = Vec<ICRC3Value>;
    /// Response type for the `add_batch_transactions` endpoint
    ///
    /// Returns a tuple containing:
    /// * A vector of transaction indices
    /// * A vector of principals that need to be notified
    pub type Response = Result<(Vec<u64>, Vec<Principal>), String>;
}
