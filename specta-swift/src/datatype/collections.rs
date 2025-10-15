//! Collection type generation (List, Map, Tuple)
//!
//! This module handles Swift generation for Rust collection types:
//!
//! - `Vec<T>` → `[T]` (Swift arrays)
//! - `HashMap<K, V>` → `[K: V]` (Swift dictionaries)
//! - `(T, U, ...)` → `(T, U, ...)` (Swift tuples)

use specta::TypeCollection;

use crate::error::Result;
use crate::swift::Swift;

/// Convert a Rust `Vec<T>` to Swift array syntax `[T]`.
///
/// # Arguments
///
/// * `swift` - Swift configuration
/// * `types` - Type collection for resolving references
/// * `list` - The list type to convert
/// * `datatype_to_swift` - Function to recursively convert the element type
///
/// # Returns
///
/// The Swift array type string
///
/// # Examples
///
/// ```rust
/// // Vec<String> → [String]
/// // Vec<Vec<i32>> → [[Int32]]
/// // Vec<User> → [User]
/// ```
pub fn list_to_swift<F>(list: &specta::datatype::List, convert: F) -> Result<String>
where
    F: FnOnce(&specta::datatype::DataType) -> Result<String>,
{
    let element_type = convert(list.ty())?;
    Ok(format!("[{}]", element_type))
}

/// Convert a Rust `HashMap<K, V>` to Swift dictionary syntax `[K: V]`.
///
/// # Arguments
///
/// * `swift` - Swift configuration
/// * `types` - Type collection for resolving references
/// * `map` - The map type to convert
/// * `datatype_to_swift` - Function to recursively convert key and value types
///
/// # Returns
///
/// The Swift dictionary type string
///
/// # Examples
///
/// ```rust
/// // HashMap<String, i32> → [String: Int32]
/// // HashMap<String, Vec<User>> → [String: [User]]
/// ```
pub fn map_to_swift<F>(map: &specta::datatype::Map, convert: F) -> Result<String>
where
    F: Fn(&specta::datatype::DataType) -> Result<String>,
{
    let key_type = convert(map.key_ty())?;
    let value_type = convert(map.value_ty())?;
    Ok(format!("[{}: {}]", key_type, value_type))
}

/// Convert a Rust tuple to Swift tuple syntax.
///
/// # Arguments
///
/// * `swift` - Swift configuration
/// * `types` - Type collection for resolving references  
/// * `tuple` - The tuple type to convert
/// * `datatype_to_swift` - Function to recursively convert element types
///
/// # Returns
///
/// The Swift tuple type string, or "Void" for empty tuples
///
/// # Examples
///
/// ```rust
/// // () → Void
/// // (String,) → String (single element unwrapped)
/// // (String, i32) → (String, Int32)
/// // (String, i32, bool) → (String, Int32, Bool)
/// ```
pub fn tuple_to_swift<F>(tuple: &specta::datatype::Tuple, convert: F) -> Result<String>
where
    F: Fn(&specta::datatype::DataType) -> Result<String>,
{
    if tuple.elements().is_empty() {
        Ok("Void".to_string())
    } else if tuple.elements().len() == 1 {
        // Single element tuple unwraps to just the type
        convert(&tuple.elements()[0])
    } else {
        let types_str = tuple
            .elements()
            .iter()
            .map(|e| convert(e))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .join(", ");
        Ok(format!("({})", types_str))
    }
}

#[cfg(test)]
mod tests {
    // Integration tests verify these work correctly with actual types
    // Unit tests would require mocking the datatype_to_swift function
}
