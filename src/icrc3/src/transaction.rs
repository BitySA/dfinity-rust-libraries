use bity_ic_types::TimestampSeconds;
use candid::Nat;
use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use icrc_ledger_types::icrc1::account::Account;
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;

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

    /// Returns the transaction data.
    fn tx(&self) -> ICRC3Value;

    fn block_type(&self) -> String;
}

/// A basic transaction type that wraps ICRC3Value.
///
/// This struct provides a simple implementation of a transaction that can be
/// used as a base for more complex transaction types.
#[derive(Clone, Debug)]
pub struct GlobalTransaction(ICRC3Value);

impl GlobalTransaction {
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
    pub fn validate_transaction_fields(&self) -> Result<(), String> {
        match &self.0 {
            ICRC3Value::Map(map) => {
                match map.get("phash") {
                    Some(phash) => match phash {
                        ICRC3Value::Blob(_) => {}
                        _ => return Err("phash is supposed to be a blob".to_string()),
                    },
                    None => return Err("phash field is required".to_string()),
                };

                match map.get("btype") {
                    Some(btype) => match btype {
                        ICRC3Value::Text(_) => Ok(()),
                        _ => Err("btype is supposed to be a string".to_string()),
                    },
                    None => Err("btype field is required".to_string()),
                }
            }
            _ => Err("Transaction is supposed to be a map".to_string()),
        }
    }
}

impl From<GlobalTransaction> for ICRC3Value {
    fn from(tx: GlobalTransaction) -> Self {
        trace(&format!("from {:?}", tx.0));
        tx.0
    }
}

#[derive(Clone, Debug)]
pub struct ICRC1Transaction {
    pub btype: String,
    pub timestamp: u64,
    pub fee: Nat,
    pub tx: ICRC1TransactionData,
}

#[derive(Clone, Debug)]
pub struct ICRC1TransactionData {
    pub op: Option<String>,
    pub amount: Nat,
    pub from: Option<Account>,
    pub to: Option<Account>,
    pub memo: Option<ByteBuf>,
    pub created_at_time: Option<Nat>,
    pub fee: Option<Nat>,
}

impl ICRC1Transaction {
    pub fn new(btype: String, timestamp: u64, fee: Nat, tx: ICRC1TransactionData) -> Self {
        Self {
            btype,
            timestamp,
            fee,
            tx,
        }
    }
}

impl TransactionType for ICRC1Transaction {
    fn validate_transaction_fields(&self) -> Result<(), String> {
        let validate_mint = || -> Result<(), String> {
            if self.tx.to.is_none() {
                return Err("To is required for mint".to_string());
            }
            if self.tx.from.is_some() {
                return Err("From is not allowed for mint".to_string());
            }
            Ok(())
        };

        let validate_burn = || -> Result<(), String> {
            if self.tx.from.is_none() {
                return Err("From is required for burn".to_string());
            }
            if self.tx.to.is_some() {
                return Err("To is not allowed for burn".to_string());
            }
            Ok(())
        };

        let validate_transfer = || -> Result<(), String> {
            if self.tx.from.is_none() {
                return Err("From is required for transfer".to_string());
            }
            if self.tx.to.is_none() {
                return Err("To is required for transfer".to_string());
            }
            Ok(())
        };

        match self.btype.as_str() {
            "1mint" => validate_mint()?,
            "1burn" => validate_burn()?,
            "1xfer" => validate_transfer()?,
            _ => return Err("Invalid ICRC1 transaction type".to_string()),
        }
        match self.tx.op.clone() {
            Some(op) if op == "mint" => validate_mint()?,
            Some(op) if op == "burn" => validate_burn()?,
            Some(op) if op == "xfer" => validate_transfer()?,
            None => {}
            _ => return Err("Invalid ICRC1 transaction type".to_string()),
        }
        Ok(())
    }

    fn timestamp(&self) -> Option<TimestampSeconds> {
        Some(self.timestamp)
    }

    fn tx(&self) -> ICRC3Value {
        self.tx.clone().into()
    }

