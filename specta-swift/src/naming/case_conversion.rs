//! Case conversion utilities
//!
//! This module provides functions for converting between different naming conventions:
//!
//! - `snake_case` (Rust convention)
//! - `camelCase` (Swift property/variable convention)
//! - `PascalCase` (Swift type convention)
//! - `kebab-case`
//! - `SCREAMING_SNAKE_CASE`
//! - And more...
//!
//! These utilities ensure consistent naming across generated Swift code.

/// Convert a snake_case string to camelCase.
///
/// # Arguments
///
/// * `s` - The snake_case string to convert
///
/// # Returns
///
/// The camelCase string
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::snake_to_camel;
/// assert_eq!(snake_to_camel("hello_world"), "helloWorld");
/// assert_eq!(snake_to_camel("user_id"), "userId");
/// assert_eq!(snake_to_camel("is_active"), "isActive");
/// ```
pub fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert a snake_case string to PascalCase.
///
/// # Arguments
///
/// * `s` - The snake_case string to convert
///
/// # Returns
///
/// The PascalCase string
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::snake_to_pascal;
/// assert_eq!(snake_to_pascal("hello_world"), "HelloWorld");
/// assert_eq!(snake_to_pascal("user_type"), "UserType");
/// assert_eq!(snake_to_pascal("api_response"), "ApiResponse");
/// ```
pub fn snake_to_pascal(s: &str) -> String {
    let camel = snake_to_camel(s);
    if camel.is_empty() {
        return camel;
    }

    let mut chars = camel.chars();
    let first = chars.next().unwrap().to_ascii_uppercase();
    format!("{}{}", first, chars.collect::<String>())
}

/// Convert a string to PascalCase, returning as-is if already PascalCase.
///
/// This is useful for enum variant names that might already be in the correct format.
///
/// # Arguments
///
/// * `s` - The string to convert
///
/// # Returns
///
/// The PascalCase string
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::to_pascal_case;
/// assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
/// assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld"); // Already PascalCase
/// assert_eq!(to_pascal_case("user_type"), "UserType");
/// ```
pub fn to_pascal_case(s: &str) -> String {
    // If it's already PascalCase (starts with uppercase), return as-is
    if s.chars().next().map_or(false, |c| c.is_uppercase()) {
        return s.to_string();
    }

    // Otherwise, convert snake_case to PascalCase
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
    }

    result
}

/// Convert a camelCase or PascalCase string to snake_case.
///
/// # Arguments
///
/// * `s` - The camelCase or PascalCase string to convert
///
/// # Returns
///
/// The snake_case string
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::to_snake_case;
/// assert_eq!(to_snake_case("helloWorld"), "hello_world");
/// assert_eq!(to_snake_case("UserType"), "user_type");
/// assert_eq!(to_snake_case("APIResponse"), "api_response");
/// ```
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_lowercase = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            // Add underscore before uppercase if:
            // 1. Not the first character
            // 2. Previous character was lowercase
            // 3. OR next character is lowercase (handles acronyms like "APIResponse" -> "api_response")
            if i > 0
                && (prev_was_lowercase
                    || s.chars()
                        .nth(i + 1)
                        .map(|c| c.is_lowercase())
                        .unwrap_or(false))
            {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_was_lowercase = false;
        } else {
            result.push(ch);
            prev_was_lowercase = ch.is_lowercase();
        }
    }

    result
}

/// Convert snake_case to kebab-case.
///
/// # Arguments
///
/// * `s` - The snake_case string to convert
///
/// # Returns
///
/// The kebab-case string
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::snake_to_kebab;
/// assert_eq!(snake_to_kebab("hello_world"), "hello-world");
/// assert_eq!(snake_to_kebab("user_type"), "user-type");
/// ```
pub fn snake_to_kebab(s: &str) -> String {
    s.replace('_', "-")
}

/// Convert snake_case to SCREAMING_SNAKE_CASE.
///
/// # Arguments
///
/// * `s` - The snake_case string to convert
///
/// # Returns
///
/// The SCREAMING_SNAKE_CASE string
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::snake_to_screaming_snake;
/// assert_eq!(snake_to_screaming_snake("hello_world"), "HELLO_WORLD");
/// assert_eq!(snake_to_screaming_snake("user_type"), "USER_TYPE");
/// ```
pub fn snake_to_screaming_snake(s: &str) -> String {
    s.to_ascii_uppercase()
}

/// Check if a string is in snake_case format.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string is in snake_case, `false` otherwise
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::is_snake_case;
/// assert!(is_snake_case("hello_world"));
/// assert!(is_snake_case("user_id"));
/// assert!(!is_snake_case("helloWorld"));
/// assert!(!is_snake_case("HelloWorld"));
/// ```
pub fn is_snake_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
}

