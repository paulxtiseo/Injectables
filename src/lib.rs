//! A procedural macro library that enables field injection between Rust structs through declarative attributes.
//!
//! This library provides two key attributes:
//! - [`#[injectable]`](macro@injectable): Marks a struct as a source for field injection
//! - [`#[inject_fields]`](macro@inject_fields): Injects fields from one or more injectable structs
//!
//! # Features
//!
//! - Respects Rust's visibility rules (`pub`, `pub(crate)`, private)
//! - Supports generic types with concrete type resolution
//! - Compile-time dependency injection and validation
//! - Detects circular dependencies and invalid injections
//! - Supports nested/transitive injections
//!
//! # Basic Usage
//!
//! ```rust,ignore
//! use injectables::{injectable, inject_fields};
//!
//! #[injectable]
//! pub struct Base {
//!     pub id: u64,
//! }
//!
//! #[inject_fields(Base)]
//! pub struct Document {
//!     pub title: String,
//! }
//!
//! let doc = Document {
//!     title: "Test".to_string(),
//!     id: 1,  // Field injected from Base
//! };
//! ```
//!
//! # Advanced Features
//!
//! ## Generic Types
//!
//! ```rust,ignore
//! #[injectable]
//! pub struct GenericBase<T> {
//!     pub data: T,
//! }
//!
//! #[inject_fields(GenericBase<String>)]
//! pub struct Container {
//!     pub name: String,
//! }
//! ```
//!
//! ## Nested Injections
//!
//! ```rust,ignore
//! #[injectable]
//! pub struct A {
//!     pub id: u64,
//! }
//!
//! #[injectable]
//! #[inject_fields(A)]
//! pub struct B {
//!     pub name: String,
//! }
//!
//! #[inject_fields(B)]
//! pub struct C {
//!     pub description: String,
//! }
//! ```
//!
//! ## Visibility Rules
//!
//! ```rust,ignore
//! mod inner {
//!     # use super::*;
//!     #[injectable]
//!     pub struct Private {
//!         id: u64,           // private field
//!         pub name: String,  // public field
//!         pub(crate) age: u32, // crate-visible field
//!     }
//! }
//!
//! #[inject_fields(inner::Private)]
//! pub struct Public {
//!     pub value: String,
//!     // Can access `name` and `age`, but not `id`
//! }
//! ```
//!
//! # Limitations
//!
//! 1. Only works with named struct fields (not tuple structs)
//! 2. Cannot inject fields into enums
//! 3. Source structs must be marked with `#[injectable]` before use in `#[inject_fields]`
//! 4. Injected fields maintain their original visibility rules
//! 5. Generic types require concrete type specifications in `#[inject_fields]`

mod registry;
mod types;
mod visibility;
mod error;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

use crate::error::compile_error;
use crate::registry::{check_and_update_injection_chain, validate_and_process_input, update_module_paths};
use crate::types::{InjectConfig, ModuleInfo};
use crate::visibility::visibility_to_kind;

/// Marks a struct as injectable, allowing its fields to be injected into other structs.
///
/// This attribute must be applied to any struct whose fields you want to inject into other structs
/// using [`macro@inject_fields`]. The struct must have named fields (not a tuple struct).
///
/// # Example
///
/// ```rust,ignore
/// #[injectable]
/// pub struct User {
///     pub id: u64,
///     pub name: String,
/// }
/// ```
///
/// # Generic Types
///
/// The attribute supports structs with generic parameters:
///
/// ```rust,ignore
/// #[injectable]
/// pub struct Container<T> {
///     pub data: T,
///     pub timestamp: u64,
/// }
/// ```
///
/// # Visibility
///
/// Fields maintain their original visibility rules when injected:
///
/// ```rust,ignore
/// #[injectable]
/// pub struct Document {
///     id: u64,              // Private - only accessible in same module
///     pub name: String,     // Public - accessible everywhere
///     pub(crate) data: Vec<u8>, // Crate-visible
/// }
/// ```
///
/// # Errors
///
/// This attribute will fail to compile if:
/// - Applied to an enum or union instead of a struct
/// - Applied to a tuple struct (must use named fields)
#[proc_macro_attribute]
pub fn injectable(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let item_clone = item.clone();
  let input = parse_macro_input!(item as DeriveInput);
  let struct_name = input.ident.to_string();

  let generic_params: Vec<String> = input
    .generics
    .params
    .iter()
    .filter_map(|param| {
      if let syn::GenericParam::Type(type_param) = param {
        Some(type_param.ident.to_string())
      } else {
        None
      }
    })
    .collect();

  let fields = match &input.data {
    syn::Data::Struct(data) => match &data.fields {
      syn::Fields::Named(fields) => fields,
      _ => return compile_error("Only named fields are supported"),
    },
    _ => return compile_error("Only structs are supported"),
  };

  let mut registry = registry::FIELD_REGISTRY.lock().unwrap();
  let field_defs = fields
    .named
    .iter()
    .map(|f| {
      let name = f.ident.as_ref().unwrap().to_string();
      let vis = visibility_to_kind(&f.vis);

      types::FieldDef {
        name,
        ty: f.ty.to_token_stream().to_string(),
        vis,
        generic_params: generic_params.clone(),
      }
    })
    .collect();

  let module_info = ModuleInfo {
    fields: field_defs,
    module_path: String::new(), // Will be populated when used in inject_fields
  };

  registry.insert(struct_name, module_info);
  item_clone
}

