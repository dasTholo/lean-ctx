//! `lean-ctx benchmark --real` — canonical production-path measurements.

use serde::Serialize;

use crate::core::task_benchmark::{
    config::{BenchConfig, ProfileMode},
    fixtures::canonical_suite,
    runner::run_benchmark,
};

#[derive(Debug, Clone, Serialize)]
struct RealBenchmarkReport {
    suite: &'static str,
    task_count: usize,
    raw_tokens: usize,
    compressed_tokens: usize,
    compression_ratio: f64,
    quality_score: f64,
    tasks: Vec<RealTaskMeasurement>,
}

#[derive(Debug, Clone, Serialize)]
struct RealTaskMeasurement {
    task_id: String,
    raw_tokens: usize,
    compressed_tokens: usize,
    compression_ratio: f64,
    quality_score: f64,
}

/// Runs the canonical ten-task suite through the production standard compressor.
pub(crate) fn cmd_benchmark_real(args: &[String]) {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        usage();
        return;
    }

    match render(args) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("benchmark: {error}");
            usage();
            std::process::exit(2);
        }
    }
}

fn render(args: &[String]) -> Result<String, String> {
    let json = parse(args)?;
    let report = measure();
    if json {
        serde_json::to_string_pretty(&report).map_err(|error| format!("serialize report: {error}"))
    } else {
        Ok(table(&report))
    }
}

fn parse(args: &[String]) -> Result<bool, String> {
    let mut real = false;
    let mut json = false;
    for arg in args {
        match arg.as_str() {
            "--real" => real = true,
            "--json" => json = true,
            unknown => return Err(format!("unknown argument {unknown:?}")),
        }
    }
    real.then_some(json)
        .ok_or_else(|| "--real is required for this benchmark mode".to_string())
}

fn measure() -> RealBenchmarkReport {
    let result = run_benchmark(
        &canonical_suite(),
        &BenchConfig::single_profile(ProfileMode::Standard),
    );
    let profile = &result.profiles[0];
    let tasks: Vec<_> = profile
        .runs
        .iter()
        .map(|run| RealTaskMeasurement {
            task_id: run.task_id.clone(),
            raw_tokens: run.raw_tokens,
            compressed_tokens: run.compressed_tokens,
            compression_ratio: ratio(run.compressed_tokens, run.raw_tokens),
            quality_score: run.quality.overall_score(),
        })
        .collect();

    RealBenchmarkReport {
        suite: "canonical-10-task",
        task_count: tasks.len(),
        raw_tokens: profile.total_raw_tokens,
        compressed_tokens: profile.total_compressed_tokens,
        compression_ratio: ratio(profile.total_compressed_tokens, profile.total_raw_tokens),
        quality_score: profile.avg_quality_score,
        tasks,
    }
}

fn table(report: &RealBenchmarkReport) -> String {
    let mut output = format!(
        "Canonical 10-task benchmark (production standard compression)\n\n{:<26} {:>10} {:>12} {:>9} {:>9}\n{}\n",
        "Task",
        "Raw",
        "Compressed",
        "Ratio",
        "Quality",
        "-".repeat(72)
    );
    for task in &report.tasks {
        output.push_str(&format!(
            "{:<26} {:>10} {:>12} {:>8.2}x {:>8.1}%\n",
            task.task_id,
            task.raw_tokens,
            task.compressed_tokens,
            task.compression_ratio,
            task.quality_score * 100.0,
        ));
    }
    output.push_str(&format!(
        "\nTotal ({} tasks): raw {} → compressed {} ({:.2}x, quality {:.1}%)\n",
        report.task_count,
        report.raw_tokens,
        report.compressed_tokens,
        report.compression_ratio,
        report.quality_score * 100.0,
    ));
    output
}

fn ratio(compressed_tokens: usize, raw_tokens: usize) -> f64 {
    if compressed_tokens == 0 {
        return 0.0;
    }
    raw_tokens as f64 / compressed_tokens as f64
}

fn usage() {
    println!(
        "Measure the canonical ten-task suite on the production compressor.\n\nUsage: lean-ctx benchmark --real [--json]"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn benchmark_cmd_produces_valid_json_structure() {
        let output = render(&args(&["--real", "--json"])).unwrap();
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(report["suite"], "canonical-10-task");
        assert_eq!(report["task_count"], 10);
        assert_eq!(report["tasks"].as_array().unwrap().len(), 10);
        for task in report["tasks"].as_array().unwrap() {
            assert!(task.get("raw_tokens").is_some());
            assert!(task.get("compressed_tokens").is_some());
            assert!(task.get("compression_ratio").is_some());
            assert!(task.get("quality_score").is_some());
        }
    }

    #[test]
    fn benchmark_cmd_table_includes_measurement_columns() {
        let output = render(&args(&["--real"])).unwrap();
        assert!(output.contains("Raw"));
        assert!(output.contains("Compressed"));
        assert!(output.contains("Ratio"));
        assert!(output.contains("Quality"));
    }
}
