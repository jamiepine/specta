//! Reference type handling
//!
//! This module handles conversion of Specta reference types (type references)
//! to Swift type names with generic parameters.

use specta::TypeCollection;

use crate::error::{Error, Result};
use crate::swift::Swift;

/// Convert a Specta reference type to Swift.
///
/// A reference represents a named type that's defined elsewhere.
/// This function resolves the reference and generates the appropriate Swift type name,
/// including any generic type parameters.
///
/// # Arguments
///
/// * `swift` - Swift configuration for naming conventions
/// * `types` - Type collection to look up the referenced type
/// * `reference` - The reference type to convert
/// * `datatype_to_swift` - Function to recursively convert generic type parameters
///
/// # Returns
///
/// The Swift type name with generic parameters if applicable
///
/// # Examples
///
/// ```rust
/// // Reference to User → User
/// // Reference to Vec<String> → Vec (but handled elsewhere)
/// // Reference to ApiResponse<T, E> → ApiResponse<T, E>
/// ```
///
/// # Errors
///
/// Returns `Error::InvalidIdentifier` if the referenced type is not found
pub fn reference_to_swift<F>(
    swift: &Swift,
    types: &TypeCollection,
    reference: &specta::datatype::Reference,
    convert: F,
) -> Result<String>
where
    F: Fn(&specta::datatype::DataType) -> Result<String>,
{
    // Get the name from the TypeCollection using the SID
    let name = if let Some(ndt) = types.get(reference.sid()) {
        swift.naming.convert(ndt.name())
    } else {
        return Err(Error::InvalidIdentifier(
            "Reference to unknown type".to_string(),
        ));
    };

    if reference.generics().is_empty() {
        Ok(name)
    } else {
        let generics = reference
            .generics()
            .iter()
            .map(|(_, t)| convert(t))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .join(", ");
        Ok(format!("{}<{}>", name, generics))
    }
}

#[cfg(test)]
mod tests {
    // Integration tests verify reference resolution works correctly
}

