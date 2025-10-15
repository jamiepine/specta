use specta::{Type, TypeCollection};
use specta_swift::Swift;

#[derive(Type)]
pub struct TestStruct {
    pub job_id: String,
    pub include_stats: bool,
    pub status: Option<String>,
}

fn main() {
    let mut types = TypeCollection::default()
        .register::<TestStruct>();

    // Test with initializers enabled
    let swift_with_init = Swift::new()
        .generate_initializers(true);
    
    let result_with_init = swift_with_init.export(&types).unwrap();
    println!("With initializers:\n{}", result_with_init);
    
    // Test with initializers disabled (default)
    let swift_without_init = Swift::new()
        .generate_initializers(false);
    
    let result_without_init = swift_without_init.export(&types).unwrap();
    println!("\nWithout initializers:\n{}", result_without_init);
}

