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
             review (analyze a file), suggest (simplify a snippet), fingerprint (predict solution rung), policy-check (validate team policy).",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "review|suggest|report|ladder|decide|fingerprint|policy-check"
                    },
                    "path": {
                        "type": "string",
                        "description": "File path (for review)"
                    },
                    "decision": {
                        "type": "string",
                        "description": "Decision text (for decide/fingerprint) or code snippet (for suggest)"
                    },
                    "category": {
                        "type": "string",
                    "description": "Decision category: stdlib|native|reuse|yagni|oneline|debt"
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
            "review" => {
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
                        out.push_str(&format!(
                            "- Line {} [{}] {}\n",
                            f.line, f.category, f.suggestion
                        ));
                    }
                }
                out.push_str(&format!(
                    "\nSession: {}\n",
                    crate::core::solution_tracker::gain_summary()
                ));
                out
            }
            "suggest" => {
                let decision = required_arg(args, "decision")?;
                let (rung, suggestion, loc_impact) = suggest_simplification(&decision);
                format!("{rung} — {suggestion} Estimated {loc_impact} LOC.")
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

struct ReviewFinding {
    line: usize,
    category: &'static str,
    suggestion: String,
}

fn analyze_solution_patterns(content: &str, path: &str) -> Vec<ReviewFinding> {
    let ext = path.rsplit('.').next().unwrap_or_default();
    let mut findings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let line_count = content.lines().count();

    let stdlib_indicators: &[(&str, &str)] = match ext {
        "rs" => &[
            (
                "lazy_static",
                "Prefer std::sync::LazyLock or OnceLock when possible.",
            ),
            (
                "once_cell",
                "Prefer std::sync::LazyLock or OnceLock when possible.",
            ),
        ],
        "js" | "ts" | "jsx" | "tsx" => &[
            (
                "lodash",
                "Use the matching built-in JavaScript method where possible.",
            ),
            (
                "underscore",
                "Use the matching built-in JavaScript method where possible.",
            ),
            (
                "moment",
                "Use Intl or Temporal-compatible APIs where possible.",
            ),
            (
                "axios",
                "Use fetch when its built-in API covers this use case.",
            ),
        ],
        "py" => &[
            ("six", "Use the Python 3 standard library directly."),
            (
                "pathlib2",
                "Use pathlib from the Python 3 standard library.",
            ),
        ],
        _ => &[],
    };
    for (indicator, suggestion) in stdlib_indicators {
        if let Some(line) = find_code_line(&lines, indicator) {
            push_finding(
                &mut findings,
                line,
                "stdlib",
                format!("`{indicator}` has a native alternative. {suggestion}"),
            );
        }
    }

    if line_count > 500 {
        push_finding(
            &mut findings,
            1,
            "yagni",
            format!(
                "{line_count} lines — split independent responsibilities before adding more abstraction."
            ),
        );
    }

    let todo_lines: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            (line.contains("TODO") || line.contains("FIXME")).then_some(index + 1)
        })
        .collect();
    if todo_lines.len() > 3 {
        push_finding(
            &mut findings,
            todo_lines[3],
            "yagni",
            format!(
                "{} TODO/FIXME markers — resolve or remove deferred paths.",
                todo_lines.len()
            ),
        );
    }

    if ext == "rs" {
        analyze_rust_patterns(&lines, &mut findings);
        let unwrap_lines: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| line.contains(".unwrap()").then_some(index + 1))
            .collect();
        if unwrap_lines.len() > 5 {
            push_finding(
                &mut findings,
                unwrap_lines[5],
                "oneline",
                format!(
                    "{} .unwrap() calls — propagate recoverable errors with `?`.",
                    unwrap_lines.len()
                ),
            );
        }
    }

    analyze_unreachable_patterns(&lines, &mut findings);

    let mut seen_imports = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_import = match ext {
            "rs" => trimmed.starts_with("use "),
            "js" | "ts" | "jsx" | "tsx" => trimmed.starts_with("import "),
            "py" => trimmed.starts_with("import ") || trimmed.starts_with("from "),
            _ => false,
        };
        if is_import {
            if seen_imports.contains(&trimmed) {
                push_finding(
                    &mut findings,
                    index + 1,
                    "reuse",
                    format!("Duplicate import `{trimmed}` — keep one shared import."),
                );
            } else {
                seen_imports.push(trimmed);
            }
        }
    }

    findings.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.category.cmp(right.category))
            .then(left.suggestion.cmp(&right.suggestion))
    });
    findings
}