    fn block_type(&self) -> String {
        self.btype.clone()
    }
}

impl From<ICRC1TransactionData> for ICRC3Value {
    fn from(tx: ICRC1TransactionData) -> Self {
        let mut tx_map = BTreeMap::new();
        if let Some(fee) = tx.fee {
            tx_map.insert("fee".to_string(), ICRC3Value::Nat(fee));
        }
        if let Some(op) = tx.op {
            tx_map.insert("op".to_string(), ICRC3Value::Text(op));
        }
        tx_map.insert("amt".to_string(), ICRC3Value::Nat(tx.amount));
        if let Some(from) = tx.from {
            tx_map.insert("from".to_string(), ICRC3Value::Text(from.owner.to_string()));
        }
        if let Some(to) = tx.to {
            tx_map.insert("to".to_string(), ICRC3Value::Text(to.owner.to_string()));
        }
        if let Some(memo) = tx.memo {
            tx_map.insert("memo".to_string(), ICRC3Value::Blob(memo));
        }
        if let Some(time) = tx.created_at_time {
            tx_map.insert("ts".to_string(), ICRC3Value::Nat(time));
        }
        ICRC3Value::Map(tx_map)
    }
}

impl From<ICRC1Transaction> for ICRC3Value {
    fn from(tx: ICRC1Transaction) -> Self {
        let mut map = BTreeMap::new();
        map.insert("btype".to_string(), ICRC3Value::Text(tx.btype));
        map.insert("ts".to_string(), ICRC3Value::Nat(Nat::from(tx.timestamp)));

        let tx_value = tx.tx.into();
        map.insert("tx".to_string(), tx_value);

        ICRC3Value::Map(map)
    }
}

#[derive(Clone, Debug)]
pub struct ICRC2Transaction {
    pub btype: String,
    pub timestamp: u64,
    pub fee: Option<Nat>,
    pub tx: ICRC2TransactionData,
}

#[derive(Clone, Debug)]
pub struct ICRC2TransactionData {
    pub op: Option<String>,
    pub amount: Nat,
    pub from: Option<Account>,
    pub to: Option<Account>,
    pub spender: Option<Account>,
    pub memo: Option<ByteBuf>,
    pub expected_allowance: Option<Nat>,
    pub expires_at: Option<Nat>,
}

impl ICRC2Transaction {
    pub fn new(btype: String, timestamp: u64, fee: Option<Nat>, tx: ICRC2TransactionData) -> Self {
        Self {
            btype,
            timestamp,
            fee,
            tx,
        }
    }
}

impl TransactionType for ICRC2Transaction {
    fn validate_transaction_fields(&self) -> Result<(), String> {
        let validate_transfer = || -> Result<(), String> {
            if self.tx.from.is_none() {
                return Err("From is required for transfer".to_string());
            }
            if self.tx.to.is_none() {
                return Err("To is required for transfer".to_string());
            }
            Ok(())
        };
        let validate_approve = || -> Result<(), String> {
            if self.tx.from.is_none() {
                return Err("From is required for approve".to_string());
            }
            if self.tx.spender.is_none() {
                return Err("Spender is required for approve".to_string());
            }
            if self.tx.to.is_some() {
                return Err("To is not allowed for approve".to_string());
            }

            Ok(())
        };
        match self.btype.as_str() {
            "2xfer" => validate_transfer()?,
            "2approve" => validate_approve()?,
            _ => return Err("Invalid ICRC2 transaction type".to_string()),
        }

        match self.tx.op.clone() {
            Some(op) if op == "xfer" => validate_transfer()?,
            Some(op) if op == "approve" => validate_approve()?,
            None => {}
            _ => return Err("Invalid ICRC2 transaction type".to_string()),
        }
        Ok(())
    }

    fn timestamp(&self) -> Option<TimestampSeconds> {
        Some(self.timestamp)
    }

    fn tx(&self) -> ICRC3Value {
        self.tx.clone().into()
    }

    fn block_type(&self) -> String {
        self.btype.clone()
    }
}

