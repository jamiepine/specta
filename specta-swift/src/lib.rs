//! [Swift](https://www.swift.org) language exporter.
//!
//! This crate provides functionality to export Rust types to Swift code.
//!
//! # Usage
//!
//! Add `specta` and `specta-swift` to your project:
//!
//! ```bash
//! cargo add specta@2.0.0-rc.22 --features derive,export
//! cargo add specta-swift@0.0.1
//! cargo add specta-serde@0.0.9
//! ```
//!
//! Next copy the following into your `main.rs` file:
//!
//! ```rust
//! use specta::{Type, TypeCollection};
//! use specta_swift::Swift;
//!
//! #[derive(Type)]
//! pub struct MyType {
//!     pub field: MyOtherType,
//! }
//!
//! #[derive(Type)]
//! pub struct MyOtherType {
//!     pub other_field: String,
//! }
//!
//! fn main() {
//!     let mut types = TypeCollection::default()
//!         // We don't need to specify `MyOtherType` because it's referenced by `MyType`
//!         .register::<MyType>();
//!
//!     Swift::default()
//!         .export_to("./Types.swift", &types)
//!         .unwrap();
//! }
//! ```
//!
//! Now you're set up with Specta Swift!
//!
//! If you get tired of listing all your types, checkout [`specta::export`].
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/specta/raw/main/.github/logo-128.png",
    html_favicon_url = "https://github.com/oscartbeaumont/specta/raw/main/.github/logo-128.png"
)]

// Core modules
mod error;
mod swift;

// New modular architecture (public for testing and external use)
pub mod codable;
pub mod datatype;
pub mod naming;
pub mod special_types;
pub mod utils;

pub use error::Error;
pub use swift::{
    DuplicateNameStrategy, GenericStyle, IndentStyle, NamingConvention, OptionalStyle,
    StructNamingStrategy, Swift,
};
