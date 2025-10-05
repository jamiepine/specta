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

// Re-export public types from submodules once they're created
// pub use struct_codable::*;
// pub use enum_codable::*;
// pub use adjacently_tagged::*;
// pub use coding_keys::*;

