// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! End-to-end tests for the public E02 verification contract boundary.

use quire_verification_contracts::{
    MAX_ARRAY_ITEMS, MAX_JSON_BYTES, MAX_JSON_DEPTH, PublicContract, VerificationErrorCode,
    contract_version, input_kinds, jcs_canonicalize, jcs_equal, jcs_sha256, parse_bounded_json,
    validate_contract, validate_resource_envelope, validate_schema_definition,
};
use serde_json::json;

fn minimal_tool_capability() -> serde_json::Value {
    json!({
        "contractVersion": "quire-verification/e02-draft-2",
        "kind": "ToolCapability",
        "capabilityId": "test-capability",
        "techniqueId": "test-technique",
        "qualification": "provisional",
        "inputKinds": ["state"],
        "profiles": ["test-profile"],
        "prerequisites": [],
        "runner": {
            "kind": "manual",
            "adapterId": "test-adapter",
            "adapterVersion": "0.1.0",
        },
        "environmentPermissions": [],
        "nativeArtifactInterface": {},
    })
}

#[test]
fn contract_version_matches_schema_generated_constant() {
    assert_eq!(contract_version(), "quire-verification/e02-draft-2");
}

#[test]
fn input_kinds_lists_the_five_public_kinds() {
    let kinds = input_kinds();
    for expected in ["state", "relational", "trace", "fault", "manual"] {
        assert!(kinds.contains(&expected), "missing input kind {expected}");
    }
}

#[test]
fn valid_tool_capability_round_trips_through_validate_contract() {
    let value = minimal_tool_capability();
    let contract = validate_contract(value).expect("minimal tool capability must validate");
    assert!(matches!(contract, PublicContract::ToolCapability(_)));
}

#[test]
fn unknown_contract_kind_is_rejected() {
    let mut value = minimal_tool_capability();
    value["kind"] = json!("NotAContract");
    let error = validate_contract(value).expect_err("unknown kind must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::SchemaViolation);
}

#[test]
fn oversized_json_document_is_rejected() {
    let oversized = vec![b'a'; MAX_JSON_BYTES + 1];
    let error = parse_bounded_json(&oversized).expect_err("oversized input must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::JsonDocumentTooLarge);
}

#[test]
fn deeply_nested_json_is_rejected() {
    let mut value = json!(1);
    for _ in 0..=MAX_JSON_DEPTH + 1 {
        value = json!([value]);
    }
    let error = validate_resource_envelope(&value).expect_err("deep nesting must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::JsonNestingTooDeep);
}

#[test]
fn oversized_array_is_rejected() {
    let value = json!(vec![0_u32; MAX_ARRAY_ITEMS + 1]);
    let error = validate_resource_envelope(&value).expect_err("oversized array must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::JsonArrayTooLarge);
}

#[test]
fn jcs_canonicalize_is_key_order_independent() {
    let left = json!({"b": 1, "a": 2});
    let right = json!({"a": 2, "b": 1});
    assert!(jcs_equal(&left, &right).expect("both values are in the JCS domain"));
    assert_eq!(
        jcs_canonicalize(&left).expect("left canonicalizes"),
        jcs_canonicalize(&right).expect("right canonicalizes"),
    );
}

#[test]
fn jcs_sha256_is_deterministic_and_domain_separated() {
    let value = json!({"kind": "example"});
    let digest = jcs_sha256(&value).expect("value canonicalizes");
    assert!(digest.starts_with("sha256-jcs:"));
    assert_eq!(
        digest,
        jcs_sha256(&value).expect("value canonicalizes again")
    );
}

#[test]
fn every_verification_error_code_round_trips_through_its_stable_string() {
    for code in VerificationErrorCode::all() {
        assert_eq!(VerificationErrorCode::from_code(code.as_str()), Some(*code));
    }
}

/// `SharedArtifactEnvelope.value` is a `$ref` into the externally retained
/// `urn:ix:shared-reference:2-draft` schema, resolved at build time (typify codegen, via
/// `wire::SharedArtifactEnvelope`) and independently at runtime through the
/// `jsonschema::Registry` this crate builds in `build_contract_validator`. An empty envelope
/// must fail on the *external* schema's own required properties, proving the registry
/// resolved the cross-schema reference rather than treating it as opaque or absent.
#[test]
fn shared_artifact_envelope_resolves_the_external_shared_reference_registry() {
    let value = json!({
        "wireVersion": "ix.shared-reference/2-draft",
        "kind": "artifact",
        "value": {},
    });
    let error = validate_schema_definition("SharedArtifactEnvelope", &value)
        .expect_err("an empty artifact value must fail the external ArtifactRef schema");
    assert_eq!(error.code(), VerificationErrorCode::SchemaViolation);
    let message = error.to_string();
    assert!(
        message.contains("authority") || message.contains("required"),
        "expected a property-level failure from the external ArtifactRef schema, got: {message}"
    );
}

#[test]
fn unknown_schema_definition_is_rejected() {
    let error = validate_schema_definition("NotADefinition", &json!({}))
        .expect_err("an unknown definition name must be rejected");
    assert_eq!(error.code(), VerificationErrorCode::UnknownSchemaDefinition);
}
