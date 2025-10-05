//! Type export orchestration
//!
//! This module contains the main orchestration logic for exporting Specta types
//! to Swift. It coordinates between all the specialized modules to generate
//! complete Swift type definitions.

use std::borrow::Cow;

use specta::{datatype::DataType, SpectaID, TypeCollection};

use crate::datatype::primitives::{literal_to_swift, primitive_to_swift};
use crate::error::{Error, Result};
use crate::naming::case_conversion::to_pascal_case;
use crate::naming::rename_rules::{generate_raw_value, generate_string_enum_raw_value};
use crate::naming::variant_naming::generate_variant_struct_name;
use crate::special_types::{is_duration_struct, is_serde_json_number_enum, is_special_std_type};
use crate::swift::Swift;
use crate::utils::formatting::{format_deprecated, format_doc_comment};
use crate::utils::validation::is_recursive_type_reference;

/// Export a single type to Swift with a custom name.
pub fn export_type_with_name(
    swift: &Swift,
    types: &TypeCollection,
    ndt: &specta::datatype::NamedDataType,
    custom_name: &str,
) -> Result<String> {
    let mut result = String::new();

    // Add JSDoc-style comments if present
    if !ndt.docs().is_empty() {
        result.push_str(&format_doc_comment(ndt.docs()));
    }

    // Add deprecated annotation if present
    if let Some(deprecated) = ndt.deprecated() {
        let message = match deprecated {
            specta::datatype::DeprecatedType::Deprecated => "This type is deprecated",
            specta::datatype::DeprecatedType::DeprecatedWithSince { note, .. } => note.as_ref(),
            _ => "This type is deprecated",
        };
        result.push_str(&format_deprecated(message));
    }

    // Generate the type definition
    let type_def = datatype_to_swift(swift, types, ndt.ty(), vec![], false, Some(ndt.sid()))?;

    // Format based on type
    match ndt.ty() {
        DataType::Struct(s) => {
            let name = swift.naming.convert(custom_name);
            let generics = if ndt.generics().is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    ndt.generics()
                        .iter()
                        .map(|g| g.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };

            result.push_str(&format!("public struct {}{}: Codable {{\n", name, generics));

            // Handle empty structs specially
            match s.fields() {
                specta::datatype::Fields::Named(fields) if fields.fields().is_empty() => {
                    // Empty struct - just close the braces
                }
                specta::datatype::Fields::Unit => {
                    // Unit struct - just close the braces
                }
                _ => {
                    // Non-empty struct - use the generated type definition
                    result.push_str(&type_def);
                }
            }

            result.push_str("}");

            // Add custom Codable implementation if struct has optional fields
            // Check if any field is wrapped in Option/Nullable
            if let specta::datatype::Fields::Named(fields) = s.fields() {
                let has_nullable_fields = fields.fields().iter().any(|(_, field)| {
                    if let Some(ty) = field.ty() {
                        matches!(ty, DataType::Nullable(_))
                    } else {
                        false
                    }
                });

                if has_nullable_fields {
                    let codable_impl = generate_struct_codable_impl(swift, types, s, &name)?;
                    result.push_str(&codable_impl);
                }
            }
        }
        DataType::Enum(e) => {
            let name = swift.naming.convert(custom_name);
            let generics = if ndt.generics().is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    ndt.generics()
                        .iter()
                        .map(|g| g.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };

            // Check if this is a string enum
            let is_string_enum = e.repr().map(|repr| repr.is_string()).unwrap_or(false);

            // Check if this enum has struct-like or tuple variants (needs custom Codable)
            let has_struct_variants = e.variants().iter().any(|(_, variant)| {
                matches!(variant.fields(), specta::datatype::Fields::Named(fields) if !fields.fields().is_empty())
                || matches!(variant.fields(), specta::datatype::Fields::Unnamed(fields) if !fields.fields().is_empty())
            });

            // Determine protocols based on whether we'll generate custom Codable
            let protocols = if is_string_enum {
                if has_struct_variants {
                    "String" // Custom Codable will be generated
                } else {
                    "String, Codable"
                }
            } else {
                if has_struct_variants {
                    "" // Custom Codable will be generated
                } else {
                    "Codable"
                }
            };

            let protocol_part = if protocols.is_empty() {
                String::new()
            } else {
                format!(": {}", protocols)
            };

            // Check if this enum is recursive (has variants that reference the enum itself)
            let _is_recursive = e
                .variants()
                .iter()
                .any(|(_, variant)| match variant.fields() {
                    specta::datatype::Fields::Named(fields) => {
                        fields.fields().iter().any(|(_, field)| {
                            if let Some(ty) = field.ty() {
                                is_recursive_type_reference(ty, ndt.sid())
                            } else {
                                false
                            }
                        })
                    }
                    specta::datatype::Fields::Unnamed(fields) => {
                        fields.fields().iter().any(|field| {
                            if let Some(ty) = field.ty() {
                                is_recursive_type_reference(ty, ndt.sid())
                            } else {
                                false
                            }
                        })
                    }
                    specta::datatype::Fields::Unit => false,
                });

            if is_string_enum {
                // String enum with raw values - always include String raw type
                let string_protocols = if has_struct_variants {
                    ": String" // Custom Codable will be generated
                } else {
                    ": String, Codable"
                };
                result.push_str(&format!(
                    "public enum {}{}{} {{\n",
                    name, generics, string_protocols
                ));

                for (variant_name, _variant) in e.variants() {
                    let swift_variant_name = swift.naming.convert_enum_case(variant_name);
                    let raw_value =
                        generate_raw_value(variant_name, e.repr().and_then(|r| r.rename_all()));
                    result.push_str(&format!(
                        "    case {} = \"{}\"\n",
                        swift_variant_name, raw_value
                    ));
                }

                result.push_str("}");

                // Add Codable extension if needed for string enums with struct variants
                if has_struct_variants {
                    result.push_str(&format!("\n\nextension {}: Codable {{\n", name));
                    result.push_str(
                        "    // TODO: Implement string enum with struct variants Codable\n",
                    );
                    result.push_str("}");
                }
            } else {
                // Regular tagged union enum
                result.push_str(&format!(
                    "public enum {}{}{} {{\n",
                    name, generics, protocol_part
                ));

                for (variant_name, variant) in e.variants() {
                    // Skip variants marked with #[serde(skip)] or #[specta(skip)]
                    if variant.skip() {
                        continue;
                    }
                    let swift_variant_name = swift.naming.convert_enum_case(variant_name);

                    match variant.fields() {
                        specta::datatype::Fields::Unit => {
                            result.push_str(&format!("    case {}\n", swift_variant_name));
                        }
                        specta::datatype::Fields::Unnamed(fields) => {
                            if fields.fields().is_empty() {
                                // Empty tuple variant - treat as unit variant
                                result.push_str(&format!("    case {}\n", swift_variant_name));
                            } else {
                                let types_str = fields
                                    .fields()
                                    .iter()
                                    .filter_map(|field| {
                                        field.ty().map(|ty| {
                                            datatype_to_swift(swift, types, ty, vec![], false, None)
                                        })
                                    })
                                    .collect::<Result<Vec<_>>>()?
                                    .join(", ");

                                if types_str.is_empty() {
                                    // Empty tuple variant - generate as unit variant
                                    result.push_str(&format!("    case {}\n", swift_variant_name));
                                } else {
                                    result.push_str(&format!(
                                        "    case {}({})\n",
                                        swift_variant_name, types_str
                                    ));
                                }
                            }
                        }
                        specta::datatype::Fields::Named(fields) => {
                            if fields.fields().is_empty() {
                                result.push_str(&format!("    case {}\n", swift_variant_name));
                            } else {
                                // Generate a struct for this variant
                                let struct_name =
                                    generate_variant_struct_name(swift, &name, variant_name);

                                result.push_str(&format!(
                                    "    case {}({})\n",
                                    swift_variant_name, struct_name
                                ));
                            }
                        }
                    }
                }

                result.push_str("}\n");

                // Generate struct definitions for variants with named fields
                if has_struct_variants {
                    result.push_str(&generate_enum_variant_structs(swift, types, e, &name)?);
                }

                // Check if this is an adjacently tagged enum
                let is_adjacently_tagged = if let Some(repr) = e.repr() {
                    matches!(repr, specta::datatype::EnumRepr::Adjacent { .. })
                } else {
                    false
                };

                // Add Codable extension if needed (struct variants OR adjacently tagged)
                if has_struct_variants || is_adjacently_tagged {
                    // Note: generate_enum_codable_impl will handle adjacently tagged detection internally
                    result.push_str(&generate_enum_codable_impl(swift, types, e, &name)?);
                }
            }
        }
        _ => {
            // For other types, just use the generated type definition
            result.push_str(&type_def);
        }
    }

    Ok(result)
}

