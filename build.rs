// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Generates Rust wire types from the authoritative E02 JSON Schema (FR-002) and, separately,
//! from the retained shared-reference draft-1 and draft-2 schemas (FR-016, VER-50).

use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use schemars::schema::RootSchema;
use serde_json::Value;
use typify::{TypeSpace, TypeSpaceSettings};

const E02_SCHEMA_PATH: &str = "contracts/e02-draft-2.schema.json";
const SHARED_V1_SCHEMA_PATH: &str = "contracts/shared-reference-1-draft.schema.json";
const SHARED_V2_SCHEMA_PATH: &str = "contracts/shared-reference-2-draft/schema.json";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=Cargo.toml");
    for path in [
        E02_SCHEMA_PATH,
        SHARED_V1_SCHEMA_PATH,
        SHARED_V2_SCHEMA_PATH,
    ] {
        println!("cargo:rerun-if-changed={path}");
    }

    let authored_schema: Value = serde_json::from_str(&fs::read_to_string(E02_SCHEMA_PATH)?)?;
    let contract_version = authored_schema
        .pointer("/$defs/TechniqueDefinition/properties/contractVersion/const")
        .and_then(Value::as_str)
        .ok_or("schema contract version is missing")?
        .to_owned();
    let input_kinds = authored_schema
        .pointer("/$defs/Input/oneOf")
        .and_then(Value::as_array)
        .ok_or("schema input kinds are missing")?
        .iter()
        .map(|entry| {
            entry
                .pointer("/properties/kind/const")
                .and_then(Value::as_str)
                .ok_or("schema input kind is missing")
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let constants = format!(
        "pub const CONTRACT_VERSION: &str = {contract_version:?}; pub const INPUT_KINDS: &[&str] = &{input_kinds:?};"
    );

    let mut schema = authored_schema;
    strip_non_constructive_keywords(&mut schema);
    localize_shared_references_for_codegen(&mut schema);
    let root: RootSchema = serde_json::from_value(schema)?;
    let mut settings = TypeSpaceSettings::default();
    settings.with_map_type("::std::collections::BTreeMap".to_owned());
    let mut type_space = TypeSpace::new(&settings);

    let mut shared: Value = serde_json::from_str(&fs::read_to_string(SHARED_V2_SCHEMA_PATH)?)?;
    strip_non_constructive_keywords(&mut shared);
    relax_alternative_digest_codegen(&mut shared)?;
    let shared: RootSchema = serde_json::from_value(shared)?;
    type_space.add_ref_types(shared.definitions)?;

    type_space.add_root_schema(root)?;

    let output =
        PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?).join("e02_contracts.rs");
    fs::write(output, format!("{constants} {}", type_space.to_stream()))?;

    generate_shared_reference_types(SHARED_V1_SCHEMA_PATH, "shared_reference_v1.rs")?;
    generate_shared_reference_types(SHARED_V2_SCHEMA_PATH, "shared_reference_v2.rs")?;
    Ok(())
}

// Generates the shared-reference wire types into their own root `TypeSpace`, independent of
// the E02 `TypeSpace` above. The E02 codegen above folds the same draft-2 fragment's `$defs`
// into E02's `TypeSpace` only to resolve `$ref`s reachable from E02's own contracts; it does
// not produce a public `wire_v1`/`wire_v2` module. This is that module's own generation.
fn generate_shared_reference_types(
    schema_path: &str,
    output_name: &str,
) -> Result<(), Box<dyn Error>> {
    let mut schema: Value = serde_json::from_str(&fs::read_to_string(schema_path)?)?;
    strip_non_constructive_keywords(&mut schema);
    if schema_path == SHARED_V2_SCHEMA_PATH {
        relax_alternative_digest_codegen(&mut schema)?;
    }
    let root: RootSchema = serde_json::from_value(schema)?;
    let mut settings = TypeSpaceSettings::default();
    settings.with_map_type("::std::collections::BTreeMap".to_owned());
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_root_schema(root)?;

    let output =
        PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?).join(output_name);
    fs::write(output, type_space.to_stream().to_string())?;
    Ok(())
}

// Typify 0.7 does not retrieve external schemas. For code generation only,
// resolve the authored URN to definitions loaded below from the exact retained
// schema. Runtime validation keeps the external reference and registry.
fn localize_shared_references_for_codegen(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            const PREFIX: &str = "urn:ix:shared-reference:2-draft#/$defs/";
            let localized = fields
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|reference| reference.strip_prefix(PREFIX))
                .map(|definition| Value::String(format!("#/$defs/{definition}")));
            if let Some(localized) = localized {
                fields.insert("$ref".to_owned(), localized);
            }
            for child in fields.values_mut() {
                localize_shared_references_for_codegen(child);
            }
        }
        Value::Array(items) => {
            for child in items {
                localize_shared_references_for_codegen(child);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

// Typify unifies the digest property shared by the two CanonicalIdentity
// alternatives. Their selected wire syntaxes have different exact lengths, so
// retaining either branch's length/pattern here would make the other valid
// branch impossible to deserialize. The complete runtime schema remains the
// validator; generated construction retains the branch shape and domain enums.
fn relax_alternative_digest_codegen(schema: &mut Value) -> Result<(), Box<dyn Error>> {
    let alternatives = schema
        .pointer_mut("/$defs/CanonicalIdentity/oneOf")
        .and_then(Value::as_array_mut)
        .ok_or("canonical identity alternatives are missing")?;
    for alternative in alternatives {
        let digest = alternative
            .pointer_mut("/properties/digest")
            .and_then(Value::as_object_mut)
            .ok_or("canonical identity digest is missing")?;
        digest.remove("minLength");
        digest.remove("maxLength");
        digest.remove("pattern");
    }
    Ok(())
}

// Conditional JSON Schema assertions constrain valid values but cannot be
// represented directly as constructive Rust types. Runtime validation retains
// the complete authoritative schema, including these removed codegen keywords.
fn strip_non_constructive_keywords(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            fields.remove("if");
            fields.remove("then");
            fields.remove("else");
            let contains_only_assertions = fields
                .get("allOf")
                .and_then(Value::as_array)
                .is_some_and(|entries| {
                    !entries.is_empty()
                        && entries.iter().all(|entry| {
                            entry.as_object().is_some_and(|assertion| {
                                assertion.contains_key("contains")
                                    && assertion.keys().all(|keyword| {
                                        matches!(
                                            keyword.as_str(),
                                            "contains" | "minContains" | "maxContains"
                                        )
                                    })
                            })
                        })
                });
            if contains_only_assertions {
                fields.remove("allOf");
            }
            for child in fields.values_mut() {
                strip_non_constructive_keywords(child);
            }
        }
        Value::Array(items) => {
            for child in items {
                strip_non_constructive_keywords(child);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}
