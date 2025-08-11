//! Common types and utilities for Internet Computer development.
//!
//! This crate provides a collection of commonly used types and type aliases
//! for developing applications on the Internet Computer platform.

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

mod build_version;

pub use build_version::*;

/// Represents an empty type, useful for functions that don't need to return data
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct Empty {}

/// Type alias for an Internet Computer canister ID
pub type CanisterId = Principal;
/// Type alias for WebAssembly binary data representing a canister
pub type CanisterWasm = Vec<u8>;
/// Type alias for cycle amounts in the Internet Computer
pub type Cycles = u64;
/// Type alias for a 32-byte hash value
pub type Hash = [u8; 32];
/// Type alias for neuron maturity values
pub type Maturity = u64;
/// Type alias for time durations in milliseconds
pub type Milliseconds = u64;
/// Type alias for Network Nervous System neuron IDs
pub type NnsNeuronId = u64;
/// Type alias for proposal IDs
pub type ProposalId = u64;
/// Type alias for Service Nervous System neuron IDs
pub type SnsNeuronId = [u8; 32];
/// Type alias for Unix timestamps in seconds
pub type TimestampSeconds = u64;
/// Type alias for Unix timestamps in milliseconds
pub type TimestampMillis = u64;
/// Type alias for Unix timestamps in nanoseconds
pub type TimestampNanos = u64;
/// Type alias for seconds
pub type Second = u64;
