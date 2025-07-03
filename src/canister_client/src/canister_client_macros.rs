//! Macros for generating Internet Computer canister client code.
//!
//! This module provides a set of macros to generate boilerplate code for interacting with
//! Internet Computer canisters. It supports various types of calls including update calls,
//! query calls, and cross-canister calls (C2C) with different serialization formats.
//!
//! # Examples
//! ```
//! use bity_ic_canister_client::*;
//!
//! // Generate an update call function
//! generate_update_call!(my_method);
//!
//! // Generate a query call function
//! generate_query_call!(get_data);
//!
//! // Generate a cross-canister call function
//! generate_c2c_call!(transfer);
//! ```
//!
//! # Features
//! - Update call generation with Candid serialization
//! - Query call generation with Candid serialization
//! - Cross-canister call generation with both Candid and MessagePack serialization
//! - Support for calls with and without arguments
//! - Support for calls with cycle payments

/// Generates a function for making update calls to a canister.
///
/// This macro creates an async function that handles the serialization, call, and
/// deserialization of update calls using Candid.
///
/// # Arguments
/// * `method_name` - The name of the method to generate
///
/// # Returns
/// A function that takes an agent, canister ID, and arguments, and returns a Result
/// with the decoded response or an error.
///
/// # Example
/// ```
/// use bity_ic_canister_client::generate_update_call;
///
/// generate_update_call!(transfer);
/// ```
#[macro_export]
macro_rules! generate_update_call {
    ($method_name:ident) => {
        pub async fn $method_name(
            agent: &ic_agent::Agent,
            canister_id: &candid::Principal,
            args: &$method_name::Args,
        ) -> Result<$method_name::Response, Box<dyn std::error::Error + Sync + std::marker::Send>> {
            use candid::{Decode, Encode};

            let candid_args = Encode!(args)?;

            let method_name = stringify!($method_name);
            let response = agent
                .update(canister_id, method_name)
                .with_arg(candid_args)
                .call_and_wait()
                .await?;

            let result = Decode!(response.as_slice(), $method_name::Response)?;
            Ok(result)
        }
    };
}

/// Generates a function for making query calls to a canister.
///
/// This macro creates an async function that handles the serialization, call, and
/// deserialization of query calls using Candid.
///
/// # Arguments
/// * `method_name` - The name of the method to generate
///
/// # Returns
/// A function that takes an agent, canister ID, and arguments, and returns a Result
/// with the decoded response or an error.
///
/// # Example
/// ```
/// use bity_ic_canister_client::generate_query_call;
///
/// generate_query_call!(get_balance);
/// ```
#[macro_export]
macro_rules! generate_query_call {
    ($method_name:ident) => {
        pub async fn $method_name(
            agent: &ic_agent::Agent,
            canister_id: &candid::Principal,
            args: &$method_name::Args,
        ) -> Result<
            $method_name::Response,
            Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>,
        > {
            use candid::{Decode, Encode};

            let candid_args = Encode!(args)?;

            let method_name = stringify!($method_name);
            let response = agent
                .query(canister_id, method_name)
                .with_arg(candid_args)
                .call()
                .await?;

            Ok(Decode!(response.as_slice(), $method_name::Response)?)
        }
    };
}

/// Generates a function for making cross-canister calls using MessagePack serialization.
///
/// This macro creates an async function that handles cross-canister calls with
/// MessagePack serialization for better performance.
///
/// # Arguments
/// * `method_name` - The name of the method to generate
///
/// # Returns
/// A function that takes a canister ID and arguments, and returns a CallResult
/// with the decoded response.
///
/// # Example
/// ```
/// use bity_ic_canister_client::generate_c2c_call;
///
/// generate_c2c_call!(transfer);
/// ```
#[macro_export]
macro_rules! generate_c2c_call {
    ($method_name:ident) => {
        pub async fn $method_name<R>(
            canister_id: bity_ic_types::CanisterId,
            args: &$method_name::Args,
        ) -> Result<R>
        where
            for<'de> R: ::serde::Deserialize<'de>,
        {
            let method_name = concat!(stringify!($method_name), "_msgpack");

            bity_ic_canister_client::make_c2c_call(
                canister_id,
                method_name,
                args,
                msgpack::serialize,
                |r| msgpack::deserialize(r),
            )
            .await
        }
    };
}

/// Generates a function for making cross-canister calls using Candid serialization.
///
/// This macro creates an async function that handles cross-canister calls with
/// Candid serialization. It supports both single-argument and tuple-argument methods.
///
/// # Arguments
/// * `method_name` - The name of the method to generate
/// * `external_canister_method_name` - (Optional) The name of the method on the target canister
///
/// # Returns
/// A function that takes a canister ID and arguments, and returns a CallResult
/// with the decoded response.
///
/// # Example
/// ```
/// use bity_ic_canister_client::generate_candid_c2c_call;
///
/// generate_candid_c2c_call!(transfer);
/// ```
pub extern crate anyhow;
pub use anyhow::Result;
pub extern crate candid;
pub extern crate serde;
pub use candid::CandidType;

