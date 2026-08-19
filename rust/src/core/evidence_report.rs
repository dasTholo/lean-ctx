//! Customer-readable rendering for a signed evidence-run directory.
//!
//! The report is deliberately derived only from the persisted evidence
//! artifacts. It does not invent a currency, workload volume, or statistical
//! conclusion that is not present in the bundle.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, ensure};
use lean_ctx_protocol::{ExecutionReceiptV1, SavingsReceiptV1};
use serde::Deserialize;

use crate::core::quality_scorecard::{
    DimensionScore, QualityComparison, QualityDimension, ScoreConfidence,
};

const BASELINE_RECEIPT_FILE: &str = "execution-receipt-baseline.json";
const TREATMENT_RECEIPT_FILE: &str = "execution-receipt-treatment.json";
const SAVINGS_RECEIPT_FILE: &str = "savings-receipt.json";
const QUALITY_COMPARISON_FILE: &str = "quality-comparison.json";
const RUN_METADATA_FILE: &str = "run-metadata.json";
const BUNDLE_FILE: &str = "evidence-bundle.zip";

/// Configuration for rendering an evidence-run directory as Markdown.
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Directory containing the persisted evidence artifacts.
    pub bundle_dir: PathBuf,
    /// Destination selected by the caller for the rendered Markdown.
    pub output_path: PathBuf,
    /// Optional customer name displayed in the report header.
    pub company_name: Option<String>,
    /// Report publication date supplied by the caller.
    pub report_date: String,
}

#[derive(Debug, Deserialize)]
struct RunMetadata {
    run_id: String,
    commit: String,
    model: String,
    provider: String,
    timestamp: String,
}

