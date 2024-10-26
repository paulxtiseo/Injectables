//! Core type definitions and parsing implementations for field injection.
//!
//! This module defines the fundamental data structures and parsing traits used
//! throughout the crate for managing field injection, including module information,
//! field definitions, and configuration parsing.

use syn::parse::{Parse, ParseStream};

/// Information about a module and its injectable fields.
///
/// This struct maintains metadata about an injectable struct, including its fields
/// and module path information.
///
/// # Fields
///
/// * `fields` - Vector of field definitions from the struct
/// * `module_path` - Full path to the module containing the struct
#[derive(Clone, Debug)]
pub struct ModuleInfo {
  pub fields:     Vec<FieldDef>,
  pub module_path:String,
}

/// Definition of an injectable field.
///
/// Contains all necessary information about a field that can be injected into
/// other structs, including its name, type, visibility, and any generic parameters.
///
/// # Fields
///
/// * `name` - Name of the field
/// * `ty` - Type of the field as a string
/// * `vis` - Visibility of the field
/// * `generic_params` - Names of generic type parameters if any
///
/// # Examples
///
/// ```rust,ignore
/// # use crate::visibility::VisibilityKind;
/// # use crate::types::FieldDef;
/// let field = FieldDef {
///     name: "id".to_string(),
///     ty: "u64".to_string(),
///     vis: VisibilityKind::Public,
///     generic_params: vec![],
/// };
/// ```
#[derive(Clone, Debug)]
pub struct FieldDef {
  pub name:          String,
  pub ty:            String,
  pub vis:           super::visibility::VisibilityKind,
  pub generic_params:Vec<String>,
}

/// Type information for a field during processing.
///
/// Used during field injection to track type and visibility information
/// as fields are processed and validated.
///
/// # Fields
///
/// * `name` - Name of the field
/// * `ty` - Type of the field
/// * `vis` - Visibility of the field
#[derive(Debug, Clone)]
pub struct FieldTypeInfo {
  pub name:String,
  pub ty:  String,
  pub vis: super::visibility::VisibilityKind,
}

/// Configuration for field injection.
///
/// Parsed from the attribute arguments of `#[inject_fields(...)]`,
/// containing the source structs to inject fields from.
///
/// # Fields
///
/// * `structs` - Vector of type paths representing source structs
///
/// # Examples
///
/// ```rust,ignore
/// // The macro invocation #[inject_fields(UserData, Timestamps)]
/// // would parse into an InjectConfig containing two TypePaths
/// ```
pub struct InjectConfig {
  pub structs:Vec<syn::TypePath>,
}

impl Parse for InjectConfig {
  fn parse(input:ParseStream) -> syn::Result<Self> {
    Ok(InjectConfig {
      structs:input
        .parse_terminated(syn::TypePath::parse, syn::Token![,])?
        .into_iter()
        .collect(),
    })
  }
}

/// Error type for injection-related failures.
///
/// Wraps a string error message describing what went wrong during
/// the injection process.
///
/// # Examples
///
/// ```rust,ignore
/// # use crate::types::InjectionError;
/// let error = InjectionError("Cannot inject private field".to_string());
/// ```
#[derive(Debug)]
pub struct InjectionError(pub String);

impl From<String> for InjectionError {
  fn from(msg:String) -> Self { InjectionError(msg) }
}
