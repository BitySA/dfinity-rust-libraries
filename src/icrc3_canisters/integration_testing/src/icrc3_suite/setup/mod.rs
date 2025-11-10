use self::setup::{TestEnv, TestEnvBuilder};
use bity_ic_icrc3::config::{ICRC3Config, ICRC3Properties};

pub mod setup;
pub mod setup_icrc3;

pub fn default_test_setup() -> TestEnv {
    TestEnvBuilder::new().build()
}

pub fn default_test_setup_with_archive() -> TestEnv {
    let mut test_env = TestEnvBuilder::new();

    let mut icrc3_constants = ICRC3Properties::default();
    icrc3_constants.max_tx_local_stable_memory_size_bytes = Some(5000);
    icrc3_constants.threshold_for_archiving_to_external_archive = Some(10);
    icrc3_constants.max_transactions_in_window = 10_u64.into();

    test_env.icrc3_constants = icrc3_constants;

    test_env.build()
}
