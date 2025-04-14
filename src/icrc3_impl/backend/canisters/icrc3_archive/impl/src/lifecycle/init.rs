use bity_ic_types::BuildVersion;
use bity_ic_utils::env::CanisterEnv;
use bity_ic_utils::env::Environment;
use ic_cdk_macros::init;
use tracing::info;

use crate::state::{Data, RuntimeState};

use super::init_canister;
use crate::utils::trace;
pub use icrc3_archive_api::lifecycle::Args;

#[init]
fn init(args: Args) {
    trace(&format!("archive canister init args: {:?}", args));
    match args {
        Args::Init(init_args) => {
            bity_ic_canister_logger::init(init_args.test_mode);

            let env = CanisterEnv::new(
                init_args.test_mode,
                BuildVersion::min(),
                init_args.commit_hash,
            );
            let mut data = Data::new(
                init_args.archive_config,
                init_args.authorized_principals,
                init_args.master_canister_id,
                init_args.block_type,
            );

            if init_args.test_mode {
                data.authorized_principals.push(env.caller());
            }

            let runtime_state = RuntimeState::new(env, data);

            init_canister(runtime_state);

            info!("Init complete.");
        }
        Args::Upgrade(_) => {
            panic!(
                "Cannot initialize the canister with an Upgrade argument. Please provide an Init argument."
            );
        }
    }
}
