// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Literal canonical bytes and refusal vectors for FR-001. This file also
//! compiles against the old Serialize boundary for bounded baseline measurement.

use std::collections::BTreeMap;

use quire_canonical::{Encode, FixedShape, nest};
use quire_verification_contracts::{
    VerificationErrorCode, jcs_canonicalize, jcs_equal, jcs_sha256,
};
use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};
use serde_json::{Value, json};

// Keep independent helper assertions running during old/new measurements.
// Only the aggregate at the end of one typed case fails the test.
#[derive(Default)]
struct Checks {
    failures: Vec<String>,
}

impl Checks {
    fn check(&mut self, label: &str, assertion: impl FnOnce()) {
        if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(assertion)) {
            let message = payload.downcast_ref::<String>().map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("non-string assertion panic");
            self.failures.push(format!("{label}: {message}"));
            println!("check={label} status=FAIL");
        } else {
            println!("check={label} status=PASS");
        }
    }

    fn finish(self) {
        assert!(self.failures.is_empty(), "independent checks failed:\n{}", self.failures.join("\n"));
    }
}

fn refused<T: Serialize + Encode>(value: &T) {
    refused_against(value, &json!({"ok": true}));
}

fn refused_against<T: Serialize + Encode, C: Serialize + Encode>(value: &T, control: &C) {
    let mut checks = Checks::default();
    checks.check("canonicalize", || {
        assert_eq!(jcs_canonicalize(value).expect_err("no canonical bytes on refusal").code(),
            VerificationErrorCode::CanonicalizationFailed);
    });
    checks.check("sha256", || {
        assert_eq!(jcs_sha256(value).expect_err("no identity on refusal").code(),
            VerificationErrorCode::CanonicalizationFailed);
    });
    checks.check("equal-left", || {
        assert_eq!(jcs_equal(value, control).expect_err("left operand refuses").code(),
            VerificationErrorCode::CanonicalizationFailed);
    });
    checks.check("equal-right", || {
        assert_eq!(jcs_equal(control, value).expect_err("right operand refuses").code(),
            VerificationErrorCode::CanonicalizationFailed);
    });
    checks.finish();
}

fn accepted_checks<T: Serialize + Encode>(checks: &mut Checks, value: &T, bytes: &[u8]) {
    checks.check("canonicalize", || {
        assert_eq!(jcs_canonicalize(value).expect("canonical bytes"), bytes);
    });
    checks.check("equal-self", || {
        assert!(jcs_equal(value, value).expect("same canonical bytes"));
    });
    checks.check("equal-different", || {
        assert!(!jcs_equal(value, &json!({"different": null})).expect("different bytes"));
    });
}

fn accepted<T: Serialize + Encode>(value: &T, bytes: &[u8]) {
    let mut checks = Checks::default();
    accepted_checks(&mut checks, value, bytes);
    checks.finish();
}

fn accepted_name<T: Serialize + Encode>(value: &T, bytes: &[u8], hash: &str) {
    let equivalent: Value = serde_json::from_slice(bytes).expect("literal name oracle is JSON");
    let mut checks = Checks::default();
    accepted_checks(&mut checks, value, bytes);
    checks.check("equal-string-names-left", || {
        assert!(jcs_equal(value, &equivalent).expect("equivalent string names"));
    });
    checks.check("equal-string-names-right", || {
        assert!(jcs_equal(&equivalent, value).expect("symmetric equivalent names"));
    });
    checks.check("sha256", || {
        assert_eq!(jcs_sha256(value).expect("content identity"), format!("sha256-jcs:{hash}"));
    });
    checks.finish();
}

