// Test to understand tuple variant serialization format

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Type, Serialize, Deserialize, Debug)]
enum TestEnum {
    Unit,
    Tuple(String, i32),
    TupleThree(String, i32, bool),
}

#[test]
fn test_tuple_variant_json_format() {
    // Test what JSON format serde uses for tuple variants
    let tuple_two = TestEnum::Tuple("hello".to_string(), 42);
    let json = serde_json::to_string(&tuple_two).unwrap();
    println!("Tuple(String, i32) serializes as: {}", json);

    let tuple_three = TestEnum::TupleThree("world".to_string(), 100, true);
    let json3 = serde_json::to_string(&tuple_three).unwrap();
    println!("Tuple(String, i32, bool) serializes as: {}", json3);

    let unit = TestEnum::Unit;
    let json_unit = serde_json::to_string(&unit).unwrap();
    println!("Unit serializes as: {}", json_unit);

    // Verify deserialization works
    let _deserialized: TestEnum = serde_json::from_str(&json).unwrap();
    println!("✅ Deserialization works!");
}
