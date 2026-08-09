//! Capability discovery and conformance contract.

use crate::common::{
    CapabilityId, ValidationError, deserialize_schema_version, validate_schema_version,
};
use crate::experiment::DataClassification;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Capability category exposed by a provider or runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityKind {
    Tool,
    Model,
    Provider,
    ContextSource,
    Validator,
    Scheduler,
    ShellOutputOptimization,
    Other,
}

/// Whether a capability's effects can be undone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    Reversible,
    Irreversible,
    Conditional,
}

/// Determinism guarantee made by a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Determinism {
    Deterministic,
    Seeded,
    NonDeterministic,
}

/// Boundary at which a capability moves data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataMovement {
    None,
    LocalOnly,
    Remote,
    CrossRegion,
}

/// Measurement dimensions a capability can report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementSupportV1 {
    pub latency: bool,
    pub tokens: bool,
    pub quality: bool,
}

/// Per-surface support details in a capability manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceSupportV1 {
    pub supported: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema_ref: Option<String>,
}

/// Provider capability manifest. Unknown top-level fields are retained for additive evolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityManifestV1 {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u32,
    pub capability_id: CapabilityId,
    pub provider: String,
    pub kind: CapabilityKind,
    pub version: String,
    pub surfaces: Vec<String>,
    pub support_matrix: BTreeMap<String, SurfaceSupportV1>,
    pub local: bool,
    pub remote: bool,
    pub reversibility: Reversibility,
    pub determinism: Determinism,
    pub data_movement: DataMovement,
    pub supported_classifications: Vec<DataClassification>,
    pub measurement_support: MeasurementSupportV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema_ref: Option<String>,
    pub conformance_version: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl CapabilityManifestV1 {
    /// Validate location and schema invariants.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_schema_version(self.schema_version)?;
        if !self.local && !self.remote {
            return Err(ValidationError::new(
                "capability must support local or remote execution",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id<T>(value: &str) -> T
    where
        T: TryFrom<String>,
        <T as TryFrom<String>>::Error: std::fmt::Debug,
    {
        T::try_from(value.to_owned()).expect("identifier should be valid")
    }

    #[test]
    fn serialization_round_trip() {
        let manifest = CapabilityManifestV1 {
            schema_version: 1,
            capability_id: id("capability:search"),
            provider: "provider-1".to_owned(),
            kind: CapabilityKind::Tool,
            version: "1.0.0".to_owned(),
            surfaces: vec!["mcp".to_owned(), "cli".to_owned()],
            support_matrix: BTreeMap::from([(
                "mcp".to_owned(),
                SurfaceSupportV1 {
                    supported: true,
                    input_schema_ref: Some("schema:input".to_owned()),
                    output_schema_ref: Some("schema:output".to_owned()),
                },
            )]),
            local: true,
            remote: true,
            reversibility: Reversibility::Reversible,
            determinism: Determinism::Deterministic,
            data_movement: DataMovement::LocalOnly,
            supported_classifications: vec![DataClassification::Public],
            measurement_support: MeasurementSupportV1 {
                latency: true,
                tokens: true,
                quality: false,
            },
            input_schema_ref: Some("schema:input".to_owned()),
            output_schema_ref: Some("schema:output".to_owned()),
            conformance_version: 1,
            extra: BTreeMap::from([("future_field".to_owned(), Value::from(true))]),
        };
        let json = serde_json::to_string(&manifest).expect("manifest should serialize");
        let decoded: CapabilityManifestV1 =
            serde_json::from_str(&json).expect("manifest should deserialize");
        assert_eq!(manifest, decoded);
        manifest
            .validate()
            .expect("manifest should satisfy invariants");
    }
}
