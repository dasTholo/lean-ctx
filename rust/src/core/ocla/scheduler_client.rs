//! Public Scheduler client boundary and deterministic OSS reference.

use lean_ctx_protocol::{CapabilityManifestV1, ExecutionPlanV1, TaskEnvelopeV1};

use super::{OclaError, OclaResult};

/// Public client boundary. Production selection intelligence remains private.
pub trait SchedulerClient: Send + Sync {
    fn select_plan(
        &self,
        envelope: &TaskEnvelopeV1,
        eligible_capabilities: &[CapabilityManifestV1],
    ) -> OclaResult<ExecutionPlanV1>;
}

/// Deterministic/manual reference selection for OSS and conformance tests.
pub struct DeterministicScheduler {
    default_capability: String,
}

impl DeterministicScheduler {
    #[must_use]
    pub fn new(default_capability: impl Into<String>) -> Self {
        Self {
            default_capability: default_capability.into(),
        }
    }

    #[must_use]
    pub fn default_capability(&self) -> &str {
        &self.default_capability
    }
}

impl Default for DeterministicScheduler {
    fn default() -> Self {
        Self::new("capability://leanctx/passthrough")
    }
}

impl SchedulerClient for DeterministicScheduler {
    fn select_plan(
        &self,
        envelope: &TaskEnvelopeV1,
        eligible_capabilities: &[CapabilityManifestV1],
    ) -> OclaResult<ExecutionPlanV1> {
        envelope.validate().map_err(|error| {
            OclaError::InvalidRequest(format!("invalid task envelope: {error}"))
        })?;
        let selected = eligible_capabilities
            .first()
            .ok_or_else(|| OclaError::InvalidRequest("no eligible capabilities supplied".into()))?;
        selected.validate().map_err(|error| {
            OclaError::InvalidRequest(format!("invalid capability manifest: {error}"))
        })?;

        let capability_id = selected.capability_id.clone();
        let plan_seed = format!(
            "{}:{}:{}",
            envelope.task_id.as_str(),
            capability_id.as_str(),
            selected.version
        );
        let plan_id = lean_ctx_protocol::PlanId::try_from(format!(
            "plan:{}",
            blake3::hash(plan_seed.as_bytes()).to_hex()
        ))
        .map_err(|error| {
            OclaError::InvalidRequest(format!("invalid deterministic plan id: {error}"))
        })?;

        let plan = ExecutionPlanV1 {
            schema_version: 1,
            plan_id,
            task_id: envelope.task_id.clone(),
            context_budget_tokens: 0,
            context_strategy: lean_ctx_protocol::ContextStrategy::Balanced,
            knowledge_refs: Vec::new(),
            capability_ids: vec![capability_id],
            model: "manual".into(),
            provider: "leanctx".into(),
            reasoning_allocation_milli: 0,
            max_retries: 0,
            fallback_refs: Vec::new(),
            stop_condition: lean_ctx_protocol::StopCondition::OnCompletion,
            expected_cost_micros: 0,
            expected_quality_milli: envelope.quality_requirement_milli.unwrap_or(0),
            expected_latency_ms: envelope.latency_budget_ms.unwrap_or(0),
            policy_decision_ref: None,
            scheduler_decision_ref: Some("scheduler:deterministic-v1".into()),
        };
        plan.validate().map_err(|error| {
            OclaError::InvalidRequest(format!("invalid execution plan: {error}"))
        })?;
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ocla::SchedulerClient;
    use lean_ctx_protocol::{
        CapabilityId, CapabilityKind, DataClassification, DataMovement, Determinism,
        MeasurementSupportV1, Reversibility, SurfaceSupportV1, TaskComplexity,
    };
    use std::collections::BTreeMap;

    fn manifest(id: &str) -> CapabilityManifestV1 {
        CapabilityManifestV1 {
            schema_version: 1,
            capability_id: CapabilityId::try_from(id).expect("capability id"),
            provider: "leanctx".into(),
            kind: CapabilityKind::Tool,
            version: "1.0.0".into(),
            surfaces: vec!["context".into()],
            support_matrix: BTreeMap::from([(
                "context".into(),
                SurfaceSupportV1 {
                    supported: true,
                    input_schema_ref: None,
                    output_schema_ref: None,
                },
            )]),
            local: true,
            remote: false,
            reversibility: Reversibility::Reversible,
            determinism: Determinism::Deterministic,
            data_movement: DataMovement::LocalOnly,
            supported_classifications: vec![DataClassification::Public],
            measurement_support: MeasurementSupportV1 {
                latency: true,
                tokens: true,
                quality: true,
            },
            input_schema_ref: None,
            output_schema_ref: None,
            conformance_version: 1,
            extra: BTreeMap::new(),
        }
    }

    fn task() -> TaskEnvelopeV1 {
        TaskEnvelopeV1 {
            schema_version: 1,
            task_id: "task-1".try_into().expect("task id"),
            trace_id: "trace-1".try_into().expect("trace id"),
            project_id: "project-1".try_into().expect("project id"),
            session_id: "session-1".try_into().expect("session id"),
            agent_id: "agent-1".try_into().expect("agent id"),
            complexity: TaskComplexity::Low,
            created_at: "2026-08-09T00:00:00Z".into(),
            parent_task_id: None,
            tenant_id: None,
            intent: None,
            task_class: None,
            risk_class: None,
            quality_requirement_milli: Some(800),
            cost_budget_micros: None,
            latency_budget_ms: Some(500),
            data_classification: Some(DataClassification::Public),
            region_policy_ref: None,
            model_policy_ref: None,
            context_state_ref: None,
            outcome_contract_ref: None,
        }
    }

    #[test]
    fn selects_first_eligible_capability_without_ranking() {
        let scheduler = DeterministicScheduler::new("ignored-default");
        let first = manifest("capability://first");
        let second = manifest("capability://second");
        let plan = scheduler
            .select_plan(&task(), &[first.clone(), second])
            .expect("deterministic plan");
        assert_eq!(plan.capability_ids, vec![first.capability_id]);
        assert_eq!(plan.task_id.as_str(), "task-1");
        assert_eq!(
            plan.scheduler_decision_ref.as_deref(),
            Some("scheduler:deterministic-v1")
        );
    }

    #[test]
    fn rejects_empty_eligibility() {
        assert!(
            DeterministicScheduler::default()
                .select_plan(&task(), &[])
                .is_err()
        );
    }
}
