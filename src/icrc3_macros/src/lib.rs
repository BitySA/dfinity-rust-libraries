//! Module for managing ICRC3 state in Internet Computer canisters.
//!
//! This module provides macros for creating thread-safe ICRC3 state management in canisters,
//! with functions for initialization and direct access to ICRC3 interface methods.

/// A macro that generates thread-safe ICRC3 state management functions.
///
/// This macro creates a set of functions for managing ICRC3 state in a thread-safe manner.
/// It provides direct access to all ICRC3 interface methods.
///
/// # Generated Functions
/// * `init_icrc3()` - Initializes the ICRC3 state
/// * `add_transaction(transaction: T) -> Result<u64, Icrc3Error>` - Adds a new transaction
/// * `icrc3_get_archives() -> Vec<ICRC3ArchiveInfo>` - Gets information about archives
/// * `icrc3_get_blocks(args: Vec<GetBlocksRequest>) -> Response` - Gets blocks
/// * `icrc3_get_properties() -> Response` - Gets blockchain properties
/// * `icrc3_get_tip_certificate() -> ICRC3DataCertificate` - Gets the tip certificate
/// * `icrc3_supported_block_types() -> Vec<SupportedBlockType>` - Gets supported block types
/// * `upgrade_archive_wasm(wasm_module: Vec<u8>)` - Upgrades the archive canister WASM
///
/// # Example
/// ```
/// use icrc3_library::icrc3_macros::icrc3_state;
///
/// icrc3_state!();
///
/// fn add_transaction(transaction: MyTransaction) -> Result<u64, Icrc3Error> {
///     add_transaction(transaction)
/// }
///
/// fn get_archives() -> Vec<ICRC3ArchiveInfo> {
///     icrc3_get_archives()
/// }
/// ```
///
///
// icrc3_macros/src/lib.rs
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn icrc3_state(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        use lazy_static::lazy_static;
        use std::sync::{Arc, RwLock};
        use icrc_ledger_types::icrc3::blocks::{GetBlocksResult, GetBlocksRequest, ICRC3DataCertificate, SupportedBlockType};
        use icrc_ledger_types::icrc3::archive::ICRC3ArchiveInfo;
        use bity_ic_icrc3::{config::{ICRC3Config, ICRC3Properties}, icrc3::ICRC3, interface::ICRC3Interface, types::Icrc3Error};
        use bity_ic_canister_time::{run_interval, MINUTE_IN_MS, HOUR_IN_MS};
        use std::time::Duration;

        lazy_static! {
            pub static ref ICRC3_INSTANCE: Arc<RwLock<Option<ICRC3>>> = Arc::new(RwLock::new(None));
        }

        const __ICRC3_NOT_INITIALIZED: &str = "ICRC3 state has not been initialized";

        pub fn init_icrc3(config: ICRC3Config) {
            let mut lock = ICRC3_INSTANCE.write().unwrap();
            *lock = Some(ICRC3::new(config));
        }

        pub fn is_initialized() -> bool {
            let lock = ICRC3_INSTANCE.read().unwrap();
            lock.is_some()
        }

        pub fn take_icrc3() -> Option<ICRC3> {
            let mut lock = ICRC3_INSTANCE.write().unwrap();
            lock.take()
        }

        pub fn replace_icrc3(icrc3: ICRC3) {
            let mut lock = ICRC3_INSTANCE.write().unwrap();
            *lock = Some(icrc3);
        }

        pub fn icrc3_add_transaction<T: TransactionType>(
            transaction: T,
        ) -> Result<u64, Icrc3Error> {
            let mut lock = ICRC3_INSTANCE.write().unwrap();
            let icrc3 = lock.as_mut().expect(__ICRC3_NOT_INITIALIZED);
            <ICRC3 as ICRC3Interface>::add_transaction(icrc3, transaction)
        }

        pub fn icrc3_prepare_transaction<T: TransactionType>(
            transaction: T,
        ) -> Result<bity_ic_icrc3::types::prepare_transaction::PreparedTransaction, Icrc3Error> {
            let mut lock = ICRC3_INSTANCE.write().unwrap();
            let icrc3 = lock.as_mut().expect(__ICRC3_NOT_INITIALIZED);
            <ICRC3 as ICRC3Interface>::prepare_transaction(icrc3, transaction)
        }

        pub fn icrc3_commit_prepared_transaction<T: TransactionType>(
            transaction: T,
            timestamp: u128,
        ) -> Result<u64, Icrc3Error> {
            let mut lock = ICRC3_INSTANCE.write().unwrap();
            let icrc3 = lock.as_mut().expect(__ICRC3_NOT_INITIALIZED);
            <ICRC3 as ICRC3Interface>::commit_prepared_transaction(icrc3, transaction, timestamp)
        }

        pub fn icrc3_get_archives() -> Vec<ICRC3ArchiveInfo> {
            let lock = ICRC3_INSTANCE.read().unwrap();
            let icrc3 = lock.as_ref().expect(__ICRC3_NOT_INITIALIZED);
            <ICRC3 as ICRC3Interface>::icrc3_get_archives(icrc3)
        }

        pub fn icrc3_get_blocks(
            args: Vec<GetBlocksRequest>,
        ) -> GetBlocksResult {
            let lock = ICRC3_INSTANCE.read().unwrap();
            let icrc3 = lock.as_ref().expect(__ICRC3_NOT_INITIALIZED);
            <ICRC3 as ICRC3Interface>::icrc3_get_blocks(icrc3, args)
        }

        pub fn icrc3_get_properties() -> ICRC3Properties {
            let lock = ICRC3_INSTANCE.read().unwrap();
            let icrc3 = lock.as_ref().expect(__ICRC3_NOT_INITIALIZED);
            <ICRC3 as ICRC3Interface>::icrc3_get_properties(icrc3)
        }

        pub fn icrc3_get_tip_certificate() -> ICRC3DataCertificate {
            let lock = ICRC3_INSTANCE.read().unwrap();
            let icrc3 = lock.as_ref().expect(__ICRC3_NOT_INITIALIZED);
            <ICRC3 as ICRC3Interface>::icrc3_get_tip_certificate(icrc3)
        }

        pub fn icrc3_supported_block_types() -> Vec<SupportedBlockType> {
            let lock = ICRC3_INSTANCE.read().unwrap();
            let icrc3 = lock.as_ref().expect(__ICRC3_NOT_INITIALIZED);
            <ICRC3 as ICRC3Interface>::icrc3_supported_block_types(icrc3)
        }

        pub fn start_archive_job(interval_ms: u64) {
            run_interval(Duration::from_millis(interval_ms), || {
                ic_cdk::futures::spawn(async {
                    match ICRC3_INSTANCE.write() {
                        Ok(mut lock) => {
                            if let Some(icrc3) = lock.as_mut() {
                                if let Err(e) = icrc3.archive_job().await {
                                    bity_ic_icrc3::utils::trace(format!("Archive job failed: {}", e));
                                } else {
                                    bity_ic_icrc3::utils::trace(format!("Archive job completed successfully"));
                                }
                            } else {
                                bity_ic_icrc3::utils::trace("ICRC3 instance not initialized");
                            }
                        },
                        Err(e) => {
                            bity_ic_icrc3::utils::trace(format!("Failed to acquire ICRC3 lock: {}", e));
                        }
                    }
                });
            });
        }

        pub fn start_cleanup_job(interval_ms: u64) {
            run_interval(Duration::from_millis(interval_ms), || {
                ic_cdk::futures::spawn(async {
                    match ICRC3_INSTANCE.write() {
                        Ok(mut lock) => {
                            if let Some(icrc3) = lock.as_mut() {
                                if let Err(e) = icrc3.cleanup_job() {
                                    bity_ic_icrc3::utils::trace(format!("Cleanup job failed: {}", e));
                                } else {
                                    bity_ic_icrc3::utils::trace(format!("Cleanup job completed successfully"));
                                }
                            } else {
                                bity_ic_icrc3::utils::trace("ICRC3 instance not initialized");
                            }
                        },
                        Err(e) => {
                            bity_ic_icrc3::utils::trace(format!("Failed to acquire ICRC3 lock: {}", e));
                        }
                    }
                });
            });
        }

        // by default you can use this method, to run archive 10mins
        pub fn start_default_archive_job() {
            start_archive_job(10 * MINUTE_IN_MS);
            start_cleanup_job(1 * HOUR_IN_MS);
        }
    };

    expanded.into()
}
