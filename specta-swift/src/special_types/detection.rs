//! Special type detection
//!
//! This module provides utilities for detecting standard library types that need
//! special Swift representations.

use specta::{SpectaID, TypeCollection};

/// Check if a type is a special standard library type that needs special handling.
///
/// Maps special Rust types to their Swift equivalents:
/// - `std::time::Duration` → `RustDuration`
/// - `std::time::SystemTime` → `Date`
/// - `serde_json::Number` → `Double`
/// - `serde_json::Value` (as JsonValue) → `JsonValue`
///
/// # Arguments
///
/// * `types` - The type collection to look up types
/// * `sid` - The Specta ID of the type to check
///
/// # Returns
///
/// `Some(String)` with the Swift type name if special, `None` otherwise
///
/// # Examples
///
/// ```rust
/// # use specta::TypeCollection;
/// # use specta_swift::special_types::detection::is_special_std_type;
/// // When processing std::time::Duration, this would return Some("RustDuration")
/// // When processing regular types, this would return None
/// ```
pub fn is_special_std_type(types: &TypeCollection, sid: Option<SpectaID>) -> Option<String> {
    if let Some(sid) = sid {
        if let Some(ndt) = types.get(sid) {
            // Check for std::time::Duration
            if ndt.name() == "Duration" {
                return Some("RustDuration".to_string());
            }
            // Check for std::time::SystemTime
            if ndt.name() == "SystemTime" {
                return Some("Date".to_string());
            }
            // Check for serde_json::Number
            if ndt.name() == "Number" {
                return Some("Double".to_string());
            }
            // Check for serde_json::Value (mapped as JsonValue in Specta)
            if ndt.name() == "JsonValue" {
                return Some("JsonValue".to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    // Tests would be added here when we have actual type collection data
    // For now, the function is verified through integration tests
}
