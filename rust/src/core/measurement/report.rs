use super::comparison::ComparisonReport;

pub fn markdown(report: &ComparisonReport) -> String {
    let baseline_quality = display_rate(report.quality.baseline_rate());
    let treatment_quality = display_rate(report.quality.treatment_rate());
    format!(
        "# lean-ctx A/B Measurement Report\n\n| Metric | Baseline | Treatment | Savings |\n| --- | ---: | ---: | ---: |\n| Tool calls | {} | {} | — |\n| Total tokens | {} | {} | {} |\n| Cost (USD) | {} | {} | {} |\n| Accepted outcomes | {} | {} | — |\n\nQuality rate: baseline {}, treatment {}.\n\nStatistical significance: {}\n",
        report.baseline_calls,
        report.treatment_calls,
        report.baseline_total_tokens,
        report.treatment_total_tokens,
        report.token_savings,
        usd(report.baseline_cost_micros),
        usd(report.treatment_cost_micros),
        usd(report.cost_savings_micros),
        report.quality.baseline_accepted,
        report.quality.treatment_accepted,
        baseline_quality,
        treatment_quality,
        report.significance_hint,
    )
}

pub fn json(report: &ComparisonReport) -> String {
    serde_json::to_string_pretty(report).expect("comparison report serializes")
}

fn usd(micros: u64) -> String {
    format!("${:.6}", micros as f64 / 1_000_000.0)
}
fn display_rate(rate: Option<f64>) -> String {
    rate.map_or_else(
        || "not recorded".to_string(),
        |value| format!("{:.1}%", value * 100.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::measurement::{QualityComparison, comparison::ComparisonReport};

    #[test]
    fn report_format_is_stable_and_deterministic() {
        let report = ComparisonReport { baseline_calls: 2, treatment_calls: 2, baseline_total_tokens: 100, treatment_total_tokens: 40, token_savings: 60, baseline_cost_micros: 100, treatment_cost_micros: 40, cost_savings_micros: 60, quality: QualityComparison::default(), statistically_meaningful: false, significance_hint: "2 paired calls recorded; collect more than 30 paired calls for a meaningful comparison.".into() };
        assert_eq!(markdown(&report), markdown(&report));
        assert_eq!(json(&report), json(&report));
    }
}