/// Render a professional Markdown report from the evidence artifacts.
///
/// This function only renders content. Callers write the returned Markdown to
/// [`ReportConfig::output_path`] after generation succeeds.
pub fn generate_report(config: &ReportConfig) -> Result<String> {
    ensure!(
        config.bundle_dir.is_dir(),
        "evidence directory does not exist or is not a directory: {}",
        config.bundle_dir.display()
    );

    let baseline: ExecutionReceiptV1 = read_json(&config.bundle_dir, BASELINE_RECEIPT_FILE)?;
    let treatment: ExecutionReceiptV1 = read_json(&config.bundle_dir, TREATMENT_RECEIPT_FILE)?;
    let savings: SavingsReceiptV1 = read_json(&config.bundle_dir, SAVINGS_RECEIPT_FILE)?;
    let quality: QualityComparison = read_json(&config.bundle_dir, QUALITY_COMPARISON_FILE)?;
    let metadata: RunMetadata = read_json(&config.bundle_dir, RUN_METADATA_FILE)?;

    validate_artifact_links(&baseline, &treatment, &savings, &quality, &metadata)?;

    let baseline_tokens = savings.baseline_tokens.provider_billed_tokens;
    let treatment_tokens = savings.treatment_tokens.provider_billed_tokens;
    let token_savings = baseline_tokens.saturating_sub(treatment_tokens);
    let quality_status = if quality.quality_preserved {
        "preserved"
    } else {
        "regression detected"
    };
    let file_list = bundle_file_list(&config.bundle_dir)?;
    let company_line = config
        .company_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .map(|name| format!("- Prepared for: {}\n", markdown_text(name)))
        .unwrap_or_default();

    let mut report = String::new();
    report.push_str("# Thinkery Evidence Report\n\n");
    report.push_str("## Header\n\n");
    report.push_str(&company_line);
    report.push_str(&format!("- Date: {}\n", markdown_text(&config.report_date)));
    report.push_str(&format!("- Run ID: `{}`\n", code_text(&metadata.run_id)));
    report.push_str(&format!(
        "- Methodology version: `{}`\n\n",
        code_text(&savings.methodology_version)
    ));

    report.push_str("## Executive Summary\n\n");
    report.push_str(&format!(
        "This matched run compared raw context with LeanCTX context optimization for task `{}` at commit `{}` using model `{}` through provider `{}`. Provider-billed input tokens decreased from {} to {}, a savings of {} ({}); recorded per-run cost decreased from {} to {}; the independent {}-dimension quality comparison reports quality {}.\n\n",
        code_text(baseline.task_id.as_str()),
        code_text(&metadata.commit),
        code_text(&metadata.model),
        code_text(&metadata.provider),
        format_tokens(baseline_tokens),
        format_tokens(treatment_tokens),
        format_tokens(token_savings),
        format_percent_milli(savings.token_savings_ratio_milli),
        format_cost(savings.baseline_cost_micros),
        format_cost(savings.treatment_cost_micros),
        quality_dimension_count(&quality),
        quality_status,
    ));
    report.push_str("Key metrics:\n\n");
    report.push_str(&format!(
        "- Token savings: {} ({})\n",
        format_tokens(token_savings),
        format_percent_milli(savings.token_savings_ratio_milli)
    ));
    report.push_str(&format!(
        "- Cost savings per matched run: {}\n",
        format_cost(savings.avoided_cost_micros)
    ));
    report.push_str(&format!("- Quality: {quality_status}\n\n"));

    report.push_str("## Methodology\n\n");
    report.push_str("- Matched controls: same task, same commit, same model, and same provider.\n");
    report.push_str("- Baseline: raw context with no optimization.\n");
    report.push_str("- Treatment: LeanCTX context optimization applied.\n");
    report.push_str(&format!(
        "- Quality: independent review on {} dimensions; tolerance recorded as {}.\n\n",
        quality_dimension_count(&quality),
        format_percent_milli(quality.tolerance_milli)
    ));

    report.push_str("## Results\n\n");
    report.push_str("### Token Usage\n\n");
    report.push_str(&format!(
        "- Baseline: {} input tokens\n",
        format_tokens(baseline_tokens)
    ));
    report.push_str(&format!(
        "- Treatment: {} input tokens\n",
        format_tokens(treatment_tokens)
    ));
    report.push_str(&format!(
        "- Savings: {} tokens ({})\n",
        format_tokens(token_savings),
        format_percent_milli(savings.token_savings_ratio_milli)
    ));
    report.push_str(&format!("- Source: `{SAVINGS_RECEIPT_FILE}`\n\n"));

    report.push_str("### Cost Impact\n\n");
    report.push_str(&format!(
        "- Baseline cost: {}\n",
        format_cost(savings.baseline_cost_micros)
    ));
    report.push_str(&format!(
        "- Treatment cost: {}\n",
        format_cost(savings.treatment_cost_micros)
    ));
    report.push_str("- Monthly savings estimate (extrapolated): unavailable; the bundle records no workload-volume assumption.\n");
    report.push_str(&format!(
        "- Source: `{SAVINGS_RECEIPT_FILE}`; receipt cost units are recorded in micros and do not identify a currency.\n\n"
    ));

    report.push_str("### Quality Assessment\n\n");
    report.push_str("| Dimension | Baseline | Treatment | Assessment |\n");
    report.push_str("| --- | ---: | ---: | --- |\n");
    for dimension in QualityDimension::ALL {
        let baseline_score = score_for(&quality.baseline_scorecard.dimensions, dimension);
        let treatment_score = score_for(&quality.treatment_scorecard.dimensions, dimension);
        report.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            quality_dimension_name(dimension),
            display_score(baseline_score),
            display_score(treatment_score),
            dimension_assessment(&quality, dimension, baseline_score, treatment_score),
        ));
    }
    report.push_str(&format!(
        "\nOverall: **{quality_status}**. Source: `{QUALITY_COMPARISON_FILE}`.\n\n",
    ));

    report.push_str("## Evidence Chain\n\n");
    report.push_str(&format!(
        "- Baseline ExecutionReceiptV1 ID: `{}`\n",
        code_text(baseline.receipt_id.as_str())
    ));
    report.push_str(&format!(
        "- Treatment ExecutionReceiptV1 ID: `{}`\n",
        code_text(treatment.receipt_id.as_str())
    ));
    report.push_str(&format!(
        "- SavingsReceiptV1 ID: `{}`\n",
        code_text(&savings.savings_id)
    ));
    report.push_str("- Signature algorithm: Ed25519\n");
    report.push_str("- Verification command: `lean-ctx verify <bundle.zip>`\n\n");

    report.push_str("## Verification\n\n");
    report.push_str("1. Change to the evidence directory.\n");
    report.push_str(&format!("2. Run `lean-ctx verify {BUNDLE_FILE}`.\n"));
    report.push_str("3. PASS means the manifest, artifact hashes, and Ed25519 signature verified; FAIL means at least one verification check did not pass and the bundle should not support a claim.\n\n");

    report.push_str("## Limitations\n\n");
    report.push_str("- Measured: provider-billed input tokens, receipt cost units, and the recorded quality comparison.\n");
    report.push_str("- Estimated: no monthly estimate is calculated because the bundle contains no run-rate or workload-volume input.\n");
    report.push_str("- Unavailable: a currency denomination and a production usage volume are not recorded in the bundle.\n");
    report.push_str("- This is a single matched run, not a statistical sample.\n");
    report.push_str(
        "- The bundle documents a local provider run, not a production-cloud workload.\n\n",
    );

    report.push_str("## Appendix\n\n");
    report.push_str("### Frozen Inputs\n\n");
    report.push_str(&format!(
        "- Task ID: `{}`\n",
        code_text(baseline.task_id.as_str())
    ));
    report.push_str(&format!("- Commit: `{}`\n", code_text(&metadata.commit)));
    report.push_str(&format!("- Model: `{}`\n", code_text(&metadata.model)));
    report.push_str(&format!(
        "- Provider: `{}`\n",
        code_text(&metadata.provider)
    ));
    report.push_str(&format!(
        "- Evidence captured: `{}`\n\n",
        code_text(&metadata.timestamp)
    ));
    report.push_str("### Full File List in Bundle\n\n");
    for file in file_list {
        report.push_str(&format!("- `{}`\n", code_text(&file)));
    }

    Ok(report)
}

