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
//! and leaves interpretation to the consumer that owns the reader.
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;

    use super::*;

    fn catalog() -> Value {
        serde_json::from_str(CHECKED_OPERATION_CATALOG_V1)
            .expect("the embedded catalog is valid JSON")
    }

    /// The document declares the catalog identity its consumers select it by.
    #[test]
    fn the_embedded_document_declares_the_catalog_identity() {
        assert_eq!(
            catalog().get("version").and_then(Value::as_str),
            Some("quire.checked-operation-catalog/v1")
        );
    }

    /// Every entry is complete, has a unique identity, and names only vocabulary the
    /// catalog declares: operand and rest families or groups, its member kind, its
    /// constraint kinds and its result form.
    #[test]
    fn the_embedded_document_carries_every_operation_in_full() {
        let catalog = catalog();
        let operations = catalog
            .get("operations")
            .and_then(Value::as_array)
            .expect("the catalog declares an operations array");
        assert!(!operations.is_empty(), "the catalog declares operations");
        let names = |vocabulary: &str| -> BTreeSet<String> {
            let value = &catalog[vocabulary];
            match value {
                Value::Array(items) => items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
                Value::Object(members) => members.keys().cloned().collect(),
                _ => BTreeSet::new(),
            }
        };
        let families = names("families");
        let groups = names("groups");
        let member_kinds = names("member_kinds");
        let constraint_kinds = names("constraint_kinds");
        let position_resolves =
            |position: &str| families.contains(position) || groups.contains(position);
        let result_resolves = |form: &str| {
            families.contains(form)
                || ["member", "absence", "arm_body"].contains(&form)
                || ["operand:", "kind:", "inner:", "return:"]
                    .iter()
                    .any(|prefix| {
                        form.strip_prefix(prefix)
                            .is_some_and(|n| n.parse::<usize>().is_ok())
                    })
        };

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
            for position in operation["operands"].as_array().expect("operands array") {
                let position = position.as_str().expect("an operand position is a string");
                assert!(
                    position_resolves(position),
                    "operation `{identity}` names undeclared operand `{position}`"
                );
            }
            if let Some(rest) = operation["rest"].as_str() {
                assert!(
                    position_resolves(rest),
                    "operation `{identity}` names undeclared rest `{rest}`"
                );
            }
            if let Some(member) = operation["member"].as_str() {
                assert!(
                    member_kinds.contains(member),
                    "operation `{identity}` names undeclared member kind `{member}`"
                );
            }
            for constraint in operation["constraints"]
                .as_array()
                .expect("constraints array")
            {
                let kind = constraint["kind"].as_str().expect("a constraint kind");
                assert!(
                    constraint_kinds.contains(kind),
                    "operation `{identity}` names undeclared constraint kind `{kind}`"
                );
            }
            let result = operation["result"].as_str().expect("a result form");
            assert!(
                result_resolves(result),
                "operation `{identity}` names undeclared result form `{result}`"
            );
        }

        let identities: BTreeSet<&str> = operations
            .iter()
            .filter_map(|operation| operation.get("identity").and_then(Value::as_str))
            .collect();
        assert_eq!(
            identities.len(),
            operations.len(),
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

    /// The object family, the `field_owner` group, the `conforming_reference` kind and
    /// the three operations that use them.
    #[test]
    fn the_embedded_document_pins_object_field_owner_and_conforming_reference() {
        let catalog = catalog();
        let strings = |value: &Value| -> Vec<String> {
            value
                .as_array()
                .expect("an array")
                .iter()
                .map(|item| item.as_str().expect("a string").to_owned())
                .collect()
        };
        assert!(strings(&catalog["families"]).contains(&"object".to_owned()));
        assert_eq!(
            strings(&catalog["groups"]["field_owner"]),
            ["record", "object"]
        );
        assert!(strings(&catalog["constraint_kinds"]).contains(&"conforming_reference".to_owned()));

        let operation = |identity: &str| -> Value {
            catalog["operations"]
                .as_array()
                .expect("operations array")
                .iter()
                .find(|entry| entry["identity"] == identity)
                .unwrap_or_else(|| panic!("operation `{identity}` is declared"))
                .clone()
        };
        for identity in ["quire.op.reference.eq", "quire.op.reference.ne"] {
            let constraints = &operation(identity)["constraints"];
            assert_eq!(constraints.as_array().map(Vec::len), Some(1));
            assert_eq!(constraints[0]["kind"], "conforming_reference");
            assert_eq!(constraints[0]["operands"], serde_json::json!([0, 1]));
        }
        assert_eq!(
            operation("quire.op.record.project")["operands"],
            serde_json::json!(["field_owner"])
        );
    }
}
