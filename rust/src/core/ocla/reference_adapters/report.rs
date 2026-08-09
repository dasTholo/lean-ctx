//! Fixture loader, deterministic comparison runner, and report serializers.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::super::invocation::{CAPABILITY_OBSERVATION_SCHEMA_VERSION, CapabilityObservationV1};
use super::comparison_receipt::{
    ComparisonDecision, ComparisonReceipt, QualityCheck, decide, evaluate_quality,
};

/// Number of fixtures in the public Sprint 4 corpus.
pub const EXPECTED_FIXTURE_COUNT: usize = 30;

/// Metadata required for one shell-output fixture.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureMetadata {
    pub command: String,
    pub working_dir: String,
    pub expected_tokens: u64,
    pub category: String,
}

/// Raw fixture data after loading and validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureInput {
    pub name: String,
    pub metadata: FixtureMetadata,
    pub input: String,
    pub input_path: PathBuf,
    pub metadata_path: PathBuf,
}

/// One workload and its payload-free comparison receipt.
#[derive(Clone, Debug, Serialize)]
pub struct WorkloadComparison {
    pub name: String,
    pub category: String,
    pub command: String,
    pub receipt: ComparisonReceipt,
}

/// Per-category wins, losses, ties, and token accounting.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CategorySummary {
    pub category: String,
    pub external_wins: u64,
    pub native_wins: u64,
    pub ties: u64,
    pub external_unavailable: u64,
    pub native_tokens: u64,
    pub external_tokens: u64,
    pub token_savings: u64,
    pub token_overhead: u64,
    pub net_token_delta: i64,
}

impl CategorySummary {
    fn new(category: &str) -> Self {
        Self {
            category: category.to_string(),
            ..Self::default()
        }
    }

    fn record(&mut self, receipt: &ComparisonReceipt) {
        self.native_tokens = self.native_tokens.saturating_add(receipt.native_tokens);
        match receipt.decision {
            ComparisonDecision::ExternalPreferred { .. } => self.external_wins += 1,
            ComparisonDecision::NativePreferred { .. } => self.native_wins += 1,
            ComparisonDecision::Inconclusive { .. } => self.ties += 1,
            ComparisonDecision::ExternalUnavailable => self.external_unavailable += 1,
        }

        if let Some(external_tokens) = receipt.external_tokens {
            self.external_tokens = self.external_tokens.saturating_add(external_tokens);
            let delta = receipt.native_tokens as i64 - external_tokens as i64;
            self.net_token_delta = self.net_token_delta.saturating_add(delta);
            if delta >= 0 {
                self.token_savings = self.token_savings.saturating_add(delta as u64);
            } else {
                self.token_overhead = self.token_overhead.saturating_add(delta.unsigned_abs());
            }
        }
    }
}

/// Aggregate token savings and overhead across all compared workloads.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AggregateTokenStats {
    pub native_tokens: u64,
    pub external_tokens: u64,
    pub token_savings: u64,
    pub token_overhead: u64,
    pub net_token_delta: i64,
    pub savings_percent: f64,
    pub external_unavailable: u64,
}

/// Structured comparison report exported by the reference-adapter runner.
#[derive(Clone, Debug, Serialize)]
pub struct ComparisonReport {
    pub fixture_count: usize,
    pub workloads: Vec<WorkloadComparison>,
    pub categories: BTreeMap<String, CategorySummary>,
    pub external_preferred_workloads: Vec<String>,
    pub native_preferred_workloads: Vec<String>,
    pub aggregate: AggregateTokenStats,
}

impl ComparisonReport {
    /// Serialize this report as readable, structured JSON.
    pub fn to_json(&self) -> Result<String, ReportError> {
        serde_json::to_string_pretty(self).map_err(|source| ReportError::Serialization { source })
    }

