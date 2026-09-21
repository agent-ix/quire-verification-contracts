# quire-verification-contracts

Public interface vocabulary for the E02 verification contract boundary and the retained shared-reference packet: schema-owned public contracts, RFC 8785 canonicalization, and bounded JSON ingestion.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # fmt-check + lint + test + deny + audit-unsafe
```

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.94` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Layout

```
src/lib.rs               # crate root
src/shared_reference.rs  # retained shared-reference packet (VER-50): schemas, wire_v1/wire_v2, fixtures, snapshot guard
src/operation_catalog.rs # home of quire.checked-operation-catalog/v1: bytes, version identity, digest
tests/integration.rs     # end-to-end tests (E02 boundary)
tests/shared_reference.rs # acceptance tests for the shared-reference packet
contracts/                # E02 schema, shared-reference schemas, snapshot manifest, provenance notes
fixtures/                # the fourteen retained amendment fixtures
benches/                 # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                    # requirements artifacts (from /spec-create-spec)
scripts/                 # local tooling
```
