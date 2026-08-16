use std::fs;

use rmcp::ErrorData;
use rmcp::model::Tool;
use serde_json::{Map, Value, json};

use crate::core::knowledge::ProjectKnowledge;
use crate::server::tool_trait::{McpTool, ToolContext, ToolOutput, get_str};
use crate::tool_defs::tool_def;
use crate::tools::ctx_optimize::{OptimizeReport, analyze_file, format_report_text};

pub struct CtxOptimizeTool;

impl McpTool for CtxOptimizeTool {
    fn name(&self) -> &'static str {
        "ctx_optimize"
    }

    fn tool_def(&self) -> Tool {
        tool_def(
            "ctx_optimize",
            "Review code for over-engineering and propose simplifications.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["review", "suggest", "report", "ladder", "resolve"],
                        "description": "Requested Solution Intelligence action."
                    },
                    "decision": {
                        "type": "string",
                        "description": "Resolved finding or chosen implementation; required for action=resolve"
                    },
                    "category": {
                        "type": "string",
                        "enum": ["stdlib", "native", "reuse", "yagni", "one-line", "debt"],
                        "description": "Resolved finding category: stdlib, native, reuse, yagni, one-line, or debt"
                    },
                    "path": {
                        "type": "string",
                        "description": "Project-relative file path for review or suggestion."
                    },
                    "scope": {
                        "type": "string",
                        "enum": ["diff", "file", "symbol"],
                        "default": "file",
                        "description": "Target review scope."
                    },
                    "symbol": {
                        "type": "string",
                        "description": "Optional symbol name when scope is symbol."
                    },
                    "format": {
                        "type": "string",
                        "enum": ["text", "json"],
                        "default": "text",
                        "description": "Response format."
                    }
                },
                "required": ["action"],
                "allOf": [
                    {
                        "if": {
                            "properties": {
                                "action": {"enum": ["review", "suggest"]}
                            }
                        },
                        "then": {"required": ["path"]}
                    },
                    {
                        "if": {
                            "properties": {
                                "action": {"const": "resolve"}
                            }
                        },
                        "then": {"required": ["decision", "category"]}
                    },
                    {
                        "if": {
                            "properties": {
                                "scope": {"const": "symbol"}
                            }
                        },
                        "then": {"required": ["symbol"]}
                    }
                ]
            }),
        )
    }

    fn handle(
        &self,
        args: &Map<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, ErrorData> {
        let action = required_enum(
            args,
            "action",
            &["review", "suggest", "report", "ladder", "resolve"],
            None,
        )?;
        let format = required_enum(args, "format", &["text", "json"], Some("text"))?;
        let scope = required_enum(args, "scope", &["diff", "file", "symbol"], Some("file"))?;
        let symbol = get_str(args, "symbol");
        if scope == "symbol" && symbol.as_deref().is_none_or(str::is_empty) {
            return Err(ErrorData::invalid_params(
                "symbol is required when scope is symbol".to_string(),
                None,
            ));
        }

        let (text, path) = match action.as_str() {
            "resolve" => {
                let decision = args
                    .get("decision")
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| {
                        ErrorData::invalid_params("decision is required for resolve", None)
                    })?;
                let category = required_enum(
                    args,
                    "category",
                    &["stdlib", "native", "reuse", "yagni", "one-line", "debt"],
                    None,
                )?;
                if let Some(project_root) = crate::server::derive_project_root_from_cwd() {
                    crate::core::solution_auto_capture::capture_optimize_resolution(
                        &project_root,
                        &category,
                        decision,
                    )
                    .map_err(|error| ErrorData::invalid_params(error, None))?;
                }
                (format!("Resolved {category} finding: {decision}"), None)
            }
            "review" | "suggest" => {
                let (content, requested_path, resolved_path) = read_target(args, ctx)?;
                let mut report = analyze_file(
                    &content,
                    &requested_path,
                    std::path::Path::new(&ctx.project_root),
                );
                if scope == "symbol" {
                    limit_to_symbol(&mut report, &content, symbol.as_deref().unwrap_or_default())?;
                }
                let text = if action == "review" {
                    format_review(&report, &requested_path, &scope, symbol.as_deref(), &format)
                } else {
                    format_suggestions(&report, &requested_path, &scope, symbol.as_deref(), &format)
                };
                let _ = resolved_path;
                (text, Some(requested_path))
            }
            "report" => (format_session_report(ctx, &format), None),
            "ladder" => (format_decision_ladder(ctx, &format), None),
            _ => unreachable!("action is constrained by required_enum"),
        };

        let mut output = ToolOutput::simple(text);
        output.mode = Some(action);
        output.path = path;
        Ok(output)
    }
}

fn required_enum(
    args: &Map<String, Value>,
    key: &str,
    allowed: &[&str],
    default: Option<&str>,
) -> Result<String, ErrorData> {
    let value = get_str(args, key)
        .or_else(|| default.map(str::to_string))
        .ok_or_else(|| ErrorData::invalid_params(format!("{key} is required"), None))?;
    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(ErrorData::invalid_params(
            format!("{key} must be one of: {}", allowed.join(", ")),
            None,
        ))
    }
}

fn read_target(
    args: &Map<String, Value>,
    ctx: &ToolContext,
) -> Result<(String, String, String), ErrorData> {
    let requested_path = get_str(args, "path")
        .filter(|path| !path.is_empty())
        .ok_or_else(|| ErrorData::invalid_params("path is required".to_string(), None))?;
    let resolved_path = ctx
        .resolved_path("path")
        .map(str::to_string)
        .map(Ok)
        .unwrap_or_else(|| ctx.resolve_path_sync(&requested_path))
        .map_err(|error| ErrorData::invalid_params(format!("path: {error}"), None))?;
    let content = fs::read_to_string(&resolved_path)
        .map_err(|error| ErrorData::invalid_params(format!("read: {error}"), None))?;
    Ok((content, requested_path, resolved_path))
}

