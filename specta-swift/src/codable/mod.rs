//! Codable protocol implementation module
//!
//! This module handles the generation of Swift Codable protocol implementations.
//! It provides specialized generators for different Codable scenarios:
//!
//! - `struct_codable`: Codable for struct types
//! - `enum_codable`: Codable for enum types
//! - `adjacently_tagged`: Adjacently tagged enum Codable implementation
//! - `coding_keys`: CodingKeys enum generation for custom serialization keys
//!
//! # Architecture
//!
//! The Codable layer is responsible for generating the protocol implementation code
//! that enables encoding and decoding of Swift types to/from JSON. This includes:
//!
//! - `init(from decoder: Decoder)` initializers
//! - `encode(to encoder: Encoder)` methods
//! - `CodingKeys` enum definitions
//! - Custom encoding/decoding logic for complex types

// Submodules
pub mod adjacently_tagged;
pub mod enum_codable;
pub mod struct_codable;

// Re-export commonly used functions
pub use adjacently_tagged::generate_adjacently_tagged_codable;
pub use enum_codable::generate_enum_codable_impl;
pub use struct_codable::generate_enum_variant_structs;

// Re-export public types from submodules once they're created
// pub use coding_keys::*;
