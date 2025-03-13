//! Client utilities for Internet Computer canister interactions.
//!
//! This module provides low-level utilities for making cross-canister calls (C2C)
//! with support for different serialization formats and cycle payments.
//!
//! # Features
//! - Cross-canister calls with custom serialization/deserialization
//! - Support for cycle payments in C2C calls
//! - Raw C2C call functionality with detailed error handling
//! - Integration with tracing for debugging and monitoring
//!
//! # Examples
//! ```
//! use bity_dfinity_library::canister_client::make_c2c_call;
//! use candid::{encode_one, decode_one};
//!
//! async fn transfer(
//!     canister_id: Principal,
//!     args: &TransferArgs,
//! ) -> CallResult<TransferResponse> {
//!     make_c2c_call(
//!         canister_id,
//!         "transfer",
//!         args,
//!         encode_one,
//!         |r| decode_one(r),
//!     )
//!     .await
//! }
//! ```

use candid::Principal;
use ic_cdk::api::call::{CallResult, RejectionCode};
use std::fmt::Debug;

pub use bity_ic_canister_client_macros::*;

/// Makes a cross-canister call with custom serialization and deserialization.
///
/// This function handles the complete flow of a cross-canister call, including:
/// - Serialization of arguments
/// - Making the actual call
/// - Deserialization of the response
///
/// # Type Parameters
/// * `A` - The type of the arguments
/// * `R` - The type of the response
/// * `S` - The type of the serializer function
/// * `D` - The type of the deserializer function
/// * `SError` - The error type of the serializer
/// * `DError` - The error type of the deserializer
///
/// # Arguments
/// * `canister_id` - The ID of the target canister
/// * `method_name` - The name of the method to call
/// * `args` - The arguments to pass to the method
/// * `serializer` - Function to serialize the arguments
/// * `deserializer` - Function to deserialize the response
///
/// # Returns
/// A `CallResult` containing either the deserialized response or an error.
///
/// # Example
/// ```
/// use bity_dfinity_library::canister_client::make_c2c_call;
/// use candid::{encode_one, decode_one};
///
/// async fn example(canister_id: Principal, args: &MyArgs) -> CallResult<MyResponse> {
///     make_c2c_call(
///         canister_id,
///         "my_method",
///         args,
///         encode_one,
///         |r| decode_one(r),
///     )
///     .await
/// }
/// ```
pub async fn make_c2c_call<A, R, S, D, SError: Debug, DError: Debug>(
    canister_id: Principal,
    method_name: &str,
    args: A,
    serializer: S,
    deserializer: D,
) -> CallResult<R>
where
    S: Fn(A) -> Result<Vec<u8>, SError>,
    D: Fn(&[u8]) -> Result<R, DError>,
{
    let payload_bytes = serializer(args).map_err(|e| {
        (
            RejectionCode::CanisterError,
            format!("Serialization error: {:?}", e),
        )
    })?;

    let response_bytes = make_c2c_call_raw(canister_id, method_name, &payload_bytes, 0).await?;

    deserializer(&response_bytes).map_err(|e| {
        (
            RejectionCode::CanisterError,
            format!("Deserialization error: {:?}", e),
        )
    })
}

/// Makes a cross-canister call with cycle payment and custom serialization.
///
/// This function is similar to `make_c2c_call` but includes cycle payment support.
/// It allows specifying the number of cycles to be transferred with the call.
///
/// # Type Parameters
/// * `A` - The type of the arguments
/// * `R` - The type of the response
/// * `S` - The type of the serializer function
/// * `D` - The type of the deserializer function
/// * `SError` - The error type of the serializer
/// * `DError` - The error type of the deserializer
///
/// # Arguments
/// * `canister_id` - The ID of the target canister
/// * `method_name` - The name of the method to call
/// * `args` - The arguments to pass to the method
/// * `serializer` - Function to serialize the arguments
/// * `deserializer` - Function to deserialize the response
/// * `cycles` - The number of cycles to transfer with the call
///
/// # Returns
/// A `CallResult` containing either the deserialized response or an error.
///
/// # Example
/// ```
/// use bity_dfinity_library::canister_client::make_c2c_call_with_payment;
/// use candid::{encode_one, decode_one};
///
/// async fn example(canister_id: Principal, args: &MyArgs, cycles: u128) -> CallResult<MyResponse> {
///     make_c2c_call_with_payment(
///         canister_id,
///         "my_method",
///         args,
///         encode_one,
///         |r| decode_one(r),
///         cycles,
///     )
///     .await
/// }
/// ```
pub async fn make_c2c_call_with_payment<A, R, S, D, SError: Debug, DError: Debug>(
    canister_id: Principal,
    method_name: &str,
    args: A,
    serializer: S,
    deserializer: D,
    cycles: u128,
) -> CallResult<R>
where
    S: Fn(A) -> Result<Vec<u8>, SError>,
    D: Fn(&[u8]) -> Result<R, DError>,
{
    let payload_bytes = serializer(args).map_err(|e| {
        (
            RejectionCode::CanisterError,
            format!("Serialization error: {:?}", e),
        )
    })?;

    let response_bytes =
        make_c2c_call_raw(canister_id, method_name, &payload_bytes, cycles).await?;

    deserializer(&response_bytes).map_err(|e| {
        (
            RejectionCode::CanisterError,
            format!("Deserialization error: {:?}", e),
        )
    })
}

/// Makes a raw cross-canister call with byte-level control.
///
/// This is the lowest-level function for making cross-canister calls. It handles
/// the actual call to the Internet Computer and includes tracing for debugging.
///
/// # Arguments
/// * `canister_id` - The ID of the target canister
/// * `method_name` - The name of the method to call
/// * `payload_bytes` - The raw bytes to send as the payload
/// * `cycles` - The number of cycles to transfer with the call
///
/// # Returns
/// A `CallResult` containing either the raw response bytes or an error.
///
/// # Example
/// ```
/// use bity_dfinity_library::canister_client::make_c2c_call_raw;
///
/// async fn example(canister_id: Principal, payload: &[u8]) -> CallResult<Vec<u8>> {
///     make_c2c_call_raw(canister_id, "my_method", payload, 0).await
/// }
/// ```
pub async fn make_c2c_call_raw(
    canister_id: Principal,
    method_name: &str,
    payload_bytes: &[u8],
    cycles: u128,
) -> CallResult<Vec<u8>> {
    tracing::trace!(method_name, %canister_id, "Starting c2c call");

    let response =
        ic_cdk::api::call::call_raw128(canister_id, method_name, payload_bytes, cycles).await;

    match response {
        Ok(response_bytes) => {
            tracing::trace!(method_name, %canister_id, "Completed c2c call successfully");
            Ok(response_bytes)
        }
        Err((error_code, error_message)) => {
            tracing::error!(method_name, %canister_id, ?error_code, error_message, "Error calling c2c");
            Err((error_code, error_message))
        }
    }
}
