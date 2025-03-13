//! Module for time-related operations in Internet Computer canisters.
//!
//! This module provides utilities for working with time in canisters, including
//! timestamp functions, time constants, and timer scheduling functions. It supports
//! both WASM and non-WASM environments.
//!
//! # Example
//! ```
//! use icrc7_nft::libraries::canister_time::{timestamp_millis, run_interval};
//! use std::time::Duration;
//!
//! let current_time = timestamp_millis();
//! run_interval(Duration::from_secs(60), || {
//!     println!("Running periodic task");
//! });
//! ```

use ic_cdk_timers::TimerId;
use std::time::Duration;

use bity_ic_types::{Milliseconds, TimestampMillis, TimestampNanos};

/// Number of milliseconds in one second
pub const SECOND_IN_MS: Milliseconds = 1000;
/// Number of milliseconds in one minute
pub const MINUTE_IN_MS: Milliseconds = SECOND_IN_MS * 60;
/// Number of milliseconds in one hour
pub const HOUR_IN_MS: Milliseconds = MINUTE_IN_MS * 60;
/// Number of milliseconds in one day
pub const DAY_IN_MS: Milliseconds = HOUR_IN_MS * 24;
/// Number of milliseconds in one week
pub const WEEK_IN_MS: Milliseconds = DAY_IN_MS * 7;

/// Number of nanoseconds in one millisecond
pub const NANOS_PER_MILLISECOND: u64 = 1_000_000;

/// Returns the current timestamp in seconds.
///
/// # Returns
/// The current Unix timestamp in seconds
pub fn timestamp_seconds() -> u64 {
    timestamp_nanos() / 1_000_000_000
}

/// Returns the current timestamp in milliseconds.
///
/// # Returns
/// The current Unix timestamp in milliseconds
pub fn timestamp_millis() -> u64 {
    timestamp_nanos() / 1_000_000
}

/// Returns the current timestamp in microseconds.
///
/// # Returns
/// The current Unix timestamp in microseconds
pub fn timestamp_micros() -> u64 {
    timestamp_nanos() / 1_000
}

/// Returns the current timestamp in nanoseconds (WASM implementation).
///
/// This function is only available when targeting the WASM architecture.
///
/// # Returns
/// The current timestamp in nanoseconds
#[cfg(target_arch = "wasm32")]
pub fn timestamp_nanos() -> u64 {
    unsafe { ic0::time() as u64 }
}

/// Returns the current timestamp in nanoseconds (non-WASM implementation).
///
/// This function is only available when not targeting the WASM architecture.
///
/// # Returns
/// The current Unix timestamp in nanoseconds
#[cfg(not(target_arch = "wasm32"))]
pub fn timestamp_nanos() -> u64 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// Returns the current time in milliseconds.
///
/// # Returns
/// The current time in milliseconds since the Unix epoch
pub fn now_millis() -> TimestampMillis {
    now_nanos() / NANOS_PER_MILLISECOND
}

/// Returns the current time in nanoseconds (WASM implementation).
///
/// This function is only available when targeting the WASM architecture.
///
/// # Returns
/// The current time in nanoseconds since the Unix epoch
#[cfg(target_arch = "wasm32")]
pub fn now_nanos() -> TimestampNanos {
    ic_cdk::api::time()
}

/// Returns the current time in nanoseconds (non-WASM implementation).
///
/// This function is only available when not targeting the WASM architecture.
///
/// # Returns
/// Always returns 0 in non-WASM environments
#[cfg(not(target_arch = "wasm32"))]
pub fn now_nanos() -> TimestampNanos {
    0
}

/// Runs a function immediately and then at the specified interval.
///
/// # Arguments
/// * `interval` - The duration between subsequent executions
/// * `func` - The function to execute
///
/// # Returns
/// The `TimerId` that can be used to cancel the timer
pub fn run_now_then_interval(interval: Duration, func: fn()) -> TimerId {
    ic_cdk_timers::set_timer(Duration::ZERO, func);
    ic_cdk_timers::set_timer_interval(interval, func)
}

/// Runs a function at the specified interval.
///
/// # Arguments
/// * `interval` - The duration between executions
/// * `func` - The function to execute
pub fn run_interval(interval: Duration, func: fn()) {
    ic_cdk_timers::set_timer_interval(interval, func);
}

/// Runs a function once immediately.
///
/// # Arguments
/// * `func` - The function to execute
pub fn run_once(func: fn()) {
    ic_cdk_timers::set_timer(Duration::ZERO, func);
}
