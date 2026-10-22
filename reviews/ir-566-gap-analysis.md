---
id: SR-15518
title: "Gap analysis of quire-verification-contracts PR #15 against FR-001 and NFR-001"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-verification-contracts@86bf51a127b8659026431c9de1c2f3204aec6e67; spec/functional/FR-001-authoritative-jcs-encode-contract.md, spec/non-functional/NFR-001-jcs-depth-safety.md, src/lib.rs, tests/jcs_contract.rs, tests/jcs_depth.rs, tests/integration.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-verification-contracts/FR-001
    type: references
  - target: ix://agent-ix/quire-verification-contracts/NFR-001
    type: references
---
# Gap analysis of quire-verification-contracts PR #15

## Summary

Ticket: IR-566. `quire matrix --scope <repo>` (quire 0.36.2) tags every Test
criterion: FR-001-AC-1..7 and NFR-001-M-1/M-2. FR-001-AC-8, AC-9 and
NFR-001-M-3 are Inspection criteria with no symbol, which is expected.

Vector-by-vector check of whether each test would fail on the old
`serde_json_canonicalizer` path. This was read from source and not run
against the old baseline:

- V-01, nested V-02, V-03, V-04, V-14, V-15, V-16, Some keys in V-18 and the
  u64::MAX newtype in V-17 would all fail on the old path, because the old
  encoder returns bytes or rounded names where the new tests expect a refusal
  or exact text.
- Preserved controls pass on both paths as the spec intends: top-level
  nonfinite V-02, None keys in V-18, V-05..V-11, char, unit-variant and
  string-newtype V-17, and V-13.
- AC-4 and AC-1 cannot compile on the old path at all.

Inspection results:

- AC-9: only `quire_canonical::to_vec` is used. `serde_json_canonicalizer` is
  gone from `Cargo.toml` and `Cargo.lock`. There is no re-export, no Value or
  text bridge and no fallback.
- NFR-001-M-3: no depth or ingestion cap was added to the `jcs_*` helpers.
- AC-8: see FND-001.

Earlier review SR-5358 FND-003 (duplicate numeric test names) is resolved at
947745b. The names are now distinct.

## Verdict

CONDITIONAL. No untagged Test criterion and no high finding. One medium
inspection gap and two low trace gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-001-AC-8 is not met. The PR records no downstream call-site check, and the PR body itself says downstream migration is not claimed. Measured here: quire-specification, which tracks QVC branch=main, passes only `&Value` and compiles under the Value feature. quire-protocol passes `json!` Values and depends on QVC through a rev-pinned quire-verification. The private quire-verification re-exports QVC and was not checked out locally, so its callers were not measured. The load-bearing part is a compile of quire-verification against this head before merge. The inventory document itself is ceremony and should be replaced by that compile check. | spec/functional/FR-001-authoritative-jcs-encode-contract.md:215 |
| FND-002 | low | `deep_value_and_iterative_arrays_agree_on_small_and_large_native_stacks` is tagged FR-001-AC-3. AC-3 covers the literal vectors V-05..V-11, V-14 and V-17, and this test checks none of them, so it inflates AC-3's binder count. | tests/jcs_depth.rs:169 |
| FND-003 | low | AC-1's negative half, that a Serialize-only struct and a generic Serialize-only wrapper fail to compile, is proven only by `compile_fail` doctests on `jcs_canonicalize`, which carry no trace tag. The matrix binds AC-1 only to positive tests. | src/lib.rs:585-608 |

## Coverage

- Criteria: 12 in scope. 9 Test criteria are tagged, 0 are untagged, and 3
  are Inspection criteria checked by hand.
- Reverse gap: none. The only production change is the three helpers that
  FR-001 owns.
- Stubs: none in source or tests.
- Semantic review: done by hand per vector (above), with no subagent fan-out.
- The `test-arbitrary-precision` lane test is tagged but no gate runs it. See
  SR-15517 FND-002.
- Plan completion: not assessed
