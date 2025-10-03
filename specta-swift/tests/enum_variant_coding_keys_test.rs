use specta::{Type, TypeCollection};
use specta_swift::{StructNamingStrategy, Swift};

#[derive(Type)]
pub enum JobEvent {
    Started {
        job_id: String,     // snake_case field
        job_type: String,   // snake_case field
        started_at: String, // snake_case field
    },
    Progress {
        job_id: String,
        progress_percent: f64,
    },
}

#[test]
fn test_enum_variant_struct_coding_keys() {
    let swift = Swift::new()
        .struct_naming(StructNamingStrategy::AutoRename)
        .naming(specta_swift::NamingConvention::PascalCase);

    let types = TypeCollection::default().register::<JobEvent>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Should generate enum variant structs with CodingKeys
    assert!(result.contains("public struct JobEventStartedData: Codable"));
    assert!(result.contains("public let jobId: String"));
    assert!(result.contains("case jobId = \"job_id\""));
    assert!(result.contains("case jobType = \"job_type\""));
    assert!(result.contains("case startedAt = \"started_at\""));
    assert!(result.contains("private enum CodingKeys: String, CodingKey"));
}
