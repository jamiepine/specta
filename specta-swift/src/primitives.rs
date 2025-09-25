//! Primitive type conversion from Rust to Swift.

use std::borrow::Cow;

use specta::{
    datatype::{DataType, Primitive},
    SpectaID, TypeCollection,
};

use crate::error::{Error, Result};
use crate::swift::Swift;

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
        let docs = ndt.docs();
        // Handle multi-line comments properly
        for line in docs.lines() {
            result.push_str("/// ");
            // Trim leading whitespace from the line to avoid extra spaces
            result.push_str(line.trim_start());
            result.push('\n');
        }
    }

    // Add deprecated annotation if present
    if let Some(deprecated) = ndt.deprecated() {
        let message = match deprecated {
            specta::datatype::DeprecatedType::Deprecated => "This type is deprecated".to_string(),
            specta::datatype::DeprecatedType::DeprecatedWithSince { note, .. } => note.to_string(),
            _ => "This type is deprecated".to_string(),
        };
        result.push_str(&format!(
            "@available(*, deprecated, message: \"{}\")\n",
            message
        ));
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

            // Check if this enum has struct-like variants (needs custom Codable)
            let has_struct_variants = e.variants().iter().any(|(_, variant)| {
                matches!(variant.fields(), specta::datatype::Fields::Named(fields) if !fields.fields().is_empty())
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
                    let raw_value = generate_string_enum_raw_value(variant_name, swift.naming);
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
                                let struct_name = match swift.struct_naming {
                                    crate::swift::StructNamingStrategy::AutoRename => {
                                        format!("{}{}", name, swift.naming.convert(variant_name))
                                    }
                                    crate::swift::StructNamingStrategy::KeepOriginal => {
                                        swift.naming.convert(variant_name)
                                    }
                                };

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
                    result.push_str(&generate_enum_codable_impl(swift, e, &name)?);
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

/// Check if a struct is a Duration by examining its fields
pub fn is_duration_struct(s: &specta::datatype::Struct) -> bool {
    match s.fields() {
        specta::datatype::Fields::Named(fields) => {
            let field_names: Vec<String> = fields
                .fields()
                .iter()
                .map(|(name, _)| name.to_string())
                .collect();
            // Duration has exactly two fields: "secs" (u64) and "nanos" (u32)
            field_names.len() == 2
                && field_names.contains(&"secs".to_string())
                && field_names.contains(&"nanos".to_string())
        }
        _ => false,
    }
}

/// Check if a type is a special standard library type that needs special handling
fn is_special_std_type(types: &TypeCollection, sid: Option<SpectaID>) -> Option<String> {
    if let Some(sid) = sid {
        if let Some(ndt) = types.get(sid) {
            // Check for std::time::Duration
            if ndt.name() == "Duration" {
                return Some("RustDuration".to_string());
            }
            // Check for std::time::SystemTime
            if ndt.name() == "SystemTime" {
                return Some("Date".to_string());
            }
            // Check for serde_json::Number
            if ndt.name() == "Number" {
                return Some("Double".to_string());
            }
            // Check for serde_json::Value (mapped as JsonValue in Specta)
            if ndt.name() == "JsonValue" {
                return Some("JsonValue".to_string());
            }
        }
    }
    None
}

/// Convert primitive types to Swift.
fn primitive_to_swift(primitive: &Primitive) -> Result<String> {
    Ok(match primitive {
        Primitive::i8 => "Int8".to_string(),
        Primitive::i16 => "Int16".to_string(),
        Primitive::i32 => "Int32".to_string(),
        Primitive::i64 => "Int64".to_string(),
        Primitive::isize => "Int".to_string(),
        Primitive::u8 => "UInt8".to_string(),
        Primitive::u16 => "UInt16".to_string(),
        Primitive::u32 => "UInt32".to_string(),
        Primitive::u64 => "UInt64".to_string(),
        Primitive::usize => "UInt".to_string(),
        Primitive::f32 => "Float".to_string(),
        Primitive::f64 => "Double".to_string(),
        Primitive::bool => "Bool".to_string(),
        Primitive::char => "Character".to_string(),
        Primitive::String => "String".to_string(),
        Primitive::i128 | Primitive::u128 => {
            return Err(Error::UnsupportedType(
                "Swift does not support 128-bit integers".to_string(),
            ));
        }
        Primitive::f16 => {
            return Err(Error::UnsupportedType(
                "Swift does not support f16".to_string(),
            ));
        }
    })
}

/// Convert literal types to Swift.
fn literal_to_swift(literal: &specta::datatype::Literal) -> Result<String> {
    Ok(match literal {
        specta::datatype::Literal::i8(v) => v.to_string(),
        specta::datatype::Literal::i16(v) => v.to_string(),
        specta::datatype::Literal::i32(v) => v.to_string(),
        specta::datatype::Literal::u8(v) => v.to_string(),
        specta::datatype::Literal::u16(v) => v.to_string(),
        specta::datatype::Literal::u32(v) => v.to_string(),
        specta::datatype::Literal::f32(v) => v.to_string(),
        specta::datatype::Literal::f64(v) => v.to_string(),
        specta::datatype::Literal::bool(v) => v.to_string(),
        specta::datatype::Literal::String(s) => format!("\"{}\"", s),
        specta::datatype::Literal::char(c) => format!("\"{}\"", c),
        specta::datatype::Literal::None => "nil".to_string(),
        _ => {
            return Err(Error::UnsupportedType(
                "Unsupported literal type".to_string(),
            ))
        }
    })
}

/// Convert list types to Swift arrays.
fn list_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    list: &specta::datatype::List,
) -> Result<String> {
    let element_type = datatype_to_swift(swift, types, list.ty(), vec![], false, None)?;
    Ok(format!("[{}]", element_type))
}

/// Convert map types to Swift dictionaries.
fn map_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    map: &specta::datatype::Map,
) -> Result<String> {
    let key_type = datatype_to_swift(swift, types, map.key_ty(), vec![], false, None)?;
    let value_type = datatype_to_swift(swift, types, map.value_ty(), vec![], false, None)?;
    Ok(format!("[{}: {}]", key_type, value_type))
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

            // Generate custom CodingKeys if field names were converted
            let needs_custom_coding_keys = field_mappings
                .iter()
                .any(|(swift_name, rust_name)| swift_name != rust_name);
            if needs_custom_coding_keys {
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

/// Generate raw value for string enum variants
fn generate_raw_value(variant_name: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("lowercase") => variant_name.to_lowercase(),
        Some("UPPERCASE") => variant_name.to_uppercase(),
        Some("camelCase") => {
            let mut chars = variant_name.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_lowercase().chain(chars).collect(),
            }
        }
        Some("PascalCase") => {
            let mut chars = variant_name.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        }
        Some("snake_case") => variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    vec!['_', c.to_lowercase().next().unwrap()]
                } else {
                    vec![c.to_lowercase().next().unwrap()]
                }
            })
            .collect(),
        Some("SCREAMING_SNAKE_CASE") => variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    vec!['_', c.to_uppercase().next().unwrap()]
                } else {
                    vec![c.to_uppercase().next().unwrap()]
                }
            })
            .collect(),
        Some("kebab-case") => variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    vec!['-', c.to_lowercase().next().unwrap()]
                } else {
                    vec![c.to_lowercase().next().unwrap()]
                }
            })
            .collect(),
        Some("SCREAMING-KEBAB-CASE") => variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    vec!['-', c.to_uppercase().next().unwrap()]
                } else {
                    vec![c.to_uppercase().next().unwrap()]
                }
            })
            .collect(),
        _ => variant_name.to_lowercase(), // Default to lowercase
    }
}

