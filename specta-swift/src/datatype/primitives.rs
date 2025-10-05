//! Primitive type mapping from Rust to Swift
//!
//! This module provides the core functionality for converting Rust primitive types
//! to their Swift equivalents. It handles:
//!
//! - Numeric types (integers, floats)
//! - Boolean and character types
//! - String types
//! - Literal value conversion
//! - Unsupported type detection
//!
//! # Type Mappings
//!
//! | Rust Type | Swift Type | Notes |
//! |-----------|------------|-------|
//! | `i8` | `Int8` | |
//! | `i16` | `Int16` | |
//! | `i32` | `Int32` | |
//! | `i64` | `Int64` | |
//! | `isize` | `Int` | Platform-dependent |
//! | `u8` | `UInt8` | |
//! | `u16` | `UInt16` | |
//! | `u32` | `UInt32` | |
//! | `u64` | `UInt64` | |
//! | `usize` | `UInt` | Platform-dependent |
//! | `f32` | `Float` | |
//! | `f64` | `Double` | |
//! | `bool` | `Bool` | |
//! | `char` | `Character` | |
//! | `String` | `String` | |
//! | `i128` | ❌ | Not supported in Swift |
//! | `u128` | ❌ | Not supported in Swift |
//! | `f16` | ❌ | Not supported in Swift |

use specta::datatype::{Literal, Primitive};

use crate::error::{Error, Result};

/// Convert a Specta primitive type to its Swift equivalent.
///
/// # Arguments
///
/// * `primitive` - The Specta primitive type to convert
///
/// # Returns
///
/// * `Ok(String)` - The Swift type name
/// * `Err(Error::UnsupportedType)` - If the type is not supported in Swift
///
/// # Examples
///
/// ```rust
/// # use specta::datatype::Primitive;
/// # use specta_swift::datatype::primitives::primitive_to_swift;
/// assert_eq!(primitive_to_swift(&Primitive::i32).unwrap(), "Int32");
/// assert_eq!(primitive_to_swift(&Primitive::f64).unwrap(), "Double");
/// assert_eq!(primitive_to_swift(&Primitive::bool).unwrap(), "Bool");
/// assert_eq!(primitive_to_swift(&Primitive::String).unwrap(), "String");
/// ```
///
/// # Errors
///
/// Returns an error for unsupported types:
/// - `i128` / `u128`: Swift doesn't support 128-bit integers
/// - `f16`: Swift doesn't support 16-bit floats
pub fn primitive_to_swift(primitive: &Primitive) -> Result<String> {
    Ok(match primitive {
        // Signed integers
        Primitive::i8 => "Int8".to_string(),
        Primitive::i16 => "Int16".to_string(),
        Primitive::i32 => "Int32".to_string(),
        Primitive::i64 => "Int64".to_string(),
        Primitive::isize => "Int".to_string(),

        // Unsigned integers
        Primitive::u8 => "UInt8".to_string(),
        Primitive::u16 => "UInt16".to_string(),
        Primitive::u32 => "UInt32".to_string(),
        Primitive::u64 => "UInt64".to_string(),
        Primitive::usize => "UInt".to_string(),

        // Floating point
        Primitive::f32 => "Float".to_string(),
        Primitive::f64 => "Double".to_string(),

        // Other primitives
        Primitive::bool => "Bool".to_string(),
        Primitive::char => "Character".to_string(),
        Primitive::String => "String".to_string(),

        // Unsupported types
        Primitive::i128 | Primitive::u128 => {
            return Err(Error::UnsupportedType(
                "Swift does not support 128-bit integers".to_string(),
            ));
        }
        Primitive::f16 => {
            return Err(Error::UnsupportedType(
                "Swift does not support f16 (16-bit float)".to_string(),
            ));
        }
    })
}

