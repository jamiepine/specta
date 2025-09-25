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
    assert!(result.contains("public struct TaskStatusInProgress: Codable"));
    assert!(result.contains("public struct TaskStatusCompleted: Codable"));
    assert!(result.contains("public struct TaskStatusFailed: Codable"));

    // Enum should reference the structs
    assert!(result.contains("case inProgress(TaskStatusInProgress)"));
    assert!(result.contains("case completed(TaskStatusCompleted)"));
    assert!(result.contains("case failed(TaskStatusFailed)"));
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
    assert!(result.contains("public struct InProgress: Codable"));
    assert!(result.contains("public struct Completed: Codable"));
    assert!(result.contains("public struct Failed: Codable"));

    // Enum should reference the structs
    assert!(result.contains("case inProgress(InProgress)"));
    assert!(result.contains("case completed(Completed)"));
    assert!(result.contains("case failed(Failed)"));
}
