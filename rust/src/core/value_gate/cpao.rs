//! Aggregate cost-efficiency metrics.

/// Total cost divided by the number of accepted outcomes.
pub fn cost_per_accepted_outcome(costs: &[u64], accepted: &[bool]) -> Option<u64> {
    let accepted_count = accepted.iter().filter(|&&value| value).count() as u128;
    if accepted_count == 0 {
        return None;
    }
    let total_cost: u128 = costs.iter().map(|&cost| cost as u128).sum();
    Some(u64::try_from(total_cost / accepted_count).unwrap_or(u64::MAX))
}

/// Effective tokens per accepted outcome.
pub fn etpao(total_tokens: u64, accepted_count: u64) -> Option<u64> {
    (accepted_count != 0).then(|| total_tokens / accepted_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpao_basic() {
        assert_eq!(
            cost_per_accepted_outcome(&[100, 200, 300], &[true, false, true]),
            Some(300)
        );
    }

    #[test]
    fn cpao_no_accepted_returns_none() {
        assert_eq!(
            cost_per_accepted_outcome(&[100, 200], &[false, false]),
            None
        );
    }

    #[test]
    fn etpao_basic() {
        assert_eq!(etpao(1_000, 4), Some(250));
    }

    #[test]
    fn etpao_zero_accepted_returns_none() {
        assert_eq!(etpao(1_000, 0), None);
    }
}
