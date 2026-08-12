//! Deterministic task-outcome acceptance evaluation.

/// Observable outcome signals from one execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeSignal {
    BuildSucceeded,
    TestsPassed,
    LintClean,
    UserAccepted,
    UserRejected,
    CompileError,
    TestFailed,
}

/// Outcome observation associated with one task.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskOutcome {
    pub task_id: String,
    pub completed: bool,
    pub signals: Vec<OutcomeSignal>,
}

/// Accept only completed outcomes without an explicit negative signal.
pub fn evaluate(outcome: &TaskOutcome) -> bool {
    outcome.completed
        && !outcome.signals.iter().any(|signal| {
            matches!(
                signal,
                OutcomeSignal::UserRejected
                    | OutcomeSignal::CompileError
                    | OutcomeSignal::TestFailed
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_accepted_all_positive() {
        let outcome = TaskOutcome {
            task_id: "task".into(),
            completed: true,
            signals: vec![
                OutcomeSignal::BuildSucceeded,
                OutcomeSignal::TestsPassed,
                OutcomeSignal::LintClean,
                OutcomeSignal::UserAccepted,
            ],
        };
        assert!(evaluate(&outcome));
    }

    #[test]
    fn outcome_rejected_on_failure() {
        let outcome = TaskOutcome {
            task_id: "task".into(),
            completed: true,
            signals: vec![OutcomeSignal::TestsPassed, OutcomeSignal::TestFailed],
        };
        assert!(!evaluate(&outcome));
    }

    #[test]
    fn outcome_rejected_not_completed() {
        let outcome = TaskOutcome {
            task_id: "task".into(),
            completed: false,
            signals: vec![OutcomeSignal::BuildSucceeded],
        };
        assert!(!evaluate(&outcome));
    }
}
