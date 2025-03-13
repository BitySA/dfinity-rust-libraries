//! Memory usage tracking utilities for Internet Computer canisters.

use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents the memory usage of a canister.
#[derive(Serialize, Deserialize, CandidType)]
pub struct MemorySize {
    /// Heap memory usage in bytes
    heap: u64,
    /// Stable memory usage in bytes
    stable: u64,
}

impl MemorySize {
    pub fn used() -> Self {
        Self {
            heap: wasm_memory_size(),
            stable: bity_ic_stable_memory::used(),
        }
    }
}

/// Returns the current WebAssembly memory size in bytes.
pub fn wasm_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        const UPPER_LIMIT_WASM_SIZE_BYTES: u64 = 3 * 1024 * 1024; // 3MB
        UPPER_LIMIT_WASM_SIZE_BYTES + ((core::arch::wasm32::memory_size(0) * 65536) as u64)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // This branch won't actually ever be taken
        1024 * 1024 * 100 // 100Mb
    }
}
