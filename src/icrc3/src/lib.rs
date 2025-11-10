//! # ICRC3 Library
//!
//! A comprehensive implementation of the ICRC3 standard for Internet Computer canisters.
//! This library provides a standardized interface for managing transactions and blocks in a blockchain system.
//!
//! ## Features
//!
//! - Transaction management and validation
//! - Blockchain operations
//! - Archive management
//! - Data certification
//! - Configurable throttling and purging
//!
//! ## Quick Start
//!
//! ```rust
//! use icrc3_library::{ICRC3, ICRC3Config};
//!
//! // Create a new ICRC3 instance
//! let config = ICRC3Config::default();
//! let mut icrc3 = ICRC3::new(config);
//!
//! // Add a transaction
//! let result = icrc3.add_transaction(transaction).await;
//!
//! // Get blocks
//! let blocks = icrc3.icrc3_get_blocks(args).await;
//! ```
//!
//! ## Modules
//!
//! - `blockchain`: Core blockchain implementation
//! - `config`: Configuration management
//! - `icrc3`: Main ICRC3 implementation
//! - `interface`: Public interfaces
//! - `transaction`: Transaction handling
//! - `types`: Custom types
//! - `utils`: Utility functions
//!
//! ## Security
//!
//! The library implements several security measures:
//! - Transaction validation
//! - Duplicate protection
//! - DOS protection through throttling
//! - Data certification
//!
//! ## Dependencies
//!
//! - `icrc_ledger_types`
//! - `candid`
//! - `serde_bytes`
//! - `bity_ic_subcanister_manager`

pub mod blockchain;
pub mod config;
pub mod icrc3;
pub mod interface;
pub mod memory;
pub mod transaction;
pub mod types;
pub mod utils;
