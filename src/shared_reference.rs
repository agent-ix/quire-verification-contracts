// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The retained shared-reference packet: draft-1 and draft-2 schemas, their derived Rust
//! wire types, the fourteen amendment fixtures, and the byte-identical snapshot guard.
//!
//! Extracted from private `quire-verification` (VER-50), alongside the E02 boundary
//! (PLAT-861): `quire-verification` depends on this crate and re-exports this module
//! unchanged at `contracts::shared_reference`, so its strict shared-reference reader keeps
//! validating and decoding schema-derived types without holding a second copy of any
//! schema, fixture, or codegen. See `contracts/shared-reference-UPSTREAM.md` for exact
//! provenance and `contracts/README.md` for the packet's scope.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};
use std::sync::OnceLock;

use jsonschema::{Draft, Validator};
use serde_json::Value;
use sha2::{Digest, Sha256};

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

/// Verifies the retained two-schema, fourteen-fixture snapshot manifest.
///
/// # Errors
/// Returns an I/O or identity error when a retained file is missing, unsafe,
/// unlisted, or byte-different from the selected private upstream revision.
pub fn verify_shared_reference_snapshot(root: &Path) -> Result<(), VerificationError> {
    const MANIFEST: &str = include_str!("../contracts/shared-reference-snapshot.sha256");
    let canonical_root = fs::canonicalize(root).map_err(|error| {
        refusal(VerificationErrorCode::IoFailure, error.to_string())
            .with_context("path", root.display().to_string())
    })?;
    let mut expected_paths = BTreeSet::new();
    let mut entries = 0_usize;
    for line in MANIFEST.lines().filter(|line| !line.is_empty()) {
        let (expected, relative) = line.split_once("  ").ok_or_else(invalid_wire)?;
        let relative = Path::new(relative);
        if relative.is_absolute()
            || relative.components().any(|part| {
                matches!(
                    part,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(invalid_wire());
        }
        expected_paths.insert(relative.to_path_buf());
        let path = root.join(relative);
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            refusal(VerificationErrorCode::IoFailure, error.to_string())
                .with_context("path", relative.display().to_string())
        })?;
        let canonical_path = fs::canonicalize(&path).map_err(|error| {
            refusal(VerificationErrorCode::IoFailure, error.to_string())
                .with_context("path", relative.display().to_string())
        })?;
        if !metadata.file_type().is_file() || !canonical_path.starts_with(&canonical_root) {
            return Err(invalid_wire().with_context("path", relative.display().to_string()));
        }
        let bytes = fs::read(&path).map_err(|error| {
            refusal(VerificationErrorCode::IoFailure, error.to_string())
                .with_context("path", relative.display().to_string())
        })?;
        let actual = format!("{:x}", Sha256::digest(&bytes));
        if actual != expected {
            return Err(refusal(
                VerificationErrorCode::IdentityMismatch,
                "retained shared-reference snapshot digest mismatch",
            )
            .with_context("path", relative.display().to_string())
            .with_context("expected", expected)
            .with_context("actual", actual));
        }
        entries += 1;
    }
    if entries != 16 {
        return Err(invalid_wire().with_context("manifest_entries", entries.to_string()));
    }
    let fixture_directory = root.join("fixtures/shared-reference-2-draft");
    let mut fixture_entries = 0_usize;
    for entry in fs::read_dir(&fixture_directory).map_err(|error| {
        refusal(VerificationErrorCode::IoFailure, error.to_string())
            .with_context("path", fixture_directory.display().to_string())
    })? {
        let entry = entry.map_err(|error| {
            refusal(VerificationErrorCode::IoFailure, error.to_string())
                .with_context("path", fixture_directory.display().to_string())
        })?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|_| invalid_wire())?
            .to_path_buf();
        if !entry
            .file_type()
            .map_err(|error| {
                refusal(VerificationErrorCode::IoFailure, error.to_string())
                    .with_context("path", relative.display().to_string())
            })?
            .is_file()
            || !expected_paths.contains(&relative)
        {
            return Err(invalid_wire().with_context("path", relative.display().to_string()));
        }
        fixture_entries += 1;
    }
    if fixture_entries != 14 {
        return Err(invalid_wire().with_context("fixture_entries", fixture_entries.to_string()));
    }
    Ok(())
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

fn invalid_wire() -> VerificationError {
    refusal(
        VerificationErrorCode::InvalidWire,
        "shared-reference wire value is malformed",
    )
}
