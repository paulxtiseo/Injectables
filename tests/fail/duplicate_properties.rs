use injectables::{injectable, inject_fields};

#[injectable]
pub struct Base {
  pub id: u64,
}

#[inject_fields(Base)]
pub struct Conflict {
  pub id: u32,  // Should fail because 'id' is already injected
}

fn main() {}