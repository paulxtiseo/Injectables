use injectables::{injectable, inject_fields};

#[injectable]
#[inject_fields(B)]
pub struct A {
  pub id: u64,
}

#[injectable]
#[inject_fields(A)]
pub struct B {
  pub name: String,
}

fn main() {}