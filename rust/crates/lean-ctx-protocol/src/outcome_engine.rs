//! Outcome-learning contracts for evaluating control-plane decisions.

use crate::{AcceptanceState, ControlPlaneDecision, OutcomeSignalsV1, TaskId, TaskProfileV1};
use serde::{Deserialize, Serialize};

/// Quality dimensions reported alongside an observed task outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityDimensionsV2 {
    pub correctness_milli: u16,
    pub completeness_milli: u16,
    pub maintainability_milli: u16,
    pub efficiency_milli: u16,
}

/// Rich task outcome signal used by the learning engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeSignalV2 {
    pub task_id: TaskId,
    pub accepted: AcceptanceState,
    pub quality_score_milli: u16,
    pub quality_dimensions: QualityDimensionsV2,
    pub signals: OutcomeSignalsV1,
    pub observed_at: String,
}

/// One decision/outcome pair available for adaptive learning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearningEvent {
    pub task_id: TaskId,
    pub decision_taken: ControlPlaneDecision,
    pub outcome_observed: OutcomeSignalV2,
    pub counterfactual_estimate: Option<f64>,
}

/// Extension point for Enterprise outcome engines such as `AdaptiveOutcomeEngine`.
pub trait OutcomeEngineContract {
    /// Record an observed outcome for a prior decision.
    fn observe(&mut self, event: LearningEvent);

    /// Estimate an outcome score for a task profile in the inclusive 0.0..=1.0 range.
    fn predict_outcome(&self, task_profile: &TaskProfileV1) -> f64;
}

/// OSS outcome engine that retains observations without producing predictions.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LocalOutcomeEngine {
    events: Vec<LearningEvent>,
}

impl LocalOutcomeEngine {
    /// Return the observations retained by this local process.
    pub fn events(&self) -> &[LearningEvent] {
        &self.events
    }
}

impl OutcomeEngineContract for LocalOutcomeEngine {
    fn observe(&mut self, event: LearningEvent) {
        self.events.push(event);
    }

    fn predict_outcome(&self, _task_profile: &TaskProfileV1) -> f64 {
        0.0
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

    fn event() -> LearningEvent {
        LearningEvent {
            task_id: id("task-1"),
            decision_taken: ControlPlaneDecision {
                selected_model: "local/default".to_owned(),
                selected_provider: "local".to_owned(),
                reasoning_budget: 1,
                context_adjustments: Vec::new(),
                confidence: 900,
            },
            outcome_observed: OutcomeSignalV2 {
                task_id: id("task-1"),
                accepted: AcceptanceState::Accepted,
                quality_score_milli: 900,
                quality_dimensions: QualityDimensionsV2 {
                    correctness_milli: 900,
                    completeness_milli: 900,
                    maintainability_milli: 900,
                    efficiency_milli: 900,
                },
                signals: OutcomeSignalsV1 {
                    build: None,
                    tests: None,
                    lint: None,
                    typecheck: None,
                    completion: None,
                    pr: None,
                    correction: None,
                    rollback: None,
                    retry: None,
                },
                observed_at: "2026-08-12T00:00:00Z".to_owned(),
            },
            counterfactual_estimate: None,
        }
    }

    #[test]
    fn local_outcome_engine_stores_events_without_prediction() {
        let mut engine = LocalOutcomeEngine::default();
        engine.observe(event());
        assert_eq!(engine.events().len(), 1);
        assert_eq!(engine.predict_outcome(&TaskProfileV1::default()), 0.0);
    }

    #[test]
    fn contract_is_object_safe() {
        let mut engine: Box<dyn OutcomeEngineContract> = Box::new(LocalOutcomeEngine::default());
        engine.observe(event());
        assert_eq!(engine.predict_outcome(&TaskProfileV1::default()), 0.0);
    }
}
