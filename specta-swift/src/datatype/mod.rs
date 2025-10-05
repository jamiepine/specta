//! Data type generation module
//!
//! This module handles the generation of Swift type definitions from Specta datatypes.
//! It provides specialized generators for different type categories:
//!
//! - `struct_gen`: Struct type generation
//! - `enum_gen`: Enum type generation  
//! - `tuple_gen`: Tuple variant generation
//! - `primitives`: Primitive type mapping and validation
//!
//! # Architecture
//!
//! The data type generation layer is responsible for converting Specta's intermediate
//! representation (IR) of types into Swift type declarations. This is a pure transformation
//! that focuses on syntax generation without concerning itself with Codable implementation.

// Submodules
pub mod collections;
pub mod export;
pub mod generic;
pub mod primitives;
pub mod reference;

// Re-export commonly used functions
pub use collections::{list_to_swift, map_to_swift, tuple_to_swift};
pub use export::{datatype_to_swift, export_type_with_name};
pub use generic::generic_to_swift;
pub use primitives::{literal_to_swift, primitive_to_swift};
pub use reference::reference_to_swift;

// Re-export public types from submodules once they're created
// pub use struct_gen::*;
// pub use enum_gen::*;
// pub use tuple_gen::*;