/// Check if an enum is the serde_json::Number enum
fn is_serde_json_number_enum(e: &specta::datatype::Enum) -> bool {
    // Check if this enum has the specific pattern of serde_json::Number:
    // - Untagged representation
    // - Exactly 3 variants: f64, i64, u64
    // - Each variant has a single primitive field

    // Check for untagged representation
    if let Some(repr) = e.repr() {
        match repr {
            specta::datatype::EnumRepr::Untagged => {
                // Continue with checking variants
            }
            _ => return false,
        }
    } else {
        return false;
    }

    let variants: Vec<_> = e.variants().iter().collect();
    if variants.len() == 3 {
        let variant_names: Vec<&str> = variants.iter().map(|(name, _)| name.as_ref()).collect();
        if variant_names.contains(&"f64")
            && variant_names.contains(&"i64")
            && variant_names.contains(&"u64")
        {
            // Check that each variant has the expected primitive type
            for (name, variant) in variants {
                if let specta::datatype::Fields::Unnamed(fields) = variant.fields() {
                    if fields.fields().len() == 1 {
                        if let Some(field) = fields.fields().first() {
                            if let Some(DataType::Primitive(p)) = field.ty() {
                                let expected_primitive = match name.as_ref() {
                                    "f64" => specta::datatype::Primitive::f64,
                                    "i64" => specta::datatype::Primitive::i64,
                                    "u64" => specta::datatype::Primitive::u64,
                                    _ => continue,
                                };
                                if *p != expected_primitive {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            return true;
        }
    }
    false
}

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

/// Generate struct definitions for enum variants with named fields
fn generate_enum_structs(
    swift: &Swift,
    types: &TypeCollection,
    e: &specta::datatype::Enum,
    location: Vec<Cow<'static, str>>,
    is_export: bool,
    sid: Option<SpectaID>,
    enum_name: &str,
) -> Result<String> {
    let mut result = String::new();

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        if let specta::datatype::Fields::Named(fields) = variant.fields() {
            if !fields.fields().is_empty() {
                let struct_name =
                    generate_variant_struct_name(swift, enum_name, original_variant_name);

                // Generate struct definition with custom CodingKeys for field name mapping
                result.push_str(&format!("\npublic struct {}: Codable {{\n", struct_name));

                // Generate struct fields
                let mut field_mappings = Vec::new();
                for (original_field_name, field) in fields.fields() {
                    if let Some(ty) = field.ty() {
                        let field_type =
                            datatype_to_swift(swift, types, ty, location.clone(), is_export, sid)?;
                        let optional_marker = if field.optional() { "?" } else { "" };
                        let swift_field_name = swift.naming.convert_field(original_field_name);
                        result.push_str(&format!(
                            "    public let {}: {}{}\n",
                            swift_field_name, field_type, optional_marker
                        ));
                        field_mappings.push((swift_field_name, original_field_name.to_string()));
                    }
                }

                // Generate custom CodingKeys if field names were converted
                let needs_custom_coding_keys = field_mappings
                    .iter()
                    .any(|(swift_name, rust_name)| swift_name != rust_name);
                if needs_custom_coding_keys {
                    result.push_str("\n    private enum CodingKeys: String, CodingKey {\n");
                    for (swift_name, rust_name) in &field_mappings {
                        result.push_str(&format!(
                            "        case {} = \"{}\"\n",
                            swift_name, rust_name
                        ));
                    }
                    result.push_str("    }\n");
                }

                result.push_str("}\n");
            }
        }
    }

    Ok(result)
}

/// Generate consistent struct name for enum variant following the same logic as export_type_with_name
fn generate_variant_struct_name(swift: &Swift, enum_name: &str, variant_name: &str) -> String {
    match swift.struct_naming {
        crate::swift::StructNamingStrategy::AutoRename => {
            format!("{}{}", enum_name, swift.naming.convert(variant_name))
        }
        crate::swift::StructNamingStrategy::KeepOriginal => swift.naming.convert(variant_name),
    }
}

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
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

/// Convert tuple types to Swift.
fn tuple_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    t: &specta::datatype::Tuple,
) -> Result<String> {
    if t.elements().is_empty() {
        Ok("Void".to_string())
    } else if t.elements().len() == 1 {
        datatype_to_swift(swift, types, &t.elements()[0], vec![], false, None)
    } else {
        let types_str = t
            .elements()
            .iter()
            .map(|e| datatype_to_swift(swift, types, e, vec![], false, None))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .join(", ");
        Ok(format!("({})", types_str))
    }
}

/// Convert reference types to Swift.
fn reference_to_swift(
    swift: &Swift,
    types: &TypeCollection,
    r: &specta::datatype::Reference,
) -> Result<String> {
    // Get the name from the TypeCollection using the SID
    let name = if let Some(ndt) = types.get(r.sid()) {
        swift.naming.convert(ndt.name())
    } else {
        return Err(Error::InvalidIdentifier(
            "Reference to unknown type".to_string(),
        ));
    };

    if r.generics().is_empty() {
        Ok(name)
    } else {
        let generics = r
            .generics()
            .iter()
            .map(|(_, t)| datatype_to_swift(swift, types, t, vec![], false, None))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .join(", ");
        Ok(format!("{}<{}>", name, generics))
    }
}

/// Convert generic types to Swift.
fn generic_to_swift(_swift: &Swift, g: &specta::datatype::Generic) -> Result<String> {
    Ok(g.to_string())
}

/// Generate custom Codable implementation for enums with struct-like variants
fn generate_enum_codable_impl(
    swift: &Swift,
    e: &specta::datatype::Enum,
    enum_name: &str,
) -> Result<String> {
    let mut result = String::new();

    result.push_str(&format!(
        "\n// MARK: - {} Codable Implementation\n",
        enum_name
    ));
    result.push_str(&format!("extension {}: Codable {{\n", enum_name));

    // Check if this is an adjacently tagged enum
    let is_adjacently_tagged = if let Some(repr) = e.repr() {
        matches!(repr, specta::datatype::EnumRepr::Adjacent { .. })
    } else {
        false
    };

    if is_adjacently_tagged {
        return generate_adjacently_tagged_codable(swift, e, enum_name);
    }

    // Generate CodingKeys enum
    result.push_str("    private enum CodingKeys: String, CodingKey {\n");
    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }
        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);
        result.push_str(&format!(
            "        case {} = \"{}\"\n",
            swift_case_name, original_variant_name
        ));
    }
    result.push_str("    }\n\n");

    // Generate init(from decoder:)
    result.push_str("    public init(from decoder: Decoder) throws {\n");
    result.push_str("        let container = try decoder.container(keyedBy: CodingKeys.self)\n");
    result.push_str("        \n");
    result.push_str("        if container.allKeys.count != 1 {\n");
    result.push_str("            throw DecodingError.dataCorrupted(\n");
    result.push_str("                DecodingError.Context(codingPath: decoder.codingPath, debugDescription: \"Invalid number of keys found, expected one.\")\n");
    result.push_str("            )\n");
    result.push_str("        }\n\n");
    result.push_str("        let key = container.allKeys.first!\n");
    result.push_str("        switch key {\n");

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            specta::datatype::Fields::Unit => {
                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!("            self = .{}\n", swift_case_name));
            }
            specta::datatype::Fields::Unnamed(fields) => {
                if fields.fields().is_empty() {
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str(&format!("            self = .{}\n", swift_case_name));
                } else {
                    // For tuple variants, decode as array
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str(&format!(
                        "            // TODO: Implement tuple variant decoding for {}\n",
                        swift_case_name
                    ));
                    result.push_str(
                        "            fatalError(\"Tuple variant decoding not implemented\")\n",
                    );
                }
            }
            specta::datatype::Fields::Named(_) => {
                let struct_name =
                    generate_variant_struct_name(swift, enum_name, original_variant_name);

                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "            let data = try container.decode({}.self, forKey: .{})\n",
                    struct_name, swift_case_name
                ));
                result.push_str(&format!("            self = .{}(data)\n", swift_case_name));
            }
        }
    }

    result.push_str("        }\n");
    result.push_str("    }\n\n");

    // Generate encode(to encoder:)
    result.push_str("    public func encode(to encoder: Encoder) throws {\n");
    result.push_str("        var container = encoder.container(keyedBy: CodingKeys.self)\n");
    result.push_str("        \n");
    result.push_str("        switch self {\n");

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            specta::datatype::Fields::Unit => {
                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "            try container.encodeNil(forKey: .{})\n",
                    swift_case_name
                ));
            }
            specta::datatype::Fields::Unnamed(_) => {
                // TODO: Handle tuple variants
                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "            // TODO: Implement tuple variant encoding for {}\n",
                    swift_case_name
                ));
                result.push_str(
                    "            fatalError(\"Tuple variant encoding not implemented\")\n",
                );
            }
            specta::datatype::Fields::Named(_) => {
                result.push_str(&format!("        case .{}(let data):\n", swift_case_name));
                result.push_str(&format!(
                    "            try container.encode(data, forKey: .{})\n",
                    swift_case_name
                ));
            }
        }
    }

    result.push_str("        }\n");
    result.push_str("    }\n");
    result.push_str("}\n");

    Ok(result)
}