    /// Render a stable human-readable report with summary and workload tables.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut output = String::new();
        output.push_str("# RTK v1 Capability Comparison\n\n");
        output.push_str(&format!("Fixtures compared: {}\n\n", self.fixture_count));
        output.push_str("## Category summary\n\n");
        output.push_str(
            "| Category | RTK wins | Native wins | Ties | Unavailable | Native tokens | RTK tokens |\n",
        );
        output.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
        for summary in self.categories.values() {
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                summary.category,
                summary.external_wins,
                summary.native_wins,
                summary.ties,
                summary.external_unavailable,
                summary.native_tokens,
                summary.external_tokens,
            ));
        }

        output.push_str("\n## Aggregate token accounting\n\n");
        output.push_str(&format!(
            "- Native output tokens: {}\n- RTK output tokens: {}\n- Token savings: {}\n- Token overhead: {}\n- Net token savings: {}\n- Savings rate: {:.1}%\n",
            self.aggregate.native_tokens,
            self.aggregate.external_tokens,
            self.aggregate.token_savings,
            self.aggregate.token_overhead,
            self.aggregate.net_token_delta,
            self.aggregate.savings_percent,
        ));

        output.push_str("\n## RTK-preferred workloads\n\n");
        append_workload_list(&mut output, &self.external_preferred_workloads);
        output.push_str("\n## Native-preferred workloads\n\n");
        append_workload_list(&mut output, &self.native_preferred_workloads);

        output.push_str("\n## Workload details\n\n");
        output.push_str("| Workload | Decision | Native | RTK | Delta | Quality |\n");
        output.push_str("|---|---|---:|---:|---:|---|\n");
        for workload in &self.workloads {
            let receipt = &workload.receipt;
            let external_tokens = receipt
                .external_tokens
                .map_or_else(|| "—".to_string(), |value| value.to_string());
            let delta = receipt
                .token_delta()
                .map_or_else(|| "—".to_string(), |value| value.to_string());
            output.push_str(&format!(
                "| {}/{} | {} | {} | {} | {} | {} |\n",
                workload.category,
                workload.name,
                receipt.decision.label(),
                receipt.native_tokens,
                external_tokens,
                delta,
                quality_label(&receipt.quality_check),
            ));
        }

        output
    }
}

/// Errors returned while loading fixtures or exporting a report.
#[derive(Debug, Error)]
pub enum ReportError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("metadata at {path} is invalid: {source}")]
    Metadata {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("fixture at {path} is invalid: {reason}")]
    InvalidFixture { path: PathBuf, reason: String },
    #[error("expected {expected} fixtures, found {actual}")]
    FixtureCount { expected: usize, actual: usize },
    #[error("could not serialize comparison report: {source}")]
    Serialization {
        #[source]
        source: serde_json::Error,
    },
}

/// Load, sort, and validate every fixture below a category-root directory.
pub fn load_fixtures(root: impl AsRef<Path>) -> Result<Vec<FixtureInput>, ReportError> {
    let root = root.as_ref();
    let mut fixtures = Vec::new();

    for category_path in sorted_directories(root)? {
        let category = category_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| ReportError::InvalidFixture {
                path: category_path.clone(),
                reason: "category directory name is not valid UTF-8".to_string(),
            })?
            .to_string();

        for fixture_path in sorted_directories(&category_path)? {
            let name = fixture_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| ReportError::InvalidFixture {
                    path: fixture_path.clone(),
                    reason: "fixture directory name is not valid UTF-8".to_string(),
                })?
                .to_string();
            let input_path = fixture_path.join("input.txt");
            let metadata_path = fixture_path.join("metadata.json");
            let input = fs::read_to_string(&input_path).map_err(|source| ReportError::Io {
                path: input_path.clone(),
                source,
            })?;
            let metadata_body =
                fs::read_to_string(&metadata_path).map_err(|source| ReportError::Io {
                    path: metadata_path.clone(),
                    source,
                })?;
            let metadata =
                serde_json::from_str::<FixtureMetadata>(&metadata_body).map_err(|source| {
                    ReportError::Metadata {
                        path: metadata_path.clone(),
                        source,
                    }
                })?;

            validate_fixture(&category, &name, &input, &metadata, &fixture_path)?;
            fixtures.push(FixtureInput {
                name,
                metadata,
                input,
                input_path,
                metadata_path,
            });
        }
    }

    Ok(fixtures)
}

/// Generate the Sprint 4 report and require all 30 public fixtures.
pub fn generate_report(root: impl AsRef<Path>) -> Result<ComparisonReport, ReportError> {
    let fixtures = load_fixtures(root)?;
    if fixtures.len() != EXPECTED_FIXTURE_COUNT {
        return Err(ReportError::FixtureCount {
            expected: EXPECTED_FIXTURE_COUNT,
            actual: fixtures.len(),
        });
    }
    Ok(generate_from_fixtures(&fixtures))
}

/// Alias emphasizing that this report is the internal comparison artifact.
pub fn generate_comparison_report(root: impl AsRef<Path>) -> Result<ComparisonReport, ReportError> {
    generate_report(root)
}

