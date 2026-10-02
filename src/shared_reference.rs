// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The retained shared-reference packet: draft-1 and draft-2 schemas, their derived Rust
//! wire types, and the fourteen amendment fixtures.
//!
//! Extracted from private `quire-verification` (VER-50), alongside the E02 boundary
//! (PLAT-861): `quire-verification` depends on this crate and re-exports this module
//! unchanged at `contracts::shared_reference`, so its strict shared-reference reader keeps
//! validating and decoding schema-derived types without holding a second copy of any
//! schema, fixture, or codegen. See `contracts/README.md` for the packet's scope.

use std::sync::OnceLock;

use jsonschema::{Draft, Validator};
use serde_json::Value;

use crate::{VerificationError, VerificationErrorCode};

/// Rust wire types generated from the retained version-1 schema.
#[allow(missing_docs, clippy::all, clippy::pedantic)]
pub mod wire_v1 {
    include!(concat!(env!("OUT_DIR"), "/shared_reference_v1.rs"));
}

/// Rust wire types generated from the retained version-2 schema.
#[allow(missing_docs, clippy::all, clippy::pedantic)]
pub mod wire_v2 {
    include!(concat!(env!("OUT_DIR"), "/shared_reference_v2.rs"));
}

/// The fourteen retained amendment fixtures, embedded at compile time.
///
/// Downstream consumers read fixture bytes through these constants instead of a runtime
/// filesystem path, so exercising them needs no second copy of the fixture packet.
#[allow(missing_docs)]
pub mod fixtures {
    pub const ADVERSE: &str = include_str!("../fixtures/shared-reference-2-draft/adverse.json");
    pub const CANONICAL_VECTORS: &str =
        include_str!("../fixtures/shared-reference-2-draft/canonical-vectors.json");
    pub const CORE: &str = include_str!("../fixtures/shared-reference-2-draft/core.json");
    pub const CORE_SCHEMA: &str =
        include_str!("../fixtures/shared-reference-2-draft/core.schema.json");
    pub const FAULT_MODEL: &str =
        include_str!("../fixtures/shared-reference-2-draft/fault-model.json");
    pub const OBSERVATION: &str =
        include_str!("../fixtures/shared-reference-2-draft/observation.json");
    pub const ORACLE: &str = include_str!("../fixtures/shared-reference-2-draft/oracle.json");
    pub const PROFILE: &str = include_str!("../fixtures/shared-reference-2-draft/profile.json");
    pub const RAW_DUPLICATE: &str =
        include_str!("../fixtures/shared-reference-2-draft/raw-duplicate.json");
    pub const REVIEW_DISPOSITION: &str =
        include_str!("../fixtures/shared-reference-2-draft/review-disposition.json");
    pub const REVIEW_PROCEDURE: &str =
        include_str!("../fixtures/shared-reference-2-draft/review-procedure.json");
    pub const ROLES: &str = include_str!("../fixtures/shared-reference-2-draft/roles.json");
    pub const SEMANTIC: &str = include_str!("../fixtures/shared-reference-2-draft/semantic.json");
    pub const TRACE: &str = include_str!("../fixtures/shared-reference-2-draft/trace.json");
}

/// Validates one decoded version-1 envelope against the retained schema.
///
/// # Errors
/// Returns `invalid_wire` when the value violates the retained version-1 schema, or a
/// schema-compilation failure if the retained schema itself fails to compile.
pub fn validate_shared_reference_v1(value: &Value) -> Result<(), VerificationError> {
    validate_schema(v1_validator()?, value)
}

/// Validates one decoded version-2 envelope against the retained schema.
///
/// # Errors
/// Returns `invalid_wire` when the value violates the retained version-2 schema, or a
/// schema-compilation failure if the retained schema itself fails to compile.
pub fn validate_shared_reference_v2(value: &Value) -> Result<(), VerificationError> {
    validate_schema(v2_validator()?, value)
}

fn validate_schema(validator: &Validator, value: &Value) -> Result<(), VerificationError> {
    if validator.is_valid(value) {
        return Ok(());
    }
    let detail = validator
        .iter_errors(value)
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    Err(refusal(
        VerificationErrorCode::InvalidWire,
        format!("shared-reference schema violation: {detail}"),
    ))
}

fn v1_validator() -> Result<&'static Validator, VerificationError> {
    static VALIDATOR: OnceLock<Result<Validator, String>> = OnceLock::new();
    cached_validator(
        &VALIDATOR,
        include_str!("../contracts/shared-reference-1-draft.schema.json"),
    )
}

fn v2_validator() -> Result<&'static Validator, VerificationError> {
    static VALIDATOR: OnceLock<Result<Validator, String>> = OnceLock::new();
    cached_validator(
        &VALIDATOR,
        include_str!("../contracts/shared-reference-2-draft/schema.json"),
    )
}

fn cached_validator(
    cell: &'static OnceLock<Result<Validator, String>>,
    schema: &str,
) -> Result<&'static Validator, VerificationError> {
    match cell.get_or_init(|| {
        let value: Value = serde_json::from_str(schema).map_err(|error| error.to_string())?;
        jsonschema::options()
            .with_draft(Draft::Draft202012)
            .build(&value)
            .map_err(|error| error.to_string())
    }) {
        Ok(validator) => Ok(validator),
        Err(detail) => Err(refusal(VerificationErrorCode::InvalidWire, detail.clone())),
    }
}

fn refusal(code: VerificationErrorCode, message: impl Into<Box<str>>) -> VerificationError {
    VerificationError::new(code, message)
}
