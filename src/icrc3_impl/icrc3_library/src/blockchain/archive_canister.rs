use crate::utils::trace;
use bity_ic_icrc3_archive_api::types::encoded_blocks::EncodedBlock;
use bity_ic_subcanister_manager::Canister;
use bity_ic_utils::retry_async::retry_async;
use candid::{CandidType, Principal};
use icrc_ledger_types::icrc3::archive::ICRC3ArchiveInfo;
use serde::{Deserialize, Serialize};

/// Represents an archive canister that stores blockchain data.
///
/// This struct manages the state and operations of a single archive canister,
/// including block insertion and space management.
#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct ArchiveCanister {
    /// The current state of the canister
    pub state: bity_ic_subcanister_manager::CanisterState,
    /// The parameters used to initialize or upgrade the canister
    pub canister_param: bity_ic_icrc3_archive_api::Args,
    /// Information about the blocks stored in this archive
    pub archive_info: ICRC3ArchiveInfo,
}

impl ArchiveCanister {
    /// Inserts a batch of blocks into the archive canister.
    ///
    /// # Arguments
    ///
    /// * `blocks` - A vector of encoded blocks to insert
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the blocks were successfully inserted
    /// * `Err(String)` if the insertion failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The canister is not in the installed state
    /// * The insertion operation fails
    pub async fn insert_blocks(&mut self, blocks: Vec<EncodedBlock>) -> Result<(), String> {
        if self.state != bity_ic_subcanister_manager::CanisterState::Installed {
            return Err("Canister is not installed".to_string());
        }

        let res = retry_async(
            || bity_ic_icrc3_archive_c2c_client::insert_blocks(self.canister_id(), &blocks),
            3,
        )
        .await;

        match res {
            Ok(data_response) => match data_response {
                bity_ic_icrc3_archive_api::insert_blocks::Response::Success => {
                    // Update the archive info: increment the `end` by the number of blocks inserted
                    let block_count = blocks.len() as u64;
                    self.archive_info.end += block_count;

                    // Log the updated archive info for debugging
                    ic_cdk::println!(
                        "Updated archive info: start = {}, end = {}",
                        self.archive_info.start,
                        self.archive_info.end
                    );

                    Ok(())
                }
                bity_ic_icrc3_archive_api::insert_blocks::Response::Error(_) => {
                    Err("Failed to insert data".to_string())
                }
            },
            Err(e) => Err(format!("{e:?}")),
        }
    }

    /// Gets the available space in the archive canister.
    ///
    /// # Returns
    ///
    /// * `Ok(u128)` containing the available space in bytes
    /// * `Err(String)` if the operation failed
    pub async fn get_available_space(&self) -> Result<u128, String> {
        let res = retry_async(
            || bity_ic_icrc3_archive_c2c_client::remaining_capacity(self.canister_id(), &()),
            3, // Retry up to 3 times
        )
        .await;

        trace(&format!(
            "Checking canister {:?} remaining capacity: {res:?}",
            self.canister_id()
        ));

        match res {
            Ok(available_space) => Ok(u128::try_from(available_space.0).unwrap()),
            Err(err) => {
                trace(&format!(
                    "Failed to get archive size for canister {:?}: {:?}",
                    self.canister_id(),
                    err
                ));
                Err(format!("Failed to fetch available space: {:?}", err))
            }
        }
    }
}

impl bity_ic_subcanister_manager::Canister for ArchiveCanister {
    type ParamType = bity_ic_icrc3_archive_api::Args;

    /// Creates a new archive canister instance.
    ///
    /// # Arguments
    ///
    /// * `canister_id` - The principal ID of the canister
    /// * `state` - The initial state of the canister
    /// * `canister_param` - The initialization parameters
    ///
    /// # Panics
    ///
    /// Panics if `canister_param` is an upgrade argument instead of an init argument
    fn new(
        canister_id: Principal,
        state: bity_ic_subcanister_manager::CanisterState,
        canister_param: Self::ParamType,
    ) -> Self {
        match &canister_param {
            bity_ic_icrc3_archive_api::Args::Init(init_args) => Self {
                state,
                canister_param: canister_param.clone(),
                archive_info: ICRC3ArchiveInfo {
                    canister_id,
                    start: init_args.archive_config.block_offset.into(),
                    end: init_args.archive_config.block_offset.into(),
                },
            },
            bity_ic_icrc3_archive_api::Args::Upgrade(_) => {
                panic!(
                    "Cannot initialize the canister with an Upgrade argument. Please provide an Init argument."
                );
            }
        }
    }

    /// Returns the canister's principal ID.
    fn canister_id(&self) -> Principal {
        self.archive_info.canister_id
    }

    /// Returns the current state of the canister.
    fn state(&self) -> bity_ic_subcanister_manager::CanisterState {
        self.state.clone()
    }

    /// Returns the canister's parameters.
    fn canister_param(&self) -> Self::ParamType {
        self.canister_param.clone()
    }

    /// Returns a reference to the canister as an Any type.
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