/// Convert a literal value to Swift syntax.
///
/// # Arguments
///
/// * `literal` - The literal value to convert
///
/// # Returns
///
/// * `Ok(String)` - The Swift literal syntax
/// * `Err(Error::UnsupportedType)` - If the literal type is not supported
///
/// # Examples
///
/// ```rust
/// # use specta::datatype::Literal;
/// # use specta_swift::datatype::primitives::literal_to_swift;
/// assert_eq!(literal_to_swift(&Literal::i32(42)).unwrap(), "42");
/// assert_eq!(literal_to_swift(&Literal::bool(true)).unwrap(), "true");
/// assert_eq!(literal_to_swift(&Literal::String("hello".into())).unwrap(), "\"hello\"");
/// assert_eq!(literal_to_swift(&Literal::None).unwrap(), "nil");
/// ```
///
/// # Errors
///
/// Returns an error for unsupported literal types.
pub fn literal_to_swift(literal: &Literal) -> Result<String> {
    Ok(match literal {
        // Integer literals
        Literal::i8(v) => v.to_string(),
        Literal::i16(v) => v.to_string(),
        Literal::i32(v) => v.to_string(),
        Literal::u8(v) => v.to_string(),
        Literal::u16(v) => v.to_string(),
        Literal::u32(v) => v.to_string(),

        // Float literals
        Literal::f32(v) => v.to_string(),
        Literal::f64(v) => v.to_string(),

        // Boolean literal
        Literal::bool(v) => v.to_string(),

        // String and character literals (need quotes)
        Literal::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
        Literal::char(c) => format!("\"{}\"", c),

        // None/null literal
        Literal::None => "nil".to_string(),

        // Unsupported literals
        _ => {
            return Err(Error::UnsupportedType(
                "Unsupported literal type".to_string(),
            ))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_integers() {
        assert_eq!(primitive_to_swift(&Primitive::i8).unwrap(), "Int8");
        assert_eq!(primitive_to_swift(&Primitive::i16).unwrap(), "Int16");
        assert_eq!(primitive_to_swift(&Primitive::i32).unwrap(), "Int32");
        assert_eq!(primitive_to_swift(&Primitive::i64).unwrap(), "Int64");
        assert_eq!(primitive_to_swift(&Primitive::isize).unwrap(), "Int");
    }

    #[test]
    fn test_unsigned_integers() {
        assert_eq!(primitive_to_swift(&Primitive::u8).unwrap(), "UInt8");
        assert_eq!(primitive_to_swift(&Primitive::u16).unwrap(), "UInt16");
        assert_eq!(primitive_to_swift(&Primitive::u32).unwrap(), "UInt32");
        assert_eq!(primitive_to_swift(&Primitive::u64).unwrap(), "UInt64");
        assert_eq!(primitive_to_swift(&Primitive::usize).unwrap(), "UInt");
    }

    #[test]
    fn test_floats() {
        assert_eq!(primitive_to_swift(&Primitive::f32).unwrap(), "Float");
        assert_eq!(primitive_to_swift(&Primitive::f64).unwrap(), "Double");
    }

    #[test]
    fn test_other_primitives() {
        assert_eq!(primitive_to_swift(&Primitive::bool).unwrap(), "Bool");
        assert_eq!(primitive_to_swift(&Primitive::char).unwrap(), "Character");
        assert_eq!(primitive_to_swift(&Primitive::String).unwrap(), "String");
    }

    #[test]
    fn test_unsupported_128bit() {
        assert!(primitive_to_swift(&Primitive::i128).is_err());
        assert!(primitive_to_swift(&Primitive::u128).is_err());
    }

    #[test]
    fn test_unsupported_f16() {
        assert!(primitive_to_swift(&Primitive::f16).is_err());
    }

    #[test]
    fn test_integer_literals() {
        assert_eq!(literal_to_swift(&Literal::i32(42)).unwrap(), "42");
        assert_eq!(literal_to_swift(&Literal::u32(100)).unwrap(), "100");
        assert_eq!(literal_to_swift(&Literal::i8(-5)).unwrap(), "-5");
    }

    #[test]
    fn test_float_literals() {
        assert_eq!(literal_to_swift(&Literal::f64(3.14)).unwrap(), "3.14");
        assert_eq!(literal_to_swift(&Literal::f32(2.5)).unwrap(), "2.5");
    }

    #[test]
    fn test_bool_literals() {
        assert_eq!(literal_to_swift(&Literal::bool(true)).unwrap(), "true");
        assert_eq!(literal_to_swift(&Literal::bool(false)).unwrap(), "false");
    }

    #[test]
    fn test_string_literals() {
        assert_eq!(
            literal_to_swift(&Literal::String("hello".into())).unwrap(),
            "\"hello\""
        );
        assert_eq!(
            literal_to_swift(&Literal::String("world".into())).unwrap(),
            "\"world\""
        );
    }

    #[test]
    fn test_string_literal_escaping() {
        assert_eq!(
            literal_to_swift(&Literal::String("say \"hello\"".into())).unwrap(),
            "\"say \\\"hello\\\"\""
        );
    }

    #[test]
    fn test_char_literal() {
        assert_eq!(literal_to_swift(&Literal::char('a')).unwrap(), "\"a\"");
        assert_eq!(literal_to_swift(&Literal::char('Z')).unwrap(), "\"Z\"");
    }

    #[test]
    fn test_none_literal() {
        assert_eq!(literal_to_swift(&Literal::None).unwrap(), "nil");
    }
}
