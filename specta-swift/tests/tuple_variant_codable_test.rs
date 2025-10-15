use specta::{Type, TypeCollection};
use specta_swift::Swift;

#[derive(Type)]
enum Status {
    Active,
    Pending(String),
    Error(String, u32),
    Loading(f64, String, bool),
}

#[test]
fn test_tuple_variant_codable_generation() {
    let types = TypeCollection::default().register::<Status>();

    let output = Swift::default().export(&types).unwrap();

    println!("\n=== GENERATED SWIFT CODE ===\n{}\n", output);

    // Verify enum declaration has tuple cases
    assert!(output.contains("case active"));
    assert!(output.contains("case pending(String)"));
    assert!(output.contains("case error(String, UInt32)"));
    assert!(output.contains("case loading(Double, String, Bool)"));

    // Verify Codable implementation is generated
    assert!(output.contains("extension Status: Codable"));

    // Verify tuple variant decoding is implemented (no fatalError)
    assert!(!output.contains("fatalError(\"Tuple variant decoding not implemented\")"));
    assert!(!output.contains("fatalError(\"Tuple variant encoding not implemented\")"));

    // Verify array container usage for tuple variants
    assert!(output.contains("arrayContainer"));
    assert!(output.contains("nestedUnkeyedContainer"));

    println!("✅ Tuple variant Codable generation successful!");
}
