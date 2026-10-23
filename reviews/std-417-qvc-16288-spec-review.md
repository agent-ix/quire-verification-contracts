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
