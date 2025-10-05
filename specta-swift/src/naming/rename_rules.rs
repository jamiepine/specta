//! Rename rules for serde attributes
//!
//! This module handles Serde's `rename_all` attribute conversion rules.
//! These rules control how variant/field names are serialized.

/// Generate raw value for string enum variants based on serde rename rules.
///
/// Handles all serde `rename_all` options:
/// - `lowercase` - all lowercase
/// - `UPPERCASE` - all uppercase  
/// - `PascalCase` - capitalize first letter
/// - `camelCase` - lowercase first letter
/// - `snake_case` - convert to snake_case
/// - `SCREAMING_SNAKE_CASE` - uppercase snake_case
/// - `kebab-case` - use dashes instead of underscores
/// - `SCREAMING-KEBAB-CASE` - uppercase kebab-case
///
/// # Arguments
///
/// * `variant_name` - The original variant name
/// * `rename_all` - The serde rename_all attribute value
///
/// # Returns
///
/// The renamed string according to the rule
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::rename_rules::generate_raw_value;
/// assert_eq!(generate_raw_value("MyVariant", Some("lowercase")), "myvariant");
/// assert_eq!(generate_raw_value("MyVariant", Some("camelCase")), "myVariant");
/// assert_eq!(generate_raw_value("MyVariant", Some("snake_case")), "my_variant");
/// assert_eq!(generate_raw_value("MyVariant", None), "myvariant");
/// ```
pub fn generate_raw_value(variant_name: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("lowercase") => variant_name.to_lowercase(),
        Some("UPPERCASE") => variant_name.to_uppercase(),
        Some("camelCase") => {
            let mut chars = variant_name.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_lowercase().chain(chars).collect(),
            }
        }
        Some("PascalCase") => {
            let mut chars = variant_name.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        }
        Some("snake_case") => {
            // Use the to_snake_case from case_conversion for proper handling
            crate::naming::case_conversion::to_snake_case(variant_name)
        }
        Some("SCREAMING_SNAKE_CASE") => variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    vec!['_', c.to_uppercase().next().unwrap()]
                } else {
                    vec![c.to_uppercase().next().unwrap()]
                }
            })
            .collect(),
        Some("kebab-case") => variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    vec!['-', c.to_lowercase().next().unwrap()]
                } else {
                    vec![c.to_lowercase().next().unwrap()]
                }
            })
            .collect(),
        Some("SCREAMING-KEBAB-CASE") => variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    vec!['-', c.to_uppercase().next().unwrap()]
                } else {
                    vec![c.to_uppercase().next().unwrap()]
                }
            })
            .collect(),
        _ => variant_name.to_lowercase(), // Default to lowercase
    }
}

/// Generate raw value for string enum variants using NamingConvention.
///
/// This is a simpler version used when we have a NamingConvention instead of
/// the full serde rename_all attribute.
///
/// # Arguments
///
/// * `variant_name` - The original variant name
/// * `naming` - The naming convention to apply
///
/// # Returns
///
/// The renamed string
pub fn generate_string_enum_raw_value(
    variant_name: &str,
    naming: crate::swift::NamingConvention,
) -> String {
    match naming {
        crate::swift::NamingConvention::SnakeCase => variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    vec!['_', c]
                } else {
                    vec![c]
                }
            })
            .collect::<String>()
            .to_lowercase(),
        _ => variant_name.to_lowercase(), // Default to lowercase
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowercase() {
        assert_eq!(generate_raw_value("MyVariant", Some("lowercase")), "myvariant");
    }

    #[test]
    fn test_uppercase() {
        assert_eq!(generate_raw_value("MyVariant", Some("UPPERCASE")), "MYVARIANT");
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(generate_raw_value("MyVariant", Some("camelCase")), "myVariant");
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(generate_raw_value("MyVariant", Some("PascalCase")), "MyVariant");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(generate_raw_value("MyVariant", Some("snake_case")), "my_variant");
        assert_eq!(generate_raw_value("APIResponse", Some("snake_case")), "api_response");
    }

    #[test]
    fn test_screaming_snake_case() {
        assert_eq!(
            generate_raw_value("MyVariant", Some("SCREAMING_SNAKE_CASE")),
            "MY_VARIANT"
        );
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(generate_raw_value("MyVariant", Some("kebab-case")), "my-variant");
    }

    #[test]
    fn test_screaming_kebab_case() {
        assert_eq!(
            generate_raw_value("MyVariant", Some("SCREAMING-KEBAB-CASE")),
            "MY-VARIANT"
        );
    }

    #[test]
    fn test_default_none() {
        assert_eq!(generate_raw_value("MyVariant", None), "myvariant");
    }
}

