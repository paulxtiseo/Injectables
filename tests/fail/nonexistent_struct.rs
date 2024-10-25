use injectables::inject_fields;

#[inject_fields(NonexistentStruct)]
pub struct Test {
    pub name: String,
}

fn main() {}