use specta::{Type, TypeCollection};
use specta_swift::Swift;

#[derive(Type)]
#[specta(tag = "type", content = "data")]
pub enum JobOutput {
    Success,
    FileCopy { copied_count: u32, total_bytes: u64 },
    Indexed { files: u32, size: u64 },
    Custom(), // Empty tuple variant - this might cause the issue
}

#[test]
fn test_adjacently_tagged_enum_with_empty_tuple() {
    let swift = Swift::new();

    let types = TypeCollection::default().register::<JobOutput>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Should generate proper adjacently tagged enum with unique TypeKeys name
    assert!(result.contains("private enum JobOutputTypeKeys: String, CodingKey"));
    assert!(result.contains("case tag = \"type\""));
    assert!(result.contains("case content = \"data\""));

    // Should handle custom variant correctly
    assert!(result.contains("case custom"));

    // Should have exhaustive switch statements
    assert!(result.contains("case .custom:"));

    // Should not have any TODO or fatalError for custom variant
    assert!(
        !result.contains("fatalError")
            || !result.contains("case .custom:")
            || result
                .lines()
                .any(|line| line.contains("case .custom:") && !line.contains("fatalError"))
    );
}
