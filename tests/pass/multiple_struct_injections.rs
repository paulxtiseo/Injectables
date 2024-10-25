use injectables::{injectable, inject_fields};

#[injectable]
pub struct GenericPair<T, U> {
  pub first: T,
  pub second: U,
}

#[inject_fields(GenericPair<i32, String>)]
pub struct MultiContainer {
  pub name: String,
}

fn main() {
  let container = MultiContainer {
    name: "Test".to_string(),
    first: 42,
    second: "Hello".to_string(),
  };
  assert_eq!(container.first, 42);
  assert_eq!(container.second, "Hello");
}