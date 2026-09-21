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
pub const CHECKED_OPERATION_CATALOG_V1_SHA256: &str =
    "4413e24af43934b0d02c51b0f14815a8b36678548c8dd7639d12af3e2f6fa508";

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

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

    /// The document declares the version this module names. A file swapped for one of
    /// a different identity is refused here, not by whichever consumer notices first.
    #[test]
    fn the_embedded_document_declares_the_published_version() {
        let value: serde_json::Value = serde_json::from_str(CHECKED_OPERATION_CATALOG_V1)
            .expect("the embedded catalog is valid JSON");
        assert_eq!(
            value.get("version").and_then(serde_json::Value::as_str),
            Some(CHECKED_OPERATION_CATALOG_V1_VERSION)
        );
    }

    /// The catalog carries operations. An empty or truncated file would otherwise
    /// satisfy both checks above while admitting nothing.
    #[test]
    fn the_embedded_document_carries_its_operations() {
        let value: serde_json::Value = serde_json::from_str(CHECKED_OPERATION_CATALOG_V1)
            .expect("the embedded catalog is valid JSON");
        let operations = value
            .get("operations")
            .and_then(serde_json::Value::as_array)
            .expect("the catalog declares an operations array");
        assert!(
            !operations.is_empty(),
            "the catalog declares no operations, so every operation identity would be refused"
        );
    }
}
