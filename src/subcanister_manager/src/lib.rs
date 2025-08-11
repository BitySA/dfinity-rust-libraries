//! A library for managing sub-canisters on the Internet Computer.
//!
//! This library provides functionality to create, manage, and update sub-canisters
//! on the Internet Computer. It handles the lifecycle of canisters including
//! creation, installation, updates, and state management.
//!
//! # Features
//!
//! - Create and manage sub-canisters
//! - Handle canister lifecycle (create, install, update, stop)
//! - Manage canister controllers and permissions
//! - Handle cycles allocation and management
//!
//! # Example
//!
//! ```rust
//! use bity_ic_subcanister_manager::{SubCanisterManager, CanisterState};
//!
//! // Create a new sub-canister manager
//! let manager = SubCanisterManager::new(
//!     master_canister_id,
//!     HashMap::new(),
//!     vec![],
//!     vec![],
//!     1_000_000_000, // initial cycles
//!     100_000_000,   // reserved cycles
//!     false,         // test mode
//!     "commit_hash".to_string(),
//!     wasm_module,
//! );
//! ```
//!
//! # License
//!
//! This project is licensed under the MIT License.

use bity_ic_utils::retry_async::retry_async;
use candid::{CandidType, Encode, Nat, Principal};
use canfund::{
    manager::{options::FundManagerOptions, RegisterOpts},
    operations::fetch::FetchCyclesBalanceFromCanisterStatus,
    FundManager,
};
use ic_cdk::management_canister::create_canister_with_extra_cycles;
use ic_cdk::management_canister::{
    canister_status, install_code, start_canister, stop_canister, CanisterInstallMode,
    CanisterSettings, CreateCanisterArgs, InstallCodeArgs, LogVisibility,
};
use ic_management_canister_types::CanisterIdRecord;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{any::Any, collections::HashMap, fmt::Debug, future::Future};

/// Error types for storage operations
#[derive(Debug)]
pub enum NewStorageError {
    /// Error when creating a new canister
    CreateCanisterError(String),
    /// Error when installing code on a canister
    InstallCodeError(String),
    /// Error when serializing initialization arguments
    FailedToSerializeInitArgs(String),
}

/// Error types for canister operations
#[derive(Debug)]
pub enum NewCanisterError {
    /// Error when creating a new canister
    CreateCanisterError(String),
    /// Error when installing code on a canister
    InstallCodeError(String),
    /// Error when serializing initialization arguments
    FailedToSerializeInitArgs(String),
}

/// Error types for canister operations
#[derive(Serialize, Deserialize, Clone)]
pub enum CanisterError {
    /// Error when controllers cannot be found
    CantFindControllers(String),
}

/// Represents the current state of a canister
#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum CanisterState {
    /// Canister has been created but not yet installed
    Created,
    /// Canister is installed and running
    Installed,
    /// Canister has been stopped
    Stopped,
}

/// Trait that must be implemented by canister types
pub trait Canister {
    /// Type of parameters used for canister initialization
    type ParamType: CandidType + Serialize + Clone + Send;

    /// Creates a new canister instance
    fn new(canister_id: Principal, state: CanisterState, canister_param: Self::ParamType) -> Self;

    /// Returns the canister's parameters
    fn canister_param(&self) -> Self::ParamType;

    /// Returns the canister's ID
    fn canister_id(&self) -> Principal;

    /// Returns the current state of the canister
    fn state(&self) -> CanisterState;

    /// Returns the canister as an Any type for type erasure
    fn as_any(&self) -> &dyn Any;

