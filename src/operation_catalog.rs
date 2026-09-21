// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The closed `quire.checked-operation-catalog/v1` operation vocabulary.
//!
//! This crate is the catalog's home. It is not a snapshot of one kept elsewhere:
//! `contracts/checked-operation-catalog-v1.json` is the only copy of these bytes in
//! the ecosystem, and every consumer — `quire-contract-ir`'s CheckedPackage V2 reader
//! first among them — reads it from here by dependency rather than by embedding its
//! own. A second copy anywhere, under any name, is a defect, not a convenience.
//!
//! The catalog declares which `operation.identity` values a V2 `application` term may
//! name, and for each one the operand families, result form, required law roles, mode
//! and member kinds, and cross-operand constraints. A reader that cannot load it
//! cannot decide whether an operation is admissible, so this module exposes the bytes
//! and their digest and leaves interpretation to the consumer that owns the reader.
//!
//! Homed here on the owner's ruling (2026-09-21), resolving
//! agent-ix/quire-contract-ir#169 and the catalog half of #166: that repository had
//! embedded its own copy, the copy was deleted, and the crate stopped compiling
//! rather than let its reader start admitting operations it used to refuse.

/// The catalog document, verbatim.
///
/// Consumers parse this into their own types. This crate deliberately publishes no
/// parsed representation: the reader that validates against the catalog lives in
/// `quire-contract-ir`, and a second parse here would be a second definition of the
/// same closed vocabulary.
pub const CHECKED_OPERATION_CATALOG_V1: &str =
    include_str!("../contracts/checked-operation-catalog-v1.json");

/// Version identity the catalog declares.
pub const CHECKED_OPERATION_CATALOG_V1_VERSION: &str = "quire.checked-operation-catalog/v1";

