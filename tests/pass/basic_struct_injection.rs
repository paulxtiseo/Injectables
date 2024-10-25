use injectables::{injectable, inject_fields};

#[injectable]
pub struct Base {
  pub id: u64,
}

#[inject_fields(Base)]
#[derive(Debug)] // the derive() must follow the inject_fields() to show up in the println!()
pub struct Document {
  pub title: String,
}

fn main() {
  let doc = Document {
    title: "Test".to_string(),
    id: 1,
  };

  assert_eq!(doc.title, "Test");
  assert_eq!(doc.id, 1);
}