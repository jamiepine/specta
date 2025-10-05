use specta::{Type, TypeCollection};
use specta_swift::Swift;

#[derive(Type)]
struct Zebra {
    field: String,
}

#[derive(Type)]
struct Apple {
    field: String,
}

#[derive(Type)]
struct Mango {
    field: String,
}

#[derive(Type)]
struct Banana {
    field: String,
}

#[test]
fn test_types_exported_in_alphabetical_order() {
    let swift = Swift::new();
    let types = TypeCollection::default()
        .register::<Zebra>()
        .register::<Apple>()
        .register::<Mango>()
        .register::<Banana>();

    let result = swift.export(&types).unwrap();

    println!("Generated output:\n{}", result);

    // Find positions of each type
    let apple_pos = result.find("public struct Apple").expect("Apple not found");
    let banana_pos = result
        .find("public struct Banana")
        .expect("Banana not found");
    let mango_pos = result.find("public struct Mango").expect("Mango not found");
    let zebra_pos = result.find("public struct Zebra").expect("Zebra not found");

    println!("\nPositions:");
    println!("  Apple: {}", apple_pos);
    println!("  Banana: {}", banana_pos);
    println!("  Mango: {}", mango_pos);
    println!("  Zebra: {}", zebra_pos);

    // Verify alphabetical order
    assert!(apple_pos < banana_pos, "Apple should come before Banana");
    assert!(banana_pos < mango_pos, "Banana should come before Mango");
    assert!(mango_pos < zebra_pos, "Mango should come before Zebra");

    println!("\n✅ Types are exported in alphabetical order!");
}

#[test]
fn test_deterministic_output_across_runs() {
    let swift = Swift::new();
    let types = TypeCollection::default()
        .register::<Zebra>()
        .register::<Apple>()
        .register::<Mango>()
        .register::<Banana>();

    // Generate multiple times
    let output1 = swift.export(&types).unwrap();
    let output2 = swift.export(&types).unwrap();
    let output3 = swift.export(&types).unwrap();

    // All outputs should be identical
    assert_eq!(output1, output2, "Output 1 and 2 should be identical");
    assert_eq!(output2, output3, "Output 2 and 3 should be identical");

    println!("✅ Output is deterministic across multiple runs!");
}
