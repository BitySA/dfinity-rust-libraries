use ic_certification::Hash;
use ic_certification::RbTree;

const MAX_U64_ENCODING_BYTES: usize = 10;

/// Creates a hash tree for the last block in the chain.
///
/// This function creates a certification tree containing the last block's index and hash.
/// It is used for data certification in the Internet Computer.
///
/// # Arguments
///
/// * `last_block_index` - The index of the last block
/// * `last_block_hash` - The hash of the last block
///
/// # Returns
///
/// An `RbTree` containing the certification data
///
/// # Example
///
/// ```rust
/// use icrc3_library::utils::last_block_hash_tree;
///
/// let hash_tree = last_block_hash_tree(42u64, [0u8; 32]);
/// ```
pub fn last_block_hash_tree<I, H>(
    last_block_index: I,
    last_block_hash: H,
) -> RbTree<&'static str, Vec<u8>>
where
    I: Into<u64>,
    H: Into<Hash>,
{
    let last_block_index: u64 = last_block_index.into();
    let last_block_hash: Hash = last_block_hash.into();

    let mut hash_tree = RbTree::new();
    let mut last_block_index_buf = Vec::with_capacity(MAX_U64_ENCODING_BYTES);
    leb128::write::unsigned(&mut last_block_index_buf, last_block_index).unwrap();

    hash_tree.insert("last_block_index", last_block_index_buf);
    hash_tree.insert("last_block_hash", last_block_hash.to_vec());

    // FIXME: write the tree somewhere while block update
    // ic_cdk::api::set_certified_data(&hash_tree.root_hash());
    hash_tree
}

/// Prints a debug message to both the IC debug output and standard output.
///
/// # Arguments
///
/// * `msg` - The message to print
use std::borrow::Cow;

pub fn trace<'a>(msg: impl Into<Cow<'a, str>>) {
    let msg: Cow<'a, str> = msg.into();

    #[cfg(feature = "debug-logs")]
    {
        // unsafe {
        //     ic0::debug_print(msg.as_ptr() as usize, msg.len());
        // }
        ic_cdk::println!("{}", msg);
    }

    #[cfg(not(feature = "debug-logs"))]
    {
        let _ = msg; // prevent unused variable warning
    }
}

use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use std::time::Duration;

/// Extracts the timestamp from a transaction as a Duration.
///
/// # Arguments
///
/// * `transaction` - The transaction to extract the timestamp from
///
/// # Returns
///
/// * `Ok(Duration)` if the timestamp is valid
/// * `Err(String)` if the timestamp is invalid or missing
///
/// # Errors
///
/// Returns an error if:
/// * The transaction is not a map
/// * The timestamp field is missing
/// * The timestamp is not a Nat
/// * The timestamp is too large for u64
pub fn get_duration_timestamp(transaction: &ICRC3Value) -> Result<Duration, String> {
    let ICRC3Value::Map(map) = transaction else {
        return Err("top_level is not a valid ICRC3Value::Map".to_string());
    };
    if let Some(value) = map.get("timestamp") {
        if let ICRC3Value::Nat(timestamp) = value {
            let seconds = u64::try_from(timestamp.0.clone())
                .ok()
                .ok_or_else(|| "\"timestamp\" field is too large to fit in u64".to_string())?;
            Ok(Duration::from_secs(seconds))
        } else {
            Err("\"timestamp\" field must be of type Nat".to_string())
        }
    } else {
        Err("\"timestamp\" field not found".to_string())
    }
}

use candid::Nat;

/// Extracts the timestamp from a transaction as a Nat.
///
/// # Arguments
///
/// * `transaction` - The transaction to extract the timestamp from
///
/// # Returns
///
/// * `Ok(Nat)` if the timestamp is valid
/// * `Err(String)` if the timestamp is invalid or missing
///
/// # Errors
///
/// Returns an error if:
/// * The transaction is not a map
/// * The timestamp field is missing
/// * The timestamp is not a Nat
pub fn get_timestamp(transaction: &ICRC3Value) -> Result<Nat, String> {
    let ICRC3Value::Map(map) = transaction else {
        return Err("top_level is not a valid ICRC3Value::Map".to_string());
    };
    if let Some(value) = map.get("timestamp") {
        if let ICRC3Value::Nat(timestamp) = value {
            Ok(timestamp.clone())
        } else {
            Err("\"timestamp\" field must be of type Nat".to_string())
        }
    } else {
        Err("\"timestamp\" field not found".to_string())
    }
}

/// Extracts the transaction hash from a transaction.
///
/// # Arguments
///
/// * `transaction` - The transaction to extract the hash from
///
/// # Returns
///
/// * `Ok(Hash)` if the hash is valid
/// * `Err(String)` if the hash is invalid or missing
///
/// # Errors
///
/// Returns an error if:
/// * The transaction is not a map
/// * The thash field is missing
/// * The thash is not a Blob
/// * The thash is not exactly 32 bytes
pub fn get_transaction_hash(transaction: &ICRC3Value) -> Result<Hash, String> {
    let ICRC3Value::Map(map) = transaction else {
        return Err("top_level is not a valid ICRC3Value::Map".to_string());
    };
    let thash = map.get("thash");
    if let Some(thash) = thash {
        if let ICRC3Value::Blob(thash) = thash {
            let bytes: [u8; 32] = thash
                .as_slice()
                .try_into()
                .map_err(|_| "thash must be exactly 32 bytes".to_string())?;
            Ok(bytes)
        } else {
            Err("thash is not a valid ICRC3Value::Blob".to_string())
        }
    } else {
        Err("thash field not found".to_string())
    }
}

/// Calculates the size of an ICRC3Value in bytes.
///
/// # Arguments
///
/// * `value` - The value to calculate the size of
///
/// # Returns
///
/// The size of the value in bytes
fn get_value_size(value: ICRC3Value) -> u128 {
    match value {
        ICRC3Value::Blob(ref blob) => blob.len() as u128,
        ICRC3Value::Text(ref text) => text.len() as u128,
        ICRC3Value::Nat(ref nat) => nat.0.to_bytes_be().len() as u128,
        ICRC3Value::Int(ref int) => int.0.to_bytes_be().1.len() as u128,
        ICRC3Value::Array(ref array) => array.iter().map(|v| get_value_size(v.clone())).sum(),
        ICRC3Value::Map(ref map) => map
            .iter()
            .map(|(k, v)| (k.len() as u128) + get_value_size(v.clone()))
            .sum(),
    }
}

/// Calculates the total size of a transaction in bytes.
///
/// # Arguments
///
/// * `transaction` - The transaction to calculate the size of
///
/// # Returns
///
/// * `Ok(u128)` containing the size in bytes
/// * `Err(String)` if the transaction is invalid
///
/// # Errors
///
/// Returns an error if the transaction is not a map
pub fn get_transaction_size(transaction: &ICRC3Value) -> Result<u128, String> {
    let ICRC3Value::Map(map) = transaction else {
        return Err("top_level is not a valid ICRC3Value::Map".to_string());
    };
    Ok(map.iter().map(|(_, v)| get_value_size(v.clone())).sum())
}
