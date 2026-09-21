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
does not retrieve schemas over the network.

**This vendoring is TEMPORARY, not a settlement.** All sixteen files are
retained here under [VER-50](https://linear.app/agent-ix/issue/VER-50)'s
standing exception — VER-50 is what authorizes publishing the complete
sixteen-file packet in this crate, consolidating what had been two copies
(this crate and private `quire-verification`) into one. The AGPL-3.0-or-later,
owner-ruling-2026-09-20 terms cited below cover distribution of one file, the
draft-2 schema fragment (`contracts/shared-reference-2-draft/schema.json`)
alone; they are not a decision that the packet may stop being temporary, and
do not extend to the other fifteen files.

The exception's expiry condition, from
[VER-51](https://linear.app/agent-ix/issue/VER-51), verbatim: **"`quire-specification`
becomes public, or the packet gets a public home."** Publishing this packet in
`quire-verification-contracts` does not itself satisfy that condition —
copying content into a public repository has never discharged a vendoring
exception; visibility was never the test. The condition means the packet gets
a public home *at its definer*: either `quire-specification` itself goes
public, or its owner gives this packet a public home there, at which point
this snapshot is deleted from this crate and replaced by a reference to that
source. Until then, this remains a copy, retained under exception, not a
resolved artifact.

Public reusable-artifact terms for the one covered file: **AGPL-3.0-or-later,
publication authorized** (owner ruling, 2026-09-20). All sixteen files are
published under that same license in this crate in the meantime, under the
VER-50 exception above.
