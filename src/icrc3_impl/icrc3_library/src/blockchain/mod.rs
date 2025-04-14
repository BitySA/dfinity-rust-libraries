//! Blockchain module for ICRC3 implementation.
//!
//! This module provides the core blockchain functionality for the ICRC3 implementation,
//! including block management, archiving, and canister management.
//!
//! # Components
//!
//! * `archive_canister` - Manages individual archive canisters
//! * `archive_canister_manager` - Manages multiple archive canisters
//! * `blockchain` - Core blockchain implementation

pub mod archive_canister;
pub mod archive_canister_manager;
pub mod blockchain;