fn push_finding(
    findings: &mut Vec<ReviewFinding>,
    line: usize,
    category: &'static str,
    suggestion: impl Into<String>,
) {
    let suggestion = suggestion.into();
    if !findings.iter().any(|finding| {
        finding.line == line && finding.category == category && finding.suggestion == suggestion
    }) {
        findings.push(ReviewFinding {
            line,
            category,
            suggestion,
        });
    }
}

fn find_code_line(lines: &[&str], needle: &str) -> Option<usize> {
    lines
        .iter()
        .position(|line| code_before_line_comment(line).contains(needle))
        .map(|index| index + 1)
}

fn code_before_line_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or_default().trim()
}

fn identifier_after_keyword(line: &str, keyword: &str) -> Option<String> {
    let code = code_before_line_comment(line);
    if code.is_empty() || code.starts_with('#') || code.starts_with("//") {
        return None;
    }

    let mut words = code.split_whitespace();
    while let Some(word) = words.next() {
        if word.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') == keyword {
            return words.next().and_then(identifier_from_start);
        }
    }
    None
}

fn identifier_from_start(value: &str) -> Option<String> {
    let identifier: String = value
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect();
    (!identifier.is_empty()).then_some(identifier)
}

fn analyze_rust_patterns(lines: &[&str], findings: &mut Vec<ReviewFinding>) {
    let traits: Vec<(usize, String)> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let code = code_before_line_comment(line);
            (!code.starts_with("impl ") && !code.starts_with("unsafe impl "))
                .then(|| identifier_after_keyword(line, "trait"))
                .flatten()
                .map(|name| (index + 1, name))
        })
        .collect();

    for (line, trait_name) in traits {
        let implementation_count = lines
            .iter()
            .filter(|candidate| rust_impls_trait(candidate, &trait_name))
            .count();
        if implementation_count == 1 {
            push_finding(
                findings,
                line,
                "yagni",
                format!(
                    "Trait `{trait_name}` has one implementation — use the concrete type until polymorphism is needed."
                ),
            );
        }
    }

    analyze_rust_factories(lines, findings);
    analyze_rust_stdlib_reimplementations(lines, findings);
    analyze_rust_wrappers(lines, findings);
}

fn rust_impls_trait(line: &str, trait_name: &str) -> bool {
    let code = code_before_line_comment(line);
    let Some(impl_position) = code.find("impl") else {
        return false;
    };
    let Some(for_position) = code.find(" for ") else {
        return false;
    };
    if for_position <= impl_position {
        return false;
    }

    let candidate = code[impl_position + "impl".len()..for_position]
        .split_whitespace()
        .last()
        .and_then(identifier_from_start);
    candidate.as_deref() == Some(trait_name)
}

fn analyze_rust_factories(lines: &[&str], findings: &mut Vec<ReviewFinding>) {
    let factories: Vec<(usize, String)> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            identifier_after_keyword(line, "struct")
                .filter(|name| name.ends_with("Factory"))
                .map(|name| (index + 1, name))
        })
        .collect();

    for (line, factory_name) in factories {
        let mut products = Vec::new();
        for (start, end) in rust_impl_blocks_for(lines, &factory_name) {
            for candidate in &lines[start..=end] {
                let code = code_before_line_comment(candidate);
                if let Some((method, product)) = rust_function_return(code) {
                    if is_factory_method(&method)
                        && product != "Self"
                        && !products.contains(&product)
                    {
                        products.push(product);
                    }
                }
            }
        }
        if products.len() == 1 {
            push_finding(
                findings,
                line,
                "yagni",
                format!(
                    "`{factory_name}` creates only `{}` — construct the product directly until multiple products exist.",
                    products[0]
                ),
            );
        }
    }
}

fn rust_impl_blocks_for(lines: &[&str], type_name: &str) -> Vec<(usize, usize)> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(start, line)| {
            let code = code_before_line_comment(line);
            let impl_position = code.find("impl")?;
            let header = code[..code.find('{')?].trim();
            if header.contains(" for ") {
                return None;
            }
            let implemented = header[impl_position + "impl".len()..]
                .split_whitespace()
                .last()
                .and_then(identifier_from_start)?;
            (implemented == type_name)
                .then(|| block_after_opening_line(lines, start))
                .flatten()
        })
        .collect()
}

