// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Acceptance tests for the retained shared-reference packet (VER-50): the snapshot guard,
//! schema validation, and generated wire types.

use std::fs;
use std::path::PathBuf;

use quire_verification_contracts::VerificationErrorCode;
use quire_verification_contracts::shared_reference::{
    fixtures, validate_shared_reference_v1, validate_shared_reference_v2,
    verify_shared_reference_snapshot, wire_v2,
};
use serde_json::json;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn minimal_v1_artifact_envelope() -> serde_json::Value {
    json!({
        "wireVersion": "ix.shared-reference/1-draft",
        "kind": "artifact",
        "value": {
            "refVersion": "ix.artifact-ref/2-draft",
            "kind": "source",
            "authority": "ix.test",
            "identity": "test-artifact",
            "revision": {"namespace": "test", "value": "1"},
            "digest": format!("sha256:{}", "0".repeat(64)),
            "wire": {"identity": "ix.test.wire", "version": "1"},
        }
    })
}

#[test]
fn verify_shared_reference_snapshot_passes_for_the_retained_packet() {
    verify_shared_reference_snapshot(&root()).expect("retained snapshot must verify");
}

#[test]
fn verify_shared_reference_snapshot_rejects_an_unlisted_fixture_file() {
    let manifest = include_str!("../contracts/shared-reference-snapshot.sha256");
    let temporary = tempfile::tempdir().expect("temporary snapshot directory");
    for line in manifest.lines().filter(|line| !line.is_empty()) {
        let (_, relative) = line.split_once("  ").expect("manifest line");
        let target = temporary.path().join(relative);
        fs::create_dir_all(target.parent().expect("snapshot parent"))
            .expect("create snapshot directory");
        fs::copy(root().join(relative), target).expect("copy retained file");
    }
    fs::write(
        temporary
            .path()
            .join("fixtures/shared-reference-2-draft/unlisted.json"),
        b"{}",
    )
    .expect("write unlisted snapshot file");
    let error = verify_shared_reference_snapshot(temporary.path())
        .expect_err("unlisted fixture file must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::InvalidWire);
}

#[test]
fn verify_shared_reference_snapshot_rejects_a_mutated_fixture() {
    let manifest = include_str!("../contracts/shared-reference-snapshot.sha256");
    let temporary = tempfile::tempdir().expect("temporary snapshot directory");
    for line in manifest.lines().filter(|line| !line.is_empty()) {
        let (_, relative) = line.split_once("  ").expect("manifest line");
        let target = temporary.path().join(relative);
        fs::create_dir_all(target.parent().expect("snapshot parent"))
            .expect("create snapshot directory");
        fs::copy(root().join(relative), target).expect("copy retained file");
    }
    fs::write(
        temporary
            .path()
            .join("fixtures/shared-reference-2-draft/trace.json"),
        b"{}",
    )
    .expect("mutate retained fixture");
    let error = verify_shared_reference_snapshot(temporary.path())
        .expect_err("mutated fixture must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::IdentityMismatch);
}

#[test]
fn validate_shared_reference_v2_accepts_the_retained_semantic_fixture() {
    let semantic: serde_json::Value =
        serde_json::from_str(fixtures::SEMANTIC).expect("retained semantic fixture is JSON");
    validate_shared_reference_v2(&semantic).expect("retained semantic fixture must validate");
    let decoded: wire_v2::SharedArtifactAndSemanticReferenceAmendmentStructuralValidationOnly =
        serde_json::from_value(semantic).expect("retained semantic fixture must decode");
    let _ = decoded;
}

#[test]
fn validate_shared_reference_v2_rejects_a_malformed_envelope() {
    let error =
        validate_shared_reference_v2(&json!({})).expect_err("empty object must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::InvalidWire);
}

#[test]
fn validate_shared_reference_v1_accepts_a_minimal_artifact_envelope() {
    validate_shared_reference_v1(&minimal_v1_artifact_envelope())
        .expect("minimal v1 artifact envelope must validate");
}

#[test]
fn validate_shared_reference_v1_rejects_a_malformed_envelope() {
    let error =
        validate_shared_reference_v1(&json!({})).expect_err("empty object must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::InvalidWire);
}
