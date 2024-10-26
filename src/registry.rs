//! Module for managing injectable field registries and dependency chains.
//!
//! This module provides the core functionality for:
//! - Tracking injectable fields and their properties
//! - Managing dependency relationships between structs
//! - Validating field visibility and accessibility
//! - Processing field injections while respecting Rust's type system
//!
//! # Implementation Details
//!
//! The module maintains two global registries:
//!
//! 1. Field Registry (`FIELD_REGISTRY`):
//!    - Maps struct names to their field definitions and module information
//!    - Populated when processing `#[injectable]` attributes
//!    - Consulted during field injection for field information
//!
//! 2. Injection Chain Registry (`INJECTION_CHAINS`):
//!    - Tracks dependency relationships between structs
//!    - Used to detect circular dependencies
//!    - Maps target structs to their dependencies (direct and transitive)
//!    - Ensures valid injection chains during compilation

use std::{
  collections::{HashMap, HashSet, VecDeque},
  sync::Mutex,
};

use lazy_static::lazy_static;
use proc_macro2::Span;
use quote::ToTokens;
use syn::Field;

use crate::{
  types::{FieldDef, FieldTypeInfo, InjectionError, ModuleInfo},
  visibility::{can_access_field, kind_to_visibility},
};

lazy_static! {
  pub static ref FIELD_REGISTRY: Mutex<HashMap<String, ModuleInfo>> = Mutex::new(HashMap::new());
}

lazy_static! {
  pub static ref INJECTION_CHAINS: Mutex<HashMap<String, HashSet<String>>> = Mutex::new(HashMap::new());
}

/// Validates and updates the injection dependency chain for a new injection.
///
/// This function checks if adding an injection from `source` to `target` would
/// create a circular dependency, and if not, updates the dependency chain.
///
/// # Arguments
///
/// * `target` - Name of the struct receiving injected fields
/// * `source` - Name of the struct providing fields for injection
///
/// # Returns
///
/// * `Ok(())` if the injection is valid and the chain was updated
/// * `Err(String)` with an error message if the injection would create a circular dependency
///
/// # Examples
///
/// ```rust,ignore
/// # use crate::registry::check_and_update_injection_chain;
/// let result = check_and_update_injection_chain("Target", "Source");
/// assert!(result.is_ok());
///
/// // Trying to create a circular dependency
/// let err = check_and_update_injection_chain("Source", "Target");
/// assert!(err.is_err());
/// ```
pub fn check_and_update_injection_chain(target:&str, source:&str) -> Result<(), String> {
  let mut chains = INJECTION_CHAINS.lock().unwrap();

  // Check for direct recursion
  if source == target {
    return Err(format!(
      "Recursive injection detected: {} tries to inject from itself",
      source
    ));
  }

  // Get the target's existing dependencies
  let target_deps = chains.get(target).cloned().unwrap_or_default();

  // Get the source's dependencies
  let mut source_deps = chains.get(source).cloned().unwrap_or_default();
  source_deps.insert(source.to_string()); // Include the source itself in its dependencies

  // Check if adding this injection would create a cycle
  if source_deps.iter().any(|dep| target_deps.contains(dep) || dep == target) {
    return Err(format!(
      "Circular injection chain detected: {} already depends on {}",
      target, source
    ));
  }

  // Update the target's dependencies
  let target_entry = chains.entry(target.to_string()).or_default();
  target_entry.insert(source.to_string());
  target_entry.extend(source_deps);

  Ok(())
}

/// Extracts the module path from a type path.
///
/// # Arguments
///
/// * `type_path` - The type path to process
///
/// # Returns
///
/// A string representation of the module path without the final type name
///
/// # Examples
///
/// ```rust,ignore
/// # use syn::parse_quote;
/// # use crate::registry::get_path_from_type;
/// let type_path = parse_quote!(crate::models::User);
/// assert_eq!(get_path_from_type(&type_path), "crate::models");
/// ```
pub fn get_path_from_type(type_path:&syn::TypePath) -> String {
  let mut module_path = String::new();
  let segments = &type_path.path.segments;
  let segment_count = segments.len();

  for (i, segment) in segments.iter().enumerate() {
    if i == segment_count - 1 {
      break;
    }
    if !module_path.is_empty() {
      module_path.push_str("::");
    }
    module_path.push_str(&segment.ident.to_string());
  }

  module_path
}

/// Processes type paths for field injection, handling field collection, visibility,
/// and type resolution.
///
/// This function is the core of the field injection process. It:
/// 1. Collects fields from all source structs
/// 2. Validates field type compatibility
/// 3. Checks visibility rules
/// 4. Handles generic type resolution
/// 5. Injects fields into the target struct
///
/// # Arguments
///
/// * `type_paths` - Vector of source struct type paths
/// * `fields` - Named fields of the target struct
/// * `registry` - Reference to the field registry
/// * `chains` - Reference to the injection chains
///
/// # Returns
///
/// * `Ok(())` if processing succeeds
/// * `Err(String)` with an error message if any validation fails
pub fn process_type_paths(
  type_paths:Vec<syn::TypePath>,
  fields:&mut syn::FieldsNamed,
  registry:&HashMap<String, ModuleInfo>,
  chains:&HashMap<String, HashSet<String>>,
) -> Result<(), String> {
  let mut added_fields = HashSet::new();
  let mut field_types:HashMap<String, FieldTypeInfo> = HashMap::new();

  for type_path in type_paths {
    let last_segment = type_path.path.segments.last().unwrap();
    let struct_name = last_segment.ident.to_string();
    let target_module = String::new();

    let all_fields = collect_fields(&struct_name, registry, chains)?;

    for field in &all_fields {
      let ty_str = process_field_type(field, last_segment);

      // Check for conflicting field types and visibility
      if let Some(existing) = field_types.get(&field.name) {
        if existing.ty != ty_str {
          return Err(format!(
            "Conflicting types for field '{}': found both '{}' and '{}'",
            field.name, existing.ty, ty_str
          ));
        }

        if existing.vis != field.vis {
          return Err(format!(
            "Conflicting visibility for field '{}': cannot have both private and public fields with the same name",
            field.name
          ));
        }
      } else {
        field_types.insert(
          field.name.clone(),
          FieldTypeInfo {
            name:field.name.clone(),
            ty:  ty_str,
            vis: field.vis.clone(),
          },
        );
      }
    }

    process_fields(
      &struct_name,
      all_fields,
      &mut added_fields,
      &target_module,
      registry,
      last_segment,
      &mut fields.named,
    )?;
  }

  Ok(())
}

/// Collects all fields from a struct and its dependencies recursively.
///
/// This function performs a breadth-first traversal of the injection dependency
/// graph to collect all fields that should be injected.
///
/// # Arguments
///
/// * `start_struct` - Name of the starting struct
/// * `registry` - Reference to the field registry
/// * `chains` - Reference to the injection chains
///
/// # Returns
///
/// * `Ok(Vec<FieldDef>)` with collected fields
/// * `Err(String)` if an error occurs during collection
fn collect_fields(
  start_struct:&str,
  registry:&HashMap<String, ModuleInfo>,
  chains:&HashMap<String, HashSet<String>>,
) -> Result<Vec<FieldDef>, String> {
  let mut all_fields = Vec::new();
  let mut visited = HashSet::new();
  let mut queue = VecDeque::new();

  queue.push_back(start_struct.to_string());

  while let Some(current_struct) = queue.pop_front() {
    if !visited.insert(current_struct.clone()) {
      continue;
    }

    if let Some(info) = registry.get(&current_struct) {
      all_fields.extend(info.fields.clone());
    }

    if let Some(deps) = chains.get(&current_struct) {
      for dep in deps {
        if !visited.contains(dep) {
          queue.push_back(dep.clone());
        }
      }
    }
  }

  Ok(all_fields)
}

/// Processes fields for injection, handling field creation and visibility.
///
/// This function creates new fields in the target struct based on the collected
/// field definitions, respecting visibility rules and handling type resolution.
///
/// # Arguments
///
/// * `struct_name` - Name of the current struct being processed
/// * `all_fields` - Vector of field definitions to process
/// * `added_fields` - Set of field names already added
/// * `target_module` - Module path of the target struct
/// * `registry` - Reference to the field registry
/// * `last_segment` - Last segment of the type path
/// * `named_fields` - Named fields of the target struct
///
/// # Returns
///
/// * `Ok(())` if processing succeeds
/// * `Err(String)` with an error message if processing fails
fn process_fields(
  struct_name:&str,
  all_fields:Vec<FieldDef>,
  added_fields:&mut HashSet<String>,
  target_module:&str,
  registry:&HashMap<String, ModuleInfo>,
  last_segment:&syn::PathSegment,
  named_fields:&mut syn::punctuated::Punctuated<Field, syn::Token![,]>,
) -> Result<(), String> {
  for field in all_fields {
    if !added_fields.insert(field.name.clone()) {
      continue;
    }

    let field_info = FieldTypeInfo {
      name:field.name.clone(),
      ty:  process_field_type(&field, last_segment),
      vis: field.vis.clone(),
    };

    if !can_access_field(&field_info.vis, &registry[struct_name].module_path, target_module) {
      return Err(format!(
        "Cannot access field '{}' with visibility {:?} from module '{}' in module '{}'",
        field_info.name, field_info.vis, registry[struct_name].module_path, target_module
      ));
    }

    let name = syn::Ident::new(&field_info.name, Span::call_site());
    let ty:syn::Type =
      syn::parse_str(&field_info.ty).unwrap_or_else(|_| panic!("Failed to parse type: {}", field_info.ty));

    // Create and add the new field
    let new_field = Field {
      attrs:vec![],
      vis:kind_to_visibility(&field_info.vis),
      mutability:syn::FieldMutability::None,
      ident:Some(name),
      colon_token:Some(Default::default()),
      ty,
    };

    named_fields.push(new_field);
  }
  Ok(())
}

/// Processes field type information, handling generic type resolution.
///
/// # Arguments
///
/// * `field` - Field definition to process
/// * `last_segment` - Last segment of the type path
///
/// # Returns
///
/// String representation of the resolved type
fn process_field_type(field:&FieldDef, last_segment:&syn::PathSegment) -> String {
  if field.generic_params.is_empty() {
    return field.ty.clone();
  }

  match &last_segment.arguments {
    syn::PathArguments::AngleBracketed(args) => {
      let mut ty = field.ty.clone();
      for (param, arg) in field.generic_params.iter().zip(args.args.iter()) {
        ty = ty.replace(param, &arg.to_token_stream().to_string());
      }
      ty
    }
    _ => field.ty.clone(),
  }
}

/// Validates input and processes injection configuration.
///
/// This function performs initial validation of the target struct and injection sources.
///
/// # Arguments
///
/// * `input` - The parsed derive input
/// * `type_paths` - Vector of source struct type paths
///
/// # Returns
///
/// * `Ok(())` if validation succeeds
/// * `Err(InjectionError)` if validation fails
pub fn validate_and_process_input(
  input:&mut syn::DeriveInput,
  type_paths:&[syn::TypePath],
) -> Result<(), InjectionError> {
  match &mut input.data {
    syn::Data::Struct(data) => {
      match &mut data.fields {
        syn::Fields::Named(_) => Ok(()),
        _ => Err(InjectionError("Only named fields are supported".to_string())),
      }
    }
    _ => {
      Err(InjectionError(
        "Only structs are supported as injection targets".to_string(),
      ))
    }
  }?;

  for type_path in type_paths {
    let struct_name = type_path.path.segments.last().unwrap().ident.to_string();

    let registry = FIELD_REGISTRY.lock().unwrap();
    if !registry.contains_key(&struct_name) {
      return Err(InjectionError(format!(
        "Cannot inject fields from '{}' as this type does not exist",
        struct_name
      )));
    }
  }

  Ok(())
}

/// Updates module paths in the registry based on type paths.
///
/// This function updates the module path information for each injectable struct
/// based on its usage in injection.
///
/// # Arguments
///
/// * `type_paths` - Vector of source struct type paths
///
/// # Returns
///
/// * `Ok(())` if update succeeds
/// * `Err(InjectionError)` if update fails
///
/// # Examples
///
/// ```rust,ignore
/// # use syn::parse_quote;
/// # use crate::registry::update_module_paths;
/// let type_paths: Vec<syn::TypePath> = vec![parse_quote!(models::User)];
/// let result = update_module_paths(&type_paths);
/// ```
pub fn update_module_paths(type_paths:&[syn::TypePath]) -> Result<(), InjectionError> {
  let mut registry = FIELD_REGISTRY.lock().unwrap();

  for type_path in type_paths {
    let struct_name = type_path.path.segments.last().unwrap().ident.to_string();
    let module_path = get_path_from_type(type_path);

    if let Some(info) = registry.get_mut(&struct_name) {
      info.module_path = module_path;
    } else {
      return Err(InjectionError(format!(
        "Cannot inject fields from '{}' as it was not marked as #[injectable] or hasn't been defined yet",
        struct_name
      )));
    }
  }

  Ok(())
}