fn block_after_opening_line(lines: &[&str], start: usize) -> Option<(usize, usize)> {
    let mut depth = 0_i32;
    let mut opened = false;
    for (index, line) in lines.iter().enumerate().skip(start) {
        let code = code_before_line_comment(line);
        let opens = code.matches('{').count() as i32;
        let closes = code.matches('}').count() as i32;
        if opens > 0 {
            opened = true;
        }
        depth += opens - closes;
        if opened && depth == 0 {
            return Some((start, index));
        }
    }
    None
}

fn rust_function_return(line: &str) -> Option<(String, String)> {
    let method = identifier_after_keyword(line, "fn")?;
    let (_, return_type) = line.split_once("->")?;
    let return_type = return_type.split('{').next().unwrap_or_default().trim();
    let return_type = return_type
        .split_once(" where ")
        .map_or(return_type, |(type_name, _)| type_name.trim());
    (!return_type.is_empty()).then(|| (method, return_type.to_string()))
}

fn is_factory_method(method: &str) -> bool {
    let method = method.to_ascii_lowercase();
    method.contains("create")
        || method.contains("build")
        || method.contains("make")
        || method.contains("produce")
}

fn analyze_rust_stdlib_reimplementations(lines: &[&str], findings: &mut Vec<ReviewFinding>) {
    for (index, line) in lines.iter().enumerate() {
        if let Some(name) = identifier_after_keyword(line, "struct") {
            let normalized = name.to_ascii_lowercase();
            if ["hashmap", "hash_map", "dictionary", "dict"]
                .iter()
                .any(|name| normalized.contains(name))
            {
                push_finding(
                    findings,
                    index + 1,
                    "stdlib",
                    format!(
                        "`{name}` looks like a custom map — prefer `std::collections::HashMap` unless it enforces domain-specific invariants."
                    ),
                );
            }
        }

        if let Some(name) = identifier_after_keyword(line, "fn") {
            if name.to_ascii_lowercase().contains("sort")
                && block_after_opening_line(lines, index)
                    .map(|(start, end)| {
                        lines[start..=end].iter().any(|candidate| {
                            let code = code_before_line_comment(candidate);
                            code.starts_with("for ") || code.starts_with("while ")
                        })
                    })
                    .unwrap_or(false)
            {
                push_finding(
                    findings,
                    index + 1,
                    "stdlib",
                    format!(
                        "`{name}` performs a manual sort — prefer `slice::sort_by` or `slice::sort_unstable_by`."
                    ),
                );
            }
        }
    }
}

fn analyze_rust_wrappers(lines: &[&str], findings: &mut Vec<ReviewFinding>) {
    for (index, line) in lines.iter().enumerate() {
        let Some(name) = identifier_after_keyword(line, "struct") else {
            continue;
        };
        if !is_single_field_struct(lines, index, &name) || !is_behaviorless_wrapper(lines, &name) {
            continue;
        }
        push_finding(
            findings,
            index + 1,
            "oneline",
            format!(
                "`{name}` wraps one value without observable behavior — use the inner type or a type alias unless the wrapper enforces an invariant."
            ),
        );
    }
}

fn is_single_field_struct(lines: &[&str], index: usize, name: &str) -> bool {
    let code = code_before_line_comment(lines[index]);
    let Some(name_position) = code.find(name) else {
        return false;
    };
    let remainder = code[name_position + name.len()..].trim_start();
    if let Some(tuple_fields) = remainder
        .strip_prefix('(')
        .and_then(|rest| rest.split(')').next())
    {
        return !tuple_fields.trim().is_empty() && !tuple_fields.contains(',');
    }
    if !remainder.contains('{') {
        return false;
    }
    let Some((start, end)) = block_after_opening_line(lines, index) else {
        return false;
    };
    let fields = lines[start..=end]
        .iter()
        .enumerate()
        .map(|(offset, line)| {
            let mut field_line = code_before_line_comment(line);
            if offset == 0 {
                field_line = field_line.split_once('{').map_or("", |(_, body)| body);
            }
            if offset + 1 == end - start + 1 {
                field_line = field_line.split('}').next().unwrap_or_default();
            }
            field_line
        })
        .collect::<Vec<_>>()
        .join("\n");
    let field_count = fields
        .split(',')
        .filter(|field| field.contains(':') && !field.contains("fn "))
        .count();
    field_count == 1
}

