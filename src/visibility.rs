//! Visibility handling and validation for field injection.
//!
//! This module provides functionality for:
//! - Converting between Rust's visibility syntax and internal visibility representation
//! - Validating field visibility rules during injection
//! - Enforcing module boundary access rules

use quote::ToTokens;
use syn::Visibility;

/// Internal representation of Rust visibility rules.
///
/// This enum represents all possible visibility levels in Rust,
/// used for tracking and validating field access during injection.
///
/// # Variants
///
/// * `Public` - Visible everywhere (`pub`)
/// * `Private` - Only visible in the current module (no modifier)
/// * `Restricted` - Custom visibility like `pub(crate)` or `pub(in path)`
#[derive(Clone, Debug, PartialEq)]
pub enum VisibilityKind {
  Public,
  Private,
  Restricted(String),
}

/// Converts syn's `Visibility` to our internal `VisibilityKind`.
///
/// # Arguments
///
/// * `vis` - The visibility syntax tree node to convert
///
/// # Returns
///
/// A `VisibilityKind` representing the visibility level
///
/// # Examples
///
/// ```rust,ignore
/// # use syn::Visibility;
/// # use crate::visibility::visibility_to_kind;
/// let vis: Visibility = syn::parse_quote!(pub);
/// let kind = visibility_to_kind(&vis);
/// ```
pub fn visibility_to_kind(vis:&Visibility) -> VisibilityKind {
  match vis {
    Visibility::Public(_) => VisibilityKind::Public,
    Visibility::Inherited => VisibilityKind::Private,
    Visibility::Restricted(restricted) => {
      let path = restricted.path.to_token_stream().to_string();
      VisibilityKind::Restricted(path)
    }
  }
}

/// Converts internal `VisibilityKind` back to syn's `Visibility`.
///
/// # Arguments
///
/// * `kind` - The visibility kind to convert
///
/// # Returns
///
/// A syn `Visibility` node representing the visibility
///
/// # Examples
///
/// ```rust,ignore
/// # use crate::visibility::{VisibilityKind, kind_to_visibility};
/// let vis = kind_to_visibility(&VisibilityKind::Public);
/// ```
pub fn kind_to_visibility(kind:&VisibilityKind) -> Visibility {
  match kind {
    VisibilityKind::Public => syn::parse_quote!(pub),
    VisibilityKind::Private => Visibility::Inherited,
    VisibilityKind::Restricted(path_str) => {
      match path_str.as_str() {
        "crate" => syn::parse_quote!(pub(crate)),
        "super" => syn::parse_quote!(pub(super)),
        "self" => syn::parse_quote!(pub(self)),
        _ => {
          let path:syn::Path = syn::parse_str(path_str).unwrap_or_else(|_| syn::parse_quote!(self));
          Visibility::Restricted(syn::VisRestricted {
            pub_token:  syn::token::Pub::default(),
            paren_token:syn::token::Paren::default(),
            in_token:   Some(syn::token::In::default()),
            path:       Box::new(path),
          })
        }
      }
    }
  }
}

/// Checks if a field with given visibility can be accessed from one module in another.
///
/// This function implements Rust's visibility rules to determine if field injection
/// would violate any access restrictions.
///
/// # Arguments
///
/// * `vis` - Visibility of the field
/// * `target_module` - Module path where the field is defined
/// * `source_module` - Module path trying to access the field
///
/// # Returns
///
/// `true` if the field can be accessed, `false` otherwise
///
/// # Examples
///
/// ```rust,ignore
/// # use crate::visibility::{VisibilityKind, can_access_field};
/// let can_access = can_access_field(
///     &VisibilityKind::Public,
///     "crate::models",
///     "crate::controllers"
/// );
/// assert!(can_access);
/// ```
pub fn can_access_field(vis:&VisibilityKind, target_module:&str, source_module:&str) -> bool {
  match vis {
    VisibilityKind::Public => true,
    VisibilityKind::Private => {
      // Private fields can only be injected within the same module
      if target_module.is_empty() && source_module.is_empty() {
        true
      } else {
        let same_module = source_module == target_module;
        let is_child =
          source_module.starts_with(target_module) && source_module[target_module.len()..].starts_with("::");
        same_module || is_child
      }
    }
    VisibilityKind::Restricted(restriction) => {
      match restriction.as_str() {
        "crate" => true,
        "super" => source_module.starts_with(target_module),
        "self" => target_module == source_module,
        path => {
          let path = path.trim_start_matches("in ");
          source_module.starts_with(path)
        }
      }
    }
  }
}