fn read_json<T: serde::de::DeserializeOwned>(bundle_dir: &Path, name: &str) -> Result<T> {
    let path = bundle_dir.join(name);
    let bytes =
        fs::read(&path).with_context(|| format!("read evidence artifact {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse evidence artifact {}", path.display()))
}

fn validate_artifact_links(
    baseline: &ExecutionReceiptV1,
    treatment: &ExecutionReceiptV1,
    savings: &SavingsReceiptV1,
    quality: &QualityComparison,
    metadata: &RunMetadata,
) -> Result<()> {
    baseline
        .validate()
        .map_err(|error| anyhow!("validate baseline execution receipt: {error}"))?;
    treatment
        .validate()
        .map_err(|error| anyhow!("validate treatment execution receipt: {error}"))?;
    savings
        .validate()
        .map_err(|error| anyhow!("validate savings receipt: {error}"))?;
    ensure!(
        savings.baseline_receipt_id == baseline.receipt_id
            && savings.treatment_receipt_id == treatment.receipt_id,
        "savings receipt does not reference the execution receipts in this directory"
    );
    ensure!(
        savings.baseline_tokens == baseline.context_balance
            && savings.treatment_tokens == treatment.context_balance,
        "savings receipt token balances do not match the execution receipts"
    );
    ensure!(
        savings.baseline_cost_micros == baseline.actual_cost_micros
            && savings.treatment_cost_micros == treatment.actual_cost_micros,
        "savings receipt costs do not match the execution receipts"
    );
    ensure!(
        savings.avoided_cost_micros
            == savings
                .baseline_cost_micros
                .saturating_sub(savings.treatment_cost_micros),
        "savings receipt avoided cost does not match baseline minus treatment"
    );
    ensure!(
        savings.token_savings_ratio_milli
            == savings_ratio_milli(
                savings.baseline_tokens.provider_billed_tokens,
                savings.treatment_tokens.provider_billed_tokens,
            ),
        "savings receipt token ratio does not match its token balances"
    );
    ensure!(
        baseline.task_id == treatment.task_id && baseline.task_id.as_str() == metadata.run_id,
        "execution receipts and run metadata do not identify the same task"
    );
    ensure!(
        baseline.selected_model == treatment.selected_model
            && baseline.requested_model == treatment.requested_model
            && baseline.selected_model == metadata.model,
        "execution receipts and metadata do not identify the same model"
    );
    ensure!(
        baseline.provider == treatment.provider && baseline.provider == metadata.provider,
        "execution receipts and metadata do not identify the same provider"
    );
    ensure!(
        quality.baseline_scorecard.run_id == metadata.run_id
            && quality.treatment_scorecard.run_id == metadata.run_id,
        "quality scorecards do not identify the evidence run"
    );
    Ok(())
}

