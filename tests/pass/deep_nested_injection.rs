use injectables::{injectable, inject_fields};

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

fn main() {
  let test = C {
    description: "Test".to_string(),
    name: "Test".to_string(),
    id: 1,
  };
  assert_eq!(test.id, 1);
  assert_eq!(test.name, "Test");
  assert_eq!(test.description, "Test");
}