/// Generate custom Codable implementation for adjacently tagged enums
fn generate_adjacently_tagged_codable(
    swift: &Swift,
    e: &specta::datatype::Enum,
    enum_name: &str,
) -> Result<String> {
    let mut result = String::new();

    // Get tag and content field names
    let (tag_field, content_field) =
        if let Some(specta::datatype::EnumRepr::Adjacent { tag, content }) = e.repr() {
            (tag.as_ref(), content.as_ref())
        } else {
            return Err(Error::UnsupportedType(
                "Expected adjacently tagged enum".to_string(),
            ));
        };

    result.push_str(&format!(
        "\n// MARK: - {} Adjacently Tagged Codable Implementation\n",
        enum_name
    ));
    result.push_str(&format!("extension {}: Codable {{\n", enum_name));

    // Generate TypeKeys enum for the tag and content fields - make name unique per enum
    result.push_str(&format!(
        "    private enum {}TypeKeys: String, CodingKey {{\n",
        enum_name
    ));
    result.push_str(&format!("        case tag = \"{}\"\n", tag_field));
    result.push_str(&format!("        case content = \"{}\"\n", content_field));
    result.push_str("    }\n\n");

    // Generate VariantType enum for variant names
    result.push_str("    private enum VariantType: String, Codable {\n");
    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }
        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);
        result.push_str(&format!(
            "        case {} = \"{}\"\n",
            swift_case_name, original_variant_name
        ));
    }
    result.push_str("    }\n\n");

    // Generate init(from decoder:)
    result.push_str("    public init(from decoder: Decoder) throws {\n");
    result.push_str(&format!(
        "        let container = try decoder.container(keyedBy: {}TypeKeys.self)\n",
        enum_name
    ));
    result.push_str(
        "        let variantType = try container.decode(VariantType.self, forKey: .tag)\n",
    );
    result.push_str("        \n");
    result.push_str("        switch variantType {\n");

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            specta::datatype::Fields::Unit => {
                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!("            self = .{}\n", swift_case_name));
            }
            specta::datatype::Fields::Unnamed(fields) => {
                if fields.fields().is_empty() {
                    // Empty tuple variant - treat as unit variant
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str(&format!("            self = .{}\n", swift_case_name));
                } else {
                    // TODO: Handle non-empty tuple variants for adjacently tagged
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str("            fatalError(\"Adjacently tagged tuple variants not implemented\")\n");
                }
            }
            specta::datatype::Fields::Named(_) => {
                let struct_name =
                    generate_variant_struct_name(swift, enum_name, original_variant_name);

                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "            let data = try container.decode({}.self, forKey: .content)\n",
                    struct_name
                ));
                result.push_str(&format!("            self = .{}(data)\n", swift_case_name));
            }
        }
    }

    result.push_str("        }\n");
    result.push_str("    }\n\n");

    // Generate encode(to encoder:)
    result.push_str("    public func encode(to encoder: Encoder) throws {\n");
    result.push_str(&format!(
        "        var container = encoder.container(keyedBy: {}TypeKeys.self)\n",
        enum_name
    ));
    result.push_str("        \n");
    result.push_str("        switch self {\n");

    for (original_variant_name, variant) in e.variants() {
        if variant.skip() {
            continue;
        }

        let swift_case_name = swift.naming.convert_enum_case(original_variant_name);

        match variant.fields() {
            specta::datatype::Fields::Unit => {
                result.push_str(&format!("        case .{}:\n", swift_case_name));
                result.push_str(&format!(
                    "            try container.encode(VariantType.{}, forKey: .tag)\n",
                    swift_case_name
                ));
            }
            specta::datatype::Fields::Unnamed(fields) => {
                if fields.fields().is_empty() {
                    // Empty tuple variant - treat as unit variant
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str(&format!(
                        "            try container.encode(VariantType.{}, forKey: .tag)\n",
                        swift_case_name
                    ));
                } else {
                    // TODO: Handle non-empty tuple variants
                    result.push_str(&format!("        case .{}:\n", swift_case_name));
                    result.push_str("            fatalError(\"Adjacently tagged tuple variants not implemented\")\n");
                }
            }
            specta::datatype::Fields::Named(_) => {
                result.push_str(&format!("        case .{}(let data):\n", swift_case_name));
                result.push_str(&format!(
                    "            try container.encode(VariantType.{}, forKey: .tag)\n",
                    swift_case_name
                ));
                result.push_str("            try container.encode(data, forKey: .content)\n");
            }
        }
    }

    result.push_str("        }\n");
    result.push_str("    }\n");
    result.push_str("}\n");

    Ok(result)
}

