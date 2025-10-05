//! serde_json type handling
//!
//! This module provides detection for serde_json-specific types that need
//! special Swift representations.

use specta::datatype::{DataType, Enum, EnumRepr, Fields};

/// Check if an enum is the serde_json::Number enum.
///
/// The serde_json::Number enum has a specific pattern:
/// - Untagged representation
/// - Exactly 3 variants: `f64(f64)`, `i64(i64)`, `u64(u64)`
/// - Each variant has a single primitive field of the corresponding type
///
/// In Swift, this maps to `Double` for simplicity.
///
/// # Arguments
///
/// * `e` - The enum to check
///
/// # Returns
///
/// `true` if this is serde_json::Number, `false` otherwise
///
/// # Examples
///
/// ```rust
/// # use specta_swift::special_types::serde_json::is_serde_json_number_enum;
/// // Would detect serde_json::Number enum pattern
/// ```
pub fn is_serde_json_number_enum(e: &Enum) -> bool {
    // Check for untagged representation
    if let Some(repr) = e.repr() {
        if !matches!(repr, EnumRepr::Untagged) {
            return false;
        }
    } else {
        return false;
    }

    let variants: Vec<_> = e.variants().iter().collect();
    if variants.len() != 3 {
        return false;
    }

    let variant_names: Vec<&str> = variants.iter().map(|(name, _)| name.as_ref()).collect();
    if !(variant_names.contains(&"f64")
        && variant_names.contains(&"i64")
        && variant_names.contains(&"u64"))
    {
        return false;
    }

    // Check that each variant has the expected primitive type
    for (name, variant) in variants {
        if let Fields::Unnamed(fields) = variant.fields() {
            if fields.fields().len() != 1 {
                return false;
            }

            if let Some(field) = fields.fields().first() {
                if let Some(DataType::Primitive(p)) = field.ty() {
                    let expected_primitive = match name.as_ref() {
                        "f64" => specta::datatype::Primitive::f64,
                        "i64" => specta::datatype::Primitive::i64,
                        "u64" => specta::datatype::Primitive::u64,
                        _ => continue,
                    };
                    if *p != expected_primitive {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    // Tests would be added here when we have actual serde_json types
    // For now, the function is verified through integration tests
}
