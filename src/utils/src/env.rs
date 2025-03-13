//! Environment-related utilities for Internet Computer canisters.

use bity_ic_canister_time::now_nanos;
use bity_ic_types::BuildVersion;
use bity_ic_types::{CanisterId, Cycles, TimestampMillis, TimestampNanos};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Represents the environment configuration of a canister.
#[derive(Default, CandidType, Serialize, Deserialize, Clone)]
pub struct CanisterEnv {
    /// Whether the canister is running in test mode
    test_mode: bool,
    /// The build version of the canister
    version: BuildVersion,
    /// The Git commit hash of the canister's code
    commit_hash: String,
}

/// Trait for accessing canister environment information.
pub trait Environment {
    /// Returns the current time in nanoseconds
    fn now_nanos(&self) -> TimestampNanos;
    /// Returns the caller's principal
    fn caller(&self) -> Principal;
    /// Returns the canister's own ID
    fn canister_id(&self) -> CanisterId;
    /// Returns the current cycle balance
    fn cycles_balance(&self) -> Cycles;

    fn now(&self) -> TimestampMillis {
        self.now_nanos() / 1_000_000
    }

    fn cycles_balance_in_tc(&self) -> f64 {
        (self.cycles_balance() as f64) / 1_000_000_000_000.0
    }
}

impl CanisterEnv {
    pub fn new(test_mode: bool, version: BuildVersion, commit_hash: String) -> Self {
        Self {
            test_mode,
            version,
            commit_hash,
        }
    }

    pub fn is_test_mode(&self) -> bool {
        self.test_mode
    }

    pub fn version(&self) -> BuildVersion {
        self.version
    }

    pub fn set_version(&mut self, version: BuildVersion) {
        self.version = version;
    }

    pub fn commit_hash(&self) -> &str {
        &self.commit_hash
    }

    pub fn set_commit_hash(&mut self, commit_hash: String) {
        self.commit_hash = commit_hash;
    }
}

impl Environment for CanisterEnv {
    fn now_nanos(&self) -> TimestampNanos {
        now_nanos()
    }

    #[cfg(target_arch = "wasm32")]
    fn caller(&self) -> Principal {
        ic_cdk::caller()
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn caller(&self) -> Principal {
        Principal::anonymous()
    }

    #[cfg(target_arch = "wasm32")]
    fn canister_id(&self) -> CanisterId {
        ic_cdk::id()
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn canister_id(&self) -> CanisterId {
        Principal::anonymous()
    }

    #[cfg(target_arch = "wasm32")]
    fn cycles_balance(&self) -> Cycles {
        ic_cdk::api::canister_balance().into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn cycles_balance(&self) -> Cycles {
        0
    }
}
