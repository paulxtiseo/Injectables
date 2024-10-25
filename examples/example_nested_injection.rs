use injectables::{inject_fields, injectable};

#[injectable]
pub struct Auditable {
  pub created_by:String,
  pub created_at:String,
}

#[injectable]
#[inject_fields(Auditable)]
pub struct Versioned {
  pub version:  u32,
  pub is_latest:bool,
}

#[inject_fields(Versioned)]
pub struct Document {
  pub title:  String,
  pub content:String,
}

// Optional: Builder pattern for easier construction
impl Document {
  pub fn builder() -> DocumentBuilder { DocumentBuilder::default() }
}

#[derive(Default)]
pub struct DocumentBuilder {
  title:     Option<String>,
  content:   Option<String>,
  version:   Option<u32>,
  is_latest: Option<bool>,
  created_by:Option<String>,
  created_at:Option<String>,
}

impl DocumentBuilder {
  pub fn title(mut self, title:impl Into<String>) -> Self {
    self.title = Some(title.into());
    self
  }

  pub fn content(mut self, content:impl Into<String>) -> Self {
    self.content = Some(content.into());
    self
  }

  pub fn version(mut self, version:u32) -> Self {
    self.version = Some(version);
    self
  }

  pub fn is_latest(mut self, is_latest:bool) -> Self {
    self.is_latest = Some(is_latest);
    self
  }

  pub fn created_by(mut self, created_by:impl Into<String>) -> Self {
    self.created_by = Some(created_by.into());
    self
  }

  pub fn created_at(mut self, created_at:impl Into<String>) -> Self {
    self.created_at = Some(created_at.into());
    self
  }

  pub fn build(self) -> Option<Document> {
    Some(Document {
      title:     self.title?,
      content:   self.content?,
      version:   self.version?,
      is_latest: self.is_latest?,
      created_by:self.created_by?,
      created_at:self.created_at?,
    })
  }
}

fn main() {
  let doc = Document::builder()
    .title("My Document")
    .content("Some content")
    .version(1)
    .is_latest(true)
    .created_by("John Doe")
    .created_at("2024-01-01")
    .build()
    .unwrap();

  assert_eq!(doc.title, "My Document");
  assert_eq!(doc.version, 1);
  assert_eq!(doc.created_by, "John Doe");

  println!("Document created successfully: {}", doc.title);
}