fn limit_to_symbol(
    report: &mut OptimizeReport,
    content: &str,
    symbol: &str,
) -> Result<(), ErrorData> {
    let Some(symbol_line) = content
        .lines()
        .position(|line| line.contains(symbol))
        .map(|line| line + 1)
    else {
        return Err(ErrorData::invalid_params(
            format!("symbol '{symbol}' was not found in the target file"),
            None,
        ));
    };
    report
        .findings
        .retain(|finding| finding.line == symbol_line);
    report.summary.total_findings = report.findings.len();
    report.summary.estimated_loc_reduction = report
        .findings
        .iter()
        .map(|finding| finding.loc_impact)
        .sum();
    report.summary.by_category.clear();
    for finding in &report.findings {
        *report
            .summary
            .by_category
            .entry(finding.category.clone())
            .or_insert(0) += 1;
    }
    Ok(())
}

fn target_json(path: &str, scope: &str, symbol: Option<&str>) -> Value {
    json!({
        "path": path,
        "scope": scope,
        "symbol": symbol,
    })
}

fn format_review(
    report: &OptimizeReport,
    path: &str,
    scope: &str,
    symbol: Option<&str>,
    format: &str,
) -> String {
    if format == "json" {
        json!({
            "schema_version": "1.0",
            "tool": "ctx_optimize",
            "target": target_json(path, scope, symbol),
            "findings": report.findings,
            "summary": report.summary,
        })
        .to_string()
    } else {
        format_report_text(report, path)
    }
}

fn format_suggestions(
    report: &OptimizeReport,
    path: &str,
    scope: &str,
    symbol: Option<&str>,
    format: &str,
) -> String {
    if format == "json" {
        let suggestions: Vec<_> = report
            .findings
            .iter()
            .map(|finding| {
                json!({
                    "line": finding.line,
                    "category": finding.category,
                    "recommendation": finding.recommended,
                    "loc_impact": finding.loc_impact,
                })
            })
            .collect();
        return json!({
            "schema_version": "1.0",
            "tool": "ctx_optimize",
            "target": target_json(path, scope, symbol),
            "suggestions": suggestions,
            "summary": report.summary,
        })
        .to_string();
    }

    let mut out = format!("Solution Intelligence suggestions: {path}\n");
    for finding in &report.findings {
        out.push_str(&format!(
            "\nLine {} [{}]: {}\n  Estimated impact: -{} LOC\n",
            finding.line, finding.category, finding.recommended, finding.loc_impact
        ));
    }
    out.push_str(&format!(
        "\nSummary: {} suggestion(s); estimated reduction: {} LOC\n",
        report.summary.total_findings, report.summary.estimated_loc_reduction
    ));
    out
}

fn format_session_report(ctx: &ToolContext, format: &str) -> String {
    let (decisions, debt) = session_data(ctx);
    let knowledge = ProjectKnowledge::load(&ctx.project_root);
    let facts = knowledge.as_ref().map_or(0, |entry| entry.facts.len());
    let patterns = knowledge.as_ref().map_or(0, |entry| entry.patterns.len());
    let history = knowledge.as_ref().map_or(0, |entry| entry.history.len());

    if format == "json" {
        return json!({
            "schema_version": "1.0",
            "tool": "ctx_optimize",
            "report": {
                "decisions": decisions,
                "savings": {
                    "knowledge_facts": facts,
                    "knowledge_patterns": patterns,
                    "consolidated_insights": history,
                },
                "debt": debt,
            }
        })
        .to_string();
    }

    let mut out = format!(
        "Solution Intelligence session report\n\nDecisions: {}\n",
        decisions.len()
    );
    for decision in &decisions {
        out.push_str(&format!("- {decision}\n"));
    }
    out.push_str(&format!(
        "\nSavings: {facts} knowledge fact(s), {patterns} pattern(s), {history} consolidated insight(s)\n"
    ));
    out.push_str(&format!("Debt: {} recorded finding(s)\n", debt.len()));
    for finding in &debt {
        out.push_str(&format!("- {finding}\n"));
    }
    out
}

fn format_decision_ladder(ctx: &ToolContext, format: &str) -> String {
    let (decisions, _) = session_data(ctx);
    if format == "json" {
        return json!({
            "schema_version": "1.0",
            "tool": "ctx_optimize",
            "ladder": decisions,
        })
        .to_string();
    }

    let mut out = "Current decision ladder\n".to_string();
    if decisions.is_empty() {
        out.push_str("\nNo recorded decisions.\n");
    } else {
        for (index, decision) in decisions.iter().enumerate() {
            out.push_str(&format!("{}. {decision}\n", index + 1));
        }
    }
    out
}

fn session_data(ctx: &ToolContext) -> (Vec<String>, Vec<String>) {
    let Some(session) = ctx.session.as_ref() else {
        return (Vec::new(), Vec::new());
    };
    let Ok(session) = session.try_read() else {
        return (Vec::new(), Vec::new());
    };
    let decisions = session
        .decisions
        .iter()
        .map(|decision| match &decision.rationale {
            Some(rationale) => format!("{} — {}", decision.summary, rationale),
            None => decision.summary.clone(),
        })
        .collect();
    let debt = session
        .findings
        .iter()
        .map(|finding| match (&finding.file, finding.line) {
            (Some(file), Some(line)) => format!("{file}:{line} — {}", finding.summary),
            (Some(file), None) => format!("{file} — {}", finding.summary),
            _ => finding.summary.clone(),
        })
        .collect();
    (decisions, debt)
}
