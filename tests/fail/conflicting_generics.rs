use injectables::{injectable, inject_fields};

#[injectable]
pub struct GenericBase<T> {
  pub data: T,
}

#[inject_fields(GenericBase<i32>, GenericBase<String>)]  // should fail - conflicting types for 'data'
pub struct ConflictingGenerics {
  pub name: String,
}

fn main() {}