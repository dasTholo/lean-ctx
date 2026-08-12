//! Baseline simulation without LeanCTX optimizations.

use crate::core::{
    shadow::recommendation::ShadowTask,
    value_gate::{OutcomeSignal, TaskOutcome, cost_tracker::calculate_cost},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineMeasurement {
    pub task_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model: String,
    pub total_cost_micros: u64,
    pub duration_ms: u64,
    pub outcome_accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineConfig {
    pub model: String,
    pub no_compression: bool,
    pub no_routing: bool,
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".into(),
            no_compression: true,
            no_routing: true,
        }
    }
}

pub fn simulate_baseline(task: &ShadowTask, config: &BaselineConfig) -> BaselineMeasurement {
    let accepted = accepted(&task.task_id, &task.outcome_signals);
    BaselineMeasurement {
        task_id: task.task_id.clone(),
        input_tokens: task.raw_input_tokens,
        output_tokens: task.output_tokens,
        model: config.model.clone(),
        total_cost_micros: calculate_cost(
            task.raw_input_tokens,
            task.output_tokens,
            0,
            &config.model,
        ),
        duration_ms: task.duration_ms,
        outcome_accepted: accepted,
    }
}

pub(crate) fn accepted(task_id: &str, signals: &[OutcomeSignal]) -> bool {
    crate::core::value_gate::outcome_evaluator::evaluate(&TaskOutcome {
        task_id: task_id.into(),
        completed: true,
        signals: signals.to_vec(),
    })
}
