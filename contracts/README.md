# Private contract packet

`e02-draft-1.schema.json` is the authoritative definition of the five E02
contracts and their shared provisional artifact-reference seam. The Rust build
derives wire types and vocabulary constants from this file. At runtime,
`src/contracts.rs` compiles the complete draft 2020-12 schema, including
conditional assertions that are not constructive Rust types. The only additional plan validation is for duplicate
task IDs, unknown dependency references, and dependency cycles, which require
cross-record graph checks. This is a private versioned schema, not a published
normative schema or an accepted replacement for FS02/FS05 identities.

The fixture packet deliberately includes state, relational, trace, fault, and
manual inputs. The schema keeps their required inputs distinct rather than
placing them in one universal predicate type.

`cargo run --locked --bin validate-fixtures` discovers every JSON file in `fixtures/`, validates
each complete fixture with a schema definition, and validates every embedded or
adapted instance of the five public contracts. An unhandled new fixture fails
the check.
