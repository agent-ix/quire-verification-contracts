---
id: SR-15517
title: "Code review of quire-verification-contracts PR #15: delegate public JCS helpers to authoritative Encode"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-verification-contracts@86bf51a127b8659026431c9de1c2f3204aec6e67; PR #15 diff against origin/main: Cargo.toml, Cargo.lock, deny.toml, README.md, src/lib.rs, tests/integration.rs, tests/jcs_contract.rs, tests/jcs_depth.rs"
review_set: subset
---
# Code review of quire-verification-contracts PR #15

## Summary

Ticket: IR-566 (spec contract QSL-693). The PR swaps `serde_json_canonicalizer`
for `quire_canonical::to_vec` behind the three public `jcs_*` helpers, narrows
their bounds from `Serialize` to `Encode + ?Sized`, and adds two test files that
cover vectors V-01..V-18 and the depth requirement NFR-001. Rust lane folded in.

- `src/lib.rs`: one encoder path, `Limits::new(u64::MAX)`, every error mapped to
  `canonicalization_failed`, left-first `?` in `jcs_equal`, digest over the
  canonical bytes alone. No re-export, no Value or text bridge. Correct.
- `tests/jcs_depth.rs` `IterativeValue::nest_array`: unwind-safe. The parent
  `Vec` is allocated before the swap. `mem::swap` cannot panic. The assignment
  drops only the swapped-in `Null`. The deep chain is never owned by a local
  that unwinding could drop recursively, and `Drop` uses `drop_value`. The
  construction-panic and encode-panic probe confirms the original panic
  survives on a 512 KiB stack.
- `deny.toml` `allow-git` for quire-canonical: acceptable. It matches the
  `allow-git` convention in quire-contract-ir and other ecosystem repos, and
  quire-canonical is AGPL-3.0-or-later, which is on the allow list.
  `make deny` runs only `licenses`, so the `[sources]` table is not checked
  by the gate in any case.
- Gates run at 86bf51a: `make ci` exit 0 (jcs_contract 84 passed, jcs_depth
  6 passed, both compile_fail doctests passed, licenses ok). Run by hand:
  `cargo test --features test-arbitrary-precision` exit 0 (85 passed, wide
  integer refusal included) and `cargo clippy --all-targets --all-features
  -D warnings` exit 0.
- Literal oracles: I recomputed every hard-coded name and V-13 hash with
  `sha256sum` over the literal bytes. All match.

## Verdict

Approve after FND-001 and FND-002. No correctness defect in production code.
FND-001 is pin ceremony that already went stale once on this branch. FND-002
leaves a spec-required lane unrun.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The operation-catalog test pins a length (38_325) and a digest captured from the encoder under test. Nothing else in the repo uses that digest. It already went stale when PR #16 changed the catalog and was repointed in 86bf51a, and it will break on every catalog edit. It also fails AC-7's "independently verified" wording, because the oracle is the encoder's own output. Replace it with invariants: the canonical bytes reparse to a Value equal to the input, canonicalizing the reparsed Value gives the same bytes, and `jcs_sha256` equals `sha256-jcs:` plus `Sha256::digest` of those bytes computed in the test. | tests/jcs_contract.rs:840-848 |
| FND-002 | medium | The `test-arbitrary-precision` lane never runs. `make ci` calls `cargo test` and `clippy --all-targets` with no features, so the cfg-gated wide-integer refusal test that FR-001 requires is neither compiled nor run by the gate. It passes when run by hand, so the defect is the missing lane, not the code. Add `cargo test --features test-arbitrary-precision --test jcs_contract` and an all-features clippy run to `make ci`. | Makefile:75; Cargo.toml:23-25; tests/jcs_contract.rs:850-860 |
| FND-003 | low | The shared-reference test hard-codes hex hashes for the utf16-and-number and member-reordering vectors. They match the fixture's literal `canonical` bytes, but only the fixture-bound core identity appears anywhere else. Computing `Sha256` of `vector["canonical"]` in the test is just as independent and does not pin the fixture's content a second time. | tests/jcs_contract.rs:812-825 |
| FND-004 | low | The module doc says the file "also compiles against the old Serialize boundary for bounded baseline measurement". It cannot, because it imports `quire_canonical`, which is not a dependency on main. The extra `Serialize` bounds on the helpers and the `println!` PASS/FAIL lines in `Checks` serve only that measurement. Drop the claim, the extra bounds and the prints, and keep the aggregate-failure harness. | tests/jcs_contract.rs:4-5,17-46,48-107 |
| FND-005 | low | The rewritten README dependency sentence leaves out the runtime dependencies `chrono` and `regress`. It also does not say that a caller who derives `FixedShape` or implements `Encode` needs its own direct quire-canonical dependency, which follows from the deliberate no-re-export rule. | README.md:146-149,78-84 |