/// Check if a string is in camelCase format.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string is in camelCase, `false` otherwise
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::is_camel_case;
/// assert!(is_camel_case("helloWorld"));
/// assert!(is_camel_case("userId"));
/// assert!(!is_camel_case("hello_world"));
/// assert!(!is_camel_case("HelloWorld"));
/// ```
pub fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_char = s.chars().next().unwrap();
    first_char.is_lowercase() && s.chars().any(|c| c.is_uppercase())
}

/// Check if a string is in PascalCase format.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string is in PascalCase, `false` otherwise
///
/// # Examples
///
/// ```rust
/// # use specta_swift::naming::case_conversion::is_pascal_case;
/// assert!(is_pascal_case("HelloWorld"));
/// assert!(is_pascal_case("UserType"));
/// assert!(!is_pascal_case("helloWorld"));
/// assert!(!is_pascal_case("hello_world"));
/// ```
pub fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_char = s.chars().next().unwrap();
    first_char.is_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel_basic() {
        assert_eq!(snake_to_camel("hello_world"), "helloWorld");
        assert_eq!(snake_to_camel("user_id"), "userId");
        assert_eq!(snake_to_camel("is_active"), "isActive");
    }

    #[test]
    fn test_snake_to_camel_single_word() {
        assert_eq!(snake_to_camel("hello"), "hello");
        assert_eq!(snake_to_camel("world"), "world");
    }

    #[test]
    fn test_snake_to_camel_multiple_underscores() {
        assert_eq!(snake_to_camel("hello_world_foo_bar"), "helloWorldFooBar");
    }

    #[test]
    fn test_snake_to_pascal_basic() {
        assert_eq!(snake_to_pascal("hello_world"), "HelloWorld");
        assert_eq!(snake_to_pascal("user_type"), "UserType");
        assert_eq!(snake_to_pascal("api_response"), "ApiResponse");
    }

    #[test]
    fn test_snake_to_pascal_single_word() {
        assert_eq!(snake_to_pascal("hello"), "Hello");
        assert_eq!(snake_to_pascal("world"), "World");
    }

    #[test]
    fn test_to_snake_case_camel() {
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("userId"), "user_id");
        assert_eq!(to_snake_case("isActive"), "is_active");
    }

    #[test]
    fn test_to_snake_case_pascal() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("UserType"), "user_type");
        assert_eq!(to_snake_case("ApiResponse"), "api_response");
    }

    #[test]
    fn test_to_snake_case_acronyms() {
        assert_eq!(to_snake_case("APIResponse"), "api_response");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("XMLParser"), "xml_parser");
    }

    #[test]
    fn test_to_snake_case_single_word() {
        assert_eq!(to_snake_case("hello"), "hello");
        assert_eq!(to_snake_case("World"), "world");
    }

    #[test]
    fn test_snake_to_kebab() {
        assert_eq!(snake_to_kebab("hello_world"), "hello-world");
        assert_eq!(snake_to_kebab("user_type"), "user-type");
        assert_eq!(snake_to_kebab("api_response"), "api-response");
    }

    #[test]
    fn test_snake_to_screaming_snake() {
        assert_eq!(snake_to_screaming_snake("hello_world"), "HELLO_WORLD");
        assert_eq!(snake_to_screaming_snake("user_type"), "USER_TYPE");
        assert_eq!(snake_to_screaming_snake("api"), "API");
    }

    #[test]
    fn test_is_snake_case() {
        assert!(is_snake_case("hello_world"));
        assert!(is_snake_case("user_id"));
        assert!(is_snake_case("is_active"));
        assert!(is_snake_case("hello"));

        assert!(!is_snake_case("helloWorld"));
        assert!(!is_snake_case("HelloWorld"));
        assert!(!is_snake_case("hello-world"));
        assert!(!is_snake_case(""));
    }

    #[test]
    fn test_is_camel_case() {
        assert!(is_camel_case("helloWorld"));
        assert!(is_camel_case("userId"));
        assert!(is_camel_case("isActive"));

        assert!(!is_camel_case("hello_world"));
        assert!(!is_camel_case("HelloWorld"));
        assert!(!is_camel_case("hello"));
        assert!(!is_camel_case(""));
    }

    #[test]
    fn test_is_pascal_case() {
        assert!(is_pascal_case("HelloWorld"));
        assert!(is_pascal_case("UserType"));
        assert!(is_pascal_case("ApiResponse"));
        assert!(is_pascal_case("H"));

        assert!(!is_pascal_case("helloWorld"));
        assert!(!is_pascal_case("hello_world"));
        assert!(!is_pascal_case("hello"));
        assert!(!is_pascal_case(""));
    }

    #[test]
    fn test_roundtrip_conversions() {
        let original = "hello_world_foo_bar";
        assert_eq!(to_snake_case(&snake_to_camel(original)), original);
        assert_eq!(to_snake_case(&snake_to_pascal(original)), original);
    }
}
