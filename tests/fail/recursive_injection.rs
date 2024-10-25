use injectables::{injectable, inject_fields};

#[injectable]
#[inject_fields(RecursiveStruct)]  // Tries to inject its own fields
pub struct RecursiveStruct {
  pub id: u64,
}

fn main() {
  let test = RecursiveStruct {
    id: 1,
  };
}