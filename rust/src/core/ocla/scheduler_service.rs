//! Public scheduler boundary shared with the private Class D scheduler.
//!
//! The enterprise implementation may implement [`SchedulerService`], but
//! this OSS contract intentionally contains no adaptive routing, learned
//! weights, customer data, or private economic information.

use lean_ctx_protocol::{CapabilityManifestV1, ExecutionPlanV1, TaskEnvelopeV1};
use serde::{Deserialize, Serialize};

use super::catalogue::TechnicalCatalogue;
use super::policy_constraints::PolicyConstraints;
use super::types::OclaResult;

/// Public interface that a private Scheduler implements.
///
/// The implementation boundary is Class D. The methods only exchange public
/// manifests, technical catalogue data, policy constraints, and plans.
pub trait SchedulerService: Send + Sync {
    /// Generate deterministic or enterprise-owned candidates for a task.
    fn generate_candidates(
        &self,
        envelope: &TaskEnvelopeV1,
        eligible: &[CapabilityManifestV1],
        catalogue: &TechnicalCatalogue,
    ) -> OclaResult<Vec<ExecutionCandidate>>;

    /// Apply hard policy, budget, and security filters before ranking.
    fn filter_candidates(
        &self,
        candidates: Vec<ExecutionCandidate>,
        policy: &PolicyConstraints,
    ) -> Vec<ExecutionCandidate>;

    /// Select a plan from already-filtered candidates.
    fn select_plan(
        &self,
        filtered: &[ExecutionCandidate],
        fallback: &ExecutionCandidate,
    ) -> SchedulerDecision;
}

/// A public candidate considered by a scheduler.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionCandidate {
    pub plan: ExecutionPlanV1,
    pub capability_id: String,
    pub model: String,
    pub provider: String,
    pub expected_cost_micros: Option<u64>,
    pub expected_quality_milli: Option<u32>,
    pub expected_latency_ms: Option<u64>,
    pub exclusion_reason: Option<String>,
}

impl ExecutionCandidate {
    /// Create a candidate while preserving the candidate's public estimates.
    #[must_use]
    pub fn new(
        plan: ExecutionPlanV1,
        capability_id: impl Into<String>,
        model: impl Into<String>,
        provider: impl Into<String>,
        expected_cost_micros: Option<u64>,
        expected_quality_milli: Option<u32>,
        expected_latency_ms: Option<u64>,
    ) -> Self {
        Self {
            plan,
            capability_id: capability_id.into(),
            model: model.into(),
            provider: provider.into(),
            expected_cost_micros,
            expected_quality_milli,
            expected_latency_ms,
            exclusion_reason: None,
        }
    }

    /// Stable public identity used when hashing a candidate set.
    #[must_use]
    pub fn identity(&self) -> String {
        format!("{}:{}:{}", self.capability_id, self.model, self.provider)
    }
}

/// Auditable scheduler output. It recommends a plan; it never executes one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerDecision {
    pub selected: ExecutionPlanV1,
    pub fallback: ExecutionPlanV1,
    pub decision_ref: String,
    pub rationale_code: String,
    /// Confidence is a bounded reference signal, not a learned score.
    pub confidence_milli: u32,
    pub candidates_evaluated: u32,
    pub candidates_excluded: u32,
}
