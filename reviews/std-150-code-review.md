---
id: SR-1184
title: "Code review of quire-verification-contracts PR #12: source revision and snapshot guard removed"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-verification-contracts@6e983f92c04f7cc80395f43d1f13712a4c854f45; PR #12 diff against origin/main 41bb306e: contracts/e02-draft-2.schema.json, contracts/shared-reference-1-draft.schema.json, contracts/shared-reference-2-draft/schema.json, fixtures/shared-reference-2-draft/{adverse,roles,semantic}.json, contracts/README.md, README.md, CLAUDE.md, Cargo.toml, Cargo.lock, src/lib.rs, src/shared_reference.rs, tests/shared_reference.rs; deleted contracts/shared-reference-snapshot.sha256 and contracts/shared-reference-UPSTREAM.md"
review_set: subset
---
# Code review of quire-verification-contracts PR #12

## Summary

Ticket: STD-150. The source revision is removed from both shared-reference
schemas, the roles, semantic and adverse fixtures, the e02 VP-04 diagnostic
`source`, and one test envelope. The snapshot manifest, its guard
(`verify_shared_reference_snapshot`, `SNAPSHOT_MANIFEST`, three tests),
`shared-reference-UPSTREAM.md` and the `tempfile` dev-dependency are deleted as
file tracking. The guard's descriptive content moves to `contracts/README.md`,
with VER-50's expiry stated (VER-51).

- All 16 VER-50 packet files are present at this head. The 5 this PR touches,
  and the other 11, are byte-identical to their QSpec #178 (ef8e9bc5)
  counterparts.
- No reference to the removed API, manifest or UPSTREAM file is left in the
  tree. `sha2` stays because `src/lib.rs` still uses it for JCS digests.
- No compatibility path: the schemas drop `revision` outright.
- Rust lane: the deletion is clean. There are no dead imports, `clippy
  --all-targets -D warnings` passes, and `make test` passes (21 tests).

## Verdict

Changes needed: one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The VP-04 parser diagnostic's `source` drops `revision` and keeps only `identity`. Under STD-150 a source is identified by its document identity plus its content digest (QSpec FR-202's key is (`document_identity`, `digest`)), so this diagnostic's `span` is no longer bound to any exact bytes: two different contents under one identity give the same `source`. Add `digest` to `source`, or state in the schema description why a parse diagnostic needs only the identity. | contracts/e02-draft-2.schema.json:1952-1958 |
