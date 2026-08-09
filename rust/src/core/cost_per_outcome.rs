use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPerAcceptedOutcome {
    pub task_class: String,
    pub total_cost_usd: f64,
    pub accepted_outcomes: u64,
    pub rejected_outcomes: u64,
    pub cost_per_accepted_usd: f64,
    pub etpao: f64,
    pub period_start: String,
    pub period_end: String,
}

impl CostPerAcceptedOutcome {
    #[must_use]
    pub fn calculate(total_cost_usd: f64, accepted: u64, rejected: u64, etpao: f64) -> Self {
        let cost_per_accepted_usd = if accepted == 0 {
            0.0
        } else {
            total_cost_usd / accepted as f64
        };

        Self {
            task_class: String::new(),
            total_cost_usd,
            accepted_outcomes: accepted,
            rejected_outcomes: rejected,
            cost_per_accepted_usd,
            etpao,
            period_start: String::new(),
            period_end: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_cost_per_accepted_outcome() {
        let metric = CostPerAcceptedOutcome::calculate(12.0, 3, 1, 750.0);

        assert_eq!(metric.total_cost_usd, 12.0);
        assert_eq!(metric.accepted_outcomes, 3);
        assert_eq!(metric.rejected_outcomes, 1);
        assert_eq!(metric.cost_per_accepted_usd, 4.0);
        assert_eq!(metric.etpao, 750.0);
    }

    #[test]
    fn zero_accepted_outcomes_has_zero_unit_cost() {
        let metric = CostPerAcceptedOutcome::calculate(12.0, 0, 2, 0.0);

        assert_eq!(metric.cost_per_accepted_usd, 0.0);
        assert!(metric.cost_per_accepted_usd.is_finite());
    }
}