fn is_behaviorless_wrapper(lines: &[&str], name: &str) -> bool {
    let blocks = rust_impl_blocks_for(lines, name);
    if blocks.is_empty() {
        return true;
    }

    let implementation = blocks
        .iter()
        .flat_map(|(start, end)| lines[*start..=*end].iter())
        .map(|line| code_before_line_comment(line))
        .collect::<Vec<_>>()
        .join("\n");
    let method_count = implementation.matches("fn ").count();
    method_count == 0
        || (method_count == 1
            && !["if ", "match ", "for ", "while "]
                .iter()
                .any(|marker| implementation.contains(marker))
            && (implementation.contains("Self(")
                || implementation.contains("Self {")
                || implementation.contains("self.0")
                || implementation.contains("self.inner")))
}

fn analyze_unreachable_patterns(lines: &[&str], findings: &mut Vec<ReviewFinding>) {
    let mut depth = 0_i32;
    let mut terminated_at = None;

    for (index, line) in lines.iter().enumerate() {
        let code = code_before_line_comment(line);
        let line_number = index + 1;
        if code.contains("if false") || code.contains("if (false)") || code.contains("while false")
        {
            push_finding(
                findings,
                line_number,
                "yagni",
                "This condition is always false — remove the unreachable branch or make the condition runtime-driven.",
            );
        }
        if code.contains("#[cfg(any())]") {
            push_finding(
                findings,
                line_number,
                "yagni",
                "`#[cfg(any())]` permanently disables this path — remove it or use a real feature gate.",
            );
        }
        if code.contains("unreachable!()") {
            push_finding(
                findings,
                line_number,
                "yagni",
                "This path is declared unreachable — remove it or make the state model exhaustive.",
            );
        }

        if let Some(terminated_depth) = terminated_at {
            if depth < terminated_depth {
                terminated_at = None;
            } else if depth == terminated_depth
                && !code.is_empty()
                && !code.starts_with('}')
                && !code.starts_with("else")
            {
                push_finding(
                    findings,
                    line_number,
                    "yagni",
                    "This statement follows a terminating control-flow expression — remove it or restructure the branch.",
                );
                terminated_at = None;
            }
        }

        if is_terminating_statement(code) {
            terminated_at = Some(depth);
        }
        depth += code.matches('{').count() as i32 - code.matches('}').count() as i32;
    }
}

fn is_terminating_statement(code: &str) -> bool {
    code.starts_with("return")
        || code.starts_with("break")
        || code.starts_with("continue")
        || code.contains("panic!(")
        || code.contains("unreachable!(")
        || code.contains("unimplemented!(")
}

fn suggest_simplification(snippet: &str) -> (&'static str, &'static str, i32) {
    let snippet = snippet.to_ascii_lowercase();

    if snippet.contains("sort") {
        ("stdlib", "Use Vec::sort() or Vec::sort_by().", -15)
    } else if snippet.contains("hash") {
        ("stdlib", "Use std::collections::HashMap or HashSet.", -10)
    } else if snippet.contains("format") {
        ("stdlib", "Use format!() or std::fmt.", -8)
    } else if snippet.contains("parse") {
        (
            "stdlib",
            "Use str::parse() or a FromStr implementation.",
            -10,
        )
    } else if snippet.contains("validate") {
        (
            "reuse",
            "Use an existing validation helper or a small predicate.",
            -10,
        )
    } else if snippet.contains("config") {
        (
            "yagni",
            "Use a plain configuration struct with defaults; remove unused indirection.",
            -12,
        )
    } else if snippet.contains("factory") {
        (
            "yagni",
            "Directly construct the value instead of maintaining a factory.",
            -12,
        )
    } else if snippet.contains("singleton") {
        (
            "stdlib",
            "Use std::sync::OnceLock or LazyLock instead of a custom singleton.",
            -20,
        )
    } else if snippet.contains("wrapper") {
        (
            "yagni",
            "Call the underlying API directly; remove the pass-through wrapper.",
            -8,
        )
    } else {
        (
            "minimum",
            "Keep the smallest clear implementation; no standard simplification pattern matched.",
            0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::suggest_simplification;

    #[test]
    fn suggest_sort_uses_stdlib() {
        assert_eq!(
            suggest_simplification("Custom sorting function that reimplements bubble sort"),
            ("stdlib", "Use Vec::sort() or Vec::sort_by().", -15)
        );
    }

    #[test]
    fn suggest_covers_supported_keywords() {
        for (keyword, expected_rung) in [
            ("hash", "stdlib"),
            ("format", "stdlib"),
            ("parse", "stdlib"),
            ("validate", "reuse"),
            ("config", "yagni"),
            ("factory", "yagni"),
            ("singleton", "stdlib"),
            ("wrapper", "yagni"),
        ] {
            assert_eq!(suggest_simplification(keyword).0, expected_rung);
        }
    }
}
