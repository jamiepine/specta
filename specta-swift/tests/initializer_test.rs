use specta::{Type, TypeCollection};
use specta_swift::Swift;

#[derive(Type)]
pub struct TestStruct {
    pub job_id: String,
    pub include_stats: bool,
    pub status: Option<String>,
}

#[test]
fn test_initializer_generation() {
    let mut types = TypeCollection::default()
        .register::<TestStruct>();

    // Test with initializers enabled
    let mut swift_with_init = Swift::new();
    swift_with_init.generate_initializers = true;
    
    let result_with_init = swift_with_init.export(&types).unwrap();
    println!("With initializers:\n{}", result_with_init);
    
    // Verify that the initializer is present
    assert!(result_with_init.contains("public init("));
    assert!(result_with_init.contains("jobId: String"));
    assert!(result_with_init.contains("includeStats: Bool"));
    assert!(result_with_init.contains("status: String?"));
    
    // Test with initializers disabled (default)
    let mut swift_without_init = Swift::new();
    swift_without_init.generate_initializers = false;
    
    let result_without_init = swift_without_init.export(&types).unwrap();
    println!("\nWithout initializers:\n{}", result_without_init);
    
    // Verify that the initializer is NOT present
    assert!(!result_without_init.contains("public init("));
}
