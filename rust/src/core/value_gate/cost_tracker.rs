//! Deterministic execution-cost calculation in USD micros.

const TOKENS_PER_MILLION: u128 = 1_000_000;

/// Cost and token accounting for one model execution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionCost {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub model: String,
    pub provider: String,
    pub estimated_cost_micros: u64,
}

/// Calculate cost using the model's published USD-per-million-token rates.
pub fn calculate_cost(input: u64, output: u64, cache: u64, model: &str) -> u64 {
    let (input_rate, output_rate) = match model {
        "gpt-4o" => (2_500_000u128, 10_000_000u128),
        "claude-sonnet" => (3_000_000u128, 15_000_000u128),
        _ => (5_000_000u128, 15_000_000u128),
    };
    let total = input as u128 * input_rate
        + output as u128 * output_rate
        + cache as u128 * (input_rate / 10);
    u64::try_from(total / TOKENS_PER_MILLION).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_calculation_gpt4o() {
        assert_eq!(
            calculate_cost(1_000_000, 1_000_000, 0, "gpt-4o"),
            12_500_000
        );
    }

    #[test]
    fn cost_calculation_claude() {
        assert_eq!(
            calculate_cost(1_000_000, 1_000_000, 0, "claude-sonnet"),
            18_000_000
        );
    }

    #[test]
    fn cost_calculation_unknown() {
        assert_eq!(calculate_cost(1_000_000, 1_000_000, 0, "other"), 20_000_000);
    }

    #[test]
    fn cache_uses_ten_percent_of_input_rate() {
        assert_eq!(calculate_cost(0, 0, 1_000_000, "gpt-4o"), 250_000);
    }
}