/// Check if a DataType references the given SpectaID (for detecting recursive types)
fn is_recursive_type_reference(
    ty: &specta::datatype::DataType,
    target_sid: specta::SpectaID,
) -> bool {
    match ty {
        specta::datatype::DataType::Reference(reference) => reference.sid() == target_sid,
        specta::datatype::DataType::Nullable(inner) => {
            is_recursive_type_reference(inner, target_sid)
        }
        specta::datatype::DataType::List(list) => {
            is_recursive_type_reference(list.ty(), target_sid)
        }
        specta::datatype::DataType::Map(map) => {
            is_recursive_type_reference(map.key_ty(), target_sid)
                || is_recursive_type_reference(map.value_ty(), target_sid)
        }
        specta::datatype::DataType::Struct(s) => match s.fields() {
            specta::datatype::Fields::Named(fields) => fields.fields().iter().any(|(_, field)| {
                if let Some(ty) = field.ty() {
                    is_recursive_type_reference(ty, target_sid)
                } else {
                    false
                }
            }),
            specta::datatype::Fields::Unnamed(fields) => fields.fields().iter().any(|field| {
                if let Some(ty) = field.ty() {
                    is_recursive_type_reference(ty, target_sid)
                } else {
                    false
                }
            }),
            _ => false,
        },
        specta::datatype::DataType::Enum(e) => {
            e.variants()
                .iter()
                .any(|(_, variant)| match variant.fields() {
                    specta::datatype::Fields::Named(fields) => {
                        fields.fields().iter().any(|(_, field)| {
                            if let Some(ty) = field.ty() {
                                is_recursive_type_reference(ty, target_sid)
                            } else {
                                false
                            }
                        })
                    }
                    specta::datatype::Fields::Unnamed(fields) => {
                        fields.fields().iter().any(|field| {
                            if let Some(ty) = field.ty() {
                                is_recursive_type_reference(ty, target_sid)
                            } else {
                                false
                            }
                        })
                    }
                    _ => false,
                })
        }
        _ => false,
    }
}

