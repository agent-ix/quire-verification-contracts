# quire-verification-contracts

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Public interface vocabulary for the E02 verification contract boundary: schema-owned public contracts, RFC 8785 canonicalization, and bounded JSON ingestion.

This crate is the public boundary extracted from the private `quire-verification` strategy
layer (see [PLAT-861](https://linear.app/agent-ix/issue/PLAT-861)). It carries the wire types,
`VerificationErrorCode`, RFC 8785 canonicalization, and bounded JSON ingestion — total,
I/O-free functions with explicit resource bounds (`MAX_JSON_BYTES`, `MAX_JSON_DEPTH`,
`MAX_ARRAY_ITEMS`). The planning and evidence-assessment strategy layer (`planner`,
`bounded_portfolio`, `evidence`, `catalog`) stays in private `quire-verification`, which
depends on this crate and re-exports it at `contracts`.

## Status: blocked on an owner decision (PLAT-861)

This first slice reflects the `e02-draft-1` shape of `contracts.rs` (the pre-shared-reference
snapshot). Private `quire-verification`'s current `contracts.rs` has since moved to
`e02-draft-2`, whose `SharedArtifactEnvelope` wire type embeds a `$ref` to
`urn:ix:shared-reference:2-draft`, resolved from `shared-reference-2-draft/schema.json`. That
schema is a snapshot of private `agent-ix/quire-specification`, and its own provenance note
(`contracts/shared-reference-UPSTREAM.md`) states public reusable-artifact terms are
unresolved and the snapshot is "excluded from public promotion until an owner selects those
terms." Promoting it into this public crate is not something to decide unilaterally, so this
crate has not yet been advanced to the `e02-draft-2` shape and `quire-verification` /
`quire-protocol` have not yet been wired to it. See PLAT-861 for the options under
consideration.

## Build

```bash
make test
```

## License

Licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
