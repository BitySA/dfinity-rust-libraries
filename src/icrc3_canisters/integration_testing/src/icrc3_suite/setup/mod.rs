use candid::Nat;

use self::setup::{TestEnv, TestEnvBuilder};

pub mod setup;
pub mod setup_icrc3;

pub fn default_test_setup() -> TestEnv {
    TestEnvBuilder::new().build()
}
