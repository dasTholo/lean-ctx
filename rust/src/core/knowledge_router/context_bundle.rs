use super::planner::ContextCandidate;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BundleStrategy {
    #[default]
    Minimal,
    Enriched,
    Governed,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextBundle {
    pub bundle_id: String,
    pub task_id: String,
    pub candidates: Vec<String>,
    pub total_tokens: u64,
    pub coverage_milli: u16,
    pub strategy: BundleStrategy,
}
pub fn create_bundle(
    task_id: &str,
    candidates: &[ContextCandidate],
    strategy: BundleStrategy,
) -> ContextBundle {
    let selected = if strategy == BundleStrategy::Minimal {
        &candidates[..candidates.len().min(3)]
    } else {
        candidates
    };
    let total_tokens = selected
        .iter()
        .map(|candidate| candidate.estimated_tokens)
        .sum();
    let coverage_milli = selected
        .iter()
        .map(|candidate| u64::from(candidate.relevance_milli))
        .sum::<u64>()
        .checked_div(selected.len() as u64)
        .unwrap_or(0) as u16;
    ContextBundle {
        bundle_id: format!("bundle-{task_id}-{}", selected.len()),
        task_id: task_id.into(),
        candidates: selected
            .iter()
            .map(|candidate| candidate.candidate_id.clone())
            .collect(),
        total_tokens,
        coverage_milli,
        strategy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn candidates() -> Vec<ContextCandidate> {
        (0..4)
            .map(|id| ContextCandidate {
                candidate_id: format!("c{id}"),
                source_id: "local_files".into(),
                kind: "file".into(),
                relevance_milli: 800 + id,
                estimated_tokens: 100,
                reference: None,
            })
            .collect()
    }
    #[test]
    fn minimal_has_at_most_three_candidates() {
        assert_eq!(
            create_bundle("task", &candidates(), BundleStrategy::Minimal)
                .candidates
                .len(),
            3
        );
    }
    #[test]
    fn coverage_is_average_relevance() {
        assert_eq!(
            create_bundle("task", &candidates()[..2], BundleStrategy::Enriched).coverage_milli,
            800
        );
    }
}