#[derive(Serialize, FixedShape)]
struct FloatObject<T: FixedShape> {
    x: T,
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_nan_scalar() {
    refused(&f32::NAN);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_nan_array() {
    refused(&[f32::NAN]);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_nan_object() {
    refused(&FloatObject { x: f32::NAN });
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_positive_infinity_scalar() {
    refused(&f32::INFINITY);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_positive_infinity_array() {
    refused(&[f32::INFINITY]);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_positive_infinity_object() {
    refused(&FloatObject { x: f32::INFINITY });
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_negative_infinity_scalar() {
    refused(&f32::NEG_INFINITY);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_negative_infinity_array() {
    refused(&[f32::NEG_INFINITY]);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f32_negative_infinity_object() {
    refused(&FloatObject { x: f32::NEG_INFINITY });
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_nan_scalar() {
    refused(&f64::NAN);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_nan_array() {
    refused(&[f64::NAN]);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_nan_object() {
    refused(&FloatObject { x: f64::NAN });
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_positive_infinity_scalar() {
    refused(&f64::INFINITY);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_positive_infinity_array() {
    refused(&[f64::INFINITY]);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_positive_infinity_object() {
    refused(&FloatObject { x: f64::INFINITY });
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_negative_infinity_scalar() {
    refused(&f64::NEG_INFINITY);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_negative_infinity_array() {
    refused(&[f64::NEG_INFINITY]);
}

/// Trace: FR-001-AC-2
#[test]
fn nonfinite_f64_negative_infinity_object() {
    refused(&FloatObject { x: f64::NEG_INFINITY });
}


#[derive(FixedShape)]
struct RepeatedNames {
    entries: [(String, i32); 3],
}

impl Serialize for RepeatedNames {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.entries.len()))?;
        for (name, value) in &self.entries {
            map.serialize_entry(name, value)?;
        }
        map.end()
    }
}

/// Trace: FR-001-AC-2
#[test]
fn repeated_member_names_refuse_instead_of_retaining_a_member() {
    refused(&RepeatedNames {
        entries: [("b".into(), 1), ("a".into(), 2), ("b".into(), 3)],
    });
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_i64_just_above_refuses() {
    refused(&9_007_199_254_740_993_i64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_i128_just_above_refuses() {
    refused(&i128::from(9_007_199_254_740_993_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_i64_just_above_refuses() {
    refused(&-9_007_199_254_740_993_i64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_i128_just_above_refuses() {
    refused(&-i128::from(9_007_199_254_740_993_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_value_just_above_refuses() {
    refused(&json!(9_007_199_254_740_993_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_value_just_above_refuses() {
    refused(&json!(-9_007_199_254_740_993_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_i64_two_pow_60_refuses() {
    refused(&1_152_921_504_606_846_976_i64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_i128_two_pow_60_refuses() {
    refused(&i128::from(1_152_921_504_606_846_976_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_i64_two_pow_60_refuses() {
    refused(&-1_152_921_504_606_846_976_i64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_i128_two_pow_60_refuses() {
    refused(&-i128::from(1_152_921_504_606_846_976_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_value_two_pow_60_refuses() {
    refused(&json!(1_152_921_504_606_846_976_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_value_two_pow_60_refuses() {
    refused(&json!(-1_152_921_504_606_846_976_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_i64_max_refuses() {
    refused(&9_223_372_036_854_775_807_i64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_i128_max_refuses() {
    refused(&i128::from(9_223_372_036_854_775_807_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_i64_max_refuses() {
    refused(&-9_223_372_036_854_775_807_i64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_i128_max_refuses() {
    refused(&-i128::from(9_223_372_036_854_775_807_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_value_max_refuses() {
    refused(&json!(9_223_372_036_854_775_807_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_negative_value_max_refuses() {
    refused(&json!(-9_223_372_036_854_775_807_i64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_u64_just_above_refuses() {
    refused(&9_007_199_254_740_993_u64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_u128_just_above_refuses() {
    refused(&u128::from(9_007_199_254_740_993_u64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_unsigned_value_just_above_refuses() {
    refused(&json!(9_007_199_254_740_993_u64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_u64_two_pow_60_refuses() {
    refused(&1_152_921_504_606_846_976_u64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_u128_two_pow_60_refuses() {
    refused(&u128::from(1_152_921_504_606_846_976_u64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_unsigned_value_two_pow_60_refuses() {
    refused(&json!(1_152_921_504_606_846_976_u64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_u64_max_refuses() {
    refused(&18_446_744_073_709_551_615_u64);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_u128_max_refuses() {
    refused(&u128::from(18_446_744_073_709_551_615_u64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_unsigned_value_max_refuses() {
    refused(&json!(18_446_744_073_709_551_615_u64));
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_i128_min_refuses() {
    refused(&i128::MIN);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_i128_max_refuses() {
    refused(&i128::MAX);
}

/// Trace: FR-001-AC-2
#[test]
fn numeric_u128_max_refuses() {
    refused(&u128::MAX);
}

/// Trace: FR-001-AC-2
#[cfg(target_pointer_width = "64")]
#[test]
fn numeric_isize_above_refuses() {
    refused(&(9_007_199_254_740_993_isize));
}

/// Trace: FR-001-AC-2
#[cfg(target_pointer_width = "64")]
#[test]
fn numeric_negative_isize_above_refuses() {
    refused(&(-9_007_199_254_740_993_isize));
}

/// Trace: FR-001-AC-2
#[cfg(target_pointer_width = "64")]
#[test]
fn numeric_usize_above_refuses() {
    refused(&(9_007_199_254_740_993_usize));
}

/// Trace: FR-001-AC-2
#[cfg(target_pointer_width = "64")]
#[test]
fn numeric_isize_two_pow_60_refuses() {
    refused(&(1_isize << 60));
}

/// Trace: FR-001-AC-2
#[cfg(target_pointer_width = "64")]
#[test]
fn numeric_negative_isize_two_pow_60_refuses() {
    refused(&(-(1_isize << 60)));
}

/// Trace: FR-001-AC-2
#[cfg(target_pointer_width = "64")]
#[test]
fn numeric_usize_two_pow_60_refuses() {
    refused(&(1_usize << 60));
}

/// Trace: FR-001-AC-2
#[cfg(target_pointer_width = "64")]
#[test]
fn numeric_usize_max_refuses() {
    refused(&(usize::MAX));
}

/// Trace: FR-001-AC-3
#[test]
fn integer_boundaries_strings_and_finite_double_values_keep_literal_bytes() {
    accepted(&9_007_199_254_740_992_u64, b"9007199254740992");
    accepted(&-9_007_199_254_740_992_i64, b"-9007199254740992");
    accepted(&"9007199254740993", b"\"9007199254740993\"");
    accepted(&1_152_921_504_606_846_976_f64, b"1152921504606847000");
    for (value, expected) in [
        (-0.0, "0"), (1e-6, "0.000001"), (1e-7, "1e-7"),
        (1e20, "100000000000000000000"), (1e21, "1e+21"),
    ] {
        accepted(&value, expected.as_bytes());
    }
    accepted(&0.1_f32, b"0.10000000149011612");
}

/// Trace: FR-001-AC-3
#[test]
fn utf16_order_escaping_unicode_and_array_order_match_literal_bytes() {
    let value = json!({"\u{e000}": 1, "\u{1f600}": 2});
    let names = ["\u{1f600}", "\u{e000}"];
    assert_eq!(names[0].chars().next(), Some('\u{1f600}'));
    assert_eq!(names[1].chars().next(), Some('\u{e000}'));
    assert!(names[0].encode_utf16().cmp(names[1].encode_utf16()).is_lt());
    accepted(&value, "{\"\u{1f600}\":2,\"\u{e000}\":1}".as_bytes());
    accepted(&"\"\\\n\0", &[34, 92, 34, 92, 92, 92, 110, 92, 117, 48, 48, 48, 48, 34]);
    accepted(&"é", "\"é\"".as_bytes());
    accepted(&"e\u{301}", "\"e\u{301}\"".as_bytes());
    assert!(!jcs_equal(&"é", &"e\u{301}").expect("no Unicode normalization"));
    accepted(&[2_i32, 1], b"[2,1]");
    accepted(&json!([{"a": 1}, {"a": 1}]), b"[{\"a\":1},{\"a\":1}]");
    assert!(!jcs_equal(&[2_i32, 1], &[1_i32, 2]).expect("array order matters"));
}

#[derive(FixedShape)]
struct SerializationRefusal {
    message: String,
}

impl Serialize for SerializationRefusal {
    fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(&self.message))
    }
}

/// Trace: FR-001-AC-4
#[test]
fn custom_serialization_errors_keep_the_stable_public_code() {
    refused(&SerializationRefusal { message: "fixture refusal".into() });
}

/// Trace: FR-001-AC-5
#[test]
fn content_identity_has_the_full_literal_digest_and_changes_with_content() {
    assert_eq!(
        jcs_sha256(&json!({"a": 1})).expect("identity"),
        "sha256-jcs:015abd7f5cc57a2dd94b7590f04ad8084273905ee33ec5cebeae62276a97f862",
    );
    assert_ne!(jcs_sha256(&json!({"a": 1})).unwrap(), jcs_sha256(&json!({"a": 2})).unwrap());
}

/// Trace: FR-001-AC-3
#[test]
fn u64_max_member_name_keeps_literal_bytes_and_identity() {
accepted_name(&BTreeMap::from([(u64::MAX, 1_i32)]), b"{\"18446744073709551615\":1}",
        "6dabbead8de70faff065e73d49e5658b06ae48cd3f5d3be2d3e4e5d799e5e0af");
}

/// Trace: FR-001-AC-3
#[test]
fn i128_min_member_name_keeps_literal_bytes_and_identity() {
    accepted_name(&BTreeMap::from([(i128::MIN, 1_i32)]), b"{\"-170141183460469231731687303715884105728\":1}",
        "6e6bf45afd1abf91f01de8b3d42199e77de25eecc2ae80d5d60eda322cef488c");
}

/// Trace: FR-001-AC-3
#[test]
fn i128_max_member_name_keeps_literal_bytes_and_identity() {
    accepted_name(&BTreeMap::from([(i128::MAX, 1_i32)]), b"{\"170141183460469231731687303715884105727\":1}",
        "fe8773e9c610c297c486dc451d2266859a69b611794fdf29cad231c76845cc12");
}

/// Trace: FR-001-AC-3
#[test]
fn u128_max_member_name_keeps_literal_bytes_and_identity() {
    accepted_name(&BTreeMap::from([(u128::MAX, 1_i32)]), b"{\"340282366920938463463374607431768211455\":1}",
        "53b86ba145a98fcaad94383fd5e0c09a6f73f2cf69492d016b6b45779100835e");
}

#[derive(FixedShape)]
struct MixedNames {
    integer: u64,
    text: String,
    reverse: bool,
}

impl Serialize for MixedNames {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        if self.reverse {
            map.serialize_entry(&self.text, &2_i32)?;
            map.serialize_entry(&self.integer, &1_i32)?;
        } else {
            map.serialize_entry(&self.integer, &1_i32)?;
            map.serialize_entry(&self.text, &2_i32)?;
        }
        map.end()
    }
}

/// Trace: FR-001-AC-2
#[test]
fn mixed_small_names_integer_first_refuse() {
    refused(&MixedNames { integer: 1_u64, text: (1_u64).to_string(), reverse: false });
}

/// Trace: FR-001-AC-2
#[test]
fn mixed_small_names_string_first_refuse() {
    refused(&MixedNames { integer: 1_u64, text: (1_u64).to_string(), reverse: true });
}

/// Trace: FR-001-AC-2
#[test]
fn mixed_large_names_integer_first_refuse() {
    refused(&MixedNames { integer: u64::MAX, text: (u64::MAX).to_string(), reverse: false });
}

/// Trace: FR-001-AC-2
#[test]
fn mixed_large_names_string_first_refuse() {
    refused(&MixedNames { integer: u64::MAX, text: (u64::MAX).to_string(), reverse: true });
}

struct SingleName<T> {
    name: T,
}

impl<T: FixedShape> FixedShape for SingleName<T> {
    const DEPTH: usize = nest(&[T::DEPTH, <i32 as FixedShape>::DEPTH]);
}

impl<T: Serialize> Serialize for SingleName<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.name, &1_i32)?;
        map.end()
    }
}

/// Trace: FR-001-AC-2
#[test]
fn true_member_name_refuses() {
    refused(&SingleName { name: true });
}

/// Trace: FR-001-AC-2
#[test]
fn false_member_name_refuses() {
    refused(&SingleName { name: false });
}

/// Trace: FR-001-AC-2
#[test]
fn f32_finite_member_name_refuses() {
    refused(&SingleName { name: 1.5_f32 });
}

/// Trace: FR-001-AC-2
#[test]
fn f32_negative_zero_member_name_refuses() {
    refused(&SingleName { name: -0.0_f32 });
}

/// Trace: FR-001-AC-2
#[test]
fn f64_finite_member_name_refuses() {
    refused(&SingleName { name: 1.5_f64 });
}

/// Trace: FR-001-AC-2
#[test]
fn f64_negative_zero_member_name_refuses() {
    refused(&SingleName { name: -0.0_f64 });
}

#[derive(Serialize, FixedShape, Eq, Ord, PartialEq, PartialOrd)]
enum Name {
    Ready,
}

#[derive(Serialize, FixedShape, Eq, Ord, PartialEq, PartialOrd)]
struct TextName(String);

#[derive(Serialize, FixedShape, Eq, Ord, PartialEq, PartialOrd)]
struct IntegerName(u64);

/// Trace: FR-001-AC-3
#[test]
fn char_member_name_keeps_literal_bytes_and_identity() {
accepted_name(&BTreeMap::from([('ö', 1_i32)]), "{\"ö\":1}".as_bytes(),
        "b7c45ab400363886a1d84c8e61ca57a344ce381bce56571e6361a75bcfcc79e4");
}

/// Trace: FR-001-AC-3
#[test]
fn unit_variant_member_name_keeps_literal_bytes_and_identity() {
    accepted_name(&BTreeMap::from([(Name::Ready, 1_i32)]), b"{\"Ready\":1}",
        "d1602dc19bcfa269d0f955fd122cdcd000861ce66ba8d338a397b47ef0ef4113");
}

/// Trace: FR-001-AC-3
#[test]
fn string_newtype_member_name_keeps_literal_bytes_and_identity() {
    accepted_name(&BTreeMap::from([(TextName("n".into()), 1_i32)]), b"{\"n\":1}",
        "2bfd14f43d17fc7cea24e0917a8879b4b2f880b8baeec1b9d90fbaad655e71bd");
}

/// Trace: FR-001-AC-3
#[test]
fn integer_newtype_member_name_keeps_literal_bytes_and_identity() {
    accepted_name(&BTreeMap::from([(IntegerName(u64::MAX), 1_i32)]), b"{\"18446744073709551615\":1}",
        "6dabbead8de70faff065e73d49e5658b06ae48cd3f5d3be2d3e4e5d799e5e0af");
}

/// Trace: FR-001-AC-2
#[test]
fn option_string_some_member_name_refuses() {
    let map: BTreeMap<Option<String>, i32> = BTreeMap::from([(Some("x".to_owned()), 1_i32)]);
    let control = BTreeMap::from([("x", 1_i32)]);
    accepted(&control, b"{\"x\":1}");
    refused_against(&map, &control);
}

/// Trace: FR-001-AC-2
#[test]
fn option_string_none_member_name_refuses() {
    let map: BTreeMap<Option<String>, i32> = BTreeMap::from([(None, 1_i32)]);
    let control = BTreeMap::from([("x", 1_i32)]);
    accepted(&control, b"{\"x\":1}");
    refused_against(&map, &control);
}

/// Trace: FR-001-AC-2
#[test]
fn option_integer_some_member_name_refuses() {
    let map: BTreeMap<Option<i32>, i32> = BTreeMap::from([(Some(1_i32), 1_i32)]);
    let control = BTreeMap::from([("1", 1_i32)]);
    accepted(&control, b"{\"1\":1}");
    refused_against(&map, &control);
}

/// Trace: FR-001-AC-2
#[test]
fn option_integer_none_member_name_refuses() {
    let map: BTreeMap<Option<i32>, i32> = BTreeMap::from([(None, 1_i32)]);
    let control = BTreeMap::from([("1", 1_i32)]);
    accepted(&control, b"{\"1\":1}");
    refused_against(&map, &control);
}

/// Trace: FR-001-AC-7
#[test]
fn original_shared_reference_vectors_keep_canonical_bytes_and_content_identities() {
    use quire_verification_contracts::shared_reference::fixtures;
    let vectors: Vec<Value> = serde_json::from_str(fixtures::CANONICAL_VECTORS).unwrap();
    let hashes = [
        ("core", "70a89fec8afa6a435d5a92f63ba9ce1d01e86b17791629bb788628ee7bfb81d5"),
        ("utf16-and-number", "dc0992959da856d69c0673cd5329199f49507b767a3f278bc686f069fa5dd585"),
        ("member-reordering", "43258cff783fe7036d8a43033f830adfc60ec037382473548ac742b888292777"),
    ];
    assert_eq!(vectors.len(), hashes.len());
    for (vector, (id, hash)) in vectors.iter().zip(hashes) {
        assert_eq!(vector["id"], id);
        let value: Value = serde_json::from_str(vector["input"].as_str().unwrap()).unwrap();
        accepted_name(&value, vector["canonical"].as_str().unwrap().as_bytes(), hash);
    }
}

/// Trace: FR-001-AC-7
#[test]
fn original_operation_catalog_keeps_its_canonical_content_identity() {
    use quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1;
    let value: Value = serde_json::from_str(CHECKED_OPERATION_CATALOG_V1).unwrap();
    assert_eq!(jcs_canonicalize(&value).unwrap().len(), 37_137);
    assert_eq!(jcs_sha256(&value).unwrap(),
        "sha256-jcs:405a883cb41f9a723222b4df73adcc783b2582ea6b764cd1a76d5d7a97da5c5a");
}

/// Trace: FR-001-AC-2
#[cfg(feature = "test-arbitrary-precision")]
#[test]
fn arbitrary_precision_value_integer_text_beyond_u64_refuses() {
    let value: Value = serde_json::from_str("340282366920938463463374607431768211455").unwrap();
    assert!(!value.as_number().unwrap().is_f64(), "feature lane must retain integer text");
    refused(&value);
}
