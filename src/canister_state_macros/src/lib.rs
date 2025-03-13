//! Module for managing canister state in Internet Computer canisters.
//!
//! This module provides a macro for creating thread-safe state management in canisters,
//! with functions for initialization, reading, and modifying the state.
//!
//! # Example
//! ```
//! use bity_dfinity_library::canister_state_macros::canister_state;
//!
//! struct MyState {
//!     counter: u64,
//! }
//!
//! canister_state!(MyState);
//!
//! fn init() {
//!     init_state(MyState { counter: 0 });
//! }
//!
//! fn increment() {
//!     mutate_state(|state| state.counter += 1);
//! }
//! ```

/// A macro that generates thread-safe state management functions for a canister.
///
/// This macro creates a set of functions for managing the canister's state in a thread-safe manner.
/// It provides functions for initialization, reading, and modifying the state.
///
/// # Arguments
/// * `$type` - The type of the state to manage
///
/// # Generated Functions
/// * `init_state(state: $type)` - Initializes the state (panics if already initialized)
/// * `replace_state(state: $type) -> $type` - Replaces the current state and returns the old one
/// * `take_state() -> $type` - Takes ownership of the current state
/// * `read_state<F, R>(f: F) -> R` - Reads the state using a closure
/// * `mutate_state<F, R>(f: F) -> R` - Mutates the state using a closure
/// * `can_borrow_state() -> bool` - Checks if the state can be borrowed
///
/// # Example
/// ```
/// use bity_dfinity_library::canister_state_macros::canister_state;
///
/// struct AppState {
///     users: Vec<String>,
/// }
///
/// canister_state!(AppState);
///
/// fn add_user(name: String) {
///     mutate_state(|state| state.users.push(name));
/// }
///
/// fn get_user_count() -> usize {
///     read_state(|state| state.users.len())
/// }
/// ```
#[macro_export]
macro_rules! canister_state {
    ($type:ty) => {
        thread_local! {
            static __STATE: std::cell::RefCell<Option<$type>> = std::cell::RefCell::default();
        }

        const __STATE_ALREADY_INITIALIZED: &str = "State has already been initialized";
        const __STATE_NOT_INITIALIZED: &str = "State has not been initialized";

        /// Initializes the canister state.
        ///
        /// # Arguments
        /// * `state` - The initial state value
        ///
        /// # Panics
        /// Panics if the state has already been initialized
        pub fn init_state(state: $type) {
            __STATE.with_borrow_mut(|s| {
                if s.is_some() {
                    panic!("{}", __STATE_ALREADY_INITIALIZED);
                } else {
                    *s = Some(state);
                }
            });
        }

        /// Replaces the current state with a new one.
        ///
        /// # Arguments
        /// * `state` - The new state value
        ///
        /// # Returns
        /// The previous state value
        ///
        /// # Panics
        /// Panics if the state has not been initialized
        pub fn replace_state(state: $type) -> $type {
            __STATE.replace(Some(state)).expect(__STATE_NOT_INITIALIZED)
        }

        /// Takes ownership of the current state.
        ///
        /// # Returns
        /// The current state value
        ///
        /// # Panics
        /// Panics if the state has not been initialized
        pub fn take_state() -> $type {
            __STATE.take().expect(__STATE_NOT_INITIALIZED)
        }

        /// Reads the state using a closure.
        ///
        /// # Arguments
        /// * `f` - A closure that takes a reference to the state and returns a value
        ///
        /// # Returns
        /// The result of the closure
        ///
        /// # Panics
        /// Panics if the state has not been initialized
        pub fn read_state<F, R>(f: F) -> R
        where
            F: FnOnce(&$type) -> R,
        {
            __STATE.with_borrow(|s| f(s.as_ref().expect(__STATE_NOT_INITIALIZED)))
        }

        /// Mutates the state using a closure.
        ///
        /// # Arguments
        /// * `f` - A closure that takes a mutable reference to the state and returns a value
        ///
        /// # Returns
        /// The result of the closure
        ///
        /// # Panics
        /// Panics if the state has not been initialized
        pub fn mutate_state<F, R>(f: F) -> R
        where
            F: FnOnce(&mut $type) -> R,
        {
            __STATE.with_borrow_mut(|s| f(s.as_mut().expect(__STATE_NOT_INITIALIZED)))
        }

        /// Checks if the state can be borrowed.
        ///
        /// # Returns
        /// `true` if the state can be borrowed, `false` otherwise
        pub fn can_borrow_state() -> bool {
            __STATE.with(|s| s.try_borrow().is_ok())
        }
    };
}