/// Injects fields from one or more injectable structs into the target struct.
///
/// This attribute copies fields from source structs marked with [`macro@injectable`] into the target struct.
/// Multiple source structs can be specified, separated by commas. For generic source structs,
/// concrete types must be specified.
///
/// # Examples
///
/// Basic field injection:
/// ```rust,ignore
/// #[injectable]
/// pub struct Base {
///     pub id: u64,
/// }
///
/// #[inject_fields(Base)]
/// pub struct Document {
///     pub title: String,  // Original field
///     // id: u64 is injected from Base
/// }
/// ```
///
/// Multiple source structs:
/// ```rust,ignore
/// #[injectable]
/// pub struct Timestamp {
///     pub created_at: String,
/// }
///
/// #[inject_fields(Base, Timestamp)]
/// pub struct Document {
///     pub title: String,
/// }
/// ```
///
/// Generic type injection:
/// ```rust,ignore
/// #[injectable]
/// pub struct Container<T> {
///     pub data: T,
/// }
///
/// #[inject_fields(Container<String>)]
/// pub struct Document {
///     pub title: String,
/// }
/// ```
///
/// # Visibility Rules
///
/// - Private fields cannot be injected across module boundaries
/// - Public and `pub(crate)` fields maintain their visibility when injected
/// - The target struct must have access rights to any injected fields
///
/// # Errors
///
/// This attribute will fail to compile if:
/// - A source struct is not marked as `#[injectable]`
/// - There are circular dependencies between structs 
/// - Field names conflict between multiple sources
/// - Visibility rules are violated
/// - Applied to an enum or tuple struct
/// - Generic type parameters are not fully specified
#[proc_macro_attribute]
pub fn inject_fields(attr: TokenStream, item: TokenStream) -> TokenStream {
  let config = parse_macro_input!(attr as InjectConfig);
  let mut input = parse_macro_input!(item as DeriveInput);
  let target_name = input.ident.to_string();
  let mut errors = Vec::new();

  // Collect all validation errors
  if let Err(err) = validate_and_process_input(&mut input, &config.structs) {
    errors.push(err.0);
  }

  // Check dependencies and update injection chains regardless of validation
  for type_path in &config.structs {
    let struct_name = type_path.path.segments.last().unwrap().ident.to_string();
    if let Err(err) = check_and_update_injection_chain(&target_name, &struct_name) {
      errors.push(err);
    }
  }

  // If we have validation errors, return them
  if !errors.is_empty() {
    return errors.into_iter()
      .map(|e| compile_error(&e))
      .fold(TokenStream::new(), |mut acc, err| {
        acc.extend(std::iter::once(err));
        acc
      });
  }

  // Update module paths and continue with processing
  if let Err(err) = update_module_paths(&config.structs) {
    return compile_error(&err.0);
  }

  // Process the rest as before...
  let fields = match &mut input.data {
    syn::Data::Struct(data) => match &mut data.fields {
      syn::Fields::Named(fields) => fields,
      _ => return compile_error("Only named fields are supported"),
    },
    _ => return compile_error("Only structs are supported as injection targets"),
  };

  let registry_clone = registry::FIELD_REGISTRY.lock().unwrap();
  let chains = registry::INJECTION_CHAINS.lock().unwrap();
  let chains_clone = chains.clone();
  drop(chains);

  match registry::process_type_paths(
    config.structs,
    fields,
    &registry_clone,
    &chains_clone,
  ) {
    Ok(_) => TokenStream::from(quote!(#input)),
    Err(e) => compile_error(&e),
  }
}