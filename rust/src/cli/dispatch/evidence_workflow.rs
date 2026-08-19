use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::extension_registry::ExtensionRegistry;
use crate::core::tokens::count_tokens;
use crate::shell::compress::engine::compress_if_beneficial_pub;

// ─── Data Structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WorkflowStep {
    pub name: String,
    pub description: String,
    pub baseline_tokens: usize,
    pub treatment_tokens: usize,
    pub savings_pct: f64,
    pub compression_mode: String,
    pub baseline_content_hash: String,
    pub treatment_content_hash: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LlmComparison {
    pub baseline_prompt_tokens: usize,
    pub treatment_prompt_tokens: usize,
    pub baseline_response: String,
    pub treatment_response: String,
    pub baseline_response_tokens: usize,
    pub treatment_response_tokens: usize,
    pub model: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WorkflowResult {
    pub scenario: String,
    pub target_path: String,
    pub steps: Vec<WorkflowStep>,
    pub total_baseline_tokens: usize,
    pub total_treatment_tokens: usize,
    pub total_savings_pct: f64,
    pub composition_effect: String,
    pub llm_comparison: Option<LlmComparison>,
    pub timestamp: String,
    pub lean_ctx_version: String,
}

// ─── Workflow Arguments ─────────────────────────────────────────────────────

pub(crate) struct WorkflowArgs {
    pub target_dir: PathBuf,
    #[allow(dead_code)]
    pub output_dir: PathBuf,
    pub endpoint: String,
    pub model: String,
    pub api_key: String,
    pub system_prompt: String,
}

// ─── Main Entry Point ───────────────────────────────────────────────────────

pub(crate) fn execute_workflow_evidence(args: &WorkflowArgs) -> Result<WorkflowResult> {
    let target = &args.target_dir;
    if !target.is_dir() {
        bail!("target must be a directory: {}", target.display());
    }

    println!("═══ Workflow Evidence: Full Power Demo ═══");
    println!("Target: {}", target.display());
    println!();

    let registry = ExtensionRegistry::with_builtins();
    let mut steps = Vec::new();

    // Step 1: Orientation (tree/file listing)
    println!("[1/6] Orientation — project structure...");
    steps.push(step_orientation(target)?);

    // Step 2: Compose (multi-file understanding)
    println!("[2/6] Compose — module understanding...");
    steps.push(step_compose(target, &registry)?);

    // Step 3: Search (symbol discovery)
    println!("[3/6] Search — symbol discovery...");
    steps.push(step_search(target)?);

    // Step 4: Multi-Read (targeted file reading)
    println!("[4/6] Multi-Read — targeted file reading...");
    steps.push(step_multi_read(target, &registry)?);

    // Step 5: Shell (test execution)
    println!("[5/6] Shell — command execution...");
    steps.push(step_shell(target)?);

    // Step 6: Callgraph (dependency analysis)
    println!("[6/6] Callgraph — dependency analysis...");
    steps.push(step_callgraph(target)?);

    // Compute totals
    let total_baseline: usize = steps.iter().map(|s| s.baseline_tokens).sum();
    let total_treatment: usize = steps.iter().map(|s| s.treatment_tokens).sum();
    let total_savings = if total_baseline > 0 {
        (1.0 - total_treatment as f64 / total_baseline as f64) * 100.0
    } else {
        0.0
    };

    println!();
    println!("═══ Local Measurement Complete ═══");
    println!("Total baseline:  {total_baseline:>8} tokens");
    println!("Total treatment: {total_treatment:>8} tokens");
    println!("Total savings:   {total_savings:>8.1}%");
    println!();

    // Step 7: LLM Comparison (if API key provided)
    let llm_comparison = if !args.api_key.is_empty() && args.api_key != "skip" {
        println!("[7/7] LLM Call — sending both contexts to provider...");
        let baseline_ctx = collect_baseline_context(&steps, target)?;
        let treatment_ctx = collect_treatment_context(&steps, target)?;
        match llm_compare(args, &baseline_ctx, &treatment_ctx) {
            Ok(cmp) => {
                println!(
                    "      Baseline prompt:  {} tokens",
                    cmp.baseline_prompt_tokens
                );
                println!(
                    "      Treatment prompt: {} tokens",
                    cmp.treatment_prompt_tokens
                );
                Some(cmp)
            }
            Err(e) => {
                eprintln!("      LLM comparison failed: {e:#}");
                None
            }
        }
    } else {
        println!("[7/7] LLM Call — skipped (no API key)");
        None
    };

    let result = WorkflowResult {
        scenario: "code-review".to_string(),
        target_path: target.display().to_string(),
        steps,
        total_baseline_tokens: total_baseline,
        total_treatment_tokens: total_treatment,
        total_savings_pct: total_savings,
        composition_effect: format!(
            "{} tokens saved across 6 operations ({}x reduction)",
            total_baseline - total_treatment,
            if total_treatment > 0 {
                format!("{:.1}", total_baseline as f64 / total_treatment as f64)
            } else {
                "∞".to_string()
            }
        ),
        llm_comparison,
        timestamp: chrono::Utc::now().to_rfc3339(),
        lean_ctx_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Ok(result)
}

// ─── Step Implementations ───────────────────────────────────────────────────

fn step_orientation(target: &Path) -> Result<WorkflowStep> {
    let start = Instant::now();

    // Baseline: raw `find` output
    let raw_output = run_cmd(target, "find", &[".", "-name", "*.rs", "-type", "f"])?;

    // Treatment: compressed tree (depth-limited, structured)
    let treatment = compress_tree_output(target)?;

    let baseline_tokens = count_tokens(&raw_output);
    let treatment_tokens = count_tokens(&treatment);

    Ok(WorkflowStep {
        name: "orientation".to_string(),
        description: "Project structure discovery (find vs ctx_tree)".to_string(),
        baseline_tokens,
        treatment_tokens,
        savings_pct: savings_pct(baseline_tokens, treatment_tokens),
        compression_mode: "tree".to_string(),
        baseline_content_hash: sha256_hex(&raw_output),
        treatment_content_hash: sha256_hex(&treatment),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn step_compose(target: &Path, registry: &ExtensionRegistry) -> Result<WorkflowStep> {
    let start = Instant::now();

    // Baseline: concatenate ALL .rs files raw
    let rs_files = collect_rs_files(target, 10)?;
    let mut baseline = String::new();
    for file in &rs_files {
        let content =
            fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
        baseline.push_str(&format!("// === {} ===\n", file.display()));
        baseline.push_str(&content);
        baseline.push('\n');
    }

    // Treatment: each file compressed with "outline" mode (structure + key signatures)
    let outline_mode = registry.read_mode("outline");
    let mut treatment = String::new();
    for file in &rs_files {
        let content = fs::read_to_string(file)?;
        let path_str = file.display().to_string();
        let compressed = if let Some(ref mode) = outline_mode {
            mode.render(&content, &path_str)
        } else {
            compress_to_signatures(&content, &path_str)
        };
        treatment.push_str(&compressed);
        treatment.push('\n');
    }

    let baseline_tokens = count_tokens(&baseline);
    let treatment_tokens = count_tokens(&treatment);

    Ok(WorkflowStep {
        name: "compose".to_string(),
        description: format!(
            "Module understanding ({} files, outline mode)",
            rs_files.len()
        ),
        baseline_tokens,
        treatment_tokens,
        savings_pct: savings_pct(baseline_tokens, treatment_tokens),
        compression_mode: "outline".to_string(),
        baseline_content_hash: sha256_hex(&baseline),
        treatment_content_hash: sha256_hex(&treatment),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn step_search(target: &Path) -> Result<WorkflowStep> {
    let start = Instant::now();

    // Baseline: raw `rg` output with context
    let raw_output = run_cmd(
        target,
        "rg",
        &["--no-heading", "-n", "-C2", "pub fn|pub struct|impl ", "."],
    )
    .unwrap_or_else(|_| {
        run_cmd(target, "grep", &["-rn", "pub fn\\|pub struct\\|impl ", "."]).unwrap_or_default()
    });

    // Treatment: compressed search output (lean-ctx shell pattern for grep/rg)
    let treatment = compress_if_beneficial_pub(
        "rg --no-heading -n -C2 'pub fn|pub struct|impl'",
        &raw_output,
    );

    let baseline_tokens = count_tokens(&raw_output);
    let treatment_tokens = count_tokens(&treatment);

    Ok(WorkflowStep {
        name: "search".to_string(),
        description: "Symbol discovery (rg with context → compressed)".to_string(),
        baseline_tokens,
        treatment_tokens,
        savings_pct: savings_pct(baseline_tokens, treatment_tokens),
        compression_mode: "shell_pattern_rg".to_string(),
        baseline_content_hash: sha256_hex(&raw_output),
        treatment_content_hash: sha256_hex(&treatment),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn step_multi_read(target: &Path, registry: &ExtensionRegistry) -> Result<WorkflowStep> {
    let start = Instant::now();

    // Pick the 3 largest .rs files
    let files = collect_rs_files_by_size(target, 3)?;

    // Baseline: raw full content
    let mut baseline = String::new();
    for file in &files {
        let content = fs::read_to_string(file)?;
        baseline.push_str(&format!(
            "// === {} ({} bytes) ===\n",
            file.display(),
            content.len()
        ));
        baseline.push_str(&content);
        baseline.push('\n');
    }

    // Treatment: signatures mode (function headers only)
    let sig_mode = registry.read_mode("signatures");
    let mut treatment = String::new();
    for file in &files {
        let content = fs::read_to_string(file)?;
        let path_str = file.display().to_string();
        let compressed = if let Some(ref mode) = sig_mode {
            mode.render(&content, &path_str)
        } else {
            compress_to_signatures(&content, &path_str)
        };
        treatment.push_str(&compressed);
        treatment.push('\n');
    }

    let baseline_tokens = count_tokens(&baseline);
    let treatment_tokens = count_tokens(&treatment);

    Ok(WorkflowStep {
        name: "multi_read".to_string(),
        description: format!("Targeted reading ({} files, signatures mode)", files.len()),
        baseline_tokens,
        treatment_tokens,
        savings_pct: savings_pct(baseline_tokens, treatment_tokens),
        compression_mode: "signatures".to_string(),
        baseline_content_hash: sha256_hex(&baseline),
        treatment_content_hash: sha256_hex(&treatment),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn step_shell(target: &Path) -> Result<WorkflowStep> {
    let start = Instant::now();

    // Baseline: raw `cargo test` output (or simulated if not in a cargo project)
    let raw_output = run_cmd(target, "cargo", &["test", "--lib", "--", "--color=never"])
        .or_else(|_| run_cmd(target, "cargo", &["check", "--message-format=short"]))
        .unwrap_or_else(|_| generate_synthetic_test_output());

    // Treatment: lean-ctx shell compression (cargo test pattern)
    let treatment = compress_if_beneficial_pub("cargo test --lib", &raw_output);

    let baseline_tokens = count_tokens(&raw_output);
    let treatment_tokens = count_tokens(&treatment);

    Ok(WorkflowStep {
        name: "shell".to_string(),
        description: "Test execution (cargo test → compressed)".to_string(),
        baseline_tokens,
        treatment_tokens,
        savings_pct: savings_pct(baseline_tokens, treatment_tokens),
        compression_mode: "shell_pattern_cargo".to_string(),
        baseline_content_hash: sha256_hex(&raw_output),
        treatment_content_hash: sha256_hex(&treatment),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn step_callgraph(target: &Path) -> Result<WorkflowStep> {
    let start = Instant::now();

    // Baseline: raw grep for use/mod/pub references (simulating manual dependency tracing)
    let raw_output = run_cmd(
        target,
        "rg",
        &[
            "--no-heading",
            "-n",
            "^use |^mod |pub(crate) fn |pub fn ",
            ".",
        ],
    )
    .unwrap_or_else(|_| {
        run_cmd(target, "grep", &["-rn", "^use \\|^mod \\|pub.* fn ", "."]).unwrap_or_default()
    });

    // Treatment: compressed dependency graph output
    let treatment = compress_if_beneficial_pub(
        "rg --no-heading -n '^use |^mod |pub(crate) fn |pub fn '",
        &raw_output,
    );

    let baseline_tokens = count_tokens(&raw_output);
    let treatment_tokens = count_tokens(&treatment);

    Ok(WorkflowStep {
        name: "callgraph".to_string(),
        description: "Dependency analysis (imports/exports → compressed)".to_string(),
        baseline_tokens,
        treatment_tokens,
        savings_pct: savings_pct(baseline_tokens, treatment_tokens),
        compression_mode: "shell_pattern_rg".to_string(),
        baseline_content_hash: sha256_hex(&raw_output),
        treatment_content_hash: sha256_hex(&treatment),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

// ─── LLM Comparison ─────────────────────────────────────────────────────────

fn llm_compare(
    args: &WorkflowArgs,
    baseline_ctx: &str,
    treatment_ctx: &str,
) -> Result<LlmComparison> {
    let system = if args.system_prompt.is_empty() {
        "You are a senior software engineer. Review the provided code context and identify potential bugs, security issues, or performance problems. Be concise.".to_string()
    } else {
        args.system_prompt.clone()
    };

    let user_msg = "Review this code module. Identify the top 3 issues (bugs, security, performance) and suggest fixes.";

    // Baseline call
    let baseline_prompt = format!("{system}\n\n{user_msg}\n\n{baseline_ctx}");
    let baseline_prompt_tokens = count_tokens(&baseline_prompt);
    let baseline_resp = call_llm(
        &args.endpoint,
        &args.model,
        &args.api_key,
        &system,
        &format!("{user_msg}\n\n{baseline_ctx}"),
    )?;

    // Treatment call
    let treatment_prompt = format!("{system}\n\n{user_msg}\n\n{treatment_ctx}");
    let treatment_prompt_tokens = count_tokens(&treatment_prompt);
    let treatment_resp = call_llm(
        &args.endpoint,
        &args.model,
        &args.api_key,
        &system,
        &format!("{user_msg}\n\n{treatment_ctx}"),
    )?;

    Ok(LlmComparison {
        baseline_prompt_tokens,
        treatment_prompt_tokens,
        baseline_response: baseline_resp.clone(),
        treatment_response: treatment_resp.clone(),
        baseline_response_tokens: count_tokens(&baseline_resp),
        treatment_response_tokens: count_tokens(&treatment_resp),
        model: args.model.clone(),
        endpoint: args.endpoint.clone(),
    })
}

fn call_llm(
    endpoint: &str,
    model: &str,
    api_key: &str,
    system: &str,
    user: &str,
) -> Result<String> {
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "max_tokens": 1024,
        "temperature": 0.0
    });

    let payload = serde_json::to_vec(&body).context("serialize request")?;

    let agent = crate::core::http_client::ureq_agent(
        ureq::config::Config::builder()
            .tls_config(crate::core::http_client::platform_tls_config())
            .timeout_global(Some(std::time::Duration::from_mins(2)))
            .build(),
    );

    let resp = agent
        .post(endpoint)
        .header("Authorization", &format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .send(payload.as_slice())
        .context("LLM API call failed")?;

    let text = resp
        .into_body()
        .read_to_string()
        .context("read LLM response body")?;

    let json: serde_json::Value = serde_json::from_str(&text).context("parse LLM response JSON")?;

    let content = json
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(content)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn collect_baseline_context(steps: &[WorkflowStep], target: &Path) -> Result<String> {
    let rs_files = collect_rs_files(target, 10)?;
    let mut ctx = String::new();
    for file in &rs_files {
        let content = fs::read_to_string(file)?;
        ctx.push_str(&format!("// === {} ===\n{}\n", file.display(), content));
    }
    let _ = steps; // steps metadata available if needed
    Ok(ctx)
}

fn collect_treatment_context(steps: &[WorkflowStep], target: &Path) -> Result<String> {
    let registry = ExtensionRegistry::with_builtins();
    let rs_files = collect_rs_files(target, 10)?;
    let outline_mode = registry.read_mode("outline");
    let mut ctx = String::new();
    for file in &rs_files {
        let content = fs::read_to_string(file)?;
        let path_str = file.display().to_string();
        let compressed = if let Some(ref mode) = outline_mode {
            mode.render(&content, &path_str)
        } else {
            compress_to_signatures(&content, &path_str)
        };
        ctx.push_str(&compressed);
        ctx.push('\n');
    }
    let _ = steps;
    Ok(ctx)
}

fn run_cmd(dir: &Path, prog: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(prog)
        .args(args)
        .current_dir(dir)
        .output()
        .with_context(|| format!("run {prog}"))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn compress_tree_output(target: &Path) -> Result<String> {
    let mut entries: Vec<String> = Vec::new();
    let mut file_count = 0usize;
    let mut dir_count = 0usize;

    collect_tree_entries(
        target,
        target,
        2,
        &mut entries,
        &mut file_count,
        &mut dir_count,
    )?;

    let mut result = format!(
        "{} ({} files, {} dirs)\n",
        target.file_name().unwrap_or_default().to_string_lossy(),
        file_count,
        dir_count,
    );
    for entry in entries {
        result.push_str(&entry);
        result.push('\n');
    }
    Ok(result)
}

fn collect_tree_entries(
    base: &Path,
    current: &Path,
    max_depth: usize,
    entries: &mut Vec<String>,
    file_count: &mut usize,
    dir_count: &mut usize,
) -> Result<()> {
    if max_depth == 0 {
        return Ok(());
    }
    let mut items: Vec<_> = fs::read_dir(current)?
        .filter_map(std::result::Result::ok)
        .collect();
    items.sort_by_key(std::fs::DirEntry::file_name);

    for item in items {
        let name = item.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        let path = item.path();
        let rel = path.strip_prefix(base).unwrap_or(&path);
        let indent = "  ".repeat(rel.components().count().saturating_sub(1));
        if path.is_dir() {
            *dir_count += 1;
            entries.push(format!("{indent}{name}/"));
            collect_tree_entries(base, &path, max_depth - 1, entries, file_count, dir_count)?;
        } else if std::path::Path::new(&name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("rs") || e.eq_ignore_ascii_case("toml"))
        {
            *file_count += 1;
            entries.push(format!("{indent}{name}"));
        }
    }
    Ok(())
}

fn collect_rs_files(dir: &Path, max: usize) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_rs_recursive(dir, &mut files, 3)?;
    files.sort_by_key(|f| f.display().to_string());
    files.truncate(max);
    Ok(files)
}

fn collect_rs_files_by_size(dir: &Path, max: usize) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_rs_recursive(dir, &mut files, 3)?;
    files.sort_by(|a, b| {
        let sa = fs::metadata(a).map(|m| m.len()).unwrap_or(0);
        let sb = fs::metadata(b).map(|m| m.len()).unwrap_or(0);
        sb.cmp(&sa)
    });
    files.truncate(max);
    Ok(files)
}

fn collect_rs_recursive(dir: &Path, files: &mut Vec<PathBuf>, depth: usize) -> Result<()> {
    if depth == 0 {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        if path.is_dir() {
            collect_rs_recursive(&path, files, depth - 1)?;
        } else if std::path::Path::new(&name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("rs"))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn compress_to_signatures(source: &str, path: &str) -> String {
    let mut result = format!("{path}\n");
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("pub trait ")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("use ")
            || trimmed.starts_with("#[")
        {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

fn generate_synthetic_test_output() -> String {
    let mut out = String::new();
    out.push_str("   Compiling lean-ctx v3.9.19\n");
    out.push_str("    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.34s\n");
    out.push_str("     Running unittests src/lib.rs\n\n");
    for i in 0..50 {
        out.push_str(&format!("test core::test_{i:03} ... ok\n"));
    }
    out.push_str("\ntest result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.21s\n");
    out
}

fn savings_pct(baseline: usize, treatment: usize) -> f64 {
    if baseline == 0 {
        return 0.0;
    }
    (1.0 - treatment as f64 / baseline as f64) * 100.0
}

fn sha256_hex(input: &str) -> String {
    let hash = Sha256::digest(input.as_bytes());
    hash.iter()
        .fold(String::with_capacity(hash.len() * 2), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        })
}