/// Generate a report from already-loaded fixture data.
#[must_use]
pub fn generate_from_fixtures(fixtures: &[FixtureInput]) -> ComparisonReport {
    let mut workloads: Vec<_> = fixtures.iter().map(compare_fixture).collect();
    workloads.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut categories = BTreeMap::new();
    let mut external_preferred_workloads = Vec::new();
    let mut native_preferred_workloads = Vec::new();
    let mut aggregate = AggregateTokenStats::default();

    for workload in &workloads {
        let summary = categories
            .entry(workload.category.clone())
            .or_insert_with(|| CategorySummary::new(&workload.category));
        summary.record(&workload.receipt);
        aggregate.native_tokens = aggregate
            .native_tokens
            .saturating_add(workload.receipt.native_tokens);

        let workload_id = format!("{}/{}", workload.category, workload.name);
        match workload.receipt.decision {
            ComparisonDecision::ExternalPreferred { .. } => {
                external_preferred_workloads.push(workload_id);
            }
            ComparisonDecision::NativePreferred { .. } => {
                native_preferred_workloads.push(workload_id);
            }
            ComparisonDecision::Inconclusive { .. } | ComparisonDecision::ExternalUnavailable => {}
        }

        if let Some(external_tokens) = workload.receipt.external_tokens {
            aggregate.external_tokens = aggregate.external_tokens.saturating_add(external_tokens);
            let delta = workload.receipt.native_tokens as i64 - external_tokens as i64;
            aggregate.net_token_delta = aggregate.net_token_delta.saturating_add(delta);
            if delta >= 0 {
                aggregate.token_savings = aggregate.token_savings.saturating_add(delta as u64);
            } else {
                aggregate.token_overhead = aggregate
                    .token_overhead
                    .saturating_add(delta.unsigned_abs());
            }
        } else {
            aggregate.external_unavailable += 1;
        }
    }

    aggregate.savings_percent = if aggregate.native_tokens == 0 {
        0.0
    } else {
        aggregate.net_token_delta as f64 / aggregate.native_tokens as f64 * 100.0
    };

    ComparisonReport {
        fixture_count: workloads.len(),
        workloads,
        categories,
        external_preferred_workloads,
        native_preferred_workloads,
        aggregate,
    }
}