/// Convert a DataType to Swift syntax.
pub fn datatype_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    dt: &DataType,
    location: Vec<Cow<'static, str>>,
    is_export: bool,
    sid: Option<SpectaID>,
) -> Result<String> {
    // Check for special standard library types first
    if let Some(special_type) = is_special_std_type(types, sid) {
        return Ok(special_type);
    }

    match dt {
        DataType::Primitive(p) => primitive_to_swift(p),
        DataType::Literal(l) => literal_to_swift(l),
        DataType::List(l) => list_to_swift(swift, types, l),
        DataType::Map(m) => map_to_swift(swift, types, m),
        DataType::Nullable(def) => {
            let inner = datatype_to_swift(swift, types, def, location, is_export, sid)?;
            Ok(match swift.optionals {
                crate::swift::OptionalStyle::QuestionMark => format!("{}?", inner),
                crate::swift::OptionalStyle::Optional => format!("Optional<{}>", inner),
            })
        }
        DataType::Struct(s) => {
            // Check if this is a Duration struct by looking at its fields
            if is_duration_struct(s) {
                return Ok("RustDuration".to_string());
            }
            struct_to_swift(swift, types, s, location, is_export, sid)
        }
        DataType::Enum(e) => {
            // If this is a recursive call (no enum_name), we need to generate a reference
            // to the enum type instead of trying to inline the variants
            if let Some(sid) = sid {
                if let Some(ndt) = types.get(sid) {
                    // Special handling for serde_json::Number which should map to Double
                    if ndt.name() == "Number" && ndt.module_path().contains("serde_json") {
                        return Ok("Double".to_string());
                    }
                    let name = swift.naming.convert(ndt.name());
                    return Ok(name);
                }
            }
            // Fallback: try to generate inline (this shouldn't happen in normal cases)
            enum_to_swift(swift, types, e, location, is_export, sid, None)
        }
        DataType::Tuple(t) => tuple_to_swift(swift, types, t),
        DataType::Reference(r) => reference_to_swift(swift, types, r),
        DataType::Generic(g) => generic_to_swift(swift, g),
    }
}

