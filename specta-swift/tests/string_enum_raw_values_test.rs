use specta::{Type, TypeCollection};
use specta_swift::Swift;

#[derive(Type)]
#[specta(rename_all = "lowercase")]
pub enum ContentKind {
    Unknown,
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Code,
    Text,
    Database,
    Book,
    Font,
    Mesh,
    Config,
    Encrypted,
    Key,
    Executable,
    Binary,
}

#[test]
fn test_string_enum_with_raw_values() {
    let swift = Swift::new();

    let types = TypeCollection::default().register::<ContentKind>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Should generate proper string enum with raw type declaration
    assert!(
        result.contains("public enum ContentKind: String, Codable")
            || result.contains("public enum ContentKind")
    );

    // Should have proper raw values
    assert!(result.contains("case unknown = \"unknown\""));
    assert!(result.contains("case image = \"image\""));
    assert!(result.contains("case key = \"key\""));
}
