---
type: master-requirements
name: quire-verification-contracts
org: agent-ix
component_type: rust-lib
implementation_language: rust
relationships:
  - target: ix://agent-ix/quire-canonical
    type: uses
---
# Master Requirements Specification

## Purpose

This specification proposes QSL-693 route (b): narrow QVC's public `jcs_*`
inputs from arbitrary `serde::Serialize` to `quire_canonical::Encode`.
It defines the contract to review before IR-566 implementation. The artifacts
and worked vectors are prospective; structural validation is not a runtime
PASS or independent spec-review verdict.

## Scope

### In Scope

The public canonical bytes, equality, digest, refusal, and input-type contract
of `jcs_canonicalize`, `jcs_equal`, and `jcs_sha256`; consumer migration from
Serialize-only inputs; preservation of the authoritative depth discipline.

### Out of Scope

New canonical encoders or an arbitrary Serialize adapter; changes to JSON
ingestion limits, schemas, shared-reference packet content, operation catalog
content, digest preimages, and the separate raw JSON reader. This proposal
does not change quire-canonical's public contract or weaken its fixed-depth
serde restriction.

## System Overview

QVC delegates canonicalization to quire-canonical. Its `Encode` sources are
genuine fixed-depth serde types, iterative `Document`/`NodeRef`, iterative
`serde_json::Value`, and consumer-owned explicit-stack event sources.
Arbitrary `Serialize` alone provides no bound on caller-controlled recursive
traversal; an iterative writer does not make that traversal iterative.

## Requirements Architecture

| Artifact | Responsibility |
| --- | --- |
| [FR-001](functional/FR-001-authoritative-jcs-encode-contract.md) | Public narrowed input contract, observable bytes/refusals, migration, and worked vectors |
| [NFR-001](non-functional/NFR-001-jcs-depth-safety.md) | Native-stack discipline and independent depth verification |

These files specify this slice only; they do not claim to backfill the entire
existing QVC library. Test coverage is derived from criterion trace tags,
never from a manually maintained matrix.

## Compatibility Decision

The proposed signatures require `Encode` instead of `Serialize`. This is a
source break for any Serialize-only downstream struct, custom serializer, or
generic function bounded only by Serialize. Existing callers with `Value`
remain admissible through quire-canonical's Value feature. A fixed-depth
struct needs a genuine `FixedShape` derivation; a value containing recursive
data needs iterative `Encode`. A generic caller must require `Encode` or
make an explicit typed conversion that preserves its input semantics.

This route deliberately corrects old encoder behaviors: repeated map names
kept the first member, numeric integer values beyond magnitude 2^53 were cast
to doubles, and large integer member names were rounded instead of emitted
as exact decimal text. Top-level nonfinite refusal is preserved; nested
nonfinite values that previously became null now refuse. Bool and finite
float member names previously accepted as text now refuse under the
authoritative serde mapping. Option::Some member names previously delegated
to their inner string or integer names now refuse; None member names retain
their existing refusal. These are runtime changes even for genuine
FixedShape maps. Large integer member names remain admissible;
the numeric-value magnitude check does not apply to names.
No compatibility encoder, fallback, lossy JSON conversion, or fake
`FixedShape` is part of the migration. Independent spec review precedes
code; source compatibility and changed refusal behavior must be assessed
against the concrete migration obligations in FR-001.

## Verification Strategy

FR-001 defines prospective independent literal byte/refusal oracles and
consumer compilation checks. NFR-001 defines small-stack depth checks. Test
implementation and runtime evidence follow review; this source-only proposal
contains neither production code nor an assertion that those checks passed.

## References

- [QSL-693](https://linear.app/agent-ix/issue/QSL-693)
- [IR-566](https://linear.app/agent-ix/issue/IR-566)
- [quire-canonical public API](https://github.com/agent-ix/quire-canonical/blob/main/src/lib.rs)
- [FixedShape contract](https://github.com/agent-ix/quire-canonical/blob/main/src/shape.rs)
- [QSL depth capabilities](ix://agent-ix/quire-spec-language/FR-259)
- [QSL depth architecture](ix://agent-ix/quire-spec-language/ADR-030)
