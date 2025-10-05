//! Struct generation for enum variants
//!
//! This module handles generation of struct types that are created for enum variants
//! with named fields. When an enum has variants like:
//!
//! ```rust
//! enum ApiResponse {
//!     Success { data: String, status: u16 },
//!     Error { message: String, code: u32 },
//! }
//! ```
//!
//! We generate separate struct types:
//! ```swift
//! public struct ApiResponseSuccessData: Codable {
//!     public let data: String
//!     public let status: UInt16
//! }
//!
//! public struct ApiResponseErrorData: Codable {
//!     public let message: String
//!     public let code: UInt32
//! }
//! ```

use specta::datatype::{Enum, Fields};
use specta::TypeCollection;

use crate::error::Result;
use crate::swift::Swift;

/// Generate struct definitions for enum variants with named fields.
///
/// This function creates standalone struct types for each enum variant that has
/// named fields (struct-like variants). These structs are then referenced by the
/// enum cases.
///
/// # Arguments
///
/// * `swift` - Swift configuration
/// * `types` - Type collection for resolving type references
/// * `e` - The enum containing the variants
/// * `enum_name` - The name of the parent enum
/// * `generate_variant_struct_name` - Function to generate struct names
/// * `datatype_to_swift` - Function to convert field types
///
/// # Returns
///
/// Swift struct definitions for all variants with named fields
///
/// # Examples
///
/// For an enum with named field variants, this generates the struct types
/// that the enum cases will reference.
pub fn generate_enum_variant_structs<F, G>(
    swift: &Swift,
    types: &TypeCollection,
    e: &Enum,
    enum_name: &str,
    generate_variant_struct_name: F,
    datatype_to_swift: G,
) -> Result<String>
where
    F: Fn(&str) -> String,
    G: Fn(&specta::datatype::DataType) -> Result<String>,
{
    let mut result = String::new();

    for (variant_name, variant) in e.variants() {
        if let Fields::Named(fields) = variant.fields() {
            if !fields.fields().is_empty() {
                let struct_name = generate_variant_struct_name(variant_name);

                result.push_str(&format!("public struct {}: Codable {{\n", struct_name));

                let mut field_mappings = Vec::new();

                for (field_name, field) in fields.fields() {
                    let swift_field_name = swift.naming.convert_field(field_name);
                    if let Some(ty) = field.ty() {
                        let field_type = datatype_to_swift(ty)?;
                        result.push_str(&format!(
                            "    public let {}: {}\n",
                            swift_field_name, field_type
                        ));
                        field_mappings.push((swift_field_name, field_name.to_string()));
                    }
                }

                // Generate custom CodingKeys if field names were converted
                let needs_custom_coding_keys = field_mappings
                    .iter()
                    .any(|(swift_name, rust_name)| swift_name != rust_name);
                if needs_custom_coding_keys {
                    result.push_str("\n    private enum CodingKeys: String, CodingKey {\n");
                    for (swift_name, rust_name) in &field_mappings {
                        result.push_str(&format!(
                            "        case {} = \"{}\"\n",
                            swift_name, rust_name
                        ));
                    }
                    result.push_str("    }\n");
                }

                result.push_str("}\n\n");
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    // Integration tests verify struct generation for enum variants
}
