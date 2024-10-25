use injectables::{injectable, inject_fields};

mod inner {
  use super::*;

  #[injectable]
  pub struct Restricted {
    pub(in crate::invalid::path) field: String,  // invalid visibility path
  }
}

#[inject_fields(inner::Restricted)]
pub struct InvalidVisibility {
  pub name: String,
}

fn main() {}