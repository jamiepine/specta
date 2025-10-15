//! Generic type parameter handling
//!
//! This module handles conversion of generic type parameters (T, U, etc.)
//! to Swift syntax.

use crate::error::Result;

/// Convert a generic type parameter to Swift.
///
/// Generic type parameters (T, U, V, etc.) are typically passed through as-is
/// since Swift uses the same syntax for generics.
///
/// # Arguments
///
/// * `generic` - The generic type parameter to convert
///
/// # Returns
///
/// The Swift generic parameter name
///
/// # Examples
///
/// ```rust
/// # use specta::datatype::Generic;
/// # use specta_swift::datatype::generic::generic_to_swift;
/// // T → T
/// // U → U
/// // Data → Data
/// ```
pub fn generic_to_swift(generic: &specta::datatype::Generic) -> Result<String> {
    Ok(generic.to_string())
}

#[cfg(test)]
mod tests {
    // Simple pass-through function, verified through integration tests
}
