use injectables::{injectable, inject_fields};

mod other_module {
    use injectables::{injectable, inject_fields};

    #[injectable]
    pub struct Private {
        id: u64,           // private field
        pub name: String,  // public field
    }
}

#[inject_fields(other_module::Private)]
pub struct IllegalAccess {
    pub value: u32,
}

fn main() {
    let test = IllegalAccess {
        value: 42,
        id: 1,      // this should fail to compile - cannot access private field
        name: "Test".to_string(),
    };

    // use the variable to prevent unused variable warnings
    println!("name: {}", test.name);
}