// Special type functions now imported from special_types module

/// Convert list types to Swift arrays.
fn list_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    list: &specta::datatype::List,
) -> Result<String> {
    crate::datatype::collections::list_to_swift(list, |ty| {
        datatype_to_swift(swift, types, ty, vec![], false, None)
    })
}

/// Convert map types to Swift dictionaries.
fn map_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    map: &specta::datatype::Map,
) -> Result<String> {
    crate::datatype::collections::map_to_swift(map, |ty| {
        datatype_to_swift(swift, types, ty, vec![], false, None)
    })
}

/// Convert struct types to Swift.
fn struct_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    s: &specta::datatype::Struct,
    location: Vec<Cow<'static, str>>,
    is_export: bool,
    sid: Option<SpectaID>,
) -> Result<String> {
    match s.fields() {
        specta::datatype::Fields::Unit => Ok("Void".to_string()),
        specta::datatype::Fields::Unnamed(fields) => {
            if fields.fields().is_empty() {
                Ok("Void".to_string())
            } else if fields.fields().len() == 1 {
                // Single field tuple struct - convert to a proper struct with a 'value' field
                let field_type = datatype_to_swift(
                    swift,
                    types,
                    &fields.fields()[0].ty().unwrap(),
                    location,
                    is_export,
                    sid,
                )?;
                Ok(format!("    let value: {}\n", field_type))
            } else {
                // Multiple field tuple struct - convert to a proper struct with numbered fields
                let mut result = String::new();
                for (i, field) in fields.fields().iter().enumerate() {
                    let field_type = datatype_to_swift(
                        swift,
                        types,
                        field.ty().unwrap(),
                        location.clone(),
                        is_export,
                        sid,
                    )?;
                    result.push_str(&format!("    public let field{}: {}\n", i, field_type));
                }
                Ok(result)
            }
        }
        specta::datatype::Fields::Named(fields) => {
            let mut result = String::new();
            let mut field_mappings = Vec::new();

            // Check if struct will need custom Codable implementation (has nullable fields)
            let has_nullable_fields = fields.fields().iter().any(|(_, field)| {
                if let Some(ty) = field.ty() {
                    matches!(ty, DataType::Nullable(_))
                } else {
                    false
                }
            });

            for (original_field_name, field) in fields.fields() {
                let field_type = if let Some(ty) = field.ty() {
                    datatype_to_swift(swift, types, ty, location.clone(), is_export, sid)?
                } else {
                    continue;
                };

                let optional_marker = if field.optional() { "?" } else { "" };
                let swift_field_name = swift.naming.convert_field(original_field_name);

                result.push_str(&format!(
                    "    public let {}: {}{}\n",
                    swift_field_name, field_type, optional_marker
                ));

                field_mappings.push((swift_field_name, original_field_name.to_string()));
            }

            // Only generate CodingKeys inside the struct if we're NOT generating custom Codable extension
            // (If we are generating extension, CodingKeys will be in the extension to avoid duplication)
            let needs_custom_coding_keys = field_mappings
                .iter()
                .any(|(swift_name, rust_name)| swift_name != rust_name);
            if needs_custom_coding_keys && !has_nullable_fields {
                result.push_str("\n    private enum CodingKeys: String, CodingKey {\n");
                for (swift_name, rust_name) in &field_mappings {
                    result.push_str(&format!(
                        "        case {} = \"{}\"\n",
                        swift_name, rust_name
                    ));
                }
                result.push_str("    }\n");
            }

            // Generate public initializer if enabled
            if swift.generate_initializers && !field_mappings.is_empty() {
                result.push_str("\n    public init(");
                let init_params: Vec<String> = field_mappings
                    .iter()
                    .map(|(swift_name, _)| {
                        // Get the field type from the original field
                        let field_type = fields
                            .fields()
                            .iter()
                            .find(|(name, _)| swift.naming.convert_field(name) == *swift_name)
                            .and_then(|(_, field)| field.ty())
                            .and_then(|ty| {
                                datatype_to_swift(
                                    swift,
                                    types,
                                    ty,
                                    location.clone(),
                                    is_export,
                                    sid,
                                )
                                .ok()
                            })
                            .unwrap_or_else(|| "Any".to_string());

                        let optional_marker = fields
                            .fields()
                            .iter()
                            .find(|(name, _)| swift.naming.convert_field(name) == *swift_name)
                            .map(|(_, field)| if field.optional() { "?" } else { "" })
                            .unwrap_or("");

                        format!("{}: {}{}", swift_name, field_type, optional_marker)
                    })
                    .collect();
                result.push_str(&init_params.join(", "));
                result.push_str(") {\n");

                // Assign parameters to properties
                for (swift_name, _) in &field_mappings {
                    result.push_str(&format!("        self.{} = {}\n", swift_name, swift_name));
                }
                result.push_str("    }\n");
            }

            Ok(result)
        }
    }
}

// Function now imported from naming::rename_rules module

/// Convert enum types to Swift.
fn enum_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    e: &specta::datatype::Enum,
    location: Vec<Cow<'static, str>>,
    is_export: bool,
    sid: Option<SpectaID>,
    enum_name: Option<&str>,
) -> Result<String> {
    // If we have a sid and this is being used as a type reference (not a full definition),
    // just return the enum name
    if let Some(sid) = sid {
        if let Some(ndt) = types.get(sid) {
            let name = swift.naming.convert(ndt.name());
            return Ok(name);
        }
    }

    // Special handling for serde_json::Number enum which should map to Double
    // This enum has specific variants: f64, i64, u64 and is from serde_json
    if is_serde_json_number_enum(e) {
        return Ok("Double".to_string());
    }

    let mut result = String::new();

    // Check if this is a string enum
    let is_string_enum = e.repr().map(|repr| repr.is_string()).unwrap_or(false);

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let variant_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            specta::datatype::Fields::Unit => {
                if is_string_enum {
                    // For string enums, generate raw value assignments
                    let raw_value = generate_raw_value(
                        original_variant_name,
                        e.repr().and_then(|r| r.rename_all()),
                    );
                    result.push_str(&format!("    case {} = \"{}\"\n", variant_name, raw_value));
                } else {
                    result.push_str(&format!("    case {}\n", variant_name));
                }
            }
            specta::datatype::Fields::Unnamed(fields) => {
                if fields.fields().is_empty() {
                    result.push_str(&format!("    case {}\n", variant_name));
                } else {
                    let types_str = fields
                        .fields()
                        .iter()
                        .map(|f| {
                            let field_ty = f.ty().unwrap();
                            // For references, use the referenced type's sid, not the parent enum's sid
                            let field_sid = if let DataType::Reference(r) = field_ty {
                                Some(r.sid())
                            } else {
                                None
                            };
                            datatype_to_swift(
                                swift,
                                types,
                                field_ty,
                                location.clone(),
                                is_export,
                                field_sid,
                            )
                        })
                        .collect::<std::result::Result<Vec<_>, _>>()?
                        .join(", ");
                    result.push_str(&format!("    case {}({})\n", variant_name, types_str));
                }
            }
            specta::datatype::Fields::Named(fields) => {
                if fields.fields().is_empty() {
                    result.push_str(&format!("    case {}\n", variant_name));
                } else {
                    // Generate struct for named fields
                    // This is the old enum_to_swift function that's called from datatype_to_swift
                    // It doesn't have access to the Swift config, so we use a simple naming approach
                    let pascal_variant_name = to_pascal_case(original_variant_name);
                    let struct_name = if let Some(enum_name) = enum_name {
                        format!("{}{}", enum_name, pascal_variant_name)
                    } else {
                        pascal_variant_name.to_string()
                    };

                    // Generate enum case that references the struct
                    result.push_str(&format!("    case {}({})\n", variant_name, struct_name));
                }
            }
        }
    }

    Ok(result)
}

// Dead code removed - generate_enum_structs was never used

// Functions now imported from naming modules

/// Convert tuple types to Swift.
fn tuple_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    t: &specta::datatype::Tuple,
) -> Result<String> {
    crate::datatype::collections::tuple_to_swift(t, |ty| {
        datatype_to_swift(swift, types, ty, vec![], false, None)
    })
}

/// Convert reference types to Swift.
fn reference_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    r: &specta::datatype::Reference,
) -> Result<String> {
    crate::datatype::reference::reference_to_swift(swift, types, r, |ty| {
        datatype_to_swift(swift, types, ty, vec![], false, None)
    })
}

/// Convert generic types to Swift.
fn generic_to_swift(_swift: &Swift, g: &specta::datatype::Generic) -> Result<String> {
    crate::datatype::generic::generic_to_swift(g)
}

/// Generate custom Codable implementation for structs with optional fields.
///
/// This generates `init(from:)` and `encode(to:)` methods that preserve `nil` values
/// as `null` in JSON, matching Rust's serde behavior.
fn generate_struct_codable_impl(
    swift: &Swift,
    types: &TypeCollection,
    s: &specta::datatype::Struct,
    struct_name: &str,
) -> Result<String> {
    let mut result = String::new();

    if let specta::datatype::Fields::Named(fields) = s.fields() {
        let mut field_info = Vec::new();

        // Collect field information
        for (original_field_name, field) in fields.fields() {
            let (field_type, base_type, is_optional) = if let Some(ty) = field.ty() {
                let is_nullable = matches!(ty, DataType::Nullable(_));
                let swift_type = datatype_to_swift(swift, types, ty, vec![], false, None)?;
                // For nullable types, extract the base type (without ?)
                let base_type = if is_nullable && swift_type.ends_with('?') {
                    swift_type[..swift_type.len() - 1].to_string()
                } else {
                    swift_type.clone()
                };
                (swift_type, base_type, is_nullable)
            } else {
                continue;
            };

            let swift_field_name = swift.naming.convert_field(original_field_name);
            let rust_field_name = original_field_name.to_string();

            field_info.push((
                swift_field_name,
                rust_field_name,
                field_type,
                base_type,
                is_optional,
            ));
        }

        // Generate extension
        result.push_str(&format!(
            "\n// MARK: - {} Custom Codable Implementation\n",
            struct_name
        ));
        result.push_str(&format!("extension {} {{\n", struct_name));

        // Generate CodingKeys enum
        result.push_str("    private enum CodingKeys: String, CodingKey {\n");
        for (swift_name, rust_name, _, _, _) in &field_info {
            result.push_str(&format!(
                "        case {} = \"{}\"\n",
                swift_name, rust_name
            ));
        }
        result.push_str("    }\n\n");

        // Generate init(from decoder:)
        result.push_str("    public init(from decoder: Decoder) throws {\n");
        result
            .push_str("        let container = try decoder.container(keyedBy: CodingKeys.self)\n");

        for (swift_name, _, _field_type, base_type, is_optional) in &field_info {
            if *is_optional {
                result.push_str(&format!(
                    "        {} = try container.decodeIfPresent({}.self, forKey: .{})\n",
                    swift_name, base_type, swift_name
                ));
            } else {
                result.push_str(&format!(
                    "        {} = try container.decode({}.self, forKey: .{})\n",
                    swift_name, base_type, swift_name
                ));
            }
        }

        result.push_str("    }\n\n");

        // Generate encode(to encoder:)
        result.push_str("    public func encode(to encoder: Encoder) throws {\n");
        result.push_str("        var container = encoder.container(keyedBy: CodingKeys.self)\n");

        for (swift_name, _, _, _, _) in &field_info {
            // Use encode() for all fields - this preserves nil as null in JSON
            result.push_str(&format!(
                "        try container.encode({}, forKey: .{})\n",
                swift_name, swift_name
            ));
        }

        result.push_str("    }\n");
        result.push_str("}\n");
    }

    Ok(result)
}

