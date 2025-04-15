use bity_ic_types::TimestampSeconds;
use icrc_ledger_types::icrc::generic_value::ICRC3Value;

use crate::utils::trace;

/// The length of transaction hashes in bytes
pub const HASH_LENGTH: usize = 32;
/// The type representing a transaction hash
pub type Hash = [u8; HASH_LENGTH];

/// Trait defining the interface for transaction types.
///
/// This trait must be implemented by any type that represents a transaction
/// in the ICRC3 system. It provides methods for validation, timestamping,
/// and hashing of transactions.
///
/// # Examples
///
/// ```rust
/// use icrc3_library::transaction::TransactionType;
///
/// struct MyTransaction {
///     // transaction fields
/// }
///
/// impl TransactionType for MyTransaction {
///     fn validate_transaction_fields(&self) -> Result<(), String> {
///         // validation logic
///         Ok(())
///     }
///     
///     fn timestamp(&self) -> Option<TimestampSeconds> {
///         // timestamp logic
///         None
///     }
///     
///     fn hash(&self) -> Hash {
///         // hashing logic
///         [0u8; 32]
///     }
/// }
/// ```
pub trait TransactionType: Sized + Clone + Into<ICRC3Value> {
    /// Validates the transaction-specific fields.
    ///
    /// This method should check that all fields specific to this transaction type
    /// are valid according to the transaction's rules and constraints.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the transaction is valid
    /// * `Err(String)` containing an error message if invalid
    fn validate_transaction_fields(&self) -> Result<(), String>;

    /// Returns the timestamp of the transaction if available.
    fn timestamp(&self) -> Option<TimestampSeconds>;

    /// Computes and returns the hash of the transaction.
    fn hash(&self) -> Hash;
}

/// Enum representing different kinds of transactions supported by the system.
#[derive(Debug)]
pub enum TransactionKind {
    /// A standard ICRC1 transaction
    Icrc1Transaction,
    /// A custom transaction type
    CustomTransaction,
}

/// A basic transaction type that wraps ICRC3Value.
///
/// This struct provides a simple implementation of a transaction that can be
/// used as a base for more complex transaction types.
#[derive(Clone, Debug)]
pub struct BasicTransaction(ICRC3Value);

impl BasicTransaction {
    /// Creates a new basic transaction from an ICRC3Value.
    ///
    /// # Arguments
    ///
    /// * `value` - The ICRC3Value to wrap
    pub fn new(value: ICRC3Value) -> Self {
        Self(value)
    }

    /// Validates the transaction fields according to ICRC3 standards.
    ///
    /// # Returns
    ///
    /// * `Ok(TransactionKind)` if the transaction is valid
    /// * `Err(String)` containing an error message if invalid
    ///
    /// # Validation Rules
    ///
    /// The transaction must:
    /// * Be a map
    /// * Contain a "phash" field that is a blob
    /// * Contain a "btype" field that is a string
    pub fn validate_transaction_fields(&self) -> Result<TransactionKind, String> {
        trace(&format!("validate_transaction_fields {:?}", self.0));
        match &self.0 {
            ICRC3Value::Map(map) => {
                trace(&format!("validate_transaction_fields {:?}", map));

                let phash = match map.get("phash") {
                    Some(phash) => match phash {
                        ICRC3Value::Blob(_) => {
                            trace(&format!("phash is a blob"));
                        }
                        _ => return Err("phash is supposed to be a blob".to_string()),
                    },
                    None => return Err("phash field is required".to_string()),
                };

                match map.get("btype") {
                    Some(btype) => match btype {
                        ICRC3Value::Text(_) => {
                            trace(&format!("btype is a text"));
                            Ok(TransactionKind::CustomTransaction)
                        }
                        _ => Err("btype is supposed to be a string".to_string()),
                    },
                    None => Err("btype field is required".to_string()),
                }
            }
            _ => Err("Transaction is supposed to be a map".to_string()),
        }
    }
}

impl From<BasicTransaction> for ICRC3Value {
    fn from(tx: BasicTransaction) -> Self {
        trace(&format!("from {:?}", tx.0));
        tx.0
    }
}

// TODO add ICRC1Transaction type
