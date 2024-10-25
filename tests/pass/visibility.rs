mod inner {
  use injectables::{injectable, inject_fields};

  #[injectable]
  #[derive(Default)]  // add Default for Private
  pub struct Private {
    id: u64,           // private field
    pub name: String,  // public field
    pub(crate) age: u32, // crate-visible field
  }

  #[inject_fields(Private)]
  #[derive(Debug, Default)]  // add Default for TestVisibility
  pub struct TestVisibility {
    value: u32,  // private field
    pub description: String, // public field
  }
  pub fn new_test_visibility(description: String, name: String, age: u32) -> TestVisibility {
    TestVisibility {
      value: 42,
      description,
      id: 1,
      name,
      age,
    }
  }

  pub fn test_same_module() -> TestVisibility {
    let test = TestVisibility {
      value: 42,
      description: "Test".to_string(),
      id: 1,
      name: "Test".to_string(),
      age: 25,
    };

    // verify we can read private fields in same module
    assert_eq!(test.value, 42);
    assert_eq!(test.id, 1);
    assert_eq!(test.age, 25);

    test
  }
}

fn test_different_module() {
  let test = inner::new_test_visibility(
    "Test".to_string(),
    "Test".to_string(),
    25
  );

  let mut test2 = inner::TestVisibility::default();
  test2.description = "Test".to_string();
  test2.name = "Test".to_string();
  test2.age = 25;

  assert_eq!(test.name, "Test");
  assert_eq!(test.description, "Test");
  assert_eq!(test.age, 25);
}

fn main() {
  test_different_module();
}