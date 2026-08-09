//! Accepted-outcome contract.

use crate::common::{
    OutcomeId, TaskId, ValidationError, deserialize_optional_milliunit, deserialize_schema_version,
    validate_milliunit, validate_schema_version,
};
use crate::evidence::EvidenceRefV1;
use serde::{Deserialize, Serialize};

/// Tri-state acceptance prevents an absent observation from being treated as a rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptanceState {
    Accepted,
    Rejected,
    Unknown,
}

/// State of an individual completion or quality signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalState {
    Passed,
    Failed,
    Unknown,
    NotRun,
}

/// Verification signals attached to an accepted-outcome observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeSignalsV1 {
    pub build: Option<SignalState>,
    pub tests: Option<SignalState>,
    pub lint: Option<SignalState>,
    pub typecheck: Option<SignalState>,
    pub completion: Option<SignalState>,
    pub pr: Option<SignalState>,
    pub correction: Option<SignalState>,
    pub rollback: Option<SignalState>,
    pub retry: Option<SignalState>,
}

/// Canonical outcome observation used for acceptance and efficiency accounting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedOutcomeV1 {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u32,
    pub outcome_id: OutcomeId,
    pub task_id: TaskId,
    pub accepted: AcceptanceState,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_milliunit"
    )]
    pub quality_score_milli: Option<u16>,
    pub signals: OutcomeSignalsV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    pub evidence_refs: Vec<EvidenceRefV1>,
    pub observed_at: String,
}

impl AcceptedOutcomeV1 {
    /// Validate invariants that also apply to values constructed in Rust.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_schema_version(self.schema_version)?;
        if let Some(value) = self.quality_score_milli {
            validate_milliunit(value, "quality_score_milli")?;
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
        let outcome = AcceptedOutcomeV1 {
            schema_version: 1,
            outcome_id: id("outcome-1"),
            task_id: id("task-1"),
            accepted: AcceptanceState::Accepted,
            quality_score_milli: Some(950),
            signals: OutcomeSignalsV1 {
                build: Some(SignalState::Passed),
                tests: Some(SignalState::Passed),
                lint: Some(SignalState::Passed),
                typecheck: Some(SignalState::Passed),
                completion: Some(SignalState::Passed),
                pr: Some(SignalState::NotRun),
                correction: Some(SignalState::NotRun),
                rollback: Some(SignalState::NotRun),
                retry: Some(SignalState::Failed),
            },
            contract_ref: Some("contract:outcome".to_owned()),
            evidence_refs: vec![],
            observed_at: "2026-08-09T12:00:00Z".to_owned(),
        };
        let json = serde_json::to_string(&outcome).expect("outcome should serialize");
        let decoded: AcceptedOutcomeV1 =
            serde_json::from_str(&json).expect("outcome should deserialize");
        assert_eq!(outcome, decoded);
        outcome
            .validate()
            .expect("outcome should satisfy invariants");
    }
}