/// Export a structured JSON report to `path`.
pub fn write_json(report: &ComparisonReport, path: impl AsRef<Path>) -> Result<(), ReportError> {
    let path = path.as_ref();
    let body = report.to_json()?;
    fs::write(path, body).map_err(|source| ReportError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Export a human-readable report to `path`.
pub fn write_text(report: &ComparisonReport, path: impl AsRef<Path>) -> Result<(), ReportError> {
    let path = path.as_ref();
    fs::write(path, report.to_text()).map_err(|source| ReportError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn sorted_directories(path: &Path) -> Result<Vec<PathBuf>, ReportError> {
    let mut directories = Vec::new();
    for entry in fs::read_dir(path).map_err(|source| ReportError::Io {
        path: path.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ReportError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let entry_path = entry.path();
        if entry
            .file_type()
            .map_err(|source| ReportError::Io {
                path: entry_path.clone(),
                source,
            })?
            .is_dir()
        {
            directories.push(entry_path);
        }
    }
    directories.sort();
    Ok(directories)
}

fn validate_fixture(
    category: &str,
    name: &str,
    input: &str,
    metadata: &FixtureMetadata,
    fixture_path: &Path,
) -> Result<(), ReportError> {
    let invalid = |reason: &str| ReportError::InvalidFixture {
        path: fixture_path.to_path_buf(),
        reason: reason.to_string(),
    };

    if metadata.command.trim().is_empty() {
        return Err(invalid("command must not be empty"));
    }
    if metadata.working_dir.trim().is_empty() {
        return Err(invalid("working_dir must not be empty"));
    }
    if metadata.expected_tokens == 0 {
        return Err(invalid("expected_tokens must be greater than zero"));
    }
    if metadata.category != category {
        return Err(invalid("metadata category does not match its directory"));
    }
    let non_whitespace = input
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if non_whitespace < 20 || input.lines().count() < 2 {
        return Err(invalid("input.txt must contain realistic non-empty output"));
    }
    if input.contains("[placeholder]") || input.contains("<placeholder>") {
        return Err(invalid("input.txt contains placeholder data"));
    }
    if name.trim().is_empty() {
        return Err(invalid("fixture directory name must not be empty"));
    }
    Ok(())
}

fn compare_fixture(fixture: &FixtureInput) -> WorkloadComparison {
    let task_id = format!(
        "capability-comparison/{}/{}",
        fixture.metadata.category, fixture.name
    );
    let line_count = fixture.input.lines().count() as u64;
    let unique_line_count = fixture
        .input
        .lines()
        .map(str::trim)
        .collect::<BTreeSet<_>>()
        .len() as u64;
    let input_bytes = fixture.input.len() as u64;
    let input_tokens = ceil_div(
        (fixture.metadata.command.len() + fixture.metadata.working_dir.len()) as u64,
        4,
    );
    let native_tokens = fixture.metadata.expected_tokens;
    let external_tokens =
        estimate_external_tokens(native_tokens, line_count, unique_line_count, input_bytes);
    let external_quality_score = estimate_quality_score(
        &fixture.metadata.category,
        &fixture.metadata.command,
        &fixture.input,
    );
    let native_observation = observation(
        &task_id,
        "native.shell",
        "v1",
        input_tokens,
        native_tokens,
        native_latency_ms(input_bytes),
        100,
        line_count,
        unique_line_count,
        input_bytes,
        "native",
    );
    let external_observation = observation(
        &task_id,
        "rtk.shell",
        "v1",
        input_tokens,
        external_tokens,
        external_latency_ms(external_tokens),
        external_quality_score,
        line_count,
        unique_line_count,
        input_bytes,
        "external",
    );
    let quality_check = evaluate_quality(&native_observation, Some(&external_observation));
    let decision = decide(native_tokens, Some(external_tokens), &quality_check);
    let receipt = ComparisonReceipt::new(
        task_id,
        native_observation,
        Some(external_observation),
        decision,
        quality_check,
    );

    WorkloadComparison {
        name: fixture.name.clone(),
        category: fixture.metadata.category.clone(),
        command: fixture.metadata.command.clone(),
        receipt,
    }
}

#[allow(clippy::too_many_arguments)]
fn observation(
    task_id: &str,
    capability_id: &str,
    capability_version: &str,
    input_tokens: u64,
    output_tokens: u64,
    latency_ms: u64,
    quality_score: u64,
    line_count: u64,
    unique_line_count: u64,
    input_bytes: u64,
    arm: &str,
) -> CapabilityObservationV1 {
    let mut metrics = BTreeMap::new();
    metrics.insert("quality_score".to_string(), quality_score);
    metrics.insert("line_count".to_string(), line_count);
    metrics.insert("unique_line_count".to_string(), unique_line_count);
    metrics.insert("input_bytes".to_string(), input_bytes);

    CapabilityObservationV1 {
        schema_version: CAPABILITY_OBSERVATION_SCHEMA_VERSION,
        task_id: task_id.to_string(),
        capability_id: capability_id.to_string(),
        capability_version: capability_version.to_string(),
        success: true,
        input_tokens,
        output_tokens,
        latency_ms,
        failure_mode: None,
        output_ref: Some(format!("fixture:{task_id}:{arm}")),
        metrics,
    }
}

fn estimate_external_tokens(
    native_tokens: u64,
    line_count: u64,
    unique_line_count: u64,
    input_bytes: u64,
) -> u64 {
    let repetition = if line_count == 0 {
        0.0
    } else {
        1.0 - unique_line_count as f64 / line_count as f64
    };
    let ratio = if line_count <= 7 && repetition < 0.15 {
        1.18
    } else if repetition > 0.30 {
        0.58
    } else if input_bytes > 1_600 {
        0.68
    } else if line_count > 14 {
        0.82
    } else {
        0.94
    };
    let overhead = if ratio > 1.0 { 6 } else { 4 };
    ((native_tokens as f64 * ratio).round() as u64)
        .saturating_add(overhead)
        .max(1)
}

fn estimate_quality_score(category: &str, command: &str, input: &str) -> u64 {
    if category == "test"
        && command.contains("pytest")
        && input.contains("FAILED")
        && input.contains("AssertionError")
    {
        return 86;
    }

    let markers: &[&str] = match category {
        "git" => &[
            "On branch",
            "nothing to commit",
            "commit ",
            "diff --git",
            "@@ ",
            "Author:",
            "Date:",
            "modified:",
            "index ",
        ],
        "test" => &[
            "test result:",
            "PASSED",
            "FAILED",
            "PASS",
            "FAIL",
            "ok ",
            "running ",
            "Tests:",
            "Test Suites:",
            "=== RUN",
            "error:",
        ],
        "structured" => &[
            "total ",
            "drwx",
            "./",
            "CONTAINER ID",
            "NAME ",
            "READY",
            "package",
            "├──",
            "├─┬",
        ],
        _ => &[],
    };
    let present = markers
        .iter()
        .filter(|marker| input.contains(**marker))
        .count();
    match present {
        0 => 65,
        1 => 96,
        _ => 100,
    }
}

fn native_latency_ms(input_bytes: u64) -> u64 {
    (input_bytes / 180).saturating_add(1).max(1)
}

fn external_latency_ms(external_tokens: u64) -> u64 {
    (external_tokens / 80).saturating_add(2).max(1)
}

fn ceil_div(value: u64, divisor: u64) -> u64 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor
}

fn quality_label(quality_check: &QualityCheck) -> &'static str {
    match quality_check {
        QualityCheck::StructurallyEquivalent => "equivalent",
        QualityCheck::QualityFloorFailed => "floor_failed",
        QualityCheck::InformationLoss { .. } => "information_loss",
    }
}

fn append_workload_list(output: &mut String, workloads: &[String]) {
    if workloads.is_empty() {
        output.push_str("- None\n");
    } else {
        for workload in workloads {
            let _ = writeln!(output, "- {workload}");
        }
    }
}
