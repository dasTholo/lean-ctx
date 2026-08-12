use super::{BaselineCall, TreatmentCall, baseline_recorder::read_jsonl};
use crate::core::value_gate::ValueAssessment;
use serde::Serialize;
use std::{collections::HashSet, io, path::Path};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct QualityComparison {
    pub baseline_accepted: usize,
    pub baseline_total: usize,
    pub treatment_accepted: usize,
    pub treatment_total: usize,
}

impl QualityComparison {
    pub fn baseline_rate(&self) -> Option<f64> {
        rate(self.baseline_accepted, self.baseline_total)
    }
    pub fn treatment_rate(&self) -> Option<f64> {
        rate(self.treatment_accepted, self.treatment_total)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ComparisonReport {
    pub baseline_calls: usize,
    pub treatment_calls: usize,
    pub baseline_total_tokens: u64,
    pub treatment_total_tokens: u64,
    pub token_savings: u64,
    pub baseline_cost_micros: u64,
    pub treatment_cost_micros: u64,
    pub cost_savings_micros: u64,
    pub quality: QualityComparison,
    pub statistically_meaningful: bool,
    pub significance_hint: String,
}

pub fn compare_paths(
    baseline: &Path,
    treatment: &Path,
    outcomes: &[ValueAssessment],
) -> io::Result<ComparisonReport> {
    compare(
        &read_jsonl::<BaselineCall>(baseline)?,
        &read_jsonl::<TreatmentCall>(treatment)?,
        outcomes,
    )
}

pub fn compare(
    baseline: &[BaselineCall],
    treatment: &[TreatmentCall],
    outcomes: &[ValueAssessment],
) -> io::Result<ComparisonReport> {
    let baseline_total_tokens = baseline.iter().map(total_baseline_tokens).sum();
    let treatment_total_tokens = treatment.iter().map(total_treatment_tokens).sum();
    let baseline_cost_micros = baseline.iter().map(|entry| entry.raw_cost).sum();
    let treatment_cost_micros = treatment.iter().map(|entry| entry.raw_cost).sum();
    let quality = quality_comparison(baseline, treatment, outcomes);
    let paired_samples = baseline.len().min(treatment.len());
    let statistically_meaningful = paired_samples > 30;
    Ok(ComparisonReport {
        baseline_calls: baseline.len(),
        treatment_calls: treatment.len(),
        baseline_total_tokens,
        treatment_total_tokens,
        token_savings: baseline_total_tokens.saturating_sub(treatment_total_tokens),
        baseline_cost_micros,
        treatment_cost_micros,
        cost_savings_micros: baseline_cost_micros.saturating_sub(treatment_cost_micros),
        quality,
        statistically_meaningful,
        significance_hint: if statistically_meaningful {
            format!(
                "{paired_samples} paired calls recorded; sample size is suitable for a directional comparison."
            )
        } else {
            format!(
                "{paired_samples} paired calls recorded; collect more than 30 paired calls for a meaningful comparison."
            )
        },
    })
}

fn total_baseline_tokens(entry: &BaselineCall) -> u64 {
    entry.input_tokens.saturating_add(entry.output_tokens)
}
fn total_treatment_tokens(entry: &TreatmentCall) -> u64 {
    entry.input_tokens.saturating_add(entry.output_tokens)
}

fn quality_comparison(
    baseline: &[BaselineCall],
    treatment: &[TreatmentCall],
    outcomes: &[ValueAssessment],
) -> QualityComparison {
    let baseline_sessions: HashSet<_> = baseline
        .iter()
        .map(|entry| entry.session_id.as_str())
        .collect();
    let treatment_sessions: HashSet<_> = treatment
        .iter()
        .map(|entry| entry.session_id.as_str())
        .collect();
    let mut quality = QualityComparison::default();
    for outcome in outcomes {
        if baseline_sessions.contains(outcome.task_id.as_str()) {
            quality.baseline_total += 1;
            quality.baseline_accepted += usize::from(outcome.outcome_accepted);
        }
        if treatment_sessions.contains(outcome.task_id.as_str()) {
            quality.treatment_total += 1;
            quality.treatment_accepted += usize::from(outcome.outcome_accepted);
        }
    }
    quality
}

fn rate(accepted: usize, total: usize) -> Option<f64> {
    (total > 0).then(|| accepted as f64 / total as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_produces_meaningful_delta_when_treatment_is_smaller() {
        let baseline = vec![BaselineCall::new("s1", "ctx_read", 0, 1_000, "gpt-4o")];
        let treatment = vec![TreatmentCall::new(
            "s1", "ctx_read", 0, 1_000, 700, "gpt-4o",
        )];
        let report = compare(&baseline, &treatment, &[]).unwrap();
        assert_eq!(report.token_savings, 700);
        assert!(report.cost_savings_micros > 0);
        assert!(!report.statistically_meaningful);
    }
}
