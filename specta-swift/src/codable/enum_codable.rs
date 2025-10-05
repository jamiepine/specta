//! Enum Codable implementation
//!
//! This module generates custom Codable implementations for enums with struct-like variants.
//! Standard Swift enums don't support associated values with Codable out of the box,
//! so we generate custom encoding/decoding logic.
//!
//! # Format
//!
//! Externally tagged enums (default serde format) serialize as:
//! ```json
//! {
//!   "Success": { "data": "hello", "status": 200 }
//! }
//! ```
//!
//! With a fallback for unit variants as strings:
//! ```json
//! "Loading"
//! ```

use specta::datatype::{Enum, Fields};

use crate::error::Result;
use crate::swift::Swift;

/// Generate custom Codable implementation for enums with struct-like variants.
///
/// This generates a complete Codable extension for enums that have variants with
/// associated data (struct-like or tuple-like variants).
///
/// # Arguments
///
/// * `swift` - Swift configuration
/// * `e` - The enum to generate Codable for
/// * `enum_name` - The Swift enum name
/// * `generate_variant_struct_name` - Function to generate struct names for named field variants
///
/// # Returns
///
/// The complete Codable extension code including:
/// - CodingKeys enum
/// - init(from decoder:) with externally-tagged and string fallback support
/// - encode(to encoder:) method
///
/// # Note
///
/// This handles externally tagged enums. For adjacently tagged enums,
/// use `generate_adjacently_tagged_codable` instead.
pub fn generate_enum_codable_impl<F, G>(
    swift: &Swift,
    e: &Enum,
    enum_name: &str,
    generate_variant_struct_name: F,
    get_field_type: G,
) -> Result<String>
where
    F: Fn(&str) -> String,
    G: Fn(&specta::datatype::DataType) -> Result<String>,
{
    let mut result = String::new();

    result.push_str(&format!(
        "\n// MARK: - {} Codable Implementation\n",
        enum_name
    ));
    result.push_str(&format!("extension {}: Codable {{\n", enum_name));

    // Generate CodingKeys enum
    result.push_str("    private enum CodingKeys: String, CodingKey {\n");
    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }
        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);
        result.push_str(&format!(
            "        case {} = \"{}\"\n",
            swift_case_name, original_variant_name
        ));
    }
    result.push_str("    }\n\n");

    // Generate init(from decoder:)
    result.push_str("    public init(from decoder: Decoder) throws {\n");
    result.push_str(
        "        // Try externally-tagged format first (e.g., {\"WaitingForConnection\": null})\n",
    );
    result.push_str(
        "        if let container = try? decoder.container(keyedBy: CodingKeys.self) {\n",
    );
    result.push_str("            if container.allKeys.count == 1 {\n");
    result.push_str("                let key = container.allKeys.first!\n");
    result.push_str("                switch key {\n");

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            Fields::Unit => {
                result.push_str(&format!("                case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "                    self = .{}\n",
                    swift_case_name
                ));
                result.push_str("                    return\n");
            }
            Fields::Unnamed(fields) => {
                if fields.fields().is_empty() {
                    result.push_str(&format!("                case .{}:\n", swift_case_name));
                    result.push_str(&format!(
                        "                    self = .{}\n",
                        swift_case_name
                    ));
                    result.push_str("                    return\n");
                } else {
                    // For tuple variants, decode as array: {"Variant": [value1, value2, ...]}
                    result.push_str(&format!("                case .{}:\n", swift_case_name));

                    // Get the Swift types for the tuple elements
                    let tuple_types: Vec<String> = fields
                        .fields()
                        .iter()
                        .filter_map(|f| f.ty())
                        .map(|ty| get_field_type(ty))
                        .collect::<std::result::Result<Vec<_>, _>>()?;

                    // Decode as array and destructure
                    result.push_str(&format!(
                        "                    var arrayContainer = try container.nestedUnkeyedContainer(forKey: .{})\n",
                        swift_case_name
                    ));

                    // Decode each element
                    for (i, type_str) in tuple_types.iter().enumerate() {
                        result.push_str(&format!(
                            "                    let value{} = try arrayContainer.decode({}.self)\n",
                            i, type_str
                        ));
                    }

                    // Construct the enum case with all values
                    let value_list = (0..tuple_types.len())
                        .map(|i| format!("value{}", i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    result.push_str(&format!(
                        "                    self = .{}({})\n",
                        swift_case_name, value_list
                    ));
                    result.push_str("                    return\n");
                }
            }
            Fields::Named(_) => {
                let struct_name = generate_variant_struct_name(original_variant_name);

                result.push_str(&format!("                case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "                    let data = try container.decode({}.self, forKey: .{})\n",
                    struct_name, swift_case_name
                ));
                result.push_str(&format!(
                    "                    self = .{}(data)\n",
                    swift_case_name
                ));
                result.push_str("                    return\n");
            }
        }
    }

    result.push_str("                }\n");
    result.push_str("                return\n");
    result.push_str("            }\n");
    result.push_str("        }\n");
    result.push_str("        \n");
    result.push_str(
        "        // Fallback: try decoding as plain string for unit variants (serde default)\n",
    );
    result.push_str("        if let stringContainer = try? decoder.singleValueContainer() {\n");
    result.push_str(
        "            if let variantString = try? stringContainer.decode(String.self) {\n",
    );
    result.push_str("                switch variantString {\n");

    // Generate string fallback cases for unit variants only
    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            Fields::Unit => {
                result.push_str(&format!(
                    "                case \"{}\":\n",
                    original_variant_name
                ));
                result.push_str(&format!(
                    "                    self = .{}\n",
                    swift_case_name
                ));
                result.push_str("                    return\n");
            }
            Fields::Unnamed(fields) if fields.fields().is_empty() => {
                result.push_str(&format!(
                    "                case \"{}\":\n",
                    original_variant_name
                ));
                result.push_str(&format!(
                    "                    self = .{}\n",
                    swift_case_name
                ));
                result.push_str("                    return\n");
            }
            _ => {
                // Struct/tuple variants can't be decoded from strings
            }
        }
    }

    result.push_str("                default:\n");
    result.push_str("                    break\n");
    result.push_str("                }\n");
    result.push_str("            }\n");
    result.push_str("        }\n");
    result.push_str("        \n");
    result.push_str("        throw DecodingError.dataCorrupted(\n");
    result.push_str("            DecodingError.Context(codingPath: decoder.codingPath, debugDescription: \"Could not decode enum - expected externally-tagged object or string for unit variants\")\n");
    result.push_str("        )\n");
    result.push_str("    }\n\n");

    // Generate encode(to encoder:)
    result.push_str("    public func encode(to encoder: Encoder) throws {\n");
    result.push_str("        var container = encoder.container(keyedBy: CodingKeys.self)\n");
    result.push_str("        \n");
    result.push_str("        switch self {\n");

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            Fields::Unit => {
                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "            try container.encodeNil(forKey: .{})\n",
                    swift_case_name
                ));
            }
            Fields::Unnamed(fields) => {
                if fields.fields().is_empty() {
                    // Empty tuple - encode as nil
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str(&format!(
                        "            try container.encodeNil(forKey: .{})\n",
                        swift_case_name
                    ));
                } else {
                    // For tuple variants, encode as array
                    let tuple_count = fields.fields().len();

                    // Generate pattern match with variable bindings
                    let bindings = (0..tuple_count)
                        .map(|i| format!("let value{}", i))
                        .collect::<Vec<_>>()
                        .join(", ");

                    result.push_str(&format!(
                        "        case .{}({}):\n",
                        swift_case_name, bindings
                    ));
                    result.push_str(&format!(
                        "            var arrayContainer = container.nestedUnkeyedContainer(forKey: .{})\n",
                        swift_case_name
                    ));

                    // Encode each value
                    for i in 0..tuple_count {
                        result.push_str(&format!(
                            "            try arrayContainer.encode(value{})\n",
                            i
                        ));
                    }
                }
            }
            Fields::Named(_) => {
                result.push_str(&format!("        case .{}(let data):\n", swift_case_name));
                result.push_str(&format!(
                    "            try container.encode(data, forKey: .{})\n",
                    swift_case_name
                ));
            }
        }
    }

    result.push_str("        }\n");
    result.push_str("    }\n");
    result.push_str("}\n");

    Ok(result)
}

#[cfg(test)]
mod tests {
    // Integration tests verify enum Codable generation
}
