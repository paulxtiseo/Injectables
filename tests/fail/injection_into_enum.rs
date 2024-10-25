use injectables::{injectable, inject_fields};

#[injectable]
pub struct Base {
    pub id: u64,
}

#[inject_fields(Base)]
pub enum InvalidTarget {  // Should fail because enums can't receive injected fields
    Variant1,
    Variant2,
}

fn main() {}