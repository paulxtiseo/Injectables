use injectables::{inject_fields, injectable};

#[injectable]
pub struct Base {
  pub id:u64,
}

#[inject_fields(Base)]
pub struct Document {
  pub title:String,
}

fn main() {
  let doc = Document {
    title:"Test".to_string(),
    id:   1,
  };
  assert_eq!(doc.id, 1);
  assert_eq!(doc.title, "Test");
}
