use crate::icrc3::ICRC3;
use crate::transaction::{self, BasicTransaction, TransactionKind, TransactionType};
use crate::types::Icrc3Error;
use crate::utils::trace;

use bity_ic_icrc3_archive_api::types::{block_interface::Block, defaultblock::DefaultBlock};
use candid::Nat;
use icrc_ledger_types::{
    icrc::generic_value::ICRC3Value,
    icrc3::archive::ICRC3ArchiveInfo,
    icrc3::blocks::{BlockWithId, ICRC3DataCertificate},
    icrc3::blocks::{GetBlocksRequest, GetBlocksResult, SupportedBlockType},
    icrc3::{archive::QueryArchiveFn, blocks::ArchivedBlocks},
};
use serde_bytes::ByteBuf;

/// The main interface for ICRC3 operations.
///
/// This trait defines the core functionality required for ICRC3 compliance,
/// including transaction management, block retrieval, and archive operations.
///
/// # Type Parameters
///
/// * `T`: The transaction type that implements `TransactionType`
///
/// # Examples
///
/// ```rust
/// use icrc3_library::{ICRC3Interface, ICRC3};
///
/// let icrc3 = ICRC3::new(config);
/// let archives = icrc3.icrc3_get_archives();
/// ```
pub trait ICRC3Interface<T: TransactionType>
where
    T: TransactionType,
{
    /// Adds a new transaction to the ledger.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to add
    ///
    /// # Returns
    ///
    /// * `Result<u64, Icrc3Error>` - The index of the added transaction or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The transaction is invalid
    /// * The transaction is a duplicate
    /// * The system is throttling transactions
    async fn add_transaction(&mut self, transaction: T) -> Result<u64, Icrc3Error>;

    /// Retrieves information about all archives.
    ///
    /// # Returns
    ///
    /// A vector of `ICRC3ArchiveInfo` containing details about each archive.
    fn icrc3_get_archives(&self) -> Vec<ICRC3ArchiveInfo>;

    /// Retrieves blocks from the blockchain.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `GetBlocksRequest` specifying which blocks to retrieve
    ///
    /// # Returns
    ///
    /// A `Response` containing the requested blocks and any archived blocks.
    async fn icrc3_get_blocks(
        &self,
        args: Vec<GetBlocksRequest>,
    ) -> crate::types::icrc3_get_blocks::Response;

    /// Retrieves the properties of the blockchain.
    ///
    /// # Returns
    ///
    /// A `Response` containing the blockchain properties.
    fn icrc3_get_properties(&self) -> crate::types::icrc3_get_properties::Response;

    /// Retrieves the tip certificate of the blockchain.
    ///
    /// # Returns
    ///
    /// An `ICRC3DataCertificate` containing the current tip certificate.
    fn icrc3_get_tip_certificate(&self) -> ICRC3DataCertificate;

    /// Lists the supported block types.
    ///
    /// # Returns
    ///
    /// A vector of `SupportedBlockType` containing information about supported block types.
    fn icrc3_supported_block_types(&self) -> Vec<SupportedBlockType>;
}

impl<T: TransactionType> ICRC3Interface<T> for ICRC3 {
    async fn add_transaction(&mut self, transaction: T) -> Result<u64, Icrc3Error> {
        let now = ic_cdk::api::time() as u128;

        let timestamp = if transaction.timestamp().is_none() {
            now
        } else {
            transaction.timestamp().unwrap() as u128
        };

        let num_pruned = self.purge_old_transactions(now);

        // If we pruned some transactions, let this one through
        // otherwise throttle if there are too many
        if num_pruned == 0 && self.is_throttling() {
            return Err(Icrc3Error::Icrc3Error("Transaction throttled".to_string()));
        }

        match transaction.validate_transaction_fields() {
            Ok(_) => {}
            Err(e) => {
                return Err(Icrc3Error::Icrc3Error(e));
            }
        }

        let mut transaction_as_icrc3: ICRC3Value = transaction.clone().into();

        self.add_phash(&mut transaction_as_icrc3);

        let basic_transaction = BasicTransaction::new(transaction_as_icrc3);

        let mut checked_transaction = match basic_transaction.validate_transaction_fields() {
            Ok(transaction_kind) => match transaction_kind {
                TransactionKind::Icrc1Transaction => {
                    // todo add check here to verify icrc1 transaction format
                    ICRC3Value::from(basic_transaction)
                }
                TransactionKind::CustomTransaction => ICRC3Value::from(basic_transaction),
            },
            Err(e) => {
                return Err(Icrc3Error::Icrc3Error(e));
            }
        };

        if !self
            .icrc3_config
            .supported_blocks
            .iter()
            .any(|b| b.block_type == transaction.block_type())
        {
            return Err(Icrc3Error::Icrc3Error("Unsupported block type".to_string()));
        }

        let transaction_hash = transaction.hash();

        checked_transaction = ICRC3Value::Map(match checked_transaction {
            ICRC3Value::Map(mut map) => {
                map.insert(
                    "thash".to_string(),
                    ICRC3Value::Blob(ByteBuf::from(transaction_hash)),
                );
                map
            }
            _ => {
                return Err(Icrc3Error::Icrc3Error(
                    "Invalid transaction format".to_string(),
                ))
            }
        });

        // Check if transaction already exists in ledger
        for existing_tx in &self.ledger {
            if let ICRC3Value::Map(ref existing_map) = existing_tx {
                if let Some(ICRC3Value::Blob(existing_thash)) = existing_map.get("thash") {
                    if existing_thash.as_slice() == transaction_hash {
                        return Err(Icrc3Error::Icrc3Error(
                            "Duplicate transaction happend in tx window. Transaction not added."
                                .to_string(),
                        ));
                    }
                }
            }
        }

        self.ledger.push_back(checked_transaction.clone());
        self.last_index += 1;
        self.last_phash = Some(ByteBuf::from(checked_transaction.clone().hash().to_vec()));

        let block = DefaultBlock::from_transaction(
            self.blockchain.last_hash,
            checked_transaction,
            timestamp,
        );

        match self.blockchain.add_block(block).await {
            Ok(_) => (),
            Err(e) => {
                return Err(Icrc3Error::Icrc3Error(e));
            }
        }

        Ok(self.last_index)
    }

    fn icrc3_get_archives(&self) -> Vec<ICRC3ArchiveInfo> {
        let sub_canisters = &self
            .blockchain
            .archive_canister_manager
            .read()
            .unwrap()
            .get_subcanisters_installed();

        sub_canisters
            .iter()
            .map(|canister| canister.archive_info.clone())
            .collect()
    }

    async fn icrc3_get_blocks(
        &self,
        args: Vec<GetBlocksRequest>,
    ) -> crate::types::icrc3_get_blocks::Response {
        let mut response = GetBlocksResult {
            log_length: Nat::from(self.last_index),
            blocks: vec![],
            archived_blocks: vec![],
        };

        for arg in args {
            let start: u64 = arg.start.0.try_into().unwrap();
            let length: u64 = arg.length.0.try_into().unwrap();

            let mut current_start = start;
            let mut current_length = 0u64;
            let mut current_canister = None;

            for i in start..start + length {
                if i > self.blockchain.chain_length() {
                    let block = self.blockchain.get_block(i);

                    match block {
                        Some(block) => {
                            let default_block = DefaultBlock::decode(block).unwrap();
                            response.blocks.push(BlockWithId {
                                id: Nat::from(i),
                                block: default_block.transaction,
                            });
                            continue;
                        }
                        None => {}
                    }
                }
                let block_canister_id = self.blockchain.get_block_canister_id(i).await;

                match block_canister_id {
                    Ok(canister_id) => match current_canister {
                        Some(current_id) if current_id == canister_id => {
                            current_length += 1;
                        }
                        _ => {
                            if let Some(current_id) = current_canister {
                                response.archived_blocks.push(ArchivedBlocks {
                                    args: vec![GetBlocksRequest {
                                        start: Nat::from(current_start),
                                        length: Nat::from(current_length),
                                    }],
                                    callback: QueryArchiveFn::new(
                                        current_id,
                                        "icrc3_get_blocks".to_string(),
                                    ),
                                });
                            }

                            current_start = i;
                            current_length = 1;
                            current_canister = Some(canister_id);
                        }
                    },
                    Err(e) => {
                        // should never happen, but we can not trap or return error. we just skip the block
                        trace(&format!("icrc3_get_blocks error: {:?}", e));
                    }
                }
            }

            if let Some(current_id) = current_canister {
                response.archived_blocks.push(ArchivedBlocks {
                    args: vec![GetBlocksRequest {
                        start: Nat::from(current_start),
                        length: Nat::from(current_length - 1),
                    }],
                    callback: QueryArchiveFn::new(current_id, "icrc3_get_blocks".to_string()),
                });
            }
        }

        response
    }

    fn icrc3_get_properties(&self) -> crate::types::icrc3_get_properties::Response {
        self.icrc3_config.constants.clone()
    }

    fn icrc3_get_tip_certificate(&self) -> ICRC3DataCertificate {
        let certificate = ic_cdk::api::data_certificate().expect("No data certificate available");

        ICRC3DataCertificate {
            certificate: certificate.into(),
            hash_tree: self.get_hash_tree().into(),
        }
    }

    fn icrc3_supported_block_types(&self) -> Vec<SupportedBlockType> {
        self.icrc3_config
            .supported_blocks
            .iter()
            .map(|b| SupportedBlockType {
                block_type: b.block_type.clone(),
                url: b.url.clone(),
            })
            .collect()
    }
}