impl From<ICRC2TransactionData> for ICRC3Value {
    fn from(tx: ICRC2TransactionData) -> Self {
        let mut tx_map = BTreeMap::new();
        if let Some(op) = tx.op {
            tx_map.insert("op".to_string(), ICRC3Value::Text(op));
        }
        tx_map.insert("amt".to_string(), ICRC3Value::Nat(tx.amount));
        if let Some(from) = tx.from {
            tx_map.insert("from".to_string(), ICRC3Value::Text(from.owner.to_string()));
        }
        if let Some(to) = tx.to {
            tx_map.insert("to".to_string(), ICRC3Value::Text(to.owner.to_string()));
        }
        if let Some(spender) = tx.spender {
            tx_map.insert(
                "spender".to_string(),
                ICRC3Value::Text(spender.owner.to_string()),
            );
        }
        if let Some(memo) = tx.memo {
            tx_map.insert("memo".to_string(), ICRC3Value::Blob(memo));
        }
        if let Some(expected_allowance) = tx.expected_allowance {
            tx_map.insert(
                "expected_allowance".to_string(),
                ICRC3Value::Nat(expected_allowance),
            );
        }
        if let Some(expires_at) = tx.expires_at {
            tx_map.insert("expires_at".to_string(), ICRC3Value::Nat(expires_at));
        }
        ICRC3Value::Map(tx_map)
    }
}

impl From<ICRC2Transaction> for ICRC3Value {
    fn from(tx: ICRC2Transaction) -> Self {
        let mut map = BTreeMap::new();
        map.insert("btype".to_string(), ICRC3Value::Text(tx.btype));
        map.insert(
            "timestamp".to_string(),
            ICRC3Value::Nat(Nat::from(tx.timestamp)),
        );
        if let Some(fee) = tx.fee {
            map.insert("fee".to_string(), ICRC3Value::Nat(fee));
        }

        let tx_value = tx.tx.into();
        map.insert("tx".to_string(), tx_value);

        ICRC3Value::Map(map)
    }
}

#[derive(Clone, Debug)]
pub struct ICRC7Transaction {
    pub btype: String,
    pub timestamp: u64,
    pub tx: ICRC7TransactionData,
}

#[derive(Clone, Debug)]
pub struct ICRC7TransactionData {
    pub op: String, // need to be == to btype
    pub tid: Option<Nat>,
    pub from: Option<Account>,
    pub to: Option<Account>,
    pub meta: Option<ICRC3Value>,
    pub memo: Option<ByteBuf>,
    pub created_at_time: Option<Nat>,
}

impl ICRC7Transaction {
    pub fn new(btype: String, timestamp: u64, tx: ICRC7TransactionData) -> Self {
        Self {
            btype,
            timestamp,
            tx,
        }
    }
}

impl TransactionType for ICRC7Transaction {
    fn validate_transaction_fields(&self) -> Result<(), String> {
        if self.btype != self.tx.op {
            return Err("btype and op must be the same".to_string());
        }
        match self.btype.as_str() {
            "7mint" => {
                if self.tx.tid.is_none() {
                    return Err("Token ID is required for mint".to_string());
                }
                if self.tx.from.is_some() {
                    return Err("From is not allowed for mint".to_string());
                }
                if self.tx.to.is_none() {
                    return Err("To is required for mint".to_string());
                }
                if self.tx.meta.is_some() {
                    return Err("Meta is not allowed for mint".to_string());
                }
            }
            "7burn" => {
                if self.tx.tid.is_none() {
                    return Err("Token ID is required for burn".to_string());
                }
                if self.tx.from.is_none() {
                    return Err("From is required for burn".to_string());
                }
                if self.tx.to.is_some() {
                    return Err("To is not allowed for burn".to_string());
                }
                if self.tx.meta.is_some() {
                    return Err("Meta is not allowed for burn".to_string());
                }
            }
            "7xfer" => {
                if self.tx.tid.is_none() {
                    return Err("Token ID is required for transfer".to_string());
                }
                if self.tx.from.is_none() {
                    return Err("From is required for transfer".to_string());
                }
                if self.tx.to.is_none() {
                    return Err("To is required for transfer".to_string());
                }
                if self.tx.meta.is_some() {
                    return Err("Meta is not allowed for transfer".to_string());
                }
            }
            "7update_token" => {
                if self.tx.tid.is_none() {
                    return Err("Token ID is required for update_token".to_string());
                }
                if self.tx.from.is_none() {
                    return Err("From is required for update_token".to_string());
                }
                if self.tx.to.is_some() {
                    return Err("To is not allowed for update_token".to_string());
                }
                if self.tx.meta.is_none() {
                    return Err("Meta is required for update_token".to_string());
                }
            }
            _ => return Err("Invalid ICRC7 transaction type".to_string()),
        }
        Ok(())
    }

