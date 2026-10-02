// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Acceptance tests for the retained shared-reference packet (VER-50): schema validation
//! and generated wire types.

use quire_verification_contracts::VerificationErrorCode;
use quire_verification_contracts::shared_reference::{
    fixtures, validate_shared_reference_v1, validate_shared_reference_v2, wire_v1, wire_v2,
};
use serde_json::json;

fn minimal_v1_artifact_envelope() -> serde_json::Value {
    json!({
        "wireVersion": "ix.shared-reference/1-draft",
        "kind": "artifact",
        "value": {
            "refVersion": "ix.artifact-ref/2-draft",
            "kind": "source",
            "authority": "ix.test",
            "identity": "test-artifact",
            "digest": format!("sha256:{}", "0".repeat(64)),
            "wire": {"identity": "ix.test.wire", "version": "1"},
        }
    })
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
    let envelope = minimal_v1_artifact_envelope();
    validate_shared_reference_v1(&envelope).expect("minimal v1 artifact envelope must validate");
    let decoded: wire_v1::ProposedSharedReferenceEnvelopeStructuralValidationOnly =
        serde_json::from_value(envelope).expect("minimal v1 artifact envelope must decode");
    let _ = decoded;
}

#[test]
fn validate_shared_reference_v1_rejects_a_malformed_envelope() {
    let error =
        validate_shared_reference_v1(&json!({})).expect_err("empty object must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::InvalidWire);
}
