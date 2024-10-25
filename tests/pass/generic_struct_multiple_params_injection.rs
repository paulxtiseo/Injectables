use injectables::{injectable, inject_fields};

#[injectable]
pub struct GenericBase<T> {
  pub data: T,
}

#[inject_fields(GenericBase<String>)]  // Specify concrete type
pub struct Container {
  pub name: String,
}

fn main() {
  let container = Container {
    name: "Test".to_string(),
    data: "Hello".to_string(),
  };
  assert_eq!(container.data, "Hello");
}