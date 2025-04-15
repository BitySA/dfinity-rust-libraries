use crate::icrc3_suite::setup::setup_icrc3::setup_icrc3_canister;
use crate::utils::random_principal;
use bity_ic_icrc3::config::{ICRC3Config, ICRC3Properties};
use bity_ic_types::{BuildVersion, CanisterId};
use candid::Principal;
use icrc3_example_api::Args;
use pocket_ic::{PocketIc, PocketIcBuilder};
use std::time::Duration;

pub struct TestEnv {
    pub controller: Principal,
    pub icrc3_id: CanisterId,
    pub pic: PocketIc,
}

use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
impl Debug for TestEnv {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestEnv")
            .field("icrc3_id", &self.icrc3_id.to_text())
            .finish()
    }
}
pub struct TestEnvBuilder {
    controller: Principal,
    icrc3_id: CanisterId,
}

impl Default for TestEnvBuilder {
    fn default() -> Self {
        Self {
            controller: random_principal(),
            icrc3_id: Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        }
    }
}

impl TestEnvBuilder {
    pub fn new() -> Self {
        TestEnvBuilder::default()
    }

    pub fn build(&mut self) -> TestEnv {
        let mut pic = PocketIcBuilder::new().with_application_subnet().build();

        self.icrc3_id = pic.create_canister_with_settings(Some(self.controller.clone()), None);

        let mut constants = ICRC3Properties::default();

        // constants.max_memory_size_bytes = 1000;
        constants.max_memory_size_bytes = 60000;
        constants.tx_window = Duration::from_millis(500);
        constants.max_transactions_in_window = 10;
        constants.max_transactions_to_purge = 5;

        let icrc3_init_args = Args::Init(icrc3_example_api::init::InitArgs {
            test_mode: true,
            version: BuildVersion::min(),
            commit_hash: "".to_string(),
            authorized_principals: vec![self.controller],
            icrc3_config: ICRC3Config {
                supported_blocks: vec![],
                constants,
            },
        });

        let icrc3_canister_id =
            setup_icrc3_canister(&mut pic, self.icrc3_id, icrc3_init_args, self.controller);

        TestEnv {
            controller: self.controller,
            icrc3_id: icrc3_canister_id,
            pic,
        }
    }
}