    /// Retrieves the controllers of the canister
    fn get_canister_controllers(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Principal>, CanisterError>> + Send
    where
        Self: Sync + Send,
    {
        async {
            match retry_async(
                async || {
                    canister_status(&CanisterIdRecord {
                        canister_id: self.canister_id(),
                    })
                    .await
                },
                3,
            )
            .await
            {
                Ok(res) => Ok(res.settings.controllers),
                Err(e) => Err(CanisterError::CantFindControllers(format!("{e:?}"))),
            }
        }
    }
}

/// Manager for handling sub-canisters
#[derive(Serialize, Deserialize)]
pub struct SubCanisterManager<T>
where
    T: Canister + Clone + Send,
{
    /// ID of the master canister
    pub master_canister_id: Principal,
    /// Map of sub-canisters
    pub sub_canisters: HashMap<Principal, Box<T>>,
    /// List of controllers
    pub controllers: Vec<Principal>,
    /// List of authorized principals
    pub authorized_principal: Vec<Principal>,
    /// Initial cycles for new canisters
    pub initial_cycles: u128,
    /// Reserved cycles for canisters
    pub reserved_cycles: u128,
    /// Whether the manager is in test mode
    pub test_mode: bool,
    /// Commit hash of the current version
    pub commit_hash: String,
    /// WASM module for canister installation
    pub wasm: Vec<u8>,
    /// Fund manager
    #[serde(skip)]
    pub fund_manager: FundManager,
    /// Funding config
    #[serde(skip)]
    pub funding_config: FundManagerOptions,
}

impl<T> SubCanisterManager<T>
where
    T: Canister + Clone + Send,
{
    pub fn new(
        master_canister_id: Principal,
        sub_canisters: HashMap<Principal, Box<T>>,
        mut controllers: Vec<Principal>,
        mut authorized_principal: Vec<Principal>,
        initial_cycles: u128,
        reserved_cycles: u128,
        test_mode: bool,
        commit_hash: String,
        wasm: Vec<u8>,
        funding_config: FundManagerOptions,
    ) -> Self {
        controllers.push(master_canister_id);
        authorized_principal.push(master_canister_id);

        Self {
            master_canister_id,
            sub_canisters,
            controllers,
            authorized_principal,
            initial_cycles,
            reserved_cycles,
            test_mode,
            commit_hash,
            wasm,
            fund_manager: FundManager::new(),
            funding_config: funding_config,
        }
    }

    pub async fn create_canister(
        &mut self,
        init_args: <T as Canister>::ParamType,
    ) -> Result<Box<T>, NewCanisterError> {
        async move {
            let mut canister_id = Principal::anonymous();

            for (_canister_id, canister) in self.sub_canisters.iter() {
                if canister.state() == CanisterState::Created {
                    canister_id = *_canister_id;
                    break;
                }
            }

            if canister_id == Principal::anonymous() {
                let settings = CanisterSettings {
                    controllers: Some(self.controllers.clone()),
                    compute_allocation: None,
                    memory_allocation: None,
                    freezing_threshold: None,
                    reserved_cycles_limit: Some(Nat::from(self.reserved_cycles)),
                    log_visibility: Some(LogVisibility::Public),
                    wasm_memory_limit: None,
                    wasm_memory_threshold: None,
                };

                canister_id = match retry_async(
                    async || {
                        create_canister_with_extra_cycles(
                            &CreateCanisterArgs {
                                settings: Some(settings.clone()),
                            },
                            self.initial_cycles,
                        )
                        .await
                    },
                    3,
                )
                .await
                {
                    Ok(canister) => canister.canister_id,
                    Err(e) => {
                        return Err(NewCanisterError::CreateCanisterError(format!("{e:?}")));
                    }
                };

                add_canisters_to_fund_manager(
                    &mut self.fund_manager,
                    self.funding_config.clone(),
                    vec![canister_id],
                );

                self.sub_canisters.insert(
                    canister_id,
                    Box::new(T::new(
                        canister_id,
                        CanisterState::Created,
                        init_args.clone(),
                    )),
                );
            }

            let encoded_init_args = match Encode!(&init_args) {
                Ok(encoded_init_args) => encoded_init_args,
                Err(e) => {
                    return Err(NewCanisterError::FailedToSerializeInitArgs(format!("{e}")));
                }
            };

            let install_args = InstallCodeArgs {
                mode: CanisterInstallMode::Install,
                canister_id,
                wasm_module: self.wasm.clone(),
                arg: encoded_init_args.clone(),
            };

            match install_code(&install_args).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(NewCanisterError::InstallCodeError(format!("{:?}", e)));
                }
            }

            let canister = Box::new(T::new(
                canister_id,
                CanisterState::Installed,
                init_args.clone(),
            ));

            self.sub_canisters.insert(canister_id, canister.clone());

            Ok(canister)
        }
    }

    pub async fn update_canisters(
        &mut self,
        update_args: <T as Canister>::ParamType,
    ) -> Result<(), Vec<String>> {
        async move {
            let init_args = match Encode!(&update_args.clone()) {
                Ok(encoded_init_args) => encoded_init_args,
                Err(e) => {
                    return Err(vec![format!(
                        "ERROR : failed to create init args with error - {e}"
                    )]);
                }
            };

            let mut canister_upgrade_errors = vec![];

            for (canister_id, _canister) in self.sub_canisters.clone().iter() {
                match retry_async(
                    async || {
                        stop_canister(&CanisterIdRecord {
                            canister_id: *canister_id,
                        })
                        .await
                    },
                    3,
                )
                .await
                {
                    Ok(_) => {
                        self.sub_canisters.insert(
                            *canister_id,
                            Box::new(T::new(
                                *canister_id,
                                CanisterState::Stopped,
                                update_args.clone(),
                            )),
                        );
                    }
                    Err(e) => {
                        canister_upgrade_errors.push(format!(
                            "ERROR: storage upgrade :: storage with principal : {} failed to stop with error {:?}",
                            *canister_id, e
                        ));
                        continue;
                    }
                }

                let result = {
                    let init_args = init_args.clone();
                    let wasm_module = self.wasm.clone();

                    let install_args = InstallCodeArgs {
                        mode: CanisterInstallMode::Upgrade(None),
                        canister_id: *canister_id,
                        wasm_module,
                        arg: init_args,
                    };
                    retry_async(|| install_code(&install_args), 3).await
                };

                match result {
                    Ok(_) => {
                        match retry_async(
                            async || {
                                start_canister(&CanisterIdRecord {
                                    canister_id: *canister_id,
                                })
                                .await
                            },
                            3,
                        )
                        .await
                        {
                            Ok(_) => {
                                self.sub_canisters.insert(
                                    *canister_id,
                                    Box::new(T::new(
                                        *canister_id,
                                        CanisterState::Installed,
                                        update_args.clone(),
                                    )),
                                );
                            }
                            Err(e) => {
                                canister_upgrade_errors.push(format!(
                                    "ERROR: storage upgrade :: storage with principal : {} failed to start with error {:?}",
                                    *canister_id, e
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        canister_upgrade_errors.push(format!(
                            "ERROR: storage upgrade :: storage with principal : {} failed to install upgrade {:?}",
                            *canister_id, e
                        ));
                    }
                }
            }

            if !canister_upgrade_errors.is_empty() {
                Err(canister_upgrade_errors)
            } else {
                Ok(())
            }
        }
    }

    pub fn list_canisters(&self) -> Vec<Box<impl Canister>> {
        self.sub_canisters.values().cloned().collect()
    }

    pub fn list_canisters_ids(&self) -> Vec<Principal> {
        self.sub_canisters.clone().into_keys().collect()
    }
}

impl<T> Clone for SubCanisterManager<T>
where
    T: Canister + Clone + Send,
{
    fn clone(&self) -> Self {
        let mut fund_manager = FundManager::new();

        add_canisters_to_fund_manager(
            &mut fund_manager,
            self.funding_config.clone(),
            self.sub_canisters.clone().into_keys().collect(),
        );

        Self {
            master_canister_id: self.master_canister_id,
            sub_canisters: self.sub_canisters.clone(),
            controllers: self.controllers.clone(),
            authorized_principal: self.authorized_principal.clone(),
            initial_cycles: self.initial_cycles,
            reserved_cycles: self.reserved_cycles,
            test_mode: self.test_mode,
            commit_hash: self.commit_hash.clone(),
            wasm: self.wasm.clone(),
            fund_manager: fund_manager,
            funding_config: self.funding_config.clone(),
        }
    }
}

pub fn add_canisters_to_fund_manager(
    fund_manager: &mut FundManager,
    funding_config: FundManagerOptions,
    canister_id_lst: Vec<Principal>,
) {
    fund_manager.stop();

    fund_manager.with_options(funding_config);

    for canister_id in canister_id_lst {
        fund_manager.register(
            canister_id,
            RegisterOpts::new()
                .with_cycles_fetcher(Arc::new(FetchCyclesBalanceFromCanisterStatus::new())),
        );
    }

    fund_manager.start();
}
