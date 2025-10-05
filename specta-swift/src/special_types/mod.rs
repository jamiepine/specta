//! Special type handling module
//!
//! This module provides specialized handling for types that require custom Swift
//! representations or helper types:
//!
//! - `json_value`: JsonValue type for arbitrary JSON
//! - `duration`: Rust Duration → Swift TimeInterval conversion
//! - `serde_json`: serde_json::Value type handling
//!
//! # Architecture
//!
//! Some Rust types don't have direct Swift equivalents or require special handling:
//!
//! ## JsonValue
//!
//! Custom recursive enum for representing arbitrary JSON values:
//! ```swift
//! public indirect enum JsonValue: Codable {
//!     case null
//!     case bool(Bool)
//!     case number(Double)
//!     case string(String)
//!     case array([JsonValue])
//!     case object([String: JsonValue])
//! }
//! ```
//!
//! ## Duration
//!
//! Rust's `Duration` type is represented as a struct with separate seconds and nanoseconds:
//! ```swift
//! public struct RustDuration: Codable {
//!     public let secs: UInt64
//!     public let nanos: UInt32
//!     
//!     public var timeInterval: TimeInterval {
//!         return Double(secs) + Double(nanos) / 1_000_000_000.0
//!     }
//! }
//! ```

// Submodules
pub mod detection;
pub mod duration;
pub mod serde_json;

// Re-export commonly used functions
pub use detection::is_special_std_type;
pub use duration::is_duration_struct;
pub use serde_json::is_serde_json_number_enum;

// Re-export public types from submodules once they're created
// pub use json_value::*;

