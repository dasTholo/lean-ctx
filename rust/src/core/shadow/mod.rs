//! Counterfactual cost and quality reporting for completed tasks.

pub mod baseline;
pub mod comparison;
pub mod persistence;
pub mod recommendation;
pub mod runtime;

pub use comparison::ShadowReport;
pub use recommendation::ShadowTask;

use baseline::{BaselineConfig, simulate_baseline};
use comparison::compare;
use recommendation::measure_treatment;

/// Runs the LeanCTX treatment beside a deterministic unoptimized baseline.
#[derive(Debug, Default)]
pub struct ShadowEngine;

impl ShadowEngine {
    pub fn run_comparison(tasks: &[ShadowTask]) -> ShadowReport {
        Self::run_comparison_with_baseline(tasks, BaselineConfig::default())
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn run_comparison_with_baseline(
        tasks: &[ShadowTask],
        config: BaselineConfig,
    ) -> ShadowReport {
        let baselines: Vec<_> = tasks
            .iter()
            .map(|task| simulate_baseline(task, &config))
            .collect();
        let treatments: Vec<_> = tasks.iter().map(measure_treatment).collect();
        compare(&baselines, &treatments)
    }
}

pub fn run_comparison(tasks: &[ShadowTask]) -> ShadowReport {
    ShadowEngine::run_comparison(tasks)
}

#[cfg(test)]
mod runtime_tests;
#[cfg(test)]
mod tests;
