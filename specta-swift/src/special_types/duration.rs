//! Duration type handling
//!
//! This module handles detection and generation of Rust `std::time::Duration` types,
//! which are represented in Swift as a custom `RustDuration` struct.

use specta::datatype::Struct;

/// Check if a struct is a Rust Duration by examining its fields.
///
/// Rust's `Duration` struct has exactly two fields:
/// - `secs: u64` - Whole seconds
/// - `nanos: u32` - Nanosecond component
///
/// # Arguments
///
/// * `s` - The struct to check
///
/// # Returns
///
/// `true` if this is a Duration struct, `false` otherwise
///
/// # Examples
///
/// ```rust
/// # use specta::{Type, TypeCollection};
/// # use specta_swift::special_types::duration::is_duration_struct;
/// # use std::time::Duration;
/// # #[derive(Type)]
/// # struct TestDuration {
/// #     secs: u64,
/// #     nanos: u32,
/// # }
/// // This would return true for std::time::Duration when introspected
/// ```
pub fn is_duration_struct(s: &Struct) -> bool {
    match s.fields() {
        specta::datatype::Fields::Named(fields) => {
            let field_names: Vec<String> = fields
                .fields()
                .iter()
                .map(|(name, _)| name.to_string())
                .collect();
            // Duration has exactly two fields: "secs" (u64) and "nanos" (u32)
            field_names.len() == 2
                && field_names.contains(&"secs".to_string())
                && field_names.contains(&"nanos".to_string())
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    // Tests would be added here when we have actual Duration type data
    // For now, the function is simple enough to be verified through integration tests
}