#[macro_export]
macro_rules! generate_candid_c2c_call {
    ($method_name:ident) => {
        generate_candid_c2c_call!($method_name, $method_name);
    };
    ($method_name:ident, $external_canister_method_name:ident) => {
        pub async fn $method_name<A, R>(
            canister_id: bity_ic_types::CanisterId,
            args: A,
        ) -> $crate::Result<R>
        where
            A: std::borrow::Borrow<$method_name::Args>,
            for<'de> R: ::serde::Deserialize<'de> + ::candid::CandidType,
        {
            let method_name = stringify!($external_canister_method_name);

            ::bity_ic_canister_client::make_c2c_call(
                canister_id,
                method_name,
                args.borrow(),
                candid::encode_one,
                |r| candid::decode_one(r),
            )
            .await
        }
    };
}

/// Generates a function for making cross-canister calls with cycle payment.
///
/// This macro creates an async function that handles cross-canister calls with
/// Candid serialization and cycle payment.
///
/// # Arguments
/// * `method_name` - The name of the method to generate
///
/// # Returns
/// A function that takes a canister ID, arguments, and cycles amount, and returns
/// a CallResult with the decoded response.
///
/// # Example
/// ```
/// use bity_ic_canister_client::generate_candid_c2c_call_with_payment;
///
/// generate_candid_c2c_call_with_payment!(transfer);
/// ```
#[macro_export]
macro_rules! generate_candid_c2c_call_with_payment {
    ($method_name:ident) => {
        pub async fn $method_name<R>(
            canister_id: bity_ic_types::CanisterId,
            args: &$method_name::Args,
            cycles: ::types::Cycles,
        ) -> ::anyhow::Result<R> {
            let method_name = stringify!($method_name);

            bity_ic_canister_client::make_c2c_call_with_payment(
                canister_id,
                method_name,
                args,
                ::candid::encode_one,
                |r| ::candid::decode_one(r),
                cycles,
            )
            .await
        }
    };
}

/// Generates a function for making cross-canister calls with tuple arguments.
///
/// This macro creates an async function that handles cross-canister calls with
/// Candid serialization and tuple arguments.
///
/// # Arguments
/// * `method_name` - The name of the method to generate
/// * `external_canister_method_name` - (Optional) The name of the method on the target canister
///
/// # Returns
/// A function that takes a canister ID and tuple arguments, and returns a CallResult
/// with the decoded response.
///
/// # Example
/// ```
/// use bity_ic_canister_client::generate_candid_c2c_call_tuple_args;
///
/// generate_candid_c2c_call_tuple_args!(transfer);
/// ```
#[macro_export]
macro_rules! generate_candid_c2c_call_tuple_args {
    ($method_name:ident) => {
        ::bity_ic_canister_client::generate_candid_c2c_call_tuple_args!($method_name, $method_name);
    };
    ($method_name:ident, $external_canister_method_name:ident) => {
        pub async fn $method_name<R>(
            canister_id: bity_ic_types::CanisterId,
            args: $method_name::Args,
        ) -> ::anyhow::Result<R> {
            let method_name = stringify!($external_canister_method_name);

            bity_ic_canister_client::make_c2c_call(
                canister_id,
                method_name,
                args,
                ::candid::encode_args,
                |r| ::candid::decode_args(r),
            )
            .await
        }
    };
}

/// Generates a function for making cross-canister calls without arguments.
///
/// This macro creates an async function that handles cross-canister calls with
/// Candid serialization and no arguments.
///
/// # Arguments
/// * `method_name` - The name of the method to generate
/// * `external_canister_method_name` - (Optional) The name of the method on the target canister
///
/// # Returns
/// A function that takes only a canister ID and returns a CallResult
/// with the decoded response.
///
/// # Example
/// ```
/// use bity_ic_canister_client::generate_candid_c2c_call_no_args;
///
/// generate_candid_c2c_call_no_args!(get_version);
/// ```
#[macro_export]
macro_rules! generate_candid_c2c_call_no_args {
    ($method_name:ident) => {
        ::bity_ic_canister_client::generate_candid_c2c_call_no_args!($method_name, $method_name);
    };
    ($method_name:ident, $external_canister_method_name:ident) => {
        pub async fn $method_name<R>(
            canister_id: bity_ic_types::CanisterId,
        ) -> ::anyhow::Result<R> {
            let method_name = stringify!($external_canister_method_name);

            bity_ic_canister_client::make_c2c_call(
                canister_id,
                method_name,
                (),
                candid::encode_one,
                |r| candid::decode_one(r),
            )
            .await
        }
    };
}