    fn timestamp(&self) -> Option<TimestampSeconds> {
        Some(self.timestamp)
    }

    fn block_type(&self) -> String {
        self.btype.clone()
    }

    fn tx(&self) -> ICRC3Value {
        self.tx.clone().into()
    }
}

impl From<ICRC7TransactionData> for ICRC3Value {
    fn from(tx: ICRC7TransactionData) -> Self {
        let mut tx_map = BTreeMap::new();
        if let Some(tid) = tx.tid {
            tx_map.insert("tid".to_string(), ICRC3Value::Nat(tid));
        }
        if let Some(from) = tx.from {
            tx_map.insert("from".to_string(), ICRC3Value::Text(from.owner.to_string()));
        }
        if let Some(to) = tx.to {
            tx_map.insert("to".to_string(), ICRC3Value::Text(to.owner.to_string()));
        }
        if let Some(meta) = tx.meta {
            tx_map.insert("meta".to_string(), meta);
        }
        if let Some(memo) = tx.memo {
            tx_map.insert("memo".to_string(), ICRC3Value::Blob(memo));
        }
        if let Some(time) = tx.created_at_time {
            tx_map.insert("created_at_time".to_string(), ICRC3Value::Nat(time));
        }
        ICRC3Value::Map(tx_map)
    }
}

impl From<ICRC7Transaction> for ICRC3Value {
    fn from(tx: ICRC7Transaction) -> Self {
        let mut map = BTreeMap::new();
        map.insert("btype".to_string(), ICRC3Value::Text(tx.btype));
        map.insert(
            "timestamp".to_string(),
            ICRC3Value::Nat(Nat::from(tx.timestamp)),
        );

        let tx_value = tx.tx.into();
        map.insert("tx".to_string(), tx_value);

        ICRC3Value::Map(map)
    }
}

#[derive(Clone, Debug)]
pub struct ICRC37Transaction {
    pub btype: String,
    pub timestamp: u64,
    pub tx: ICRC37TransactionData,
}

#[derive(Clone, Debug)]
pub struct ICRC37TransactionData {
    pub op: String, // need to be == to btype
    pub tid: Option<Nat>,
    pub from: Option<Account>,
    pub to: Option<Account>,
    pub memo: Option<ByteBuf>,
    pub created_at_time: Option<Nat>,
    pub spender: Option<Account>,
    pub exp: Option<Nat>,
}

impl ICRC37Transaction {
    pub fn new(btype: String, timestamp: u64, tx: ICRC37TransactionData) -> Self {
        Self {
            btype,
            timestamp,
            tx,
        }
    }
}

