# quire-verification-contracts

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Public interface vocabulary for the E02 verification contract boundary: schema-owned public contracts, RFC 8785 canonicalization, and bounded JSON ingestion.

This crate is the public boundary extracted from the private `quire-verification` strategy
layer (see [PLAT-861](https://linear.app/agent-ix/issue/PLAT-861)). It carries the wire types,
`VerificationErrorCode`, RFC 8785 canonicalization, and bounded JSON ingestion — total,
I/O-free functions with explicit resource bounds (`MAX_JSON_BYTES`, `MAX_JSON_DEPTH`,
`MAX_ARRAY_ITEMS`). The planning and evidence-assessment strategy layer (`planner`,
`bounded_portfolio`, `evidence`, `catalog`) stays in private `quire-verification`, which
depends on this crate and re-exports it at `contracts`, so its own internal call sites are
unchanged. `quire-protocol` depends on this crate directly.

This crate carries the `e02-draft-2` shape: `SharedArtifactEnvelope` and the other wire types
resolve a shared-reference schema fragment (`contracts/shared-reference-2-draft/schema.json`)
both at build time (typify codegen) and at runtime (`jsonschema::Registry`). That fragment is
a byte-identical, data-only snapshot of private `agent-ix/quire-specification`; see
[`contracts/shared-reference-UPSTREAM.md`](contracts/shared-reference-UPSTREAM.md) for its
exact provenance. Public reusable-artifact terms for it have been selected by the owner:
AGPL-3.0-or-later, publication authorized (2026-09-20).

## Build

```bash
make test
```

## License

Licensed under the GNU Affero General Public License, version 3 or later
([LICENSE](LICENSE)).
