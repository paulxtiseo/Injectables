//! Error handling utilities for compilation errors.
//!
//! This module provides functionality for generating compile-time errors
//! when invalid field injections are detected.

use proc_macro::TokenStream;
use quote::quote;

/// Generates a compile-time error with the given message.
///
/// This function creates a TokenStream that will cause the Rust compiler
/// to emit an error at the macro expansion site.
///
/// # Arguments
///
/// * `msg` - The error message to display
///
/// # Returns
///
/// A `TokenStream` that will generate a compilation error
///
/// # Examples
///
/// ```rust,ignore
/// # use crate::error::compile_error;
/// let error = compile_error("Invalid field injection");
/// ```
pub fn compile_error(msg: &str) -> TokenStream {
  TokenStream::from(quote! {
        compile_error!(#msg);
    })
}

/* Commented implementation for potential future use
/// Generates multiple compile-time errors.
///
/// This function would create a TokenStream that generates multiple
/// compilation errors at once.
///
/// # Arguments
///
/// * `errors` - Vector of error messages
///
/// # Returns
///
/// A `TokenStream` that will generate multiple compilation errors
pub fn compile_errors(errors: Vec<String>) -> TokenStream {
    let error_tokens = errors.iter().map(|msg| {
        quote! {
            compile_error!(#msg);
        }
    });

    TokenStream::from(quote! {
        #(#error_tokens)*
    })
}
*/