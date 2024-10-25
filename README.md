# injectables

<p align="center">
  <img src="images/logo-100h.png" alt="My Image">
</p>

A Rust procedural macro library that enables field injection between structs through declarative attributes. This library allows you to compose structs by automatically injecting fields from one or more source structs while respecting Rust's visibility rules and ownership semantics.

## Features

- 🔒 Respects Rust's visibility rules (`pub`, `pub(crate)`, private)
- 🧬 Supports generic types with concrete type resolution
- ⚡ Compile-time dependency injection and validation
- 🔍 Detects circular dependencies and invalid injections at compile time
- 🌳 Supports nested/transitive injections
- 📦 Zero runtime overhead

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
injectables = "0.1.0"
```

## Basic Usage

```rust
use injectables::{injectable, inject_fields};

// Mark source struct as injectable
#[injectable]
pub struct Base {
    pub id: u64,
}

// Inject fields from Base into Document
#[inject_fields(Base)]
pub struct Document {
    pub title: String,
}

fn main() {
    let doc = Document {
        title: "Test".to_string(),
        id: 1,  // Field injected from Base
    };
    
    assert_eq!(doc.id, 1);
    assert_eq!(doc.title, "Test");
}
```

## Advanced Features

### Generic Type Support

The library handles generic types with concrete type specifications:

```rust
#[injectable]
pub struct GenericBase<T> {
    pub data: T,
}

#[inject_fields(GenericBase<String>)]
pub struct Container {
    pub name: String,
}

let container = Container {
    name: "Test".to_string(),
    data: "Hello".to_string(),  // Injected with concrete String type
};
```

### Nested Injections

Fields can be injected transitively through multiple structs:

```rust
#[injectable]
pub struct A {
    pub id: u64,
}

#[injectable]
#[inject_fields(A)]
pub struct B {
    pub name: String,
}

#[inject_fields(B)]
pub struct C {
    pub description: String,
}

// C will have fields: description, name (from B), and id (from A)
```

### Visibility Rules

The library respects Rust's visibility rules:

```rust
mod inner {
    #[injectable]
    pub struct Private {
        id: u64,           // private field
        pub name: String,  // public field
        pub(crate) age: u32, // crate-visible field
    }
}

// Can only inject visible fields based on module boundaries
#[inject_fields(inner::Private)]
pub struct Public {
    pub value: String,
    // Can access `name` and `age`, but not `id`
}
```

## Compile-Time Validations

The library performs several compile-time checks to ensure correct usage:

- ❌ Prevents circular dependencies between structs
- ❌ Detects duplicate field names
- ❌ Validates visibility access rules
- ❌ Ensures source structs are marked as `#[injectable]`
- ❌ Prevents injection into enums or non-struct types
- ❌ Validates generic type parameters

## Current Limitations

1. Only works with named struct fields (not tuple structs)
2. Cannot inject fields into enums
3. Source structs must be marked with `#[injectable]` before being used in `inject_fields`
4. Injected fields maintain their original visibility rules
5. When using generic types, concrete types must be specified in `inject_fields`

## Error Messages

The library provides clear compile-time error messages:

- When attempting circular injections:
  ```
  error: Circular injection chain detected: A already depends on B
  ```

- When accessing private fields from invalid module:
  ```
  error: Cannot access private field 'id' from struct 'Private' defined in module 'other_module'
  ```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure 100% safe Rust.