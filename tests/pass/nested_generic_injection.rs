use injectables::{injectable, inject_fields};

#[injectable]
pub struct GenericWrapper<T> {
  pub data: Option<T>,
}

#[inject_fields(GenericWrapper<Vec<i32>>)]
pub struct NestedContainer {
  pub name: String,
}

fn main() {
  let container = NestedContainer {
    name: "Test".to_string(),
    data: Some(vec![1, 2, 3]),
  };
  assert_eq!(container.data.unwrap(), vec![1, 2, 3]);
}