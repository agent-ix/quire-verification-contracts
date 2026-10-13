---
id: SR-7873
title: "Gap analysis of quire-verification-contracts PR #16 against FR-322 and QSL-735"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-verification-contracts@255598673d52b963780325671e7952e244615e26; contracts/checked-operation-catalog-v1.json, src/operation_catalog.rs; compared with QSpec FR-322 \"Operation families, groups and constraints\" on quire-specification main"
review_set: subset
---
# Gap analysis of quire-verification-contracts PR #16

## Summary

Ticket: QSL-735. Quoin's matrix does not cover this check, because FR-322
lives in another repository. This is a manual comparison of the catalog with
the FR-322 section that the catalog copies in machine-readable form.

- `families`: FR-322 names `text`, `string`, `uuid` and `timestamp` as
  scalar families in that order. The catalog now matches that order.
- `any_value`: equal, member for member and in order, to the FR-322 row.
- `any_term`: equal to `any_value` plus `function`, `population`,
  `aggregate`, `binder`, `clause` and `temporal`, as FR-322 states.
- `exact_numeric`, `rational_promotable`, `ieee_float`, `collection_kind`,
  `structural_kind`, `enum_kind` and `field_owner` are correctly left alone.
  FR-322 lists none of the three families in those rows.
- Literal `value_kind` list (ticket AC-5): the catalog has no `value_kind`
  vocabulary, so this is a QSpec-only change and not a gap in this PR.

## Verdict

The catalog equals merged FR-322 for the three families and every group, and
no group is wrongly extended or missed. One ticket criterion is unmet. It
cannot be met in this PR until QSpec defines the operations.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Ticket AC-4 asks this PR to carry `quire.op.string.eq/ne`, `quire.op.uuid.eq/ne` and `quire.op.timestamp.eq/ne`. They are absent. That is correct against merged FR-322, which names no such identities, and adding them first would make the catalog disagree with its normative source. Until they exist, a checked package cannot express the UUID and Timestamp equality FR-149-AC-14 requires. Either hold #16 for the QSpec change and add the six identities, or merge #16 and move AC-4 to a follow-up ticket that starts in QSpec. | contracts/checked-operation-catalog-v1.json:42-210 |
