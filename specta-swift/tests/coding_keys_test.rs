use specta::{Type, TypeCollection};
use specta_swift::Swift;

#[derive(Type)]
pub struct TestStruct {
    pub job_id: String,     // snake_case field
    pub job_type: String,   // snake_case field
    pub created_at: String, // snake_case field
}

#[test]
fn test_coding_keys_generation() {
    let swift = Swift::new().naming(specta_swift::NamingConvention::PascalCase);

    let types = TypeCollection::default().register::<TestStruct>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Should generate proper CodingKeys for snake_case → camelCase mapping
    assert!(result.contains("public let jobId: String"));
    assert!(result.contains("public let jobType: String"));
    assert!(result.contains("case jobId = \"job_id\""));
    assert!(result.contains("case jobType = \"job_type\""));
    assert!(result.contains("private enum CodingKeys: String, CodingKey"));
}
