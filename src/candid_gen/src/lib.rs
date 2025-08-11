//! Procedural macros for generating Candid method implementations.
//!
//! This module provides procedural macros to generate boilerplate code for Candid
//! method implementations in Internet Computer canisters. It supports both query
//! and update methods, with and without arguments.
//!
//! # Features
//! - Generation of Candid method implementations
//! - Support for query and update methods
//! - Support for methods with and without arguments
//! - Automatic type generation for Args and Response
//!
//! # Examples
//! ```
//! use bity_ic_candid_gen::*;
//!
//! // Generate a method with arguments
//! generate_candid_method!(my_canister, transfer, update);
//!
//! // Generate a method without arguments
//! generate_candid_method_no_args!(my_canister, get_balance, query);
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Ident, Token};

/// Represents the attributes needed to generate a Candid method.
///
/// This struct contains the information required to generate a method implementation,
/// including the canister name, method name, and method type (query/update).
struct MethodAttribute {
    /// The name of the canister (without the "_canister" suffix)
    canister_name: String,
    /// The name of the method to generate
    method_name: String,
    /// The type of method ("query" or "update")
    method_type: String,
}

/// Generates a Candid method implementation with arguments.
///
/// This procedural macro generates a method implementation with the specified
/// arguments and response types. The generated method will be marked with the
/// appropriate Candid method attribute.
///
/// # Arguments
/// The macro takes three comma-separated identifiers:
/// * `canister_name` - The name of the canister (without "_canister" suffix)
/// * `method_name` - The name of the method to generate
/// * `method_type` - The type of method ("query" or "update")
///
/// # Returns
/// A TokenStream containing the generated method implementation.
///
/// # Example
/// ```
/// use bity_ic_candid_gen::generate_candid_method;
///
/// generate_candid_method!(my_canister, transfer, update);
/// ```
#[proc_macro]
pub fn generate_candid_method(input: TokenStream) -> TokenStream {
    let inputs = parse_macro_input!(input with Punctuated::<Ident, Token![,]>::parse_terminated)
        .into_iter()
        .map(|i| i.to_string())
        .collect();

    let attribute = get_method_attribute(inputs);

    let canister_name = format_ident!("{}", attribute.canister_name);
    let method_name = format_ident!("{}", attribute.method_name);
    let method_type = format_ident!("{}", attribute.method_type);

    let args_name = quote! { #canister_name::#method_name::Args };
    let response_name = quote! { #canister_name::#method_name::Response };

    let tokens = quote! {
        #[candid::candid_method(#method_type)]
        fn #method_name(_: #args_name) -> #response_name {
            unimplemented!();
        }
    };

    TokenStream::from(tokens)
}

/// Generates a Candid method implementation without arguments.
///
/// This procedural macro generates a method implementation without arguments,
/// only returning a response type. The generated method will be marked with the
/// appropriate Candid method attribute.
///
/// # Arguments
/// The macro takes three comma-separated identifiers:
/// * `canister_name` - The name of the canister (without "_canister" suffix)
/// * `method_name` - The name of the method to generate
/// * `method_type` - The type of method ("query" or "update")
///
/// # Returns
/// A TokenStream containing the generated method implementation.
///
/// # Example
/// ```
/// use bity_ic_candid_gen::generate_candid_method_no_args;
///
/// generate_candid_method_no_args!(my_canister, get_balance, query);
/// ```
#[proc_macro]
pub fn generate_candid_method_no_args(input: TokenStream) -> TokenStream {
    let inputs = parse_macro_input!(input with Punctuated::<Ident, Token![,]>::parse_terminated)
        .into_iter()
        .map(|i| i.to_string())
        .collect();

    let attribute = get_method_attribute(inputs);

    let canister_name = format_ident!("{}", attribute.canister_name);
    let method_name = format_ident!("{}", attribute.method_name);
    let method_type = format_ident!("{}", attribute.method_type);

    let response_name = quote! { #canister_name::#method_name::Response };

    let tokens = quote! {
        #[candid::candid_method(#method_type)]
        fn #method_name() -> #response_name {
            unimplemented!();
        }
    };

    TokenStream::from(tokens)
}

/// Extracts method attributes from the input tokens.
///
/// This function processes the input tokens to create a `MethodAttribute` struct
/// containing the canister name, method name, and method type.
///
/// # Arguments
/// * `inputs` - A vector of strings containing the input tokens
///
/// # Returns
/// A `MethodAttribute` struct with the processed information.
///
/// # Panics
/// Panics if:
/// - The input vector has fewer than 3 elements
/// - The method type is not "query" or "update"
fn get_method_attribute(inputs: Vec<String>) -> MethodAttribute {
    let first_arg = inputs.get(0).unwrap();
    let second_arg = inputs.get(1).unwrap();
    let third_arg = inputs.get(2).unwrap();

    let canister_name = format!("{first_arg}_canister");

    let method_name = second_arg.to_string();

    let method_type = match third_arg.as_str() {
        "query" | "update" => third_arg.to_string(),
        _ => panic!("Unrecognised 'method_type' value: {third_arg}"),
    };

    MethodAttribute {
        canister_name,
        method_name,
        method_type,
    }
}