/// Lowercase SHA-256 of [`CHECKED_OPERATION_CATALOG_V1`].
///
/// Published so a consumer can assert the bytes it compiled against without
/// re-embedding them. Content, not provenance: this digest is checkable offline and
/// forever, where a commit id depends on another repository's history surviving.
///
/// Over the raw file bytes, not this crate's `sha256-jcs:` convention
/// ([`crate::jcs_sha256`]) and not the shared-reference snapshot manifest's. Consumers
/// reach the catalog through `include_str!` of these exact bytes, so the digest they
/// can check without a JSON parser is the one over what they compiled. The trade is
/// that a whitespace-only reformat invalidates it; that is the intended reading, since
/// the bytes themselves are what is published. `.gitattributes` pins `*.json -text` so
/// a checkout cannot change them underneath a consumer.
pub const CHECKED_OPERATION_CATALOG_V1_SHA256: &str =
    "4413e24af43934b0d02c51b0f14815a8b36678548c8dd7639d12af3e2f6fa508";

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;
    use sha2::{Digest, Sha256};

    use super::*;

    /// Operations the catalog declares. Pinned so a truncating regeneration fails
    /// here rather than at whichever consumer first meets an operation that vanished.
    const OPERATION_COUNT: usize = 135;

    fn catalog() -> Value {
        serde_json::from_str(CHECKED_OPERATION_CATALOG_V1)
            .expect("the embedded catalog is valid JSON")
    }

    /// The published digest describes the bytes this crate actually carries. A catalog
    /// edit that does not update the constant fails here rather than reaching a
    /// consumer that pinned the old digest.
    #[test]
    fn the_published_digest_is_over_the_embedded_bytes() {
        let measured = format!(
            "{:x}",
            Sha256::digest(CHECKED_OPERATION_CATALOG_V1.as_bytes())
        );
        assert_eq!(
            measured, CHECKED_OPERATION_CATALOG_V1_SHA256,
            "CHECKED_OPERATION_CATALOG_V1_SHA256 does not describe \
             contracts/checked-operation-catalog-v1.json"
        );
    }

    /// The document declares the v1 identity, asserted as a literal rather than
    /// through [`CHECKED_OPERATION_CATALOG_V1_VERSION`]. Comparing the document to a
    /// constant an editor edits in the same commit is not an anchor: a v2 catalog
    /// dropped in with the constant "fixed" to match would pass, and every consumer
    /// that pins the v1 name would then validate against a v2 vocabulary under it.
    #[test]
    fn the_embedded_document_declares_the_v1_identity() {
        assert_eq!(
            catalog().get("version").and_then(Value::as_str),
            Some("quire.checked-operation-catalog/v1")
        );
        assert_eq!(
            CHECKED_OPERATION_CATALOG_V1_VERSION,
            "quire.checked-operation-catalog/v1"
        );
    }

    /// The catalog carries its whole operation set, each entry complete.
    ///
    /// A bare non-empty check is not enough, and the digest test does not cover this:
    /// the digest is recomputed by whoever edits the catalog, which is what the digest
    /// test instructs them to do. Measured — a catalog reduced to a single
    /// `{"identity": "bogus.operation"}` entry, with the digest constant updated to
    /// match, passed all three of this module's earlier criteria at exit 0. It would
    /// have left this repository green while `quire-contract-ir` refused every real
    /// `quire.op.*` in every CheckedPackage V2 as uncatalogued.
    #[test]
    fn the_embedded_document_carries_every_operation_in_full() {
        let catalog = catalog();
        let operations = catalog
            .get("operations")
            .and_then(Value::as_array)
            .expect("the catalog declares an operations array");
        assert_eq!(
            operations.len(),
            OPERATION_COUNT,
            "the catalog's operation count changed; a consumer pinning this vocabulary \
             refuses whatever went missing"
        );

        const MEMBERS: [&str; 10] = [
            "identity",
            "operator",
            "operands",
            "rest",
            "result",
            "laws",
            "mode",
            "member",
            "constraints",
            "leaves",
        ];
        for operation in operations {
            let entry = operation
                .as_object()
                .expect("every operations[] entry is an object");
            let identity = entry
                .get("identity")
                .and_then(Value::as_str)
                .expect("every entry declares an identity");
            for member in MEMBERS {
                assert!(
                    entry.contains_key(member),
                    "operation `{identity}` is missing `{member}`, so a reader deserializing \
                     the closed entry shape refuses the whole catalog"
                );
            }
        }

        let identities: BTreeSet<&str> = operations
            .iter()
            .filter_map(|operation| operation.get("identity").and_then(Value::as_str))
            .collect();
        assert_eq!(
            identities.len(),
            OPERATION_COUNT,
            "two operations share an identity, so one is unreachable by lookup"
        );
    }

    /// Every closed vocabulary the catalog is built from is present.
    ///
    /// The operation entries reference these by name — operand families and groups,
    /// law roles, mode kinds, member kinds, constraint kinds, result forms, leaf
    /// sources. Measured: deleting all eleven left the earlier criteria green, so a
    /// regeneration that dropped one would publish a catalog whose entries name
    /// vocabularies that are not there.
    #[test]
    fn the_embedded_document_carries_every_closed_vocabulary() {
        const VOCABULARIES: [&str; 11] = [
            "version",
            "families",
            "groups",
            "law_roles",
            "profile_law_roles",
            "modes",
            "type_pinned_modes",
            "member_kinds",
            "constraint_kinds",
            "result_forms",
            "leaf_sources",
        ];
        let catalog = catalog();
        for vocabulary in VOCABULARIES {
            let value = catalog
                .get(vocabulary)
                .unwrap_or_else(|| panic!("the catalog declares `{vocabulary}`"));
            let populated = match value {
                Value::Array(items) => !items.is_empty(),
                Value::Object(members) => !members.is_empty(),
                Value::String(text) => !text.is_empty(),
                _ => false,
            };
            assert!(
                populated,
                "`{vocabulary}` is empty, so every entry referring to it is unresolvable"
            );
        }
    }
}
