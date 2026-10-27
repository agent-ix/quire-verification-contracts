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
    /// catalog declares: operand and rest families or groups, its law roles, mode
    /// kind, member kind, constraint kinds, leaf source and result form.
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
        let result_forms = names("result_forms");
        let law_roles: BTreeSet<String> = names("law_roles")
            .into_iter()
            .chain(names("profile_law_roles"))
            .collect();
        let modes = names("modes");
        let leaf_sources = names("leaf_sources");
        let position_resolves =
            |position: &str| families.contains(position) || groups.contains(position);
        // A declared form is `<family>` (any family), a `<prefix>:<n>` pattern
        // (an operand index), or a literal form name.
        let result_resolves = |form: &str| {
            result_forms.iter().any(|declared| {
                if declared == "<family>" {
                    families.contains(form)
                } else if let Some(prefix) = declared.strip_suffix("<n>") {
                    form.strip_prefix(prefix)
                        .is_some_and(|index| index.parse::<usize>().is_ok())
                } else {
                    declared == form
                }
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
            for law in operation["laws"].as_array().expect("laws array") {
                let law = law.as_str().expect("a law role is a string");
                assert!(
                    law_roles.contains(law),
                    "operation `{identity}` names undeclared law role `{law}`"
                );
            }
            if let Some(mode) = operation["mode"].as_str() {
                assert!(
                    modes.contains(mode),
                    "operation `{identity}` names undeclared mode kind `{mode}`"
                );
            }
            if let Some(leaves) = operation["leaves"].as_str() {
                assert!(
                    leaf_sources.contains(leaves),
                    "operation `{identity}` names undeclared leaf source `{leaves}`"
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
    /// The temporal operations of QSpec FR-370 and the union `case` of FR-440, with
    /// the families and groups they use.
    /// Trace: FR-370-AC-14.
    #[test]
    fn the_embedded_document_pins_temporal_operations_and_union_case() {
        use serde_json::json;

        let catalog = catalog();
        let strings = |value: &Value| -> Vec<String> {
            value
                .as_array()
                .expect("an array")
                .iter()
                .map(|item| item.as_str().expect("a string").to_owned())
                .collect()
        };
        let operation = |identity: &str| -> Value {
            catalog["operations"]
                .as_array()
                .expect("operations array")
                .iter()
                .find(|entry| entry["identity"] == identity)
                .unwrap_or_else(|| panic!("operation `{identity}` is declared"))
                .clone()
        };

        let families = strings(&catalog["families"]);
        for family in ["temporal", "union"] {
            assert!(families.contains(&family.to_owned()), "family `{family}`");
        }
        assert!(strings(&catalog["groups"]["any_term"]).contains(&"temporal".to_owned()));
        for group in ["any_value", "any_term", "structural_kind"] {
            assert!(
                strings(&catalog["groups"][group]).contains(&"union".to_owned()),
                "`union` is in `{group}`"
            );
        }
        let members = strings(&catalog["member_kinds"]);
        for kind in ["temporal_interval", "fairness"] {
            assert!(members.contains(&kind.to_owned()), "member kind `{kind}`");
        }
        assert!(strings(&catalog["result_forms"]).contains(&"arm_body".to_owned()));
        assert!(strings(&catalog["constraint_kinds"]).contains(&"union_arms".to_owned()));

        let clause = operation("quire.op.temporal.clause");
        assert_eq!(clause["operator"], "temporal");
        assert_eq!(
            clause["operands"],
            json!([
                "reference",
                "text",
                "aggregate",
                "aggregate",
                "aggregate",
                "temporal"
            ])
        );
        assert_eq!(clause["rest"], Value::Null);
        assert_eq!(clause["result"], "clause");
        assert_eq!(clause["laws"], json!(["temporal_profile"]));
        assert_eq!(clause["member"], Value::Null);

        // (identities, operator class, operands, member) per FR-370 table row.
        let rows: [(&[&str], &str, Value, Value); 8] = [
            (
                &["holds"],
                "temporal_formula",
                json!(["boolean"]),
                Value::Null,
            ),
            (
                &["aggregate"],
                "temporal_formula",
                json!(["aggregate"]),
                Value::Null,
            ),
            (
                &["true", "false"],
                "temporal_formula",
                json!([]),
                Value::Null,
            ),
            (
                &["not"],
                "temporal_formula",
                json!(["temporal"]),
                Value::Null,
            ),
            (
                &["and", "or", "implies"],
                "temporal_formula",
                json!(["temporal", "temporal"]),
                Value::Null,
            ),
            (
                &["eventually", "always", "once", "historically"],
                "temporal_formula",
                json!(["temporal"]),
                json!("temporal_interval"),
            ),
            (
                &["until", "release", "since", "triggered"],
                "temporal_formula",
                json!(["temporal", "temporal"]),
                json!("temporal_interval"),
            ),
            (&["fair"], "temporal_fairness", json!([]), json!("fairness")),
        ];
        for (names, operator, operands, member) in rows {
            for name in names {
                let entry = operation(&format!("quire.op.temporal.{name}"));
                assert_eq!(entry["operator"], operator, "{name} operator class");
                assert_eq!(entry["operands"], operands, "{name} operands");
                assert_eq!(entry["rest"], Value::Null, "{name} rest");
                assert_eq!(entry["result"], "temporal", "{name} result");
                assert_eq!(entry["laws"], json!([]), "{name} laws");
                assert_eq!(entry["member"], member, "{name} member");
            }
        }

        let aggregate = operation("quire.op.temporal.aggregate");
        assert_eq!(aggregate["mode"], Value::Null);
        assert_eq!(aggregate["constraints"], json!([]));
        assert_eq!(aggregate["leaves"], Value::Null);

        let case = operation("quire.op.control.case");
        assert_eq!(case["operator"], "case");
        assert_eq!(case["operands"], json!(["union"]));
        assert_eq!(case["rest"], "binder");
        assert_eq!(case["result"], "arm_body");
        assert_eq!(case["laws"], json!([]));
        assert_eq!(case["mode"], Value::Null);
        assert_eq!(case["member"], Value::Null);
        assert_eq!(case["leaves"], Value::Null);
        assert_eq!(
            case["constraints"],
            json!([{"kind": "union_arms", "operands": [0], "family": null}])
        );
    }

    /// The `string`, `uuid` and `timestamp` scalar families are value families: members
    /// of `any_value` and `any_term` and of no other group, so the operations that take
    /// any value accept them and no ordering or arithmetic operation does.
    #[test]
    fn the_embedded_document_admits_string_uuid_timestamp_as_values_only() {
        let catalog = catalog();
        let strings = |value: &Value| -> Vec<String> {
            value
                .as_array()
                .expect("an array")
                .iter()
                .map(|item| item.as_str().expect("a string").to_owned())
                .collect()
        };
        let groups = catalog["groups"].as_object().expect("groups object");
        let operations = catalog["operations"].as_array().expect("operations array");
        for family in ["string", "uuid", "timestamp"] {
            assert!(strings(&catalog["families"]).contains(&family.to_owned()));
            let containing: BTreeSet<&str> = groups
                .iter()
                .filter(|(_, members)| strings(members).contains(&family.to_owned()))
                .map(|(name, _)| name.as_str())
                .collect();
            assert_eq!(
                containing,
                BTreeSet::from(["any_term", "any_value"]),
                "`{family}` belongs to exactly any_value and any_term"
            );

            // A position admits the family when it names it or a group holding it.
            let admits = |position: &str| position == family || containing.contains(position);
            let accepting: BTreeSet<&str> = operations
                .iter()
                .filter(|entry| {
                    entry["operands"]
                        .as_array()
                        .expect("operands array")
                        .iter()
                        .chain(std::iter::once(&entry["rest"]))
                        .any(|position| position.as_str().is_some_and(admits))
                })
                .map(|entry| entry["identity"].as_str().expect("an identity"))
                .collect();
            for identity in [
                "quire.op.control.if",
                "quire.op.control.let",
                "quire.op.state.pre",
                "quire.op.collection.contains",
                "quire.op.collection.set",
            ] {
                assert!(
                    accepting.contains(identity),
                    "`{identity}` accepts `{family}`"
                );
            }
            const ORDERING_AND_ARITHMETIC: [&str; 12] = [
                "lt", "le", "gt", "ge", "add", "sub", "mul", "div", "rem", "mod", "pow", "negate",
            ];
            for identity in &accepting {
                let operator = identity.rsplit('.').next().expect("an operator segment");
                assert!(
                    !ORDERING_AND_ARITHMETIC.contains(&operator),
                    "`{identity}` is ordering or arithmetic and must not accept `{family}`"
                );
            }
        }
    }

    /// The six equality identities over `string`, `uuid` and `timestamp`: two operands
    /// of the identity's own family, a `boolean` result, no law, mode, member or leaves,
    /// and no ordering or arithmetic identity for these families.
    #[test]
    fn the_embedded_document_pins_scalar_equality_operations() {
        use serde_json::json;

        let catalog = catalog();
        let operations = catalog["operations"].as_array().expect("operations array");
        let find = |identity: &str| {
            operations
                .iter()
                .find(|entry| entry["identity"] == identity)
        };
        for family in ["string", "uuid", "timestamp"] {
            for operator in ["eq", "ne"] {
                let identity = format!("quire.op.{family}.{operator}");
                let entry = find(&identity).unwrap_or_else(|| panic!("`{identity}` is declared"));
                assert_eq!(entry["operator"], "binary", "{identity}");
                assert_eq!(entry["operands"], json!([family, family]), "{identity}");
                assert_eq!(entry["rest"], Value::Null, "{identity}");
                assert_eq!(entry["result"], "boolean", "{identity}");
                assert_eq!(entry["laws"], json!([]), "{identity}");
                assert_eq!(entry["mode"], Value::Null, "{identity}");
                assert_eq!(entry["member"], Value::Null, "{identity}");
                assert_eq!(entry["constraints"], json!([]), "{identity}");
                assert_eq!(entry["leaves"], Value::Null, "{identity}");
            }
            for operator in ["lt", "le", "gt", "ge", "add", "sub", "mul", "div", "negate"] {
                assert!(
                    find(&format!("quire.op.{family}.{operator}")).is_none(),
                    "`{family}` has no `{operator}` identity"
                );
            }
        }
        // Equality uses only the identity of its own family. The `string` family also
        // participates as a source of FR-478 text-to-number conversions.
        for family in ["string", "uuid", "timestamp"] {
            let direct: Vec<&str> = operations
                .iter()
                .filter(|entry| {
                    entry["operands"]
                        .as_array()
                        .is_some_and(|ops| ops.iter().any(|p| p == family))
                })
                .map(|entry| entry["identity"].as_str().expect("an identity"))
                .collect();
            let expected = if family == "string" {
                [
                    "quire.op.string.eq",
                    "quire.op.string.ne",
                    "quire.op.string.to_integer",
                    "quire.op.string.to_decimal",
                    "quire.op.string.to_rational",
                    "quire.op.string.to_float32",
                    "quire.op.string.to_float64",
                    "quire.op.string.to_quantity",
                ]
                .map(str::to_owned)
                .to_vec()
            } else {
                vec![
                    format!("quire.op.{family}.eq"),
                    format!("quire.op.{family}.ne"),
                ]
            };
            assert_eq!(
                direct, expected,
                "the `{family}` operand identities match the catalog"
            );
        }
    }

    /// The twelve FR-478 conversions pair each numeric target with `Text` and
    /// `String`, retaining the source and target type contract on every row.
    #[test]
    fn the_embedded_document_pins_text_and_string_number_conversions() {
        use serde_json::json;

        let catalog = catalog();
        let operations = catalog["operations"].as_array().expect("operations array");
        let targets = [
            ("integer", "integer", None, false),
            ("decimal", "decimal", Some("rounding"), false),
            ("rational", "rational", None, false),
            ("float32", "float32", Some("rounding"), true),
            ("float64", "float64", Some("rounding"), true),
            ("quantity", "quantity", Some("rounding"), false),
        ];
        let mut expected_order = Vec::new();
        for source in ["text", "string"] {
            for (target, family, mode, ieee) in targets {
                let identity = format!("quire.op.{source}.to_{target}");
                expected_order.push(identity.clone());
                let entry = operations
                    .iter()
                    .find(|entry| entry["identity"] == identity)
                    .unwrap_or_else(|| panic!("`{identity}` is catalogued"));
                let mut laws = Vec::new();
                if source == "text" {
                    laws.push("text_profile");
                }
                if ieee {
                    laws.push("ieee_profile");
                }
                assert_eq!(entry["operator"], "convert", "{identity}");
                assert_eq!(entry["operands"], json!([source]), "{identity}");
                assert_eq!(entry["rest"], Value::Null, "{identity}");
                assert_eq!(entry["result"], "member", "{identity}");
                assert_eq!(entry["laws"], json!(laws), "{identity}");
                assert_eq!(
                    entry["mode"],
                    mode.map_or(Value::Null, |mode| json!(mode)),
                    "{identity}"
                );
                assert_eq!(entry["member"], "type_argument", "{identity}");
                assert_eq!(
                    entry["constraints"],
                    json!([{"kind": "member_family", "operands": [], "family": family}]),
                    "{identity}"
                );
                assert_eq!(entry["leaves"], Value::Null, "{identity}");
            }
        }
        let actual_order: Vec<&str> = operations
            .iter()
            .filter_map(|entry| entry["identity"].as_str())
            .filter(|identity| {
                identity.starts_with("quire.op.text.to_")
                    || identity.starts_with("quire.op.string.to_")
            })
            .collect();
        assert_eq!(actual_order, expected_order);
        let all_identities: Vec<&str> = operations
            .iter()
            .map(|entry| entry["identity"].as_str().expect("an identity"))
            .collect();
        let text_anchor = all_identities
            .iter()
            .position(|identity| *identity == "quire.op.text.size")
            .expect("text.size is catalogued");
        let string_anchor = all_identities
            .iter()
            .position(|identity| *identity == "quire.op.string.ne")
            .expect("string.ne is catalogued");
        assert_eq!(
            &all_identities[text_anchor + 1..text_anchor + 7],
            expected_order[..6]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            &all_identities[string_anchor + 1..string_anchor + 7],
            expected_order[6..]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
    }

    /// The `bytes` and `json_object` scalar families are value families: members of
    /// `any_value` and `any_term` and of no other group. `bytes` has equality, `length`
    /// and `concat`; `json_object` has equality only.
    #[test]
    fn the_embedded_document_pins_bytes_and_json_object_operations() {
        use serde_json::json;

        let catalog = catalog();
        let operations = catalog["operations"].as_array().expect("operations array");
        let find = |identity: &str| {
            operations
                .iter()
                .find(|entry| entry["identity"] == identity)
        };
        let strings = |value: &Value| -> Vec<String> {
            value
                .as_array()
                .expect("an array")
                .iter()
                .map(|item| item.as_str().expect("a string").to_owned())
                .collect()
        };
        for family in ["bytes", "json_object"] {
            assert!(strings(&catalog["families"]).contains(&family.to_owned()));
            let containing: BTreeSet<&str> = catalog["groups"]
                .as_object()
                .expect("groups object")
                .iter()
                .filter(|(_, members)| strings(members).contains(&family.to_owned()))
                .map(|(name, _)| name.as_str())
                .collect();
            assert_eq!(
                containing,
                BTreeSet::from(["any_term", "any_value"]),
                "`{family}` belongs to exactly any_value and any_term"
            );
            for operator in ["eq", "ne"] {
                let identity = format!("quire.op.{family}.{operator}");
                let entry = find(&identity).unwrap_or_else(|| panic!("`{identity}` is declared"));
                assert_eq!(entry["operator"], "binary", "{identity}");
                assert_eq!(entry["operands"], json!([family, family]), "{identity}");
                assert_eq!(entry["rest"], Value::Null, "{identity}");
                assert_eq!(entry["result"], "boolean", "{identity}");
                assert_eq!(entry["laws"], json!([]), "{identity}");
                assert_eq!(entry["mode"], Value::Null, "{identity}");
                assert_eq!(entry["member"], Value::Null, "{identity}");
                assert_eq!(entry["constraints"], json!([]), "{identity}");
                assert_eq!(entry["leaves"], Value::Null, "{identity}");
            }
            for operator in ["lt", "le", "gt", "ge", "add", "sub", "mul", "div", "negate"] {
                assert!(
                    find(&format!("quire.op.{family}.{operator}")).is_none(),
                    "`{family}` has no `{operator}` identity"
                );
            }
        }
        let length = find("quire.op.bytes.length").expect("`quire.op.bytes.length` is declared");
        assert_eq!(length["operator"], "unary");
        assert_eq!(length["operands"], json!(["bytes"]));
        assert_eq!(length["result"], "integer");
        assert_eq!(length["laws"], json!([]));
        assert_eq!(length["mode"], Value::Null);
        let concat = find("quire.op.bytes.concat").expect("`quire.op.bytes.concat` is declared");
        assert_eq!(concat["operator"], "binary");
        assert_eq!(concat["operands"], json!(["bytes", "bytes"]));
        assert_eq!(concat["result"], "bytes");
        assert_eq!(concat["laws"], json!([]));
        assert_eq!(concat["mode"], Value::Null);
        let bytes_identities: BTreeSet<&str> = operations
            .iter()
            .filter_map(|entry| entry["identity"].as_str())
            .filter(|identity| identity.starts_with("quire.op.bytes."))
            .collect();
        assert_eq!(
            bytes_identities,
            BTreeSet::from([
                "quire.op.bytes.concat",
                "quire.op.bytes.eq",
                "quire.op.bytes.length",
                "quire.op.bytes.ne"
            ])
        );
        let json_identities: BTreeSet<&str> = operations
            .iter()
            .filter_map(|entry| entry["identity"].as_str())
            .filter(|identity| identity.starts_with("quire.op.json_object."))
            .collect();
        assert_eq!(
            json_identities,
            BTreeSet::from(["quire.op.json_object.eq", "quire.op.json_object.ne"])
        );
    }

    /// IEEE negate is the sign-bit flip of one float: one operand of its own width and
    /// the operand's own type (rounding mode included) as result, the IEEE profile law and no rounding mode (the operation is
    /// exact). IEEE equality and ordering stay the `numeric_equal`, `total_order` and
    /// `bit_identical` identities; no `eq`, `ne` or ordering identity exists.
    #[test]
    fn the_embedded_document_pins_ieee_negate() {
        use serde_json::json;

        let catalog = catalog();
        let operations = catalog["operations"].as_array().expect("operations array");
        for width in ["float32", "float64"] {
            let identity = format!("quire.op.ieee.{width}.negate");
            let entry = operations
                .iter()
                .find(|entry| entry["identity"] == identity.as_str())
                .unwrap_or_else(|| panic!("`{identity}` is declared"));
            assert_eq!(entry["operator"], "unary", "{identity}");
            assert_eq!(entry["operands"], json!([width]), "{identity}");
            assert_eq!(entry["rest"], Value::Null, "{identity}");
            assert_eq!(entry["result"], "operand:0", "{identity}");
            assert_eq!(entry["laws"], json!(["ieee_profile"]), "{identity}");
            assert_eq!(entry["mode"], Value::Null, "{identity}");
            assert_eq!(entry["member"], Value::Null, "{identity}");
            assert_eq!(entry["constraints"], json!([]), "{identity}");
            assert_eq!(entry["leaves"], Value::Null, "{identity}");
            for operator in ["eq", "ne", "lt", "le", "gt", "ge"] {
                assert!(
                    !operations.iter().any(|entry| entry["identity"]
                        == format!("quire.op.ieee.{width}.{operator}").as_str()),
                    "`{width}` has no `{operator}` identity"
                );
            }
        }
    }

    /// Quantity negate is the exact unary minus of one quantity: one `quantity` operand and the
    /// operand's own type as result, no law, no mode and no constraint (the result unit is the
    /// operand's unit, so there is nothing to agree).
    #[test]
    fn the_embedded_document_pins_quantity_negate() {
        use serde_json::json;

        let catalog = catalog();
        let operations = catalog["operations"].as_array().expect("operations array");
        let entry = operations
            .iter()
            .find(|entry| entry["identity"] == "quire.op.quantity.negate")
            .expect("`quire.op.quantity.negate` is declared");
        assert_eq!(entry["operator"], "unary");
        assert_eq!(entry["operands"], json!(["quantity"]));
        assert_eq!(entry["rest"], Value::Null);
        assert_eq!(entry["result"], "operand:0");
        assert_eq!(entry["laws"], json!([]));
        assert_eq!(entry["mode"], Value::Null);
        assert_eq!(entry["member"], Value::Null);
        assert_eq!(entry["constraints"], json!([]));
        assert_eq!(entry["leaves"], Value::Null);
    }

    /// The one-occurrence collection forms: `drop_first` takes one `sequence` or one
    /// `ordered_set` (a set or bag has no first occurrence, so no identity names them) and
    /// returns a collection of its operand's kind; `remove_one` takes any collection and a value of its
    /// element type, compares as `contains` does (leaves `inner:0`) and returns a collection
    /// of its operand's kind.
    #[test]
    fn the_embedded_document_pins_one_occurrence_collection_forms() {
        use serde_json::json;

        let catalog = catalog();
        let operations = catalog["operations"].as_array().expect("operations array");
        let find = |identity: &str| {
            operations
                .iter()
                .find(|entry| entry["identity"] == identity)
                .unwrap_or_else(|| panic!("`{identity}` is declared"))
        };
        for family in ["sequence", "ordered_set"] {
            let identity = format!("quire.op.collection.{family}.drop_first");
            let entry = find(&identity);
            assert_eq!(entry["operator"], "collection", "{identity}");
            assert_eq!(entry["operands"], json!([family]), "{identity}");
            assert_eq!(entry["rest"], Value::Null, "{identity}");
            assert_eq!(entry["result"], "kind:0", "{identity}");
            assert_eq!(entry["laws"], json!([]), "{identity}");
            assert_eq!(entry["mode"], Value::Null, "{identity}");
            assert_eq!(entry["member"], Value::Null, "{identity}");
            assert_eq!(entry["constraints"], json!([]), "{identity}");
            assert_eq!(entry["leaves"], Value::Null, "{identity}");
        }
        for family in ["set", "bag"] {
            assert!(
                !operations.iter().any(|entry| entry["identity"]
                    == format!("quire.op.collection.{family}.drop_first").as_str()),
                "`{family}` has no `drop_first` identity"
            );
        }
        let entry = find("quire.op.collection.remove_one");
        assert_eq!(entry["operator"], "collection");
        assert_eq!(entry["operands"], json!(["collection_kind", "any_value"]));
        assert_eq!(entry["rest"], Value::Null);
        assert_eq!(entry["result"], "kind:0");
        assert_eq!(entry["laws"], json!([]));
        assert_eq!(entry["mode"], Value::Null);
        assert_eq!(
            entry["constraints"],
            json!([{"kind": "inner_type", "operands": [0, 1], "family": null}])
        );
        assert_eq!(entry["leaves"], "inner:0");
    }
}