/// Generate custom Codable implementation for enums with struct-like variants
fn generate_enum_codable_impl(
    swift: &Swift,
    types: &TypeCollection,
    e: &specta::datatype::Enum,
    enum_name: &str,
) -> Result<String> {
    // Check if this is an adjacently tagged enum first
    let is_adjacently_tagged = if let Some(repr) = e.repr() {
        matches!(repr, specta::datatype::EnumRepr::Adjacent { .. })
    } else {
        false
    };

    if is_adjacently_tagged {
        return generate_adjacently_tagged_codable(swift, e, enum_name);
    }

    // Use the extracted enum_codable module
    crate::codable::enum_codable::generate_enum_codable_impl(
        swift,
        e,
        enum_name,
        |variant_name| generate_variant_struct_name(swift, enum_name, variant_name),
        |ty| datatype_to_swift(swift, types, ty, vec![], false, None),
    )
}

/// Generate custom Codable implementation for adjacently tagged enums
fn generate_adjacently_tagged_codable(
    swift: &Swift,
    e: &specta::datatype::Enum,
    enum_name: &str,
) -> Result<String> {
    crate::codable::adjacently_tagged::generate_adjacently_tagged_codable(
        swift,
        e,
        enum_name,
        |variant_name| generate_variant_struct_name(swift, enum_name, variant_name),
    )
}

// Function now imported from codable::struct_codable module

/// Generate struct definitions for enum variants with named fields
fn generate_enum_variant_structs(
    swift: &Swift,
    types: &TypeCollection,
    e: &specta::datatype::Enum,
    enum_name: &str,
) -> Result<String> {
    crate::codable::struct_codable::generate_enum_variant_structs(
        swift,
        types,
        e,
        enum_name,
        |variant_name| generate_variant_struct_name(swift, enum_name, variant_name),
        |ty| datatype_to_swift(swift, types, ty, vec![], false, None),
    )
}
