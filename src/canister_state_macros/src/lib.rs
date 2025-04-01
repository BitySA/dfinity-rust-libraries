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
            static __STATE: std::cell::RefCell<Option<std::rc::Rc<$type>>> = std::cell::RefCell::default();
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
                    *s = Some(std::rc::Rc::new(state));
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
            __STATE.replace(Some(std::rc::Rc::new(state))).expect(__STATE_NOT_INITIALIZED).as_ref().clone()
        }

        /// Takes ownership of the current state.
        ///
        /// # Returns
        /// The current state value
        ///
        /// # Panics
        /// Panics if the state has not been initialized
        pub fn take_state() -> $type {
            __STATE.take().expect(__STATE_NOT_INITIALIZED).as_ref().clone()
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
            __STATE.with_borrow(|s| f(s.as_ref().expect(__STATE_NOT_INITIALIZED).as_ref()))
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
            __STATE.with_borrow_mut(|s| {
                let state = std::rc::Rc::make_mut(s.as_mut().expect(__STATE_NOT_INITIALIZED));
                f(state)
            })
        }

        /// Trait for async functions that can be used with state operations
        /// This is temporary trait to be replaced once rust release official AsyncFn trait definition.
        /// see https://github.com/rust-lang/rust/pull/132706
        pub trait AsyncFn<Args>: FnOnce(Args) {
            type Future: std::future::Future<Output = Self::Output> + Send + 'static;
            type Output;
        }

        impl<F, Args, Fut, Out> AsyncFn<Args> for F
        where
            F: FnOnce(Args, Output = Fut),
            Fut: std::future::Future<Output = Out> + Send + 'static,
        {
            type Future = Fut;
            type Output = Out;
        }

        /// Reads the state using an async closure.
        ///
        /// # Arguments
        /// * `f` - An async closure that takes a reference to the state and returns a value
        ///
        /// # Returns
        /// A Future that will resolve to the result of the closure
        ///
        /// # Panics
        /// Panics if the state has not been initialized
        pub fn read_state_async<F>(f: F) -> F::Future
        where
            F: AsyncFn<(&$type,)>,
            F::Future: Send + 'static,
        {
            __STATE.with_borrow(|s| {
                let state = s.as_ref().expect(__STATE_NOT_INITIALIZED).as_ref();
                f(state)
            })
        }

        /// Mutates the state using an async closure.
        ///
        /// # Arguments
        /// * `f` - An async closure that takes a mutable reference to the state and returns a value
        ///
        /// # Returns
        /// A Future that will resolve to the result of the closure
        ///
        /// # Panics
        /// Panics if the state has not been initialized
        pub fn mutate_state_async<F>(f: F) -> F::Future
        where
            F: AsyncFn<(&mut $type,)>,
            F::Future: Send + 'static,
        {
            __STATE.with_borrow_mut(|s| {
                let state = std::rc::Rc::make_mut(s.as_mut().expect(__STATE_NOT_INITIALIZED));
                f(state)
            })
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
