//! Adjacently tagged enum Codable implementation
//!
//! This module generates Codable implementations for adjacently tagged enums.
//! These enums use separate fields for the tag (variant name) and content (variant data).
//!
//! # Format
//!
//! Adjacently tagged enums serialize as:
//! ```json
//! {
//!   "type": "Success",
//!   "data": { "value": 42, "message": "OK" }
//! }
//! ```
//!
//! Instead of the default externally tagged format:
//! ```json
//! {
//!   "Success": { "value": 42, "message": "OK" }
//! }
//! ```

use specta::datatype::{Enum, EnumRepr, Fields};

use crate::error::{Error, Result};
use crate::swift::Swift;

/// Generate custom Codable implementation for adjacently tagged enums.
///
/// Adjacently tagged enums use `#[serde(tag = "type", content = "data")]` in Rust
/// and require custom Codable implementation in Swift.
///
/// # Arguments
///
/// * `swift` - Swift configuration
/// * `e` - The enum to generate Codable for
/// * `enum_name` - The Swift enum name
/// * `generate_variant_struct_name` - Function to generate struct names for variants
///
/// # Returns
///
/// The complete Codable extension code
///
/// # Errors
///
/// Returns error if the enum is not adjacently tagged
pub fn generate_adjacently_tagged_codable<F>(
    swift: &Swift,
    e: &Enum,
    enum_name: &str,
    generate_variant_struct_name: F,
) -> Result<String>
where
    F: Fn(&str) -> String,
{
    let mut result = String::new();

    // Get tag and content field names
    let (tag_field, content_field) = if let Some(EnumRepr::Adjacent { tag, content }) = e.repr() {
        (tag.as_ref(), content.as_ref())
    } else {
        return Err(Error::UnsupportedType(
            "Expected adjacently tagged enum".to_string(),
        ));
    };

    result.push_str(&format!(
        "\n// MARK: - {} Adjacently Tagged Codable Implementation\n",
        enum_name
    ));
    result.push_str(&format!("extension {}: Codable {{\n", enum_name));

    // Generate TypeKeys enum for the tag and content fields - make name unique per enum
    result.push_str(&format!(
        "    private enum {}TypeKeys: String, CodingKey {{\n",
        enum_name
    ));
    result.push_str(&format!("        case tag = \"{}\"\n", tag_field));
    result.push_str(&format!("        case content = \"{}\"\n", content_field));
    result.push_str("    }\n\n");

    // Generate VariantType enum for variant names
    result.push_str("    private enum VariantType: String, Codable {\n");
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
    result.push_str(&format!(
        "        let container = try decoder.container(keyedBy: {}TypeKeys.self)\n",
        enum_name
    ));
    result.push_str(
        "        let variantType = try container.decode(VariantType.self, forKey: .tag)\n",
    );
    result.push_str("        \n");
    result.push_str("        switch variantType {\n");

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            Fields::Unit => {
                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!("            self = .{}\n", swift_case_name));
            }
            Fields::Unnamed(fields) => {
                if fields.fields().is_empty() {
                    // Empty tuple variant - treat as unit variant
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str(&format!("            self = .{}\n", swift_case_name));
                } else {
                    // TODO: Handle non-empty tuple variants for adjacently tagged
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str("            fatalError(\"Adjacently tagged tuple variants not implemented\")\n");
                }
            }
            Fields::Named(_) => {
                let struct_name = generate_variant_struct_name(original_variant_name);

                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "            let data = try container.decode({}.self, forKey: .content)\n",
                    struct_name
                ));
                result.push_str(&format!("            self = .{}(data)\n", swift_case_name));
            }
        }
    }

    result.push_str("        }\n");
    result.push_str("    }\n\n");

    // Generate encode(to encoder:)
    result.push_str("    public func encode(to encoder: Encoder) throws {\n");
    result.push_str(&format!(
        "        var container = encoder.container(keyedBy: {}TypeKeys.self)\n",
        enum_name
    ));
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
                    "            try container.encode(VariantType.{}, forKey: .tag)\n",
                    swift_case_name
                ));
            }
            Fields::Unnamed(fields) => {
                if fields.fields().is_empty() {
                    // Empty tuple variant - treat as unit variant
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str(&format!(
                        "            try container.encode(VariantType.{}, forKey: .tag)\n",
                        swift_case_name
                    ));
                } else {
                    // TODO: Handle non-empty tuple variants
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str("            fatalError(\"Adjacently tagged tuple variants not implemented\")\n");
                }
            }
            Fields::Named(_) => {
                result.push_str(&format!("        case .{}(let data):\n", swift_case_name));
                result.push_str(&format!(
                    "            try container.encode(VariantType.{}, forKey: .tag)\n",
                    swift_case_name
                ));
                result.push_str("            try container.encode(data, forKey: .content)\n");
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
    // Integration tests verify adjacently tagged enum generation
}
