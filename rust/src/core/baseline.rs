//! Versioned baseline observations for task-level efficiency accounting.

use serde::{Deserialize, Serialize};

/// Normalized token categories shared by baselines and the ETPAO formula.
///
/// The fields intentionally contain only token observations. Cost and latency
/// are optional on [`BaselineObservationV1`] because an absent measurement must
/// remain unknown rather than being represented as zero.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedUsage {
    pub fresh_input: u64,
    pub cached_input: u64,
    pub output: u64,
    pub reasoning: u64,
}

impl NormalizedUsage {
    /// Returns the unweighted sum of all normalized token categories.
    #[must_use]
    pub fn total_tokens(&self) -> u64 {
        self.fresh_input
            .saturating_add(self.cached_input)
            .saturating_add(self.output)
            .saturating_add(self.reasoning)
    }
}

/// Method used to obtain a baseline observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineMethod {
    /// Real execution without optimization, assigned as a holdout.
    ObservedHoldout,
    /// The path and constraints originally requested by the task.
    RequestedPath,
    /// Deterministic replay of a prior execution.
    Replay,
    /// Model-based estimate; less trustworthy than an observation or replay.
    Estimated,
}

impl BaselineMethod {
    /// Stable wire name for this method.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ObservedHoldout => "observed_holdout",
            Self::RequestedPath => "requested_path",
            Self::Replay => "replay",
            Self::Estimated => "estimated",
        }
    }

    /// Whether this method is an estimate rather than an observation/replay.
    #[must_use]
    pub const fn is_estimated(self) -> bool {
        matches!(self, Self::Estimated)
    }
}

/// Versioned, evidence-labelled baseline observation for one task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineObservationV1 {
    pub method: BaselineMethod,
    pub method_version: String,
    pub task_id: String,
    pub baseline_tokens: NormalizedUsage,
    pub baseline_cost_micros: Option<u64>,
    pub baseline_latency_ms: Option<u64>,
    pub evidence_refs: Vec<String>,
    /// True only when the same constraints were applied to both arms.
    pub policy_equivalent: bool,
}

impl BaselineObservationV1 {
    /// Creates an observation while preserving missing cost/latency values.
    #[must_use]
    pub fn new(
        method: BaselineMethod,
        method_version: impl Into<String>,
        task_id: impl Into<String>,
        baseline_tokens: NormalizedUsage,
        baseline_cost_micros: Option<u64>,
        baseline_latency_ms: Option<u64>,
        evidence_refs: Vec<String>,
        policy_equivalent: bool,
    ) -> Self {
        Self {
            method,
            method_version: method_version.into(),
            task_id: task_id.into(),
            baseline_tokens,
            baseline_cost_micros,
            baseline_latency_ms,
            evidence_refs,
            policy_equivalent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BaselineMethod, BaselineObservationV1, NormalizedUsage};

    #[test]
    fn missing_measurements_remain_unknown() {
        let observation = BaselineObservationV1::new(
            BaselineMethod::ObservedHoldout,
            "1.0.0",
            "task-1",
            NormalizedUsage {
                fresh_input: 10,
                ..NormalizedUsage::default()
            },
            None,
            None,
            vec!["evidence:task-1".to_owned()],
            true,
        );

        assert_eq!(observation.baseline_cost_micros, None);
        assert_eq!(observation.baseline_latency_ms, None);
    }

    #[test]
    fn methods_have_stable_wire_names() {
        assert_eq!(BaselineMethod::ObservedHoldout.as_str(), "observed_holdout");
        assert_eq!(BaselineMethod::RequestedPath.as_str(), "requested_path");
        assert_eq!(BaselineMethod::Replay.as_str(), "replay");
        assert!(BaselineMethod::Estimated.is_estimated());
    }
}
