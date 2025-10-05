use specta::{Type, TypeCollection};
use specta_swift::Swift;

/// Test struct with optional fields to verify encoding behavior
#[derive(Type)]
struct JobListInput {
    status: Option<String>,
}

/// Test struct with multiple optional fields
#[derive(Type)]
struct MultiOptionalFields {
    name: Option<String>,
    age: Option<u32>,
    email: Option<String>,
    is_active: Option<bool>,
}

/// Test struct with mix of optional and required fields
#[derive(Type)]
struct MixedFields {
    required_field: String,
    optional_field: Option<String>,
    another_required: u32,
    another_optional: Option<bool>,
}

#[test]
fn test_optional_field_generates_custom_codable() {
    let swift = Swift::new();
    let types = TypeCollection::default().register::<JobListInput>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Should generate the struct
    assert!(result.contains("public struct JobListInput: Codable"));
    assert!(result.contains("public let status: String?"));

    // The key question: Does it generate custom encode/decode to preserve nil?
    // If this assertion fails, it means specta-swift is relying on Swift's default
    // Codable which omits nil values
    let has_custom_encode = result.contains("public func encode(to encoder: Encoder)");
    let has_custom_decode = result.contains("public init(from decoder: Decoder)");

    println!("\nHas custom encode: {}", has_custom_encode);
    println!("Has custom decode: {}", has_custom_decode);

    // Document the current behavior
    if !has_custom_encode {
        println!("\n⚠️  WARNING: No custom encode implementation found!");
        println!("This means nil values will be OMITTED from JSON, not encoded as null.");
        println!("Expected: {{\"status\": null}}");
        println!("Actual:   {{}}");
    }

    if !has_custom_decode {
        println!("\n⚠️  WARNING: No custom decode implementation found!");
    }
}

#[test]
fn test_multiple_optional_fields() {
    let swift = Swift::new();
    let types = TypeCollection::default().register::<MultiOptionalFields>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Check that all optional fields are generated
    assert!(result.contains("public let name: String?"));
    assert!(result.contains("public let age: UInt32?"));
    assert!(result.contains("public let email: String?"));
    assert!(result.contains("public let isActive: Bool?"));

    // Check for custom Codable implementation
    let has_custom_encode = result.contains("public func encode(to encoder: Encoder)");

    println!(
        "\nHas custom encode for multiple optionals: {}",
        has_custom_encode
    );

    if !has_custom_encode {
        println!("\n⚠️  ISSUE CONFIRMED: Multiple optional fields also lack custom encoding!");
        println!("All nil values will be omitted from JSON output.");
    }
}

#[test]
fn test_mixed_required_and_optional_fields() {
    let swift = Swift::new();
    let types = TypeCollection::default().register::<MixedFields>();

    let result = swift
        .export(&types)
        .expect("Failed to generate Swift types");

    println!("Generated Swift code:\n{}", result);

    // Check that fields are generated correctly
    assert!(result.contains("public let requiredField: String"));
    assert!(result.contains("public let optionalField: String?"));
    assert!(result.contains("public let anotherRequired: UInt32"));
    assert!(result.contains("public let anotherOptional: Bool?"));

    // Check for custom Codable implementation
    let has_custom_encode = result.contains("public func encode(to encoder: Encoder)");
    let has_encode_nil_preservation = result.contains("try container.encode(")
        || result.contains("try container.encodeIfPresent(");

    println!("\nHas custom encode: {}", has_custom_encode);
    println!("Has encode method: {}", has_encode_nil_preservation);

    if has_custom_encode {
        // If custom encode exists, check which method is used
        if result.contains("encodeIfPresent") {
            println!("\n⚠️  ISSUE: Using encodeIfPresent which OMITS nil values!");
            println!("Should use encode() for all fields to preserve nil as null.");
        } else if result.contains("try container.encode(") {
            println!("\n✅ GOOD: Using encode() which preserves nil as null.");
        }
    } else {
        println!("\n⚠️  ISSUE: No custom encode, relying on Swift default which omits nil!");
    }
}

#[test]
fn test_documents_expected_behavior() {
    // This test documents what the CORRECT behavior should be

    println!("\n=== EXPECTED BEHAVIOR ===");
    println!("\nFor a struct like:");
    println!("  struct JobListInput {{ status: Option<String> }}");
    println!("\nWhen encoded with status = None:");
    println!("  ❌ Current (wrong): {{}}");
    println!("  ✅ Expected (correct): {{\"status\": null}}");

    println!("\n=== SOLUTION ===");
    println!("\nGenerated Swift should include:");
    println!("  public func encode(to encoder: Encoder) throws {{");
    println!("      var container = encoder.container(keyedBy: CodingKeys.self)");
    println!("      try container.encode(status, forKey: .status)  // preserves nil as null");
    println!("  }}");

    println!("\n=== NOT THIS ===");
    println!("  public func encode(to encoder: Encoder) throws {{");
    println!("      var container = encoder.container(keyedBy: CodingKeys.self)");
    println!("      try container.encodeIfPresent(status, forKey: .status)  // omits nil!");
    println!("  }}");
}