fn savings_ratio_milli(baseline_tokens: u64, treatment_tokens: u64) -> u16 {
    if baseline_tokens == 0 {
        return 0;
    }
    (((baseline_tokens.saturating_sub(treatment_tokens) as u128 * 1_000)
        / u128::from(baseline_tokens))
    .min(1_000)) as u16
}

fn bundle_file_list(bundle_dir: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    collect_files(bundle_dir, bundle_dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("list evidence directory {}", directory.display()))?
    {
        let entry =
            entry.with_context(|| format!("read evidence directory {}", directory.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, files)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .with_context(|| format!("resolve evidence file {}", path.display()))?;
            files.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn quality_dimension_count(quality: &QualityComparison) -> usize {
    quality
        .baseline_scorecard
        .dimensions
        .len()
        .max(quality.treatment_scorecard.dimensions.len())
}

fn score_for(scores: &[DimensionScore], dimension: QualityDimension) -> Option<&DimensionScore> {
    scores.iter().find(|score| score.dimension == dimension)
}

fn display_score(score: Option<&DimensionScore>) -> String {
    match score {
        Some(score) if score.confidence != ScoreConfidence::Unavailable => {
            format_percent_milli(score.score_milli)
        }
        _ => "unavailable".to_string(),
    }
}

fn dimension_assessment(
    quality: &QualityComparison,
    dimension: QualityDimension,
    baseline: Option<&DimensionScore>,
    treatment: Option<&DimensionScore>,
) -> &'static str {
    if baseline.is_none()
        || treatment.is_none()
        || baseline.is_some_and(|score| score.confidence == ScoreConfidence::Unavailable)
        || treatment.is_some_and(|score| score.confidence == ScoreConfidence::Unavailable)
    {
        "Unavailable"
    } else if quality.regression_dimensions.contains(&dimension) {
        "Regression detected"
    } else {
        "Preserved"
    }
}

fn quality_dimension_name(dimension: QualityDimension) -> &'static str {
    match dimension {
        QualityDimension::Correctness => "Correctness",
        QualityDimension::Completeness => "Completeness",
        QualityDimension::Actionability => "Actionability",
        QualityDimension::Safety => "Safety",
        QualityDimension::Relevance => "Relevance",
    }
}

fn format_tokens(tokens: u64) -> String {
    let value = tokens.to_string();
    let first_group = value.len() % 3;
    let mut formatted = String::with_capacity(value.len() + value.len() / 3);
    for (index, digit) in value.chars().enumerate() {
        if index > 0 && (index - first_group).is_multiple_of(3) {
            formatted.push(',');
        }
        formatted.push(digit);
    }
    formatted
}

fn format_percent_milli(value: u16) -> String {
    format!("{}.{}%", value / 10, value % 10)
}

fn format_cost(micros: u64) -> String {
    format!(
        "{}.{:06} receipt cost units",
        micros / 1_000_000,
        micros % 1_000_000
    )
}

fn markdown_text(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

fn code_text(value: &str) -> String {
    markdown_text(value).replace('`', "'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn generates_customer_readable_report_from_evidence_artifacts() {
        let directory = std::env::temp_dir().join(format!(
            "lean-ctx-evidence-report-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&directory).unwrap();
        write_fixture(&directory);

        let report = generate_report(&ReportConfig {
            bundle_dir: directory.clone(),
            output_path: directory.join("report.md"),
            company_name: Some("Example Company".to_owned()),
            report_date: "2026-08-19".to_owned(),
        })
        .unwrap();

        for section in [
            "## Header",
            "## Executive Summary",
            "## Methodology",
            "## Results",
            "### Token Usage",
            "### Cost Impact",
            "### Quality Assessment",
            "## Evidence Chain",
            "## Verification",
            "## Limitations",
            "## Appendix",
        ] {
            assert!(report.contains(section), "missing section: {section}");
        }
        assert!(report.contains("- Baseline: 1,000 input tokens"));
        assert!(report.contains("- Treatment: 400 input tokens"));
        assert!(report.contains("- Savings: 600 tokens (60.0%)"));
        assert!(report.contains("- Baseline cost: 0.025000 receipt cost units"));
        assert!(report.contains("- Treatment cost: 0.010000 receipt cost units"));
        assert!(report.contains("- SavingsReceiptV1 ID: `savings-run-123`"));
        assert!(report.contains("| Correctness | 90.0% | 90.0% | Preserved |"));

        let _ = fs::remove_dir_all(directory);
    }

    fn write_fixture(directory: &Path) {
        let baseline = receipt("baseline-receipt", 1_000, 25_000);
        let treatment = receipt("treatment-receipt", 400, 10_000);
        let savings = json!({
            "schema_version": 1,
            "savings_id": "savings-run-123",
            "task_id": "run-123",
            "baseline_receipt_id": "baseline-receipt",
            "treatment_receipt_id": "treatment-receipt",
            "baseline_cost_micros": 25_000,
            "treatment_cost_micros": 10_000,
            "avoided_cost_micros": 15_000,
            "baseline_tokens": balance(1_000),
            "treatment_tokens": balance(400),
            "token_savings_ratio_milli": 600,
            "quality_preserved": true,
            "quality_baseline_score_milli": 900,
            "quality_treatment_score_milli": 900,
            "measurement_method": "provider_reported",
            "context_strategy": "provider-run",
            "methodology_version": "savings-receipt-v1",
            "evidence_refs": [],
            "decision_refs": [],
            "signature": "signature"
        });
        let dimensions = json!([
            score("correctness", 900),
            score("completeness", 900),
            score("actionability", 900),
            score("safety", 900),
            score("relevance", 900)
        ]);
        let quality = json!({
            "baseline_scorecard": scorecard(dimensions.clone()),
            "treatment_scorecard": scorecard(dimensions),
            "quality_preserved": true,
            "regression_dimensions": [],
            "tolerance_milli": 50
        });
        let metadata = json!({
            "run_id": "run-123",
            "commit": "abc123",
            "model": "example-model",
            "provider": "example-provider",
            "timestamp": "2026-08-19T00:00:00Z"
        });

        write_json(directory, BASELINE_RECEIPT_FILE, baseline);
        write_json(directory, TREATMENT_RECEIPT_FILE, treatment);
        write_json(directory, SAVINGS_RECEIPT_FILE, savings);
        write_json(directory, QUALITY_COMPARISON_FILE, quality);
        write_json(directory, RUN_METADATA_FILE, metadata);
        fs::write(directory.join(BUNDLE_FILE), b"fixture bundle").unwrap();
    }

    fn receipt(receipt_id: &str, tokens: u64, cost_micros: u64) -> serde_json::Value {
        json!({
            "schema_version": 1,
            "receipt_id": receipt_id,
            "task_id": "run-123",
            "plan_id": "provider-run-run-123",
            "context_balance": balance(tokens),
            "fresh_input_tokens": tokens,
            "cached_input_tokens": 0,
            "output_tokens": 100,
            "reasoning_tokens": 0,
            "requested_model": "example-model",
            "selected_model": "example-model",
            "provider": "example-provider",
            "model_calls": 1,
            "retries": 0,
            "latency_ms": 100,
            "actual_cost_micros": cost_micros,
            "baseline_cost_micros": cost_micros,
            "avoided_cost_micros": 0,
            "etpao_milli": 0,
            "decision_refs": [],
            "evidence_refs": [],
            "signature": "signature"
        })
    }

    fn balance(tokens: u64) -> serde_json::Value {
        json!({
            "original_tokens": tokens,
            "materialized_tokens": tokens,
            "delivered_tokens": tokens,
            "provider_billed_tokens": tokens
        })
    }

    fn score(dimension: &str, score_milli: u16) -> serde_json::Value {
        json!({
            "dimension": dimension,
            "score_milli": score_milli,
            "confidence": "automated"
        })
    }

    fn scorecard(dimensions: serde_json::Value) -> serde_json::Value {
        json!({
            "scorecard_id": "scorecard-run-123",
            "run_id": "run-123",
            "arm_type": "matched",
            "dimensions": dimensions,
            "overall_score_milli": 900,
            "reviewer": "independent-reviewer",
            "timestamp": "2026-08-19T00:00:00Z"
        })
    }

    fn write_json(directory: &Path, name: &str, value: serde_json::Value) {
        fs::write(directory.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    }
}
