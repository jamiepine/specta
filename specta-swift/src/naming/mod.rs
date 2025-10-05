//! Naming strategies and case conversion module
//!
//! This module handles all naming-related functionality including:
//!
//! - `strategies`: Duplicate name resolution strategies
//! - `case_conversion`: Converting between naming conventions (snake_case ↔ camelCase)
//! - `resolver`: Name conflict detection and resolution
//!
//! # Architecture
//!
//! The naming layer provides tools for ensuring Swift type names are:
//!
//! 1. **Valid**: Follow Swift naming conventions
//! 2. **Unique**: No duplicate type names in the same scope
//! 3. **Idiomatic**: Use appropriate case conventions (camelCase, PascalCase)
//! 4. **Consistent**: Apply transformations uniformly across all types
//!
//! # Naming Strategies
//!
//! Four strategies are available for handling duplicate names:
//!
//! - `Warn`: Log warnings but allow duplicates (default)
//! - `Error`: Fail generation on duplicates
//! - `Qualify`: Auto-generate qualified names from module paths
//! - `Custom`: User-provided naming function

// Submodules
pub mod case_conversion;
pub mod rename_rules;
pub mod variant_naming;

// Re-export commonly used functions
pub use case_conversion::{snake_to_camel, snake_to_pascal, to_pascal_case, to_snake_case};
pub use rename_rules::{generate_raw_value, generate_string_enum_raw_value};
pub use variant_naming::generate_variant_struct_name;

// Re-export public types from submodules once they're created
// pub use strategies::*;
// pub use resolver::*;
