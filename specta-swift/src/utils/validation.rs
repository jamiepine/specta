//! Type validation utilities
//!
//! This module provides utilities for validating types and detecting patterns:
//!
//! - Recursive type detection
//! - Circular reference detection
//! - Type safety checks

use specta::{datatype::DataType, SpectaID};

/// Check if a DataType references the given SpectaID (for detecting recursive types).
///
/// This function recursively traverses a type to detect if it contains a reference
/// to a specific type ID, which is used to detect recursive/self-referential types.
///
/// Recursive types in Swift require the `indirect` keyword:
/// ```swift
/// public indirect enum Tree {
///     case leaf(Int)
///     case branch(left: Tree, right: Tree)
/// }
/// ```
///
/// # Arguments
///
/// * `ty` - The data type to check
/// * `target_sid` - The Specta ID to look for
///
/// # Returns
///
/// `true` if the type contains a reference to `target_sid`, `false` otherwise
///
/// # Examples
///
/// ```rust
/// # use specta::{Type, SpectaID};
/// # use specta_swift::utils::validation::is_recursive_type_reference;
/// // This would be used to detect self-referential types like:
/// // enum Tree { Leaf(i32), Branch(Box<Tree>, Box<Tree>) }
/// ```
pub fn is_recursive_type_reference(ty: &DataType, target_sid: SpectaID) -> bool {
    match ty {
        DataType::Reference(reference) => reference.sid() == target_sid,
        DataType::Nullable(inner) => is_recursive_type_reference(inner, target_sid),
        DataType::List(list) => is_recursive_type_reference(list.ty(), target_sid),
        DataType::Map(map) => {
            is_recursive_type_reference(map.key_ty(), target_sid)
                || is_recursive_type_reference(map.value_ty(), target_sid)
        }
        DataType::Struct(s) => match s.fields() {
            specta::datatype::Fields::Named(fields) => fields.fields().iter().any(|(_, field)| {
                if let Some(ty) = field.ty() {
                    is_recursive_type_reference(ty, target_sid)
                } else {
                    false
                }
            }),
            specta::datatype::Fields::Unnamed(fields) => fields.fields().iter().any(|field| {
                if let Some(ty) = field.ty() {
                    is_recursive_type_reference(ty, target_sid)
                } else {
                    false
                }
            }),
            _ => false,
        },
        DataType::Enum(e) => e
            .variants()
            .iter()
            .any(|(_, variant)| match variant.fields() {
                specta::datatype::Fields::Named(fields) => {
                    fields.fields().iter().any(|(_, field)| {
                        if let Some(ty) = field.ty() {
                            is_recursive_type_reference(ty, target_sid)
                        } else {
                            false
                        }
                    })
                }
                specta::datatype::Fields::Unnamed(fields) => fields.fields().iter().any(|field| {
                    if let Some(ty) = field.ty() {
                        is_recursive_type_reference(ty, target_sid)
                    } else {
                        false
                    }
                }),
                _ => false,
            }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would verify recursive detection with actual type data
    #[test]
    fn test_detects_direct_reference() {
        // Would test direct circular references
    }

    #[test]
    fn test_detects_nested_reference() {
        // Would test references nested in lists, maps, etc.
    }

    #[test]
    fn test_no_false_positives() {
        // Would test that non-recursive types return false
    }
}