/// Generate raw value for string enum variants
fn generate_string_enum_raw_value(
    variant_name: &str,
    naming: crate::swift::NamingConvention,
) -> String {
    match naming {
        crate::swift::NamingConvention::SnakeCase => variant_name
            .chars()
            .map(|c| if c.is_uppercase() { '_' } else { c })
            .collect::<String>()
            .to_lowercase(),
        _ => variant_name.to_lowercase(), // Default to lowercase
    }
}

/// Generate struct definitions for enum variants with named fields
fn generate_enum_variant_structs(
    swift: &Swift,
    types: &TypeCollection,
    e: &specta::datatype::Enum,
    enum_name: &str,
) -> Result<String> {
    let mut result = String::new();

    for (variant_name, variant) in e.variants() {
        if let specta::datatype::Fields::Named(fields) = variant.fields() {
            if !fields.fields().is_empty() {
                let struct_name = match swift.struct_naming {
                    crate::swift::StructNamingStrategy::AutoRename => {
                        format!("{}{}", enum_name, swift.naming.convert(variant_name))
                    }
                    crate::swift::StructNamingStrategy::KeepOriginal => {
                        swift.naming.convert(variant_name)
                    }
                };

                result.push_str(&format!("public struct {}: Codable {{\n", struct_name));

                let mut field_mappings = Vec::new();

                for (field_name, field) in fields.fields() {
                    let swift_field_name = swift.naming.convert_field(field_name);
                    if let Some(ty) = field.ty() {
                        let field_type = datatype_to_swift(swift, types, ty, vec![], false, None)?;
                        result.push_str(&format!(
                            "    public let {}: {}\n",
                            swift_field_name, field_type
                        ));
                        field_mappings.push((swift_field_name, field_name.to_string()));
                    }
                }

                // Generate custom CodingKeys if field names were converted
                let needs_custom_coding_keys = field_mappings
                    .iter()
                    .any(|(swift_name, rust_name)| swift_name != rust_name);
                if needs_custom_coding_keys {
                    result.push_str("\n    private enum CodingKeys: String, CodingKey {\n");
                    for (swift_name, rust_name) in &field_mappings {
                        result.push_str(&format!(
                            "        case {} = \"{}\"\n",
                            swift_name, rust_name
                        ));
                    }
                    result.push_str("    }\n");
                }

                result.push_str("}\n\n");
            }
        }
    }

    Ok(result)
}
