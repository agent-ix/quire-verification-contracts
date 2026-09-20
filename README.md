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

## Build

```bash
make test
```

## License

Licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
