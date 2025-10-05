//! Utility functions and helpers module
//!
//! This module provides common utilities used throughout the codebase:
//!
//! - `formatting`: Code formatting helpers (indentation, line wrapping, etc.)
//! - `validation`: Type validation and sanity checks
//! - `testing`: Test utilities and helpers
//!
//! # Architecture
//!
//! The utils layer contains pure helper functions that don't fit into other categories.
//! These are typically stateless functions that provide common functionality:
//!
//! ## Formatting
//!
//! - Indentation management
//! - Doc comment formatting
//! - Code block formatting
//! - String escaping
//!
//! ## Validation
//!
//! - Type name validation
//! - Circular reference detection
//! - Invalid character detection
//! - Reserved keyword checking
//!
//! ## Testing
//!
//! - Test assertion helpers
//! - Mock data generation
//! - Output comparison utilities

// Submodules
pub mod formatting;
pub mod validation;

// Re-export commonly used functions
pub use formatting::{escape_string, format_doc_comment, indent};
pub use validation::is_recursive_type_reference;

// Re-export public types from submodules once they're created
// pub use testing::*;
