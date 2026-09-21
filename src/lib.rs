// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Schema-owned public contracts, RFC 8785 canonicalization, and bounded JSON ingestion.
//!
//! Extracted from private `quire-verification` (PLAT-861) as the public interface-vocabulary
//! boundary consumed directly by `quire-protocol`. `quire-verification` depends on this crate
//! and re-exports it at `contracts`, so its own internal call sites — including the internal,
//! non-public schema definitions validated through [`validate_schema_definition`] — are
//! unchanged.
//!
//! Governing requirements: FR-001, FR-002, FR-008, FR-022, FR-024, NFR-002, and NFR-005.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use jsonschema::{Draft, Registry, Validator};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Maximum serialized JSON bytes accepted by a public ingestion boundary.
pub const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;
/// Maximum nested arrays and objects accepted by contract validation.
pub const MAX_JSON_DEPTH: usize = 128;
/// Maximum number of items accepted in one JSON array.
pub const MAX_ARRAY_ITEMS: usize = 100_000;

/// Rust wire types generated directly from the authoritative E02 JSON Schema.
#[allow(missing_docs, clippy::all, clippy::pedantic)]
pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/e02_contracts.rs"));
}

/// Stable error codes exposed by the crate boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationErrorCode {
    /// A public input has an invalid shape or value.
    InvalidInput,
    /// A timestamp is not valid RFC 3339.
    InvalidTimestamp,
    /// Timestamp ordering makes the evidence temporally impossible.
    InvalidTemporalOrder,
    /// Input exceeded the serialized byte limit.
    JsonDocumentTooLarge,
    /// Input exceeded the JSON nesting limit.
    JsonNestingTooDeep,
    /// An array exceeded the item limit.
    JsonArrayTooLarge,
    /// Bytes were not valid JSON.
    InvalidJson,
    /// JSON did not satisfy the authoritative schema.
    SchemaViolation,
    /// A requested internal schema definition does not exist.
    UnknownSchemaDefinition,
    /// RFC 8785 canonicalization failed.
    CanonicalizationFailed,
    /// A plan contains a duplicate task identifier.
    DuplicateTaskId,
    /// A plan dependency names no task.
    UnknownTaskDependency,
    /// A plan dependency graph contains a cycle.
    TaskDependencyCycle,
    /// A catalog repeats a technique identifier.
    DuplicateTechniqueId,
    /// A request repeats a property identifier.
    DuplicatePropertyId,
    /// A selection names no property.
    UnknownProperty,
    /// More than one capability matches without an explicit selection.
    AmbiguousCapability,
    /// Evidence repeats a result identifier.
    DuplicateResultId,
    /// A result repeats an oracle contribution identifier.
    DuplicateOracleContributionId,
    /// Derived oracle contribution lineage is absent, self-referential, or cyclic.
    InvalidOracleLineage,
    /// A structured request uses an unsupported contract version.
    UnsupportedContractVersion,
    /// A shared-reference envelope, artifact reference, or selected profile version is unsupported.
    UnsupportedReferenceVersion,
    /// A semantic reference requires a feature outside the selected reader profile.
    UnknownRequiredFeature,
    /// Shared-reference bytes are malformed or outside the selected wire schema.
    InvalidWire,
    /// A valid reference does not match the locally selected identity or content.
    IdentityMismatch,
    /// One immutable reference identity names conflicting content in an input set.
    IdentityContentConflict,
    /// A selected local artifact revision is unavailable.
    Stale,
    /// A source map does not preserve exact, ordered byte correspondence.
    MalformedSourceMap,
    /// A semantic reference selects an unsupported canonicalization profile.
    UnsupportedCanonicalization,
    /// More than one explicit migration applies to the selected reference pair.
    AmbiguousEquivalence,
    /// A local or process I/O operation failed.
    IoFailure,
    /// A discovered JSON fixture has no registered validator.
    UnhandledFixture,
    /// External catalog output differs from the pinned baseline.
    CatalogParityMismatch,
    /// A native execution request has no finite admitted deadline.
    InvalidExecutionDeadline,
    /// A native execution request has no single admitted numeric exit status.
    InvalidExpectedExitStatus,
    /// A native executable is not an exact regular file inside its selected root.
    InvalidExecutablePath,
    /// A native child remained live at its admitted deadline.
    ExecutionTimeout,
    /// A native child emitted the first byte beyond an admitted stream limit.
    OutputLimitExceeded,
    /// A native child terminated because of a signal.
    ProcessSignaled,
    /// A native child returned an unexpected nonzero exit status.
    UnexpectedExitStatus,
    /// A native child stream was not valid UTF-8.
    InvalidProcessOutput,
    /// A completed native child emitted a malformed method response.
    MalformedMethodResponse,
    /// A VP04 measurement series violates its fixed identity, order, or summary contract.
    InvalidMeasurementSeries,
    /// A qualification corpus violates its frozen population, retention, or lineage contract.
    InvalidCorpusManifest,
    /// A qualification oracle manifest violates its frozen coverage or dependence contract.
    InvalidQualificationOracleManifest,
    /// A qualification oracle binding carries an invalid exact reference or reuse identity.
    InvalidQualificationOracleBinding,
    /// A qualification technique manifest violates its frozen coverage or configuration contract.
    InvalidQualificationTechniqueManifest,
    /// A qualification case or replay differs from its frozen technique configuration.
    InvalidQualificationCase,
    /// A required qualification technique or accepted producer capability is unavailable.
    QualificationTechniqueUnavailable,
    /// More than one human disposition was supplied for one report core.
    DuplicateDisposition,
    /// A disposition names another core, procedure, or reviewer.
    ForeignDisposition,
    /// A disposition or its exact shared-reference role is unsupported.
    UnsupportedDisposition,
    /// A report-core or disposition byte digest is invalid.
    InvalidReportDigest,
    /// The selected Linux descriptor or process-group profile is unavailable.
    UnsupportedExecutionProfile,
    /// A native descendant remained after its direct child completed.
    UnexpectedProcessDescendant,
    /// Native process-group signaling, reaping, or termination observation failed.
    ProcessContainmentFailure,
    /// A checked model domain, identity, predicate, or bound is malformed.
    InvalidModelDomain,
    /// Checked preconditions and relationships admit no model assignment.
    UnsatisfiableModelDomain,
    /// Model-domain expansion exceeded its explicit construction bound.
    ModelConstructionBoundExceeded,
    /// Complete boundary partitions exceeded the explicit case bound.
    ModelCaseBoundExceeded,
}

impl VerificationErrorCode {
    /// Returns the stable wire code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::InvalidTimestamp => "invalid_timestamp",
            Self::InvalidTemporalOrder => "invalid_temporal_order",
            Self::JsonDocumentTooLarge => "json_document_too_large",
            Self::JsonNestingTooDeep => "json_nesting_too_deep",
            Self::JsonArrayTooLarge => "json_array_too_large",
            Self::InvalidJson => "invalid_json",
            Self::SchemaViolation => "schema_violation",
            Self::UnknownSchemaDefinition => "unknown_schema_definition",
            Self::CanonicalizationFailed => "canonicalization_failed",
            Self::DuplicateTaskId => "duplicate_task_id",
            Self::UnknownTaskDependency => "unknown_task_dependency",
            Self::TaskDependencyCycle => "task_dependency_cycle",
            Self::DuplicateTechniqueId => "duplicate_technique_id",
            Self::DuplicatePropertyId => "duplicate_property_id",
            Self::UnknownProperty => "unknown_property",
            Self::AmbiguousCapability => "ambiguous_capability",
            Self::DuplicateResultId => "duplicate_result_id",
            Self::DuplicateOracleContributionId => "duplicate_oracle_contribution_id",
            Self::InvalidOracleLineage => "invalid_oracle_lineage",
            Self::UnsupportedContractVersion => "unsupported_contract_version",
            Self::UnsupportedReferenceVersion => "unsupported_reference_version",
            Self::UnknownRequiredFeature => "unknown_required_feature",
            Self::InvalidWire => "invalid_wire",
            Self::IdentityMismatch => "identity_mismatch",
            Self::IdentityContentConflict => "identity_content_conflict",
            Self::Stale => "stale",
            Self::MalformedSourceMap => "malformed_source_map",
            Self::UnsupportedCanonicalization => "unsupported_canonicalization",
            Self::AmbiguousEquivalence => "ambiguous_equivalence",
            Self::IoFailure => "io_failure",
            Self::UnhandledFixture => "unhandled_fixture",
            Self::CatalogParityMismatch => "catalog_parity_mismatch",
            Self::InvalidExecutionDeadline => "invalid_execution_deadline",
            Self::InvalidExpectedExitStatus => "invalid_expected_exit_status",
            Self::InvalidExecutablePath => "invalid_executable_path",
            Self::ExecutionTimeout => "execution_timeout",
            Self::OutputLimitExceeded => "output_limit_exceeded",
            Self::ProcessSignaled => "process_signaled",
            Self::UnexpectedExitStatus => "unexpected_exit_status",
            Self::InvalidProcessOutput => "invalid_process_output",
            Self::MalformedMethodResponse => "malformed_method_response",
            Self::InvalidMeasurementSeries => "invalid_measurement_series",
            Self::InvalidCorpusManifest => "invalid_corpus_manifest",
            Self::InvalidQualificationOracleManifest => "invalid_qualification_oracle_manifest",
            Self::InvalidQualificationOracleBinding => "invalid_qualification_oracle_binding",
            Self::InvalidQualificationTechniqueManifest => {
                "invalid_qualification_technique_manifest"
            }
            Self::InvalidQualificationCase => "invalid_qualification_case",
            Self::QualificationTechniqueUnavailable => "qualification_technique_unavailable",
            Self::DuplicateDisposition => "duplicate_disposition",
            Self::ForeignDisposition => "foreign_disposition",
            Self::UnsupportedDisposition => "unsupported_disposition",
            Self::InvalidReportDigest => "invalid_report_digest",
            Self::UnsupportedExecutionProfile => "unsupported_execution_profile",
            Self::UnexpectedProcessDescendant => "unexpected_process_descendant",
            Self::ProcessContainmentFailure => "process_containment_failure",
            Self::InvalidModelDomain => "invalid_model_domain",
            Self::UnsatisfiableModelDomain => "unsatisfiable_model_domain",
            Self::ModelConstructionBoundExceeded => "model_construction_bound_exceeded",
            Self::ModelCaseBoundExceeded => "model_case_bound_exceeded",
        }
    }

    /// Returns every stable error code.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::InvalidInput,
            Self::InvalidTimestamp,
            Self::InvalidTemporalOrder,
            Self::JsonDocumentTooLarge,
            Self::JsonNestingTooDeep,
            Self::JsonArrayTooLarge,
            Self::InvalidJson,
            Self::SchemaViolation,
            Self::UnknownSchemaDefinition,
            Self::CanonicalizationFailed,
            Self::DuplicateTaskId,
            Self::UnknownTaskDependency,
            Self::TaskDependencyCycle,
            Self::DuplicateTechniqueId,
            Self::DuplicatePropertyId,
            Self::UnknownProperty,
            Self::AmbiguousCapability,
            Self::DuplicateResultId,
            Self::DuplicateOracleContributionId,
            Self::InvalidOracleLineage,
            Self::UnsupportedContractVersion,
            Self::UnsupportedReferenceVersion,
            Self::UnknownRequiredFeature,
            Self::InvalidWire,
            Self::IdentityMismatch,
            Self::IdentityContentConflict,
            Self::Stale,
            Self::MalformedSourceMap,
            Self::UnsupportedCanonicalization,
            Self::AmbiguousEquivalence,
            Self::IoFailure,
            Self::UnhandledFixture,
            Self::CatalogParityMismatch,
            Self::InvalidExecutionDeadline,
            Self::InvalidExpectedExitStatus,
            Self::InvalidExecutablePath,
            Self::ExecutionTimeout,
            Self::OutputLimitExceeded,
            Self::ProcessSignaled,
            Self::UnexpectedExitStatus,
            Self::InvalidProcessOutput,
            Self::MalformedMethodResponse,
            Self::InvalidMeasurementSeries,
            Self::InvalidCorpusManifest,
            Self::InvalidQualificationOracleManifest,
            Self::InvalidQualificationOracleBinding,
            Self::InvalidQualificationTechniqueManifest,
            Self::InvalidQualificationCase,
            Self::QualificationTechniqueUnavailable,
            Self::DuplicateDisposition,
            Self::ForeignDisposition,
            Self::UnsupportedDisposition,
            Self::InvalidReportDigest,
            Self::UnsupportedExecutionProfile,
            Self::UnexpectedProcessDescendant,
            Self::ProcessContainmentFailure,
            Self::InvalidModelDomain,
            Self::UnsatisfiableModelDomain,
            Self::ModelConstructionBoundExceeded,
            Self::ModelCaseBoundExceeded,
        ]
    }

    /// Parses one stable wire code.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|item| item.as_str() == code)
    }
}

impl std::fmt::Display for VerificationErrorCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Typed crate-boundary error with deterministic context ordering.
#[derive(Debug, Error)]
#[error("{code}: {message}")]
pub struct VerificationError {
    code: VerificationErrorCode,
    message: Box<str>,
    context: BTreeMap<String, String>,
}

impl VerificationError {
    /// Creates an error with a stable code and human-readable message.
    #[must_use]
    pub fn new(code: VerificationErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
            context: BTreeMap::new(),
        }
    }

    /// Adds deterministic diagnostic context.
    #[must_use]
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Returns the stable error code.
    #[must_use]
    pub const fn code(&self) -> VerificationErrorCode {
        self.code
    }

    /// Returns structured diagnostic context.
    #[must_use]
    pub const fn context(&self) -> &BTreeMap<String, String> {
        &self.context
    }
}

static CONTRACT_SCHEMA: OnceLock<Result<Value, String>> = OnceLock::new();
static CONTRACT_VALIDATOR: OnceLock<Result<Validator, String>> = OnceLock::new();
static DEFINITION_VALIDATORS: OnceLock<Mutex<BTreeMap<String, Arc<Validator>>>> = OnceLock::new();

/// One of the five schema-generated public contract values.
#[derive(Clone, Debug)]
pub enum PublicContract {
    /// A portable verification technique definition.
    TechniqueDefinition(Box<wire::TechniqueDefinition>),
    /// A qualified producer capability.
    ToolCapability(Box<wire::ToolCapability>),
    /// An authored selection policy.
    SelectionPolicy(Box<wire::SelectionPolicy>),
    /// A prerequisite-aware verification plan.
    VerificationPlan(Box<wire::VerificationPlan>),
    /// A result from one verification technique execution.
    TechniqueResult(Box<wire::TechniqueResult>),
}

