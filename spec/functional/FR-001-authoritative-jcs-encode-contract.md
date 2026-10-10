---
id: FR-001
title: "Canonicalize Encode values through the authoritative JCS encoder"
type: FR
relationships:
  - target: ix://agent-ix/quire-canonical
    type: uses
---
# FR-001: Canonicalize Encode values through the authoritative JCS encoder

## Description

When a caller invokes a public `jcs_*` helper, QVC SHALL encode its input
through quire-canonical's `Encode` contract.

This is the proposed QSL-693 route (b), replacing the public arbitrary
Serialize contract. It deliberately narrows admissible Rust types and adopts
the authoritative duplicate-name and integer-magnitude refusals. It does not
claim preservation of the old encoder's first-duplicate retention or its
out-of-range integer casts.

## Inputs

The proposed public signatures are:

```rust
pub fn jcs_canonicalize<T: quire_canonical::Encode + ?Sized>(
    value: &T,
) -> Result<Vec<u8>, VerificationError>;
pub fn jcs_equal<L: quire_canonical::Encode + ?Sized,
                 R: quire_canonical::Encode + ?Sized>(
    left: &L, right: &R,
) -> Result<bool, VerificationError>;
pub fn jcs_sha256<T: quire_canonical::Encode + ?Sized>(
    value: &T,
) -> Result<String, VerificationError>;
```

`Serialize` alone no longer qualifies. `Value` inputs use the authoritative
Value feature, fixed-depth DTOs use genuine `FixedShape`, and dynamic-depth
types use explicit-stack `Encode`; see [NFR-001](../non-functional/NFR-001-jcs-depth-safety.md).
These bounds also admit already supported unsized `Encode` types such as `str`.

## Outputs

Complete UTF-8 canonical bytes, equality of complete canonical bytes, or the
existing content identity `sha256-jcs:` followed by 64 lowercase hex digits
of SHA-256 over canonical bytes alone. On refusal there is no successful
byte vector, equality result, or digest.

## Behavior

1. QVC SHALL obtain canonical bytes from `quire_canonical::to_vec`.
2. QVC SHALL pass `Limits::new(u64::MAX)` to the authoritative encoder.
   This supplies no new QVC policy ceiling. Canonical's documented object
   buffer representation bound, allocation failures, and other authoritative
   refusals still apply; this is not a promise of infinite resources.
3. QVC SHALL map every returned canonical encoding error to the existing
   public code `canonicalization_failed`. Diagnostic text may describe the
   authoritative cause; old third-party diagnostic wording is not retained.
4. QVC SHALL compare complete successfully encoded bytes for equality.
5. If the left equality operand refuses, QVC SHALL return that refusal
   before invoking the right operand's `Encode` implementation.
6. QVC SHALL compute `jcs_sha256` as the existing textual prefix plus
   SHA-256 of canonical bytes alone. The prefix is not hashed; a
   length-prefixed domain helper is not this preimage.
7. QVC SHALL retain the authoritative refusal for duplicate names in the
   same object. Equal names in different objects are not duplicates.
8. QVC SHALL retain the authoritative refusal of NaN and positive or
   negative infinity emitted as floating-point values.
9. QVC SHALL retain the authoritative inclusive Rust integer range
   [-9007199254740992, 9007199254740992]. Every integer outside it refuses,
   including integers exactly representable as a double. Floating-point
   values follow the authoritative finite-double ECMAScript rule instead.
10. QVC SHALL keep ingestion validation separate from the `jcs_*` helpers.
    The ingestion limits on bytes, nesting, and array items do not become
    canonicalization limits.
11. QVC SHALL remove the replaced third-party canonicalization path when
    the authoritative path is implemented. No copied encoder, compatibility
    fallback, serde_json bridge, or canonical API re-export is introduced.

The API preserves the canonical writer's existing serde model mapping,
including integer map keys encoded as textual member names, non-string
unsupported key refusal, sequence ordering, and canonical string escaping.
An input already materialized as `Value` has already lost any duplicate
member history or nonfinite values that an earlier conversion removed; QVC
does not claim to reconstruct that history. Migration must not create that
loss by converting arbitrary inputs to `Value` or serialized JSON text.

## Worked Vectors

All outcomes below are prospective oracles, not executed test results. The
old/lossy columns describe source-derived differences, not measured RED.
Bytes are UTF-8 with no trailing newline. A fixed-depth custom serializer
must compute its `FixedShape::DEPTH` from every field type; a literal depth
is allowed only for a fieldless scalar serializer. Alternatively, emit the
same vector using explicit-stack `Encode`/`Writer` events.

| Vector | Typed input or events | Required authoritative result | Old or lossy difference |
| --- | --- | --- | --- |
| V-01 | One map emits b=1, a=2, b=3 | Duplicate name b; all helpers return `canonicalization_failed` | Old encoder keeps first b: `{"a":2,"b":1}`; a to_value bridge keeps last b: `{"a":2,"b":3}` |
| V-02 | f32 and f64 NaN, +Inf, -Inf, each top-level and inside an array/object | Nonfinite refusal; all helpers return `canonicalization_failed` | Old refusal preserved; serde_json text/Value conversion substitutes `null` |
| V-03 | Integer +9007199254740993 or -9007199254740993, native and Value forms | Integer-magnitude refusal; all helpers return `canonicalization_failed` | Old rounds to +9007199254740992 or -9007199254740992; text/read bridge also loses integer classification |
| V-04 | u64::MAX; native signed/unsigned integer 2^60 | Integer-magnitude refusal | Old accepts cast doubles, including exact double 2^60; refusal is a deliberate correction |
| V-05 | Integer +9007199254740992; integer -9007199254740992; decimal string "9007199254740993" | `9007199254740992`; `-9007199254740992`; `"9007199254740993"` | Inclusive boundary and string remain valid |
| V-06 | Finite f64 2^60 | `1152921504606847000` | Float remains valid despite integer 2^60 refusal |
| V-07 | Object keys U+E000 and U+1F600 with values 1 and 2 | `{"😀":2,"":1}` | UTF-16 order, not UTF-8 or codepoint order |
| V-08 | f64 -0.0; f32 0.1; f64 1e-6; f64 1e-7; f64 1e20; f64 1e21 | `0`; `0.10000000149011612`; `0.000001`; `1e-7`; `100000000000000000000`; `1e+21` | Finite-double behavior preserved |
| V-09 | String with quote, backslash, LF, U+0000 | `"\"\\\n\u0000"` | Exact escaping; no replacement or normalization |
| V-10 | Composed é and decomposed e followed by U+0301, as separate strings | `"é"` and `"é"`; equality false | Unicode normalization is forbidden |
| V-11 | Array [2,1]; two separate objects each with a=1 | `[2,1]`; `[{"a":1},{"a":1}]` | Array order preserved; separate-object names allowed |
| V-12 | Fixed-depth custom Serialize returns Error::custom("fixture refusal") | `canonicalization_failed` | Stable public code preserved |
| V-13 | Object a=1 | `{"a":1}`; digest `sha256-jcs:015abd7f5cc57a2dd94b7590f04ad8084273905ee33ec5cebeae62276a97f862` | Existing prefix and content-only preimage preserved |

For V-01 through V-04, test each applicable helper: canonicalize, sha256, and
equal with the refusing value in each operand position and a successful
other operand. For V-02 test each of the six scalar type/value combinations
at all three positions. For V-03/04 test every authoritative Rust signed
and unsigned integer width capable of holding each value, including i128
and u128; test Value numbers where that representation exists. A separate
Value feature-unification lane with arbitrary_precision must refuse integer
text beyond 64-bit storage using the authoritative wide-integer rule.

## Consumer Migration Obligations

The source break is by type class, not by whether existing fixtures happen
to fit the byte limits. Each migrated call requires a compile check plus the
relevant behavioral oracle; a repository search is not compile evidence.

| Public source or consumer class | Required migration and verification |
| --- | --- |
| QVC src/lib.rs jcs_equal and jcs_sha256 internal calls | Replace Serialize bounds with the exact proposed Encode bounds; retain left-first refusal and digest preimage |
| QVC tests/integration.rs canonicalization/equality/digest callers | Value feature admits existing Value calls; replace key-order-only oracle with literal bytes and full literal digest |
| QVC shared-reference fixtures and public operation catalog | Use original public constants/fixtures as regression inputs; independently verify canonical bytes and existing content identities, without changing fixture/catalog content |
| Serialize-only fixed-depth DTO and generic Serialize-only callers | Add a genuine FixedShape derivation where every field and serialization override has fixed depth, or require Encode at the generic boundary; the old bound alone must fail compilation |
| DTO/custom serializer containing recursive Value or recursive fields | Implement explicit-stack Encode; never add a fake FixedShape depth or blanket wrapper |
| Existing public IR checked-package wire types | Keep their authoritative FixedShape/iterative Encode split; IR-533 is not proof that every arbitrary QVC Serialize caller qualifies |
| Consumers already passing Value | Retain input types and enable the authoritative Value path; verify deep traversal and deliberate wide-integer refusal rather than converting Value through text/read |
| Any caller emitting repeated names or integers outside magnitude 2^53 | Expect the deliberate new refusal; do not preserve old bytes/identities generated by the erroneous encoder |
| Any caller emitting nonfinite numbers | Preserve refusal before any lossy conversion |

Implementation review must enumerate every live public QVC call site and
affected public IR integration, including generic wrappers and generated
DTO consumers. This table is the migration contract, not a claim that all
downstream repositories have already been compiled. Consumers outside this
public repository retain ownership of their migration; no blanket global
FixedShape derive is justified for generated schemas with open JSON fields.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-001-AC-1 | The three public signatures use the exact Encode bounds above; a Serialize-only struct and a generic Serialize-only wrapper fail to compile, while genuine FixedShape, iterative Encode, Value, Document/NodeRef, and str inputs compile. | Test |
| FR-001-AC-2 | V-01 through V-04 return the exact public refusal code in every stated helper/type/position combination, and do not return bytes or a digest. | Test |
| FR-001-AC-3 | V-05 through V-11 equal their literal byte oracles; same bytes compare equal and each pair with different bytes compares unequal. | Test |
| FR-001-AC-4 | V-12 returns canonicalization_failed and equality's right operand is not invoked when the left refuses, verified by an invocation counter. | Test |
| FR-001-AC-5 | V-13 equals the full literal digest; a content change changes it, and neither the prefix bytes nor a length-prefixed domain occur in the hash preimage. | Test |
| FR-001-AC-6 | Canonicalization succeeds for Value nesting 129, an array with 100001 null items, and a string longer than the separate 16 MiB ingestion ceiling, demonstrating that ingestion policy is not invoked. | Test |
| FR-001-AC-7 | The public shared-reference fixtures and operation catalog retain independently verified canonical bytes and content identities wherever their inputs are in the authoritative domain; any out-of-domain input is classified as a deliberate refusal correction. | Test |
| FR-001-AC-8 | The migration inventory names each live public QVC and affected public IR call site and records its input class, compile outcome, applicable vector outcome, and any deliberate behavior change. | Inspection |
| FR-001-AC-9 | Production contains one authoritative encoder path and no third-party canonicalizer, copied encoder, serde_json text/Value bridge, fake FixedShape, compatibility fallback, or canonical API re-export. | Inspection |

## Verification Procedure

After independent spec review, implement prospective tests with criterion
trace tags. Encode byte oracles as literal UTF-8 bytes; for V-07 also assert
the explicit key codepoints to avoid editor ambiguity. For V-09 assert the
ASCII bytes [34,92,34,92,92,92,110,92,117,48,48,48,48,34]. Compute content
digests independently from those literal bytes, not from an encoder's output.
V-13's expected digest is SHA-256 of the seven literal bytes `{"a":1}`.

Separate source predictions from measured old-baseline failures. If a
baseline run is performed, distinguish semantic assertion failures from
compile/setup failures; no baseline run has occurred for this proposal.
Raw malformed JSON, lone surrogates, and out-of-range raw number tokens
belong to the separate reader API, not to a new QVC raw canonicalization
entrypoint. Returned errors are the promised refusal contract; arbitrary
consumer code panics or allocation aborts are not converted into an error
promise by the narrowed bound.

## Dependencies

- The [quire-canonical Encode API](https://github.com/agent-ix/quire-canonical/blob/main/src/lib.rs),
  [FixedShape contract](https://github.com/agent-ix/quire-canonical/blob/main/src/shape.rs),
  and [authoritative refusals](https://github.com/agent-ix/quire-canonical/blob/main/src/error.rs)
  own canonical encoding policy.
- [NFR-001](../non-functional/NFR-001-jcs-depth-safety.md) constrains traversal safety.
- QSL-693 independent spec review precedes the IR-566 implementation.
