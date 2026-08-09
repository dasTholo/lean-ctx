//! Decision audit contract.

use crate::common::{
    DecisionId, PlanId, TaskId, ValidationError, deserialize_schema_version,
    validate_schema_version,
};
use crate::evidence::EvidenceRefV1;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Kind of policy, routing, or execution decision being recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionKind {
    Routing,
    Scheduling,
    CapabilitySelection,
    ContextSelection,
    Policy,
    Fallback,
    Retry,
    Stop,
    Other,
}

/// Signed, evidence-linked record of one deterministic decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRecordV1 {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u32,
    pub decision_id: DecisionId,
    pub task_id: TaskId,
    pub plan_id: PlanId,
    pub decision_kind: DecisionKind,
    pub input_refs: Vec<String>,
    pub constraint_refs: Vec<String>,
    pub selected_result: Value,
    pub rationale_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_ref: Option<String>,
    pub decision_system_name: String,
    pub decision_system_version: String,
    pub evidence_refs: Vec<EvidenceRefV1>,
    pub observed_at: String,
    pub signature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<DecisionId>,
}

impl DecisionRecordV1 {
    /// Validate schema invariants for a decision record.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_schema_version(self.schema_version)
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
        let decision = DecisionRecordV1 {
            schema_version: 1,
            decision_id: id("decision-1"),
            task_id: id("task-1"),
            plan_id: id("plan-1"),
            decision_kind: DecisionKind::Routing,
            input_refs: vec!["input:task".to_owned()],
            constraint_refs: vec!["constraint:budget".to_owned()],
            selected_result: serde_json::json!({ "model": "model-1" }),
            rationale_code: "lowest_cost_within_slo".to_owned(),
            rationale_ref: Some("rationale:1".to_owned()),
            policy_ref: Some("policy:model".to_owned()),
            decision_system_name: "scheduler".to_owned(),
            decision_system_version: "1.0.0".to_owned(),
            evidence_refs: vec![],
            observed_at: "2026-08-09T12:00:00Z".to_owned(),
            signature: "signature".to_owned(),
            supersedes: None,
        };
        let json = serde_json::to_string(&decision).expect("decision should serialize");
        let decoded: DecisionRecordV1 =
            serde_json::from_str(&json).expect("decision should deserialize");
        assert_eq!(decision, decoded);
        decision
            .validate()
            .expect("decision should satisfy invariants");
    }
}