/// Parses one bounded JSON document and enforces the structural resource envelope.
///
/// # Errors
/// Returns a stable resource or JSON error when the input is invalid.
pub fn parse_bounded_json(bytes: &[u8]) -> Result<Value, VerificationError> {
    if bytes.len() > MAX_JSON_BYTES {
        return Err(VerificationError::new(
            VerificationErrorCode::JsonDocumentTooLarge,
            "serialized JSON exceeds 16 MiB",
        )
        .with_context("bytes", bytes.len().to_string())
        .with_context("limit", MAX_JSON_BYTES.to_string()));
    }
    let value = serde_json::from_slice(bytes).map_err(|error| {
        VerificationError::new(VerificationErrorCode::InvalidJson, error.to_string())
    })?;
    validate_resource_envelope(&value)?;
    Ok(value)
}

/// Enforces nesting and array-cardinality limits with an iterative walk.
///
/// # Errors
/// Returns a resource-limit error when the value exceeds a configured limit.
pub fn validate_resource_envelope(value: &Value) -> Result<(), VerificationError> {
    let mut pending = vec![(value, 0_usize)];
    while let Some((current, depth)) = pending.pop() {
        if depth > MAX_JSON_DEPTH {
            return Err(VerificationError::new(
                VerificationErrorCode::JsonNestingTooDeep,
                "JSON nesting exceeds 128 levels",
            )
            .with_context("depth", depth.to_string())
            .with_context("limit", MAX_JSON_DEPTH.to_string()));
        }
        match current {
            Value::Array(items) => {
                if items.len() > MAX_ARRAY_ITEMS {
                    return Err(VerificationError::new(
                        VerificationErrorCode::JsonArrayTooLarge,
                        "JSON array exceeds 100,000 items",
                    )
                    .with_context("items", items.len().to_string())
                    .with_context("limit", MAX_ARRAY_ITEMS.to_string()));
                }
                pending.extend(items.iter().map(|item| (item, depth + 1)));
            }
            Value::Object(fields) => {
                pending.extend(fields.values().map(|item| (item, depth + 1)));
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }
    Ok(())
}

/// Validates and decodes exactly one of the five public contracts.
///
/// # Errors
/// Returns a schema or graph error when the value is outside the public packet.
pub fn validate_contract(value: Value) -> Result<PublicContract, VerificationError> {
    validate_resource_envelope(&value)?;
    reject_unknown_contract_version(&value)?;
    validate_with(contract_validator()?, &value, "E02 contract")?;
    let kind = value.get("kind").and_then(Value::as_str).ok_or_else(|| {
        VerificationError::new(
            VerificationErrorCode::SchemaViolation,
            "public contract kind is missing",
        )
    })?;
    match kind {
        "TechniqueDefinition" => decode(value)
            .map(Box::new)
            .map(PublicContract::TechniqueDefinition),
        "ToolCapability" => decode(value)
            .map(Box::new)
            .map(PublicContract::ToolCapability),
        "SelectionPolicy" => decode(value)
            .map(Box::new)
            .map(PublicContract::SelectionPolicy),
        "VerificationPlan" => {
            let plan: wire::VerificationPlan = decode(value)?;
            validate_plan_graph(&plan)?;
            Ok(PublicContract::VerificationPlan(Box::new(plan)))
        }
        "TechniqueResult" => decode(value)
            .map(Box::new)
            .map(PublicContract::TechniqueResult),
        _ => Err(VerificationError::new(
            VerificationErrorCode::SchemaViolation,
            "unknown public contract kind",
        )
        .with_context("kind", kind)),
    }
}

/// Validates a verification plan, including graph invariants, without consuming it.
///
/// # Errors
/// Returns a schema or graph error for invalid task content or dependencies.
pub fn validate_plan_value(value: &Value) -> Result<(), VerificationError> {
    validate_schema_definition("VerificationPlan", value)?;
    let plan: wire::VerificationPlan = decode(value.clone())?;
    validate_plan_graph(&plan)
}

/// Validates one value against a named definition in the authoritative schema.
///
/// Used both by this crate's own [`validate_contract`]/[`validate_plan_value`] and directly by
/// `quire-verification`'s internal, non-public schema definitions (for example
/// `QualificationProfile`, `QualificationCorpusManifest`, and the `Vp04*` records), which are
/// additional `$defs` in the same authoritative schema rather than a sixth public E02 root
/// contract.
///
/// # Errors
/// Returns an unknown-definition, resource, schema-compilation, or validation error.
pub fn validate_schema_definition(name: &str, value: &Value) -> Result<(), VerificationError> {
    validate_resource_envelope(value)?;
    reject_unknown_contract_version(value)?;
    let validator = definition_validator(name)?;
    validate_with(&validator, value, name)
}

fn definition_validator(name: &str) -> Result<Arc<Validator>, VerificationError> {
    let cache = DEFINITION_VALIDATORS.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(validator) = cache
        .lock()
        .map_err(|_| validator_cache_error())?
        .get(name)
        .cloned()
    {
        return Ok(validator);
    }
    let schema = contract_schema()?;
    if schema.pointer(&format!("/$defs/{name}")).is_none() {
        return Err(VerificationError::new(
            VerificationErrorCode::UnknownSchemaDefinition,
            "unknown schema definition",
        )
        .with_context("definition", name));
    }
    let definitions = schema.get("$defs").cloned().ok_or_else(|| {
        VerificationError::new(
            VerificationErrorCode::SchemaViolation,
            "authoritative schema has no definitions object",
        )
    })?;
    let wrapper = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$defs": definitions,
        "$ref": format!("#/$defs/{name}"),
    });
    let validator = build_contract_validator(&wrapper)
        .map_err(|error| VerificationError::new(VerificationErrorCode::SchemaViolation, error))?;
    let validator = Arc::new(validator);
    let mut validators = cache.lock().map_err(|_| validator_cache_error())?;
    Ok(validators
        .entry(name.to_owned())
        .or_insert_with(|| Arc::clone(&validator))
        .clone())
}

fn validator_cache_error() -> VerificationError {
    VerificationError::new(
        VerificationErrorCode::SchemaViolation,
        "schema validator cache is poisoned",
    )
}

fn reject_unknown_contract_version(value: &Value) -> Result<(), VerificationError> {
    if let Some(version) = value.get("contractVersion").and_then(Value::as_str)
        && version != contract_version()
    {
        return Err(VerificationError::new(
            VerificationErrorCode::UnsupportedContractVersion,
            "unsupported contract version",
        )
        .with_context("contract_version", version));
    }
    Ok(())
}

/// Returns the contract version from the authoritative schema.
#[must_use]
pub fn contract_version() -> &'static str {
    wire::CONTRACT_VERSION
}

/// Returns the five input-kind constants from the authoritative schema without allocation.
#[must_use]
pub const fn input_kinds() -> &'static [&'static str] {
    wire::INPUT_KINDS
}

/// Serializes a JSON value according to RFC 8785 JCS.
///
/// # Errors
/// Returns `canonicalization_failed` when serialization is outside the JCS domain.
pub fn jcs_canonicalize<T: Serialize>(value: &T) -> Result<Vec<u8>, VerificationError> {
    let mut output = Vec::new();
    serde_json_canonicalizer::to_writer(value, &mut output).map_err(|error| {
        VerificationError::new(
            VerificationErrorCode::CanonicalizationFailed,
            error.to_string(),
        )
    })?;
    Ok(output)
}

/// Compares two serializable values by RFC 8785 canonical bytes.
///
/// # Errors
/// Returns `canonicalization_failed` when either value is outside the JCS domain.
pub fn jcs_equal<L: Serialize, R: Serialize>(
    left: &L,
    right: &R,
) -> Result<bool, VerificationError> {
    Ok(jcs_canonicalize(left)? == jcs_canonicalize(right)?)
}

/// Computes a domain-separated SHA-256 digest over RFC 8785 canonical bytes.
///
/// # Errors
/// Returns `canonicalization_failed` when the value is outside the JCS domain.
pub fn jcs_sha256<T: Serialize>(value: &T) -> Result<String, VerificationError> {
    let digest = Sha256::digest(jcs_canonicalize(value)?);
    Ok(format!("sha256-jcs:{digest:x}"))
}

/// Validates duplicate ids, dependency existence, and acyclicity iteratively.
///
/// # Errors
/// Returns a stable graph error for duplicates, missing dependencies, or cycles.
pub fn validate_plan_graph(plan: &wire::VerificationPlan) -> Result<(), VerificationError> {
    let mut by_id = BTreeMap::<&str, Vec<&str>>::new();
    for task in &plan.tasks {
        let task_id = task.task_id.as_str();
        let dependencies = task
            .dependencies
            .iter()
            .map(|dependency| dependency.as_str())
            .collect::<Vec<_>>();
        if by_id.insert(task_id, dependencies).is_some() {
            return Err(VerificationError::new(
                VerificationErrorCode::DuplicateTaskId,
                "duplicate task identifier",
            )
            .with_context("task_id", task_id));
        }
    }
    for dependencies in by_id.values() {
        for dependency in dependencies {
            if !by_id.contains_key(dependency) {
                return Err(VerificationError::new(
                    VerificationErrorCode::UnknownTaskDependency,
                    "task dependency names no task",
                )
                .with_context("dependency", *dependency));
            }
        }
    }

    let mut indegree = by_id
        .keys()
        .map(|id| (*id, 0_usize))
        .collect::<BTreeMap<_, _>>();
    let mut dependents = BTreeMap::<&str, Vec<&str>>::new();
    for (task_id, dependencies) in &by_id {
        for dependency in dependencies {
            *indegree.get_mut(task_id).ok_or_else(|| {
                VerificationError::new(
                    VerificationErrorCode::UnknownTaskDependency,
                    "task disappeared during graph validation",
                )
            })? += 1;
            dependents.entry(dependency).or_default().push(task_id);
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect::<Vec<_>>();
    let mut visited = 0_usize;
    while let Some(task_id) = ready.pop() {
        visited += 1;
        for dependent in dependents.get(task_id).into_iter().flatten() {
            let count = indegree.get_mut(dependent).ok_or_else(|| {
                VerificationError::new(
                    VerificationErrorCode::UnknownTaskDependency,
                    "dependent task disappeared during graph validation",
                )
            })?;
            *count -= 1;
            if *count == 0 {
                ready.push(dependent);
            }
        }
    }
    if visited != by_id.len() {
        return Err(VerificationError::new(
            VerificationErrorCode::TaskDependencyCycle,
            "task dependency graph contains a cycle",
        ));
    }
    Ok(())
}

fn validate_with(
    validator: &Validator,
    value: &Value,
    label: &str,
) -> Result<(), VerificationError> {
    if validator.is_valid(value) {
        return Ok(());
    }
    let detail = validator
        .iter_errors(value)
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    Err(VerificationError::new(
        VerificationErrorCode::SchemaViolation,
        format!("{label} failed JSON Schema validation: {detail}"),
    ))
}

fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, VerificationError> {
    serde_json::from_value(value).map_err(|error| {
        VerificationError::new(VerificationErrorCode::SchemaViolation, error.to_string())
    })
}

fn contract_schema() -> Result<&'static Value, VerificationError> {
    match CONTRACT_SCHEMA.get_or_init(|| {
        serde_json::from_str(include_str!("../contracts/e02-draft-2.schema.json"))
            .map_err(|error| error.to_string())
    }) {
        Ok(schema) => Ok(schema),
        Err(detail) => Err(VerificationError::new(
            VerificationErrorCode::SchemaViolation,
            detail.clone(),
        )),
    }
}

fn contract_validator() -> Result<&'static Validator, VerificationError> {
    match CONTRACT_VALIDATOR.get_or_init(|| {
        let schema: Value =
            serde_json::from_str(include_str!("../contracts/e02-draft-2.schema.json"))
                .map_err(|error| error.to_string())?;
        build_contract_validator(&schema)
    }) {
        Ok(validator) => Ok(validator),
        Err(detail) => Err(VerificationError::new(
            VerificationErrorCode::SchemaViolation,
            detail.clone(),
        )),
    }
}

fn build_contract_validator(schema: &Value) -> Result<Validator, String> {
    let shared: Value = serde_json::from_str(include_str!(
        "../contracts/shared-reference-2-draft/schema.json"
    ))
    .map_err(|error| error.to_string())?;
    let registry = Registry::new()
        .add("urn:ix:shared-reference:2-draft", &shared)
        .map_err(|error| error.to_string())?
        .prepare()
        .map_err(|error| error.to_string())?;
    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .with_registry(&registry)
        .build(schema)
        .map_err(|error| error.to_string())
}
