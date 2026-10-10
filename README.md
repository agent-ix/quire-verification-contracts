# quire-verification-contracts

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/k8DVhuYBR2)

Public interface vocabulary for the E02 verification contract boundary and the retained
shared-reference packet: schema-owned public contracts, RFC 8785 canonicalization, and
bounded JSON ingestion.

This crate is the public boundary extracted from the private `quire-verification` strategy
layer (see [PLAT-861](https://linear.app/agent-ix/issue/PLAT-861)). It carries the wire types,
`VerificationErrorCode`, RFC 8785 canonicalization, and bounded JSON ingestion — total,
I/O-free functions with explicit resource bounds (`MAX_JSON_BYTES`, `MAX_JSON_DEPTH`,
`MAX_ARRAY_ITEMS`). The planning and evidence-assessment strategy layer (`planner`,
`bounded_portfolio`, `evidence`, `catalog`) stays in private `quire-verification`, which
depends on this crate and re-exports it at `contracts`, so its own internal call sites are
unchanged. `quire-protocol` depends on this crate directly.

It also carries the retained shared-reference packet (see [VER-50](https://linear.app/agent-ix/issue/VER-50)):
the draft-1 and draft-2 schemas, their `typify`-generated `wire_v1`/`wire_v2` Rust types, the
fourteen amendment fixtures, all at `shared_reference`. `quire-verification` depends
on this crate for that packet too and re-exports it unchanged at
`contracts::shared_reference`, so it holds no second copy of any schema, fixture, or codegen.

This crate carries the `e02-draft-2` shape: `SharedArtifactEnvelope` and the other wire types
resolve a shared-reference schema fragment (`contracts/shared-reference-2-draft/schema.json`)
both at build time (typify codegen) and at runtime (`jsonschema::Registry`). That fragment is
a data-only copy from private `agent-ix/quire-specification`; see
[`contracts/README.md`](contracts/README.md) for the packet's scope and expiry. Public reusable-artifact terms for it have been selected by the owner:
AGPL-3.0-or-later, publication authorized (2026-09-20).

## Canonicalization inputs

The `jcs_canonicalize`, `jcs_equal`, and `jcs_sha256` helpers accept
`quire_canonical::Encode + ?Sized`. This replaces the earlier `Serialize`
bound. Existing `serde_json::Value` callers use the authoritative iterative
Value encoder. Fixed-depth DTOs derive `quire_canonical::FixedShape`; generic
wrappers require `Encode`. Recursive values need an explicit-stack `Encode`
implementation, with no blanket `FixedShape` wrapper or JSON conversion.

The helpers delegate to the authoritative encoder and return
`canonicalization_failed` for its refusals. This corrects duplicate-name
retention, nested nonfinite numbers becoming null, integer values beyond
magnitude 2^53 being rounded, and integer member names being rounded.
Bool, float, and Option member names now refuse; char, unit-variant, and
newtype names follow the authoritative mapping. The content identity remains
`sha256-jcs:` plus SHA-256 of canonical bytes alone. Ingestion limits remain
separate. See the [input and migration contract](spec/functional/FR-001-authoritative-jcs-encode-contract.md)
and [depth safety requirement](spec/non-functional/NFR-001-jcs-depth-safety.md).

## Build

```bash
make test
```

## License

Licensed under the GNU Affero General Public License, version 3 or later
([LICENSE](LICENSE)).
