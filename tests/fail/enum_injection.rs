use injectables::{injectable, inject_fields};

#[injectable]
pub enum Invalid {
    Variant1,
    Variant2,
}

#[inject_fields(Invalid)]
pub struct Test {
    pub name: String,
}

fn main() {}