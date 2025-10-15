//! Enum variant naming utilities
//!
//! This module provides utilities for generating struct names for enum variants
//! with named fields.

use crate::swift::{NamingConvention, StructNamingStrategy, Swift};

/// Generate a struct name for an enum variant with named fields.
///
/// When an enum variant has named fields, we generate a separate struct type.
/// This function determines the name of that struct based on the naming strategy.
///
/// # Naming Strategies
///
/// ## AutoRename (Default)
/// Prefixes the struct name with the enum name to avoid conflicts:
/// - `ApiResponse::Success` → `ApiResponseSuccessData`
/// - `Event::JobStarted` → `EventJobStartedData`
///
/// ## KeepOriginal
/// Uses just the variant name without enum prefix:
/// - `ApiResponse::Success` → `SuccessData`
/// - `Event::JobStarted` → `JobStartedData`
///
/// # Arguments
///
/// * `swift` - Swift configuration with naming strategy
/// * `enum_name` - The name of the parent enum
/// * `variant_name` - The name of the variant
///
/// # Returns
///
/// The generated struct name for the variant
///
/// # Examples
///
/// ```rust
/// # use specta_swift::Swift;
/// # use specta_swift::naming::variant_naming::generate_variant_struct_name;
/// let swift = Swift::default(); // Uses AutoRename
/// let name = generate_variant_struct_name(&swift, "ApiResponse", "Success");
/// assert_eq!(name, "ApiResponseSuccessData");
/// ```
pub fn generate_variant_struct_name(swift: &Swift, enum_name: &str, variant_name: &str) -> String {
    match swift.struct_naming {
        StructNamingStrategy::AutoRename => {
            format!("{}{}Data", enum_name, swift.naming.convert(variant_name))
        }
        StructNamingStrategy::KeepOriginal => {
            format!("{}Data", swift.naming.convert(variant_name))
        }
    }
}

/// Convert a variant name to PascalCase if needed.
///
/// This is a helper for enum variant struct naming that ensures the variant
/// name is in the correct case before appending it to the enum name.
///
/// # Arguments
///
/// * `variant_name` - The original variant name
/// * `naming` - The naming convention to apply
///
/// # Returns
///
/// The variant name in the appropriate case
pub fn convert_variant_name(variant_name: &str, naming: &NamingConvention) -> String {
    naming.convert(variant_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swift::StructNamingStrategy;

    #[test]
    fn test_auto_rename_strategy() {
        let swift = Swift::default(); // Uses AutoRename by default
        let name = generate_variant_struct_name(&swift, "ApiResponse", "Success");
        assert_eq!(name, "ApiResponseSuccessData");
    }

    #[test]
    fn test_auto_rename_with_snake_case() {
        let swift = Swift::default();
        let name = generate_variant_struct_name(&swift, "JobOutput", "file_copy");
        assert_eq!(name, "JobOutputFileCopyData");
    }

    #[test]
    fn test_keep_original_strategy() {
        let swift = Swift::new().struct_naming(StructNamingStrategy::KeepOriginal);
        let name = generate_variant_struct_name(&swift, "ApiResponse", "Success");
        assert_eq!(name, "SuccessData");
    }

    #[test]
    fn test_keep_original_with_snake_case() {
        let swift = Swift::new().struct_naming(StructNamingStrategy::KeepOriginal);
        let name = generate_variant_struct_name(&swift, "JobOutput", "file_copy");
        assert_eq!(name, "FileCopyData");
    }
}
