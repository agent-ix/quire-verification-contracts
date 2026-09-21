# Public contract packet

`e02-draft-2.schema.json` is the authoritative definition of the five E02
public contracts (`TechniqueDefinition`, `ToolCapability`, `SelectionPolicy`,
`VerificationPlan`, `TechniqueResult`). Its artifact envelopes resolve the
retained shared-reference draft-2 schema rather than restating that schema's
fields. The Rust build derives wire types and vocabulary constants from those
two sources. At runtime, `src/lib.rs` compiles the complete draft 2020-12
schema, including conditional assertions that are not constructive Rust
types.

The same schema file also owns a number of `$defs` used only internally by
private `quire-verification` (for example `QualificationProfile`,
`QualificationCorpusManifest`, `QualificationOracleManifest`,
`QualificationTechniqueManifest`, and the `Vp04*` measurement and report
records). Those are not additional public E02 root contracts; `quire-verification`
reaches them through this crate's `validate_schema_definition`, which looks up
a named definition in this same schema document. This crate does not
interpret or depend on their internal semantics — it only owns the schema file
and the generic by-name validator.

`shared-reference-1-draft.schema.json` and `shared-reference-2-draft/schema.json`,
and the fourteen fixture files under `fixtures/shared-reference-2-draft/`, are a
byte-identical, data-only snapshot retained from private
`agent-ix/quire-specification`; see `shared-reference-UPSTREAM.md` for exact
provenance and license terms. This is the complete retained shared-reference
packet (VER-50): both schemas, all fourteen fixtures, both `typify` codegens
(`build.rs`, `pub mod wire_v1`/`wire_v2` in `src/shared_reference.rs`), and the
byte-identical snapshot guard (`shared-reference-snapshot.sha256`,
`verify_shared_reference_snapshot`) all live in this crate. Private
`quire-verification` holds no copy of any of it and re-exports this module
unchanged at `contracts::shared_reference`.

`cargo test` exercises `validate_contract`, `parse_bounded_json`,
`validate_resource_envelope`, RFC 8785 JCS canonicalization, and the
shared-reference schema validation and snapshot guard directly.
