use injectables::inject_fields;

// We put our source structs in a module to demonstrate realistic visibility boundaries
mod models {
  use injectables::injectable;

  // This struct will inject fields with different visibility levels.
  // Note: Private fields cannot be injected across module boundaries,
  // so we only include public and crate-visible fields here.
  #[injectable]
  #[derive(Debug, PartialEq)]
  pub struct Timestamps {
    pub created_at:       String, // publicly accessible
    pub(crate) updated_at:String, // only accessible within the current crate
  }

  #[cfg(test)]
  impl Timestamps {
    pub fn new(created_at:impl Into<String>) -> Self {
      let created = created_at.into();
      Self {
        created_at:created.clone(),
        updated_at:created,
      }
    }

    pub(crate) fn describe(&self) -> String { format!("Created: {}, Updated: {}", self.created_at, self.updated_at) }

    pub(crate) fn update(&mut self, new_time:impl Into<String>) { self.updated_at = new_time.into(); }
  }

  impl Default for Timestamps {
    fn default() -> Self {
      Self {
        created_at:String::new(),
        updated_at:String::new(),
      }
    }
  }

  // Another injectable struct that provides metadata functionality
  #[injectable]
  #[derive(Debug, PartialEq)]
  pub struct Metadata {
    pub author:String,
    pub tags:  Vec<String>,
  }

  #[cfg(test)]
  impl Metadata {
    pub fn new(author:impl Into<String>, tags:Vec<String>) -> Self {
      Self {
        author:author.into(),
        tags,
      }
    }

    pub fn summarize(&self) -> String { format!("By {} ({})", self.author, self.tags.join(", ")) }

    pub fn add_tag(&mut self, tag:impl Into<String>) { self.tags.push(tag.into()); }
  }

  impl Default for Metadata {
    fn default() -> Self {
      Self {
        author:String::new(),
        tags:  Vec::new(),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[test]
    fn test_timestamps() {
      let mut ts = Timestamps::new("2024-01-01");
      assert_eq!(ts.created_at, "2024-01-01");
      assert_eq!(ts.updated_at, "2024-01-01");

      ts.update("2024-01-02");
      assert_eq!(ts.updated_at, "2024-01-02");
      assert_eq!(ts.describe(), "Created: 2024-01-01, Updated: 2024-01-02");
    }

    #[test]
    fn test_metadata() {
      let mut meta = Metadata::new("Test Author", vec!["test".to_string()]);
      assert_eq!(meta.author, "Test Author");
      assert_eq!(meta.tags, vec!["test"]);

      meta.add_tag("example");
      assert_eq!(meta.summarize(), "By Test Author (test, example)");
    }
  }
}

// When we inject fields, we inherit their visibility levels
// - created_at will be pub
// - updated_at will be pub(crate)
// - author and tags will be pub
//
// Note: We can only inject fields that are visible from this module.
// Private fields in source structs cannot be injected across module boundaries.
#[inject_fields(models::Timestamps, models::Metadata)]
#[derive(Default)]
pub struct BlogPost {
  pub title:  String,
  pub content:String,
}

impl BlogPost {
  pub fn new(
    title:impl Into<String>,
    content:impl Into<String>,
    author:impl Into<String>,
    tags:Vec<String>,
    created_at:impl Into<String>,
  ) -> Self {
    let mut post = Self {
      title:title.into(),
      content:content.into(),
      ..Default::default()
    };

    post.author = author.into();
    post.tags = tags;
    post.created_at = created_at.into();
    post.updated_at = post.created_at.clone();

    post
  }

  pub fn get_metadata_summary(&self) -> String { format!("By {} ({})", self.author, self.tags.join(", ")) }

  pub fn get_timestamp_info(&self) -> String {
    format!("Created: {}, Last Updated: {}", self.created_at, self.updated_at)
  }

  pub fn update_content(&mut self, new_content:impl Into<String>) {
    self.content = new_content.into();
    self.updated_at = "2024-01-02".to_string();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_blog_post() {
    let mut post = BlogPost::new(
      "Test Title",
      "Test Content",
      "Test Author",
      vec!["test".to_string()],
      "2024-01-01",
    );

    assert_eq!(post.title, "Test Title");
    assert_eq!(post.content, "Test Content");
    assert_eq!(post.author, "Test Author");
    assert_eq!(post.tags, vec!["test"]);
    assert_eq!(post.created_at, "2024-01-01");
    assert_eq!(post.updated_at, "2024-01-01");

    post.update_content("Updated Content");
    assert_eq!(post.content, "Updated Content");
    assert_eq!(post.updated_at, "2024-01-02");
  }
}

fn main() {
  let mut post = BlogPost::new(
    "Hello World",
    "My first post",
    "John Doe",
    vec!["rust".to_string(), "tutorial".to_string()],
    "2024-01-01",
  );

  println!("Post info: {}", post.get_metadata_summary());
  println!("Timestamps: {}", post.get_timestamp_info());

  post.update_content("Updated content");
  println!("\nAfter update:");
  println!("New timestamps: {}", post.get_timestamp_info());

  assert_eq!(post.title, "Hello World");
  assert_eq!(post.content, "Updated content");
  assert_eq!(post.author, "John Doe");
  assert_eq!(post.tags.len(), 2);
  assert_eq!(post.created_at, "2024-01-01");
  assert_eq!(post.updated_at, "2024-01-02");

  println!("\nKey Takeaways:");
  println!("1. Injected fields maintain their original visibility");
  println!("2. Default trait helps handle field initialization");
  println!("3. Crate-visible fields are accessible within the same crate");
  println!("4. Field injection respects Rust's visibility rules");
  println!("5. Private fields cannot be injected across module boundaries");
}
