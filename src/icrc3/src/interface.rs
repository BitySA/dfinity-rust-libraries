use crate::icrc3::ICRC3;
use crate::transaction::{GlobalTransaction, TransactionType};
use crate::types::{commit_transaction, prepare_transaction, Icrc3Error};
use crate::utils::trace;

use bity_ic_icrc3_archive_api::types::{block_interface::Block, defaultblock::DefaultBlock};
use candid::Nat;
use hex;
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
///
/// ## Example with async operations:
///
/// ```rust
/// use icrc3_library::{ICRC3Interface, ICRC3};
///
/// // Prepare the transaction first
/// let prepared_tx = icrc3.prepare_transaction(transaction)?;
///
/// // Perform async operations
/// let async_result = some_async_operation().await?;
///
/// // Commit the transaction after async operations complete
/// let tx_index = icrc3.commit_prepared_transaction(prepared_tx)?;
/// ```
pub trait ICRC3Interface {
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
    fn add_transaction<T: TransactionType>(&mut self, transaction: T) -> Result<u64, Icrc3Error>;

    /// Prepares a transaction for later commit without adding it to the ledger.
    ///
    /// This method validates the transaction and creates a prepared transaction
    /// that can be committed later. This is useful for scenarios where you need
    /// to perform async operations before committing.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to prepare
    ///
    /// # Returns
    ///
    /// * `Result<PreparedTransaction, Icrc3Error>` - The prepared transaction or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The transaction is invalid
    /// * The transaction is a duplicate
    /// * The system is throttling transactions
    fn prepare_transaction<T: TransactionType>(
        &mut self,
        transaction: T,
    ) -> prepare_transaction::Response;

    /// Commits a previously prepared transaction to the ledger.
    ///
    /// This method adds the prepared transaction to the ledger and updates
    /// the blockchain. It should be called after any async operations are completed.
    ///
    /// # Arguments
    ///
    /// * `prepared_transaction` - The prepared transaction to commit
    ///
    /// # Returns
    ///
    /// * `Result<u64, Icrc3Error>` - The index of the committed transaction or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The prepared transaction is invalid
    /// * The transaction has become a duplicate since preparation
    /// * The system is now throttling transactions
    fn commit_prepared_transaction<T: TransactionType>(
        &mut self,
        transaction: T,
        timestamp: u128,
    ) -> commit_transaction::Response;

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
    fn icrc3_get_blocks(
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

    /// Cleans up expired prepared transactions.
    ///
    /// Removes prepared transactions that have been in the ledger for more than 24 hours.
    /// This is automatically called during transaction preparation, but can also be called manually.
    ///
    /// # Returns
    ///
    /// The number of expired prepared transactions that were removed
    fn cleanup_expired_prepared_transactions(&mut self) -> usize;
}

impl ICRC3Interface for ICRC3 {
    fn add_transaction<T: TransactionType>(&mut self, transaction: T) -> Result<u64, Icrc3Error> {
        let now = ic_cdk::api::time() as u128;

        let timestamp: u128 = if let Some(timestamp) = transaction.timestamp() {
            timestamp.into()
        } else {
            now
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

        let basic_transaction = GlobalTransaction::new(transaction_as_icrc3);

        let mut checked_transaction = match basic_transaction.validate_transaction_fields() {
            Ok(_) => ICRC3Value::from(basic_transaction),
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

        let transaction_hash = transaction.tx().hash();

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
        for (i, existing_tx) in self.ledger.iter().enumerate() {
            if let ICRC3Value::Map(ref existing_map) = existing_tx {
                if let Some(ICRC3Value::Blob(existing_thash)) = existing_map.get("thash") {
                    if existing_thash.as_slice() == transaction_hash {
                        return Err(Icrc3Error::DuplicateTransaction {
                            duplicate_of: self.last_index - i as u64,
                        });
                    }
                }
            }
        }

        self.ledger.push_back(checked_transaction.clone());
        self.last_index += 1;
        self.last_phash = Some(ByteBuf::from(transaction_hash));

        let block = DefaultBlock::from_transaction(
            self.blockchain.last_hash,
            checked_transaction,
            timestamp,
        );

        match self.blockchain.add_block(block) {
            Ok(_) => (),
            Err(e) => {
                return Err(Icrc3Error::Icrc3Error(e));
            }
        }

        Ok(self.last_index)
    }

    fn prepare_transaction<T: TransactionType>(
        &mut self,
        transaction: T,
    ) -> prepare_transaction::Response {
        let now = ic_cdk::api::time() as u128;

        let timestamp: u128 = if let Some(timestamp) = transaction.timestamp() {
            timestamp.into()
        } else {
            now
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

        let basic_transaction = GlobalTransaction::new(transaction_as_icrc3);

        let mut checked_transaction = match basic_transaction.validate_transaction_fields() {
            Ok(_) => ICRC3Value::from(basic_transaction),
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

        let transaction_hash = transaction.tx().hash().to_vec();

        checked_transaction = ICRC3Value::Map(match checked_transaction {
            ICRC3Value::Map(mut map) => {
                map.insert(
                    "thash".to_string(),
                    ICRC3Value::Blob(ByteBuf::from(transaction_hash.clone())),
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
        for (i, existing_tx) in self.ledger.iter().enumerate() {
            if let ICRC3Value::Map(ref existing_map) = existing_tx {
                if let Some(ICRC3Value::Blob(existing_thash)) = existing_map.get("thash") {
                    if existing_thash.as_slice() == transaction_hash {
                        return Err(Icrc3Error::DuplicateTransaction {
                            duplicate_of: self.last_index - i as u64,
                        });
                    }
                }
            }
        }

        self.ledger.push_back(checked_transaction.clone());
        let transaction_hash_string = hex::encode(&transaction_hash);
        self.add_prepared_transaction(transaction_hash_string, timestamp as u64);

        Ok(prepare_transaction::PreparedTransaction {
            transaction_hash,
            timestamp,
        })
    }

    fn commit_prepared_transaction<T: TransactionType>(
        &mut self,
        transaction: T,
        timestamp: u128,
    ) -> commit_transaction::Response {
        let mut transaction_as_icrc3: ICRC3Value = transaction.clone().into();

        self.add_phash(&mut transaction_as_icrc3);

        let basic_transaction = GlobalTransaction::new(transaction_as_icrc3);

        let mut icrc3_transaction = ICRC3Value::from(basic_transaction);

        let transaction_hash = transaction.tx().hash().to_vec();

        let transaction_hash_string = hex::encode(&transaction_hash);

        let prepared_transaction = self
            .prepared_transactions
            .iter()
            .position(|(hash, _)| hash == &transaction_hash_string);

        if let Some(index) = prepared_transaction {
            let (_, prepared_timestamp) = self.prepared_transactions[index];
            if prepared_timestamp != timestamp as u64 {
                return Err(Icrc3Error::Icrc3Error(
                    "Transaction timestamp mismatch".to_string(),
                ));
            }

            self.prepared_transactions.remove(index);
        } else {
            return Err(Icrc3Error::Icrc3Error(
                "Transaction not found in prepared transactions".to_string(),
            ));
        }

        icrc3_transaction = ICRC3Value::Map(match icrc3_transaction {
            ICRC3Value::Map(mut map) => {
                map.insert(
                    "thash".to_string(),
                    ICRC3Value::Blob(ByteBuf::from(transaction_hash.clone())),
                );
                map
            }
            _ => {
                return Err(Icrc3Error::Icrc3Error(
                    "Invalid transaction format".to_string(),
                ))
            }
        });

        self.last_index += 1;
        self.last_phash = Some(ByteBuf::from(icrc3_transaction.clone().hash().to_vec()));

        // Add block to blockchain
        let block =
            DefaultBlock::from_transaction(self.blockchain.last_hash, icrc3_transaction, timestamp);

        match self.blockchain.add_block(block) {
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

    fn icrc3_get_blocks(
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

                    if let Some(block) = block {
                        let default_block = DefaultBlock::decode(block).unwrap();
                        response.blocks.push(BlockWithId {
                            id: Nat::from(i),
                            block: default_block.transaction,
                        });
                        continue;
                    }
                }
                let block_canister_id = self.blockchain.get_block_canister_id(i);

                trace(format!("block_canister_id: {:?}", block_canister_id));

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
                        trace(format!("icrc3_get_blocks error: {:?}", e));
                    }
                }
            }

            if let Some(current_id) = current_canister {
                response.archived_blocks.push(ArchivedBlocks {
                    args: vec![GetBlocksRequest {
                        start: Nat::from(current_start),
                        length: Nat::from(current_length),
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

    fn cleanup_expired_prepared_transactions(&mut self) -> usize {
        let now = ic_cdk::api::time() as u128;
        self.cleanup_expired_prepared_transactions(now)
    }
}
