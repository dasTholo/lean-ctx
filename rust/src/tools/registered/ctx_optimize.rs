use rmcp::ErrorData;
use rmcp::model::Tool;
use serde_json::{Map, Value, json};

use crate::server::tool_trait::{McpTool, ToolContext, ToolOutput, get_str};
use crate::tool_defs::tool_def;

pub struct CtxOptimizeTool;

impl McpTool for CtxOptimizeTool {
    fn name(&self) -> &'static str {
        "ctx_optimize"
    }

    fn tool_def(&self) -> Tool {
        tool_def(
            "ctx_optimize",
            "Solution Intelligence — review code for optimizations, record decisions, get session report.\n\
             Actions: ladder (show optimization ladder), decide (record a decision), report (session summary),\n\
             review/suggest (analyze a file), fingerprint (predict solution rung), policy-check (validate team policy).",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "review|suggest|report|ladder|decide|fingerprint|policy-check"
                    },
                    "path": {
                        "type": "string",
                        "description": "File path (for review/suggest)"
                    },
                    "decision": {
                        "type": "string",
                        "description": "Decision text (for decide/fingerprint)"
                    },
                    "category": {
                        "type": "string",
                        "description": "Decision category: stdlib|native|reuse|yagni|one_line|debt"
                    }
                },
                "required": ["action"]
            }),
        )
    }

    fn handle(
        &self,
        args: &Map<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, ErrorData> {
        let action = get_str(args, "action")
            .ok_or_else(|| ErrorData::invalid_params("action is required", None))?;

        let text = match action.as_str() {
            "ladder" => crate::core::config::solution::SolutionConfig::default()
                .ladder_text()
                .to_string(),
            "decide" => {
                let decision = required_arg(args, "decision")?;
                let category = get_str(args, "category").unwrap_or_else(|| "general".to_string());
                crate::core::solution_tracker::record_decision(&category);
                format!("Recorded {category} decision: {decision}")
            }
            "report" => {
                let mut report = crate::core::solution_tracker::gain_summary();
                let snap = crate::core::solution_tracker::snapshot();
                let cfg = crate::core::config::Config::load();
                if let Some(rec) = crate::core::solution_commercial::recommend_intensity(
                    &cfg.solution.commercial.adaptive,
                    &snap,
                ) {
                    report.push_str(&format!(
                        "\n\nAdaptive: suggest '{}' (confidence {:.0}%) — {}",
                        rec.suggested_intensity,
                        rec.confidence * 100.0,
                        rec.reason
                    ));
                }
                report
            }
            "review" | "suggest" => {
                let path_str = required_arg(args, "path")?;
                let resolved = ctx.resolved_path("path").ok_or_else(|| {
                    ErrorData::invalid_params("path must be a project path", None)
                })?;
                let content = std::fs::read_to_string(resolved)
                    .map_err(|e| ErrorData::invalid_params(format!("read: {e}"), None))?;
                let findings = analyze_solution_patterns(&content, &path_str);
                let line_count = content.lines().count();
                let mut out =
                    format!("## Solution Review: {path_str}\n\nFile: {line_count} lines\n");
                if findings.is_empty() {
                    out.push_str("\nNo optimization opportunities found.\n");
                } else {
                    out.push_str(&format!("\n{} finding(s):\n", findings.len()));
                    for f in &findings {
                        out.push_str(&format!("- {f}\n"));
                    }
                }
                out.push_str(&format!(
                    "\nSession: {}\n",
                    crate::core::solution_tracker::gain_summary()
                ));
                out
            }
            "fingerprint" => {
                let decision = required_arg(args, "decision")?;
                let fp = crate::core::solution_commercial::predict_rung(&decision);
                format!(
                    "Prediction: rung='{}' confidence={:.0}% pattern='{}'",
                    fp.predicted_rung,
                    fp.confidence * 100.0,
                    fp.task_pattern
                )
            }
            "policy-check" => {
                let cfg = crate::core::config::Config::load();
                let intensity = cfg.solution.intensity.label();
                match crate::core::solution_commercial::validate_team_policy(
                    &cfg.solution.commercial.team_policy,
                    intensity,
                ) {
                    Ok(()) => format!("Policy check passed. Intensity '{intensity}' is compliant."),
                    Err(e) => format!("Policy violation: {e}"),
                }
            }
            _ => {
                return Err(ErrorData::invalid_params(
                    format!("unsupported action: {action}"),
                    None,
                ));
            }
        };

        Ok(ToolOutput {
            text,
            original_tokens: 0,
            saved_tokens: 0,
            mode: Some(action.clone()),
            path: None,
            changed: false,
            shell_outcome: None,
            content_blocks: None,
        })
    }
}

fn required_arg(args: &Map<String, Value>, key: &str) -> Result<String, ErrorData> {
    get_str(args, key)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| ErrorData::invalid_params(format!("{key} is required"), None))
}

fn analyze_solution_patterns(content: &str, path: &str) -> Vec<String> {
    let ext = path.rsplit('.').next().unwrap_or_default();
    let mut findings = Vec::new();
    let line_count = content.lines().count();

    let stdlib_indicators: &[&str] = match ext {
        "rs" => &["lazy_static", "once_cell"],
        "js" | "ts" | "jsx" | "tsx" => &["lodash", "underscore", "moment", "axios"],
        "py" => &["six", "pathlib2"],
        _ => &[],
    };
    let stdlib_hits = stdlib_indicators
        .iter()
        .filter(|ind| content.contains(**ind))
        .count();
    if stdlib_hits > 0 {
        findings.push(format!(
            "[stdlib] {stdlib_hits} dependency indicator(s) may have native alternatives"
        ));
    }

    if line_count > 500 {
        findings.push(format!(
            "[complexity] {line_count} lines — consider splitting"
        ));
    }

    let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
    if todo_count > 3 {
        findings.push(format!("[debt] {todo_count} TODO/FIXME markers"));
    }

    let empty_blocks = content.matches("{}").count();
    if empty_blocks > 2 {
        findings.push(format!("[yagni] {empty_blocks} empty blocks {{}}"));
    }

    if ext == "rs" {
        let unwrap_count = content.matches(".unwrap()").count();
        if unwrap_count > 5 {
            findings.push(format!(
                "[quality] {unwrap_count} .unwrap() calls — use ? operator"
            ));
        }
    }

    let mut seen_imports = Vec::new();
    let mut dup_count = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        let is_import = match ext {
            "rs" => trimmed.starts_with("use "),
            "js" | "ts" | "jsx" | "tsx" => trimmed.starts_with("import "),
            "py" => trimmed.starts_with("import ") || trimmed.starts_with("from "),
            _ => false,
        };
        if is_import {
            if seen_imports.contains(&trimmed) {
                dup_count += 1;
            } else {
                seen_imports.push(trimmed);
            }
        }
    }
    if dup_count > 0 {
        findings.push(format!("[reuse] {dup_count} duplicate import(s)"));
    }

    findings
}
