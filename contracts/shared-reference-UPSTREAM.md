# Shared-reference snapshot provenance

The sixteen files named by `shared-reference-snapshot.sha256` are a
byte-identical, data-only snapshot from private repository
`agent-ix/quire-specification` at merged revision
`8ab058beff03cc76c0f39cc06c87ef98f76e8d78`.

- `contracts/shared-reference-2-draft/schema.json` comes from
  `proposals/shared-reference-2-draft/schema.json`.
- `contracts/shared-reference-1-draft.schema.json` comes from
  `proposals/state-core/schemas/shared-reference-1-draft.schema.json`.
- `fixtures/shared-reference-2-draft/*.json` are the fourteen JSON files from
  `proposals/shared-reference-2-draft/fixtures/`.

This repository consumes only the retained schemas and fixture data. It does
not import or execute the producer qualification crate, and runtime validation
does not retrieve schemas over the network. Public reusable-artifact terms have
been selected for the complete packet: **AGPL-3.0-or-later, publication
authorized** (owner ruling, 2026-09-20). All sixteen files are published under
that license in this crate.
