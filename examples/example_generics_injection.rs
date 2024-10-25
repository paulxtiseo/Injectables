use injectables::{inject_fields, injectable};

#[injectable]
pub struct Pageable<T> {
  pub items:      Vec<T>,
  pub page:       u32,
  pub total_pages:u32,
}

#[inject_fields(Pageable<String>)]
pub struct SearchResults {
  pub query:         String,
  pub search_time_ms:u64,
}

fn main() {
  let results = SearchResults {
    query:         "rust tutorials".to_string(),
    search_time_ms:42,
    items:         vec!["Result 1".to_string(), "Result 2".to_string()],
    page:          1,
    total_pages:   5,
  };

  assert_eq!(results.items.len(), 2);
  assert_eq!(results.total_pages, 5);
  println!("Search completed in {}ms", results.search_time_ms);
}
