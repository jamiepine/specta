use specta::{Type, TypeCollection};
use specta_swift::{StructNamingStrategy, Swift};

#[derive(Type)]
pub enum TaskStatus {
    Pending,
    InProgress {
        started_at: String,
        estimated_completion: Option<String>,
    },
    Completed {
        completed_at: String,
        duration_minutes: u32,
    },
    Failed {
        error_message: String,
        retry_count: u32,
    },
}

#[test]
fn test_enum_struct_variants_auto_rename() {
    let swift = Swift::new().struct_naming(StructNamingStrategy::AutoRename);

    let types = TypeCollection::default().register::<TaskStatus>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Should generate separate structs for each variant
    assert!(result.contains("public struct TaskStatusInProgressData: Codable"));
    assert!(result.contains("public struct TaskStatusCompletedData: Codable"));
    assert!(result.contains("public struct TaskStatusFailedData: Codable"));

    // Enum should reference the structs
    assert!(result.contains("case inProgress(TaskStatusInProgressData)"));
    assert!(result.contains("case completed(TaskStatusCompletedData)"));
    assert!(result.contains("case failed(TaskStatusFailedData)"));
}

#[test]
fn test_enum_struct_variants_keep_original() {
    let swift = Swift::new().struct_naming(StructNamingStrategy::KeepOriginal);

    let types = TypeCollection::default().register::<TaskStatus>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Should generate separate structs with original names
    assert!(result.contains("public struct InProgressData: Codable"));
    assert!(result.contains("public struct CompletedData: Codable"));
    assert!(result.contains("public struct FailedData: Codable"));

    // Enum should reference the structs
    assert!(result.contains("case inProgress(InProgressData)"));
    assert!(result.contains("case completed(CompletedData)"));
    assert!(result.contains("case failed(FailedData)"));
}
