// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Encode caller classes and traversal safety for FR-001 and NFR-001.

use std::cell::Cell;

use quire_canonical::{Encode, Error, FixedShape, Limits, Sink, Writer, drop_value, read};
use quire_verification_contracts::{
    MAX_ARRAY_ITEMS, MAX_JSON_BYTES, MAX_JSON_DEPTH, VerificationErrorCode,
    jcs_canonicalize, jcs_equal, jcs_sha256, validate_resource_envelope,
};
use serde_json::{Value, json};

struct ArrayChain {
    depth: usize,
}

impl Encode for ArrayChain {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        for _ in 0..self.depth { writer.begin_array()?; }
        writer.null()?;
        for _ in 0..self.depth { writer.end_array()?; }
        Ok(())
    }
}

struct Observed<'a> {
    calls: &'a Cell<usize>,
    refuse: bool,
}

impl Encode for Observed<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        self.calls.set(self.calls.get() + 1);
        if self.refuse { return Err(Error::Serialize("observed refusal".into())); }
        writer.null()
    }
}

/// Trace: FR-001-AC-4
#[test]
fn equality_refuses_left_before_invoking_right_and_reports_right_refusal() {
    let left_calls = Cell::new(0);
    let right_calls = Cell::new(0);
    let left = Observed { calls: &left_calls, refuse: true };
    let right = Observed { calls: &right_calls, refuse: false };
    assert_eq!(jcs_equal(&left, &right).unwrap_err().code(), VerificationErrorCode::CanonicalizationFailed);
    assert_eq!(left_calls.get(), 1);
    assert_eq!(right_calls.get(), 0);
    let left = Observed { calls: &left_calls, refuse: false };
    let right = Observed { calls: &right_calls, refuse: true };
    assert_eq!(jcs_equal(&left, &right).unwrap_err().code(), VerificationErrorCode::CanonicalizationFailed);
    assert_eq!(left_calls.get(), 2);
    assert_eq!(right_calls.get(), 1);
}

#[derive(serde::Serialize, FixedShape)]
struct Dto {
    a: u8,
}

fn generic_encode<T: Encode + ?Sized>(value: &T) -> Vec<u8> {
    jcs_canonicalize(value).unwrap()
}

/// Trace: FR-001-AC-1
#[test]
fn fixed_shape_value_document_noderef_iterative_and_unsized_callers_compile_and_encode() {
    assert_eq!(generic_encode(&Dto { a: 1 }), b"{\"a\":1}");
    assert_eq!(generic_encode(&json!({"a": 1})), b"{\"a\":1}");
    let document = read(b"{\"a\":1}", 7).unwrap();
    assert_eq!(generic_encode(&document), b"{\"a\":1}");
    assert_eq!(generic_encode(&document.root()), b"{\"a\":1}");
    assert_eq!(generic_encode(&ArrayChain { depth: 2 }), b"[[null]]");
    assert_eq!(generic_encode("text"), b"\"text\"");
}

/// Trace: FR-001-AC-6
#[test]
fn canonicalization_does_not_apply_ingestion_depth_array_or_byte_caps() {
    let mut value = Value::Null;
    for _ in 0..MAX_JSON_DEPTH + 1 { value = Value::Array(vec![value]); }
    let mut expected = vec![b'['; MAX_JSON_DEPTH + 1];
    expected.extend_from_slice(b"null");
    expected.extend(std::iter::repeat_n(b']', MAX_JSON_DEPTH + 1));
    assert_eq!(jcs_canonicalize(&value).unwrap(), expected);
    assert_eq!(validate_resource_envelope(&value).unwrap_err().code(), VerificationErrorCode::JsonNestingTooDeep);
    drop_value(value);
    let value = Value::Array(vec![Value::Null; MAX_ARRAY_ITEMS + 1]);
    let bytes = jcs_canonicalize(&value).unwrap();
    assert_eq!(bytes.len(), 5 * (MAX_ARRAY_ITEMS + 1) + 1);
    assert!(bytes.starts_with(b"[null,null,"));
    assert!(bytes.ends_with(b",null]"));
    assert_eq!(validate_resource_envelope(&value).unwrap_err().code(), VerificationErrorCode::JsonArrayTooLarge);
    let text = "x".repeat(MAX_JSON_BYTES + 1);
    let bytes = jcs_canonicalize(&text).unwrap();
    assert_eq!(bytes.len(), MAX_JSON_BYTES + 3);
    assert_eq!(bytes.first(), Some(&b'"'));
    assert_eq!(bytes.last(), Some(&b'"'));
    assert!(bytes[1..bytes.len() - 1].iter().all(|&byte| byte == b'x'));
    assert_eq!(validate_resource_envelope(&Value::String(text)).unwrap_err().code(), VerificationErrorCode::JsonDocumentTooLarge);
}

/// Trace: FR-001-AC-1, FR-001-AC-3, FR-001-AC-6
#[test]
fn deep_value_and_iterative_arrays_agree_on_small_and_large_native_stacks() {
    const DEPTH: usize = 100_000;
    // Independent Python hashlib over 100000 '[' + b'null' + 100000 ']'.
    const IDENTITY: &str = "sha256-jcs:0123e98d2df3f5d14fca4161772124d717f3b1734874c5b69a79fac186698f2f";
    let mut outcomes = Vec::new();
    for stack_size in [512 * 1024, 8 * 1024 * 1024] {
        outcomes.push(std::thread::Builder::new().stack_size(stack_size).spawn(|| {
            let events = ArrayChain { depth: DEPTH };
            let mut value = Value::Null;
            for _ in 0..DEPTH { value = Value::Array(vec![value]); }
            let mut expected = vec![b'['; DEPTH];
            expected.extend_from_slice(b"null");
            expected.extend(std::iter::repeat_n(b']', DEPTH));
            let event_bytes = jcs_canonicalize(&events);
            let value_bytes = jcs_canonicalize(&value);
            let forward_equal = jcs_equal(&events, &value);
            let reverse_equal = jcs_equal(&value, &events);
            let event_digest = jcs_sha256(&events);
            let value_digest = jcs_sha256(&value);
            // Dispose of recursive Value before any assertion can unwind.
            drop_value(value);
            assert_eq!(event_bytes.unwrap(), expected);
            assert_eq!(value_bytes.unwrap(), expected);
            assert!(forward_equal.unwrap());
            assert!(reverse_equal.unwrap());
            let event_digest = event_digest.unwrap();
            assert_eq!(event_digest, IDENTITY);
            assert_eq!(value_digest.unwrap(), IDENTITY);
            (expected, event_digest)
        }).unwrap().join().expect("depth does not consume native stack"));
    }
    assert_eq!(outcomes[0], outcomes[1]);
}

/// Trace: FR-001-AC-6
#[test]
fn deep_object_chain_uses_authoritative_output_byte_accounting() {
    const DEPTH: usize = 10_000;
    let mut input = Vec::new();
    for _ in 0..DEPTH { input.extend_from_slice(b"{\"x\":"); }
    input.extend_from_slice(b"null");
    input.extend(std::iter::repeat_n(b'}', DEPTH));
    let bound = u64::try_from(input.len()).unwrap();
    let document = read(&input, bound).unwrap();
    assert_eq!(jcs_canonicalize(&document).unwrap(), input);
    assert_eq!(quire_canonical::to_vec(&document, Limits::new(bound)).unwrap(), input);
    assert!(matches!(quire_canonical::to_vec(&document, Limits::new(bound - 1)), Err(Error::Limit(_))));
}
