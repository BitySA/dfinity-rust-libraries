//! Module for procedural macros that add tracing capabilities to canister functions.
//!
//! This module provides macros that automatically add tracing instrumentation to functions,
//! making it easier to debug and monitor canister behavior. It wraps functions with tracing
//! capabilities while preserving their original functionality.
//!
//! # Example
//! ```
//! use bity_ic_canister_tracing_macros::trace;
//!
//! #[trace]
//! async fn my_function(arg1: u64, arg2: String) -> Result<(), String> {
//!     // Function implementation
//!     Ok(())
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatIdent, PatType, Signature};

/// A procedural macro attribute that adds tracing capabilities to a function.
///
/// This macro wraps the target function with tracing instrumentation, automatically
/// logging function entry, arguments, and return values at the trace level.
///
/// # Usage
/// Add the `#[trace]` attribute above any function you want to trace:
/// ```rust
/// #[trace]
/// fn my_function(arg: u64) -> u64 {
///     arg * 2
/// }
/// ```
///
/// # Features
/// * Automatically traces function entry and exit
/// * Logs all function arguments
/// * Logs the return value
/// * Works with both synchronous and asynchronous functions
/// * Preserves the original function signature
#[proc_macro_attribute]
pub fn trace(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut inner = parse_macro_input!(item as ItemFn);

    // We will wrap the original fn in a new fn whose signature matches the original fn
    #[allow(clippy::redundant_clone)] // clippy doesn't realise that this is used in the macro
    let wrapper_sig = inner.sig.clone();

    // Change the name of the inner fn so that it doesn't clash with the wrapper fn
    let inner_method_name = format_ident!("{}_inner_", inner.sig.ident);
    inner.sig.ident = inner_method_name.clone();

    let is_async = inner.sig.asyncness.is_some();
    let arg_names = get_arg_names(&inner.sig);

    let function_call = if is_async {
        quote! { #inner_method_name ( #(#arg_names),* ) .await }
    } else {
        quote! { #inner_method_name ( #(#arg_names),* ) }
    };

    let expanded = quote! {
        #[allow(unused_mut)]
        #[tracing::instrument(level = "trace")]
        #wrapper_sig {
            let result = #function_call;
            tracing::trace!(?result);
            result
        }
        #inner
    };

    TokenStream::from(expanded)
}

/// Extracts argument names from a function signature.
///
/// This helper function processes a function signature to extract the names of all arguments,
/// including the receiver (self) if present.
///
/// # Arguments
/// * `signature` - The function signature to process
///
/// # Returns
/// A vector of identifiers representing the argument names
///
/// # Panics
/// Panics if it encounters an argument pattern that it cannot process
fn get_arg_names(signature: &Signature) -> Vec<Ident> {
    signature
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(r) => r.self_token.into(),
            FnArg::Typed(PatType { pat, .. }) => {
                if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                    ident.clone()
                } else {
                    panic!("Unable to determine arg name");
                }
            }
        })
        .collect()
}
