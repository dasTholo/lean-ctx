//! Task admission and lineage contract.

use crate::common::{
    AgentId, ProjectId, SessionId, TaskId, TenantId, TraceId, ValidationError,
    deserialize_optional_milliunit, deserialize_schema_version, validate_milliunit,
    validate_schema_version,
};
use crate::experiment::DataClassification;
use serde::{Deserialize, Serialize};

/// Coarse task complexity used by policy and scheduling decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexity {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk class used for safety-sensitive routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskClass {
    Low,
    Medium,
    High,
    Critical,
}

/// Canonical task envelope for a V1 execution lineage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskEnvelopeV1 {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u32,
    pub task_id: TaskId,
    pub trace_id: TraceId,
    pub project_id: ProjectId,
    pub session_id: SessionId,
    pub agent_id: AgentId,
    pub complexity: TaskComplexity,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<TaskId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_class: Option<RiskClass>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_milliunit"
    )]
    pub quality_requirement_milli: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_budget_micros: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_budget_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_classification: Option<DataClassification>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_policy_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_policy_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_state_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome_contract_ref: Option<String>,
}

impl TaskEnvelopeV1 {
    /// Schema version represented by this type.
    pub const SCHEMA_VERSION: u32 = 1;

    /// Validate invariants that also apply to values constructed in Rust.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_schema_version(self.schema_version)?;
        if let Some(value) = self.quality_requirement_milli {
            validate_milliunit(value, "quality_requirement_milli")?;
        }
        Ok(())
    }

    /// Validate that this envelope is a child of `parent` in the same trace.
    pub fn validate_child_of(&self, parent: &Self) -> Result<(), ValidationError> {
        self.validate()?;
        parent.validate()?;
        if self.parent_task_id.as_ref() != Some(&parent.task_id) {
            return Err(ValidationError::new(
                "child parent_task_id does not match the parent task_id",
            ));
        }
        if self.trace_id != parent.trace_id {
            return Err(ValidationError::new(
                "child task must retain the parent trace_id",
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
        let task = TaskEnvelopeV1 {
            schema_version: 1,
            task_id: id("task-1"),
            trace_id: id("trace-1"),
            project_id: id("project-1"),
            session_id: id("session-1"),
            agent_id: id("agent-1"),
            complexity: TaskComplexity::Medium,
            created_at: "2026-08-09T12:00:00Z".to_owned(),
            parent_task_id: None,
            tenant_id: Some(id("tenant-1")),
            intent: Some("implement".to_owned()),
            task_class: Some("coding".to_owned()),
            risk_class: Some(RiskClass::Low),
            quality_requirement_milli: Some(900),
            cost_budget_micros: Some(10_000),
            latency_budget_ms: Some(2_000),
            data_classification: Some(DataClassification::Internal),
            region_policy_ref: Some("policy:region".to_owned()),
            model_policy_ref: Some("policy:model".to_owned()),
            context_state_ref: Some("context:state".to_owned()),
            outcome_contract_ref: Some("contract:outcome".to_owned()),
        };
        let json = serde_json::to_string(&task).expect("task should serialize");
        let decoded: TaskEnvelopeV1 = serde_json::from_str(&json).expect("task should deserialize");
        assert_eq!(task, decoded);
        task.validate().expect("task should satisfy invariants");
    }

    #[test]
    fn quality_requirement_is_bounded() {
        let json = r#"{
            "schema_version": 1,
            "task_id": "task-1",
            "trace_id": "trace-1",
            "project_id": "project-1",
            "session_id": "session-1",
            "agent_id": "agent-1",
            "complexity": "low",
            "created_at": "2026-08-09T12:00:00Z",
            "quality_requirement_milli": 1001
        }"#;
        assert!(serde_json::from_str::<TaskEnvelopeV1>(json).is_err());
    }
}
