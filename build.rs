// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) Agent IX

//! Generates Rust wire types from the authoritative E02 JSON Schema (FR-002).

use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use schemars::schema::RootSchema;
use serde_json::Value;
use typify::{TypeSpace, TypeSpaceSettings};

fn main() -> Result<(), Box<dyn Error>> {
    const SCHEMA_PATH: &str = "contracts/e02-draft-1.schema.json";
    println!("cargo:rerun-if-changed={SCHEMA_PATH}");

    let authored_schema: Value = serde_json::from_str(&fs::read_to_string(SCHEMA_PATH)?)?;
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
    let mut constructive_schema = authored_schema;
    strip_non_constructive_keywords(&mut constructive_schema);
    let root: RootSchema = serde_json::from_value(constructive_schema)?;
    let mut settings = TypeSpaceSettings::default();
    settings.with_map_type("::std::collections::BTreeMap".to_owned());
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_root_schema(root)?;

    let output =
        PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?).join("e02_contracts.rs");
    let constants = format!(
        "pub const CONTRACT_VERSION: &str = {contract_version:?}; pub const INPUT_KINDS: &[&str] = &{input_kinds:?};"
    );
    fs::write(output, format!("{constants} {}", type_space.to_stream()))?;
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
