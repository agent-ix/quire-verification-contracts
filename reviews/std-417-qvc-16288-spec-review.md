---
id: SR-16288
title: "spec-review of the bytes, json_object and IEEE negate catalog entries (quire-verification-contracts PR 21)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-verification-contracts@8553b36d; contracts/checked-operation-catalog-v1.json; src/operation_catalog.rs"
review_set: subset
---

# Spec review of the bytes, json_object and IEEE negate catalog entries (quire-verification-contracts PR 21)

## Summary

Ticket: STD-417 (also STD-397). One spec-review pass over `git diff origin/main...8553b36d`. Checked: the families `bytes` and `json_object` in `families`, `any_value` and `any_term` only; the six operations of the ticket with exactly the ticket's shape (laws [], mode null, member null, constraints [], leaves null); the two negate identities (unary, result `operand:0`, law `ieee_profile`, mode null) against QSpec #419 FR-322-AC-111 and #408 FR-322-AC-103; the identity spellings against QSpec #408 and #419; the two new tests, which assert exact entry shapes and the closed identity sets. `make ci` exit 0 (fmt-check, clippy both feature sets, tests, cargo deny licenses, docs). No deferral or narrowing language in the diff. The QSpec Cargo.lock move and schema enum additions are QSpec-side steps in the PR's merge order, not part of this repository's catalog change.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Verdict

Mergeable: no defects found in this PR's own delta.

Merge: yes: origin/main is an ancestor; no other open PR in the repository.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | low | The fix delta adds three STD-401 identities (`quire.op.collection.sequence.drop_first`, `.ordered_set.drop_first`, `quire.op.collection.remove_one`) and a test to a PR titled and described as STD-417 only; the PR body does not mention STD-401. The shapes match QSpec #407 FR-322 (one operand of the family, result `kind:0`, `remove_one` with `inner_type` and leaves `inner:0` as `contains`), and `make ci` exits 0. Fix: add STD-401 to the PR title or body so the merge record names it. | contracts/checked-operation-catalog-v1.json | missing-requirement |

## Dispositions

Round 1 at 780c2d90. Round 1. The original review was clean. The new commits (c7e9e85 STD-401 catalog entries, 780c2d9 the SR file) were read in full: three added identities whose names, operand shapes and result form match QSpec #407 FR-322; `set` and `bag` correctly have no `drop_first`. The existing `kind:0`, `collection_kind`, `inner_type` and `inner:0` forms are already on main. `make ci` exit 0.

| FND | Outcome | Reason |
| --- | --- | --- |
| - | - | No finding of the review pass to disposition. |

Verdict after round 1: Mergeable: no finding of the review pass; one low new finding.