impl TransactionType for ICRC37Transaction {
    fn validate_transaction_fields(&self) -> Result<(), String> {
        if self.btype != self.tx.op {
            return Err("btype and op must be the same".to_string());
        }
        match self.btype.as_str() {
            "37approve" => {
                if self.tx.tid.is_none() {
                    return Err("Token ID is required for token approval".to_string());
                }
                if self.tx.from.is_none() {
                    return Err("From is required for token approval".to_string());
                }
                if self.tx.to.is_some() {
                    return Err("To is not allowed for token approval".to_string());
                }
                if self.tx.spender.is_none() {
                    return Err("Spender is required for token approval".to_string());
                }
            }
            "37approve_coll" => {
                if self.tx.tid.is_some() {
                    return Err("Token ID is not allowed for collection approval".to_string());
                }
                if self.tx.from.is_none() {
                    return Err("From is required for collection approval".to_string());
                }
                if self.tx.to.is_some() {
                    return Err("To is not allowed for collection approval".to_string());
                }
                if self.tx.spender.is_none() {
                    return Err("Spender is required for collection approval".to_string());
                }
            }
            "37revoke" => {
                if self.tx.tid.is_none() {
                    return Err("Token ID is required for token revocation".to_string());
                }
                if self.tx.from.is_none() {
                    return Err("From is required for token revocation".to_string());
                }
                if self.tx.to.is_some() {
                    return Err("To is not allowed for token revocation".to_string());
                }
                if self.tx.exp.is_some() {
                    return Err("Exp is not allowed for token revocation".to_string());
                }
            }
            "37revoke_coll" => {
                if self.tx.tid.is_some() {
                    return Err("Token ID is not allowed for collection revocation".to_string());
                }
                if self.tx.from.is_none() {
                    return Err("From is required for collection revocation".to_string());
                }
                if self.tx.to.is_some() {
                    return Err("To is not allowed for collection revocation".to_string());
                }
                if self.tx.exp.is_some() {
                    return Err("Exp is not allowed for collection revocation".to_string());
                }
            }
            "37xfer" => {
                if self.tx.tid.is_none() {
                    return Err("Token ID is required for transfer from".to_string());
                }
                if self.tx.from.is_none() {
                    return Err("From is required for transfer from".to_string());
                }
                if self.tx.to.is_none() {
                    return Err("To is required for transfer from".to_string());
                }
                if self.tx.spender.is_none() {
                    return Err("Spender is required for transfer from".to_string());
                }
                if self.tx.exp.is_some() {
                    return Err("Exp is not allowed for transfer from".to_string());
                }
            }
            _ => return Err("Invalid ICRC37 transaction type".to_string()),
        }
        Ok(())
    }

    fn timestamp(&self) -> Option<TimestampSeconds> {
        Some(self.timestamp)
    }

    fn block_type(&self) -> String {
        self.btype.clone()
    }

    fn tx(&self) -> ICRC3Value {
        self.tx.clone().into()
    }
}

impl From<ICRC37TransactionData> for ICRC3Value {
    fn from(tx: ICRC37TransactionData) -> Self {
        let mut tx_map = BTreeMap::new();
        if let Some(tid) = tx.tid {
            tx_map.insert("tid".to_string(), ICRC3Value::Nat(tid));
        }
        if let Some(from) = tx.from {
            tx_map.insert("from".to_string(), ICRC3Value::Text(from.owner.to_string()));
        }
        if let Some(spender) = tx.spender {
            tx_map.insert(
                "spender".to_string(),
                ICRC3Value::Text(spender.owner.to_string()),
            );
        }
        if let Some(exp) = tx.exp {
            tx_map.insert("exp".to_string(), ICRC3Value::Nat(exp));
        }
        if let Some(to) = tx.to {
            tx_map.insert("to".to_string(), ICRC3Value::Text(to.owner.to_string()));
        }
        if let Some(memo) = tx.memo {
            tx_map.insert("memo".to_string(), ICRC3Value::Blob(memo));
        }
        if let Some(time) = tx.created_at_time {
            tx_map.insert("created_at_time".to_string(), ICRC3Value::Nat(time));
        }
        ICRC3Value::Map(tx_map)
    }
}

impl From<ICRC37Transaction> for ICRC3Value {
    fn from(tx: ICRC37Transaction) -> Self {
        let mut map = BTreeMap::new();
        map.insert("btype".to_string(), ICRC3Value::Text(tx.btype));
        map.insert(
            "timestamp".to_string(),
            ICRC3Value::Nat(Nat::from(tx.timestamp)),
        );

        let tx_value = tx.tx.into();
        map.insert("tx".to_string(), tx_value);

        ICRC3Value::Map(map)
    }
}
