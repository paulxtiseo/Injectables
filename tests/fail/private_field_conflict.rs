use injectables::{injectable, inject_fields};

#[injectable]
pub struct Base {
  id: u64,  // private field
}

#[injectable]
pub struct Derived {
  pub id: u64,  // public field with same name
}

#[inject_fields(Base, Derived)]
pub struct Conflict {
  pub name: String,
}

fn main() {}