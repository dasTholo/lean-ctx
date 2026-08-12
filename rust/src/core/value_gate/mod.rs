//! End-to-end task value assessment: execution cost → outcome → CPAO.

pub mod cost_tracker;
pub mod cpao;
pub mod outcome_evaluator;
pub mod report;
pub mod store;

pub use cost_tracker::ExecutionCost;
pub use outcome_evaluator::{OutcomeSignal, TaskOutcome};
pub use store::ValueGateStore;

/// Stateless orchestrator for the value-gate loop.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValueGate;

pub fn store() -> &'static store::ValueGateStore {
    static STORE: std::sync::OnceLock<store::ValueGateStore> = std::sync::OnceLock::new();
    STORE.get_or_init(store::ValueGateStore::default)
}

/// Assessment produced after one execution and deterministic outcome check.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValueAssessment {
    pub task_id: String,
    pub model: String,
    pub total_tokens: u64,
    pub cost_micros: u64,
    pub outcome_accepted: bool,
    pub cpao_micros: Option<u64>,
    pub evidence: Vec<String>,
    pub timestamp: String,
}

impl ValueGate {
    pub fn evaluate_task(
        task_id: &str,
        execution_cost: &ExecutionCost,
        outcome: &TaskOutcome,
    ) -> ValueAssessment {
        evaluate_task(task_id, execution_cost, outcome)
    }
}

/// Run the complete task → cost → outcome → CPAO path.
pub fn evaluate_task(
    task_id: &str,
    execution_cost: &ExecutionCost,
    outcome: &TaskOutcome,
) -> ValueAssessment {
    let task_id_matches = outcome.task_id == task_id;
    let outcome_accepted = task_id_matches && outcome_evaluator::evaluate(outcome);
    let cost_micros = execution_cost.estimated_cost_micros;
    let cpao_micros = cpao::cost_per_accepted_outcome(&[cost_micros], &[outcome_accepted]);
    let mut evidence = vec![
        format!("task_id_matches={task_id_matches}"),
        format!("execution_cost_micros={cost_micros}"),
        format!("outcome_completed={}", outcome.completed),
        format!("outcome_accepted={outcome_accepted}"),
    ];
    evidence.extend(
        outcome
            .signals
            .iter()
            .map(|signal| format!("signal={signal:?}")),
    );
    let assessment = ValueAssessment {
        task_id: task_id.to_owned(),
        model: execution_cost.model.clone(),
        total_tokens: execution_cost
            .input_tokens
            .saturating_add(execution_cost.output_tokens)
            .saturating_add(execution_cost.cache_read_tokens),
        cost_micros,
        outcome_accepted,
        cpao_micros,
        evidence,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    store().record(&assessment);
    assessment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_gate_e2e() {
        let cost = ExecutionCost {
            input_tokens: 1_000_000,
            output_tokens: 100_000,
            cache_read_tokens: 0,
            model: "gpt-4o".into(),
            provider: "openai".into(),
            estimated_cost_micros: cost_tracker::calculate_cost(1_000_000, 100_000, 0, "gpt-4o"),
        };
        let outcome = TaskOutcome {
            task_id: "task-e2e".into(),
            completed: true,
            signals: vec![OutcomeSignal::BuildSucceeded, OutcomeSignal::UserAccepted],
        };
        let assessment = evaluate_task("task-e2e", &cost, &outcome);
        assert_eq!(assessment.task_id, "task-e2e");
        assert!(assessment.outcome_accepted);
        assert_eq!(assessment.cpao_micros, Some(3_500_000));
        assert!(!assessment.timestamp.is_empty());
        assert!(!assessment.evidence.is_empty());
    }

    #[test]
    fn rejects_outcome_for_another_task() {
        let cost = ExecutionCost {
            input_tokens: 1,
            output_tokens: 1,
            cache_read_tokens: 0,
            model: "gpt-4o".into(),
            provider: "openai".into(),
            estimated_cost_micros: 1,
        };
        let outcome = TaskOutcome {
            task_id: "other-task".into(),
            completed: true,
            signals: vec![OutcomeSignal::TestsPassed],
        };
        let assessment = evaluate_task("expected-task", &cost, &outcome);
        assert!(!assessment.outcome_accepted);
        assert_eq!(assessment.cpao_micros, None);
        assert!(
            assessment
                .evidence
                .contains(&"task_id_matches=false".into())
        );
    }
}
