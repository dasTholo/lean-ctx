//! Lightweight Solution Intelligence checks used by `ctx_optimize`.
//!
//! The analyzer intentionally favours explainable regex-based heuristics over a
//! complete parser. Findings are suggestions, not compiler diagnostics.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// Keep regexes compiled once, matching the static-regex convention used by the
// rest of the crate's lightweight analyzers.
macro_rules! static_regex {
    ($pattern:expr_2021) => {{
        static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex::Regex::new($pattern).expect("valid static regex"))
    }};
}

/// One simplification opportunity identified in a source file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizeFinding {
    pub line: usize,
    pub category: String,
    pub severity: String,
    pub current: String,
    pub recommended: String,
    pub loc_impact: i32,
}

/// Complete analysis result for one source file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizeReport {
    pub findings: Vec<OptimizeFinding>,
    pub summary: OptimizeSummary,
}

/// Aggregate counts for an [`OptimizeReport`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizeSummary {
    pub total_findings: usize,
    pub estimated_loc_reduction: i32,
    pub by_category: HashMap<String, usize>,
}

/// Analyze `content` for simple, high-signal simplification opportunities.
pub fn analyze_file(content: &str, path: &str, project_root: &Path) -> OptimizeReport {
    let mut findings = Vec::new();
    findings.extend(detect_single_impl_traits(content));
    findings.extend(detect_single_product_factories(content));
    findings.extend(detect_stdlib_alternatives(content));
    findings.extend(detect_native_feature_replacements(content, project_root));
    findings.extend(detect_yagni(content));
    findings.extend(detect_oneline_candidates(content));
    findings.extend(detect_reuse_candidates(content, path, project_root));

    findings.sort_by(|left, right| {
        finding_score(right)
            .cmp(&finding_score(left))
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.category.cmp(&right.category))
    });
    findings.dedup_by(|left, right| {
        left.line == right.line
            && left.category == right.category
            && left.current == right.current
            && left.recommended == right.recommended
    });

    let mut by_category = HashMap::new();
    for finding in &findings {
        *by_category.entry(finding.category.clone()).or_insert(0) += 1;
    }

    OptimizeReport {
        summary: OptimizeSummary {
            total_findings: findings.len(),
            estimated_loc_reduction: findings.iter().map(|finding| finding.loc_impact).sum(),
            by_category,
        },
        findings,
    }
}

/// Detect Rust traits which currently exist only to abstract one implementation.
pub fn detect_single_impl_traits(content: &str) -> Vec<OptimizeFinding> {
    let trait_declarations =
        static_regex!(r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)\b");
    let trait_implementations = static_regex!(
        r"(?m)^[ \t]*impl(?:\s*<[^>\n]*>)?\s+([A-Za-z_][A-Za-z0-9_:]*)\s*(?:<[^>{}\n]*>)?\s+for\s+"
    );

    let implementations: HashMap<String, usize> = trait_implementations
        .captures_iter(content)
        .filter_map(|captures| {
            captures
                .get(1)
                .map(|name| last_path_segment(name.as_str()).to_string())
        })
        .fold(HashMap::new(), |mut counts, trait_name| {
            *counts.entry(trait_name).or_insert(0) += 1;
            counts
        });

    trait_declarations
        .captures_iter(content)
        .filter_map(|captures| {
            let trait_name = captures.get(1)?.as_str();
            (implementations.get(trait_name) == Some(&1)).then(|| {
                let start = captures.get(0).map_or(0, |m| m.start());
                OptimizeFinding {
                    line: line_number_at(content, start),
                    category: "single-impl-trait".into(),
                    severity: "medium".into(),
                    current: format!("Trait `{trait_name}` has exactly one implementation."),
                    recommended: format!(
                        "Use its concrete type directly; restore `{trait_name}` when a second implementation is needed."
                    ),
                    loc_impact: 4,
                }
            })
        })
        .collect()
}

/// Detect code which likely duplicates a standard-library facility.
pub fn detect_stdlib_alternatives(content: &str) -> Vec<OptimizeFinding> {
    let custom_map = static_regex!(
        r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*(?:HashMap|Map|Dictionary|Lookup)[A-Za-z0-9_]*)\b"
    );
    let string_builder = static_regex!(r"\bString::new\s*\(\s*\)");
    let push_str = static_regex!(r"\.[ \t]*push_str\s*\(");
    let manual_sort =
        static_regex!(r"(?m)^[ \t]*(?:for|while)\b[\s\S]{0,800}?(?:swap\s*\(|\.swap\s*\()");
    let mut findings = Vec::new();

    for captures in custom_map.captures_iter(content) {
        let Some(name) = captures.get(1) else {
            continue;
        };
        let Some(full_match) = captures.get(0) else {
            continue;
        };
        push_unique(
            &mut findings,
            OptimizeFinding {
                line: line_number_at(content, full_match.start()),
                category: "stdlib-alternative".into(),
                severity: "medium".into(),
                current: format!("Custom collection `{}` resembles a map or lookup table.", name.as_str()),
                recommended: "Prefer `std::collections::HashMap` unless this type enforces a domain invariant.".to_string(),
                loc_impact: 8,
            },
        );
    }

    if string_builder.is_match(content) && push_str.find_iter(content).count() >= 2 {
        let start = string_builder
            .find(content)
            .map_or(0, |matched| matched.start());
        push_unique(
            &mut findings,
            OptimizeFinding {
                line: line_number_at(content, start),
                category: "stdlib-alternative".into(),
                severity: "low".into(),
                current: "A `String` is assembled with repeated `push_str` calls.".to_string(),
                recommended: "Use `join`, `format!`, or an iterator collected into `String` when it makes the construction clearer.".to_string(),
                loc_impact: 3,
            },
        );
    }

    if let Some(matched) = manual_sort.find(content) {
        push_unique(
            &mut findings,
            OptimizeFinding {
                line: line_number_at(content, matched.start()),
                category: "stdlib-alternative".into(),
                severity: "medium".into(),
                current: "A loop performs a manual element swap while sorting.".to_string(),
                recommended: "Use `slice::sort_by` or `slice::sort_unstable_by` instead of maintaining a sorting loop.".to_string(),
                loc_impact: 6,
            },
        );
    }

    findings
}

/// Detect abstractions whose present form is unlikely to justify their cost.
pub fn detect_yagni(content: &str) -> Vec<OptimizeFinding> {
    let empty_trait = static_regex!(
        r"(?ms)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{\s*\}"
    );
    let private_abstraction =
        static_regex!(r"(?m)^[ \t]*(?:struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b");
    let function_parameters = static_regex!(
        r"(?ms)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)[^\n{]*\(([^)]*)\)"
    );
    let mut findings = Vec::new();
    let mut empty_trait_names = HashSet::new();

    for captures in empty_trait.captures_iter(content) {
        let (Some(name), Some(full_match)) = (captures.get(1), captures.get(0)) else {
            continue;
        };
        empty_trait_names.insert(name.as_str().to_string());
        push_unique(
            &mut findings,
            OptimizeFinding {
                line: line_number_at(content, full_match.start()),
                category: "yagni".into(),
                severity: "low".into(),
                current: format!(
                    "Marker trait `{}` has no behaviour or bounds.",
                    name.as_str()
                ),
                recommended: "Remove it until it expresses a real capability or constraint."
                    .to_string(),
                loc_impact: 2,
            },
        );
    }

    for captures in private_abstraction.captures_iter(content) {
        let (Some(name), Some(full_match)) = (captures.get(1), captures.get(0)) else {
            continue;
        };
        if empty_trait_names.contains(name.as_str())
            || count_identifier_occurrences(content, name.as_str()) > 1
        {
            continue;
        }
        push_unique(
            &mut findings,
            OptimizeFinding {
                line: line_number_at(content, full_match.start()),
                category: "yagni".into(),
                severity: "low".into(),
                current: format!(
                    "Private abstraction `{}` is never referenced after its declaration.",
                    name.as_str()
                ),
                recommended:
                    "Remove the unused abstraction, or add it only with its first concrete use."
                        .to_string(),
                loc_impact: 2,
            },
        );
    }

    for captures in function_parameters.captures_iter(content) {
        let (Some(name), Some(parameters), Some(full_match)) =
            (captures.get(1), captures.get(2), captures.get(0))
        else {
            continue;
        };
        let parameter_count = parameters
            .as_str()
            .split(',')
            .map(str::trim)
            .filter(|parameter| !parameter.is_empty())
            .count();
        if parameter_count < 6 {
            continue;
        }
        push_unique(
            &mut findings,
            OptimizeFinding {
                line: line_number_at(content, full_match.start()),
                category: "yagni".into(),
                severity: "low".into(),
                current: format!(
                    "Function `{}` accepts {parameter_count} parameters.",
                    name.as_str()
                ),
                recommended:
                    "Group parameters that change together into a focused options or domain type."
                        .to_string(),
                loc_impact: 3,
            },
        );
    }

    findings
}

/// Detect multi-line Rust functions whose body is already a single expression.
pub fn detect_oneline_candidates(content: &str) -> Vec<OptimizeFinding> {
    let single_expression_function = static_regex!(
        r"(?ms)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)(?:<[^>{}]*>)?\s*\([^{};]*\)\s*(?:->\s*[^\n{]+)?\{\s*\n\s*([^{}\n]+?)\s*\n\s*\}"
    );

    single_expression_function
        .captures_iter(content)
        .filter_map(|captures| {
            let (Some(name), Some(body), Some(full_match)) =
                (captures.get(1), captures.get(2), captures.get(0))
            else {
                return None;
            };
            let expression = body.as_str().trim();
            let is_expression = !expression.ends_with(';')
                && !expression.starts_with("let ")
                && !expression.starts_with("return ")
                && !expression.starts_with("if ")
                && !expression.starts_with("match ")
                && !expression.starts_with("//");
            is_expression.then(|| OptimizeFinding {
                line: line_number_at(content, full_match.start()),
                category: "oneline-candidate".into(),
                severity: "low".into(),
                current: format!(
                    "Function `{}` uses a multi-line single-expression body.",
                    name.as_str()
                ),
                recommended: format!(
                    "Collapse `{}` to a one-line expression body if that matches nearby style.",
                    name.as_str()
                ),
                loc_impact: 2,
            })
        })
        .collect()
}

/// Detect repeated three-line code sequences in the current file or nearby project sources.
pub fn detect_reuse_candidates(
    content: &str,
    path: &str,
    project_root: &Path,
) -> Vec<OptimizeFinding> {
    let current_windows = code_windows(content);
    let mut findings = Vec::new();
    let mut first_occurrence = HashMap::new();
    let mut reported = HashSet::new();

    for (line, window) in &current_windows {
        if let Some(first_line) = first_occurrence.get(window) {
            if *line >= *first_line + 3 && reported.insert(window.clone()) {
                push_unique(
                    &mut findings,
                    OptimizeFinding {
                        line: *line,
                        category: "reuse-candidate".into(),
                        severity: "low".into(),
                        current: format!("Three-line sequence duplicates code beginning at line {first_line}."),
                        recommended: "Extract the shared sequence into a small helper with a domain-focused name.".to_string(),
                        loc_impact: 3,
                    },
                );
            }
        } else {
            first_occurrence.insert(window.clone(), *line);
        }
    }

    if first_occurrence.is_empty() {
        return findings;
    }

    let target_path = normalized_target_path(path, project_root);
    for source_path in project_source_files(project_root, &target_path) {
        let Ok(other_content) = fs::read_to_string(&source_path) else {
            continue;
        };
        for (line, window) in code_windows(&other_content) {
            let Some(current_line) = first_occurrence.get(&window) else {
                continue;
            };
            if !reported.insert(window) {
                continue;
            }
            let source_label = source_path
                .strip_prefix(project_root)
                .unwrap_or(&source_path)
                .display();
            push_unique(
                &mut findings,
                OptimizeFinding {
                    line: *current_line,
                    category: "reuse-candidate".into(),
                    severity: "low".into(),
                    current: format!(
                        "Three-line sequence is also present near line {line} in `{source_label}`."
                    ),
                    recommended: "Extract a shared helper only if both call sites have the same domain meaning.".to_string(),
                    loc_impact: 3,
                },
            );
        }
    }

    findings
}

/// Render a human-readable optimization report.
pub fn detect_single_product_factories(content: &str) -> Vec<OptimizeFinding> {
    let factory = static_regex!(
        r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum)\s+([A-Za-z_][A-Za-z0-9_]*(?:Factory|Builder|Creator))\b"
    );
    let return_type = static_regex!(
        r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+[A-Za-z_][A-Za-z0-9_]*[^\n{]*->\s*([A-Za-z_][A-Za-z0-9_:]*)"
    );

    factory
        .captures_iter(content)
        .filter_map(|captures| {
            let name = captures.get(1)?.as_str();
            let declaration = captures.get(0)?;
            let impl_start = content.find(&format!("impl {name}"))?;
            let impl_end = content[impl_start + 1..]
                .find("\nimpl ")
                .map_or(content.len(), |offset| impl_start + 1 + offset);
            let products: HashSet<_> = return_type
                .captures_iter(&content[impl_start..impl_end])
                .filter_map(|return_capture| return_capture.get(1))
                .map(|product| product.as_str())
                .filter(|product| !matches!(*product, "Self" | "Result" | "Option"))
                .collect();

            (products.len() == 1).then(|| {
                let product = products.into_iter().next().unwrap_or_default();
                OptimizeFinding {
                    line: line_number_at(content, declaration.start()),
                    category: "single-product-factory".to_string(),
                    severity: "low".to_string(),
                    current: format!("'{name}' only constructs '{product}'."),
                    recommended: format!(
                        "Construct '{product}' directly; retain '{name}' when it supports multiple products."
                    ),
                    loc_impact: 4,
                }
            })
        })
        .collect()
}

pub fn detect_native_feature_replacements(
    content: &str,
    project_root: &Path,
) -> Vec<OptimizeFinding> {
    let Ok(manifest) = fs::read_to_string(project_root.join("Cargo.toml")) else {
        return Vec::new();
    };
    let replacements = [
        (
            "once_cell",
            "once_cell::",
            "std::sync::OnceLock or std::sync::LazyLock",
        ),
        ("lazy_static", "lazy_static!", "std::sync::LazyLock"),
    ];

    replacements
        .iter()
        .filter_map(|(dependency, usage, native)| {
            let declared = manifest.lines().any(|line| {
                let line = line.trim_start();
                line.starts_with(dependency)
                    && matches!(
                        line.as_bytes().get(dependency.len()),
                        Some(b' ' | b'\t' | b'=')
                    )
            });
            let offset = content.find(usage)?;
            declared.then(|| OptimizeFinding {
                line: line_number_at(content, offset),
                category: "native-feature-replacement".to_string(),
                severity: "medium".to_string(),
                current: format!("'{dependency}' is declared and '{usage}' is used in this file."),
                recommended: format!(
                    "Use '{native}' when its standard-library behavior covers this use case."
                ),
                loc_impact: 1,
            })
        })
        .collect()
}

fn finding_score(finding: &OptimizeFinding) -> i32 {
    let severity = match finding.severity.as_str() {
        "critical" => 400,
        "high" => 300,
        "medium" => 200,
        _ => 100,
    };
    severity + finding.loc_impact.max(0)
}

pub fn format_report_text(report: &OptimizeReport, path: &str) -> String {
    let mut out = format!(
        "Solution Intelligence: {path}\n\n{} finding(s); estimated reduction: {} LOC\n",
        report.summary.total_findings, report.summary.estimated_loc_reduction
    );

    if report.findings.is_empty() {
        out.push_str("\nNo simplification opportunities detected.\n");
        return out;
    }

    for finding in &report.findings {
        out.push_str(&format!(
            "\nLine {} [{}] {}\n  Current: {}\n  Recommended: {}\n  Estimated impact: -{} LOC\n",
            finding.line,
            finding.severity,
            finding.category,
            finding.current,
            finding.recommended,
            finding.loc_impact
        ));
    }

    let mut categories: Vec<_> = report.summary.by_category.iter().collect();
    categories.sort_by(|left, right| left.0.cmp(right.0));
    out.push_str("\nBy category:\n");
    for (category, count) in categories {
        out.push_str(&format!("  {category}: {count}\n"));
    }
    out
}

/// Render a machine-readable optimization report.
pub fn format_report_json(report: &OptimizeReport, path: &str) -> Value {
    json!({
        "path": path,
        "findings": &report.findings,
        "summary": &report.summary,
    })
}

fn push_unique(findings: &mut Vec<OptimizeFinding>, candidate: OptimizeFinding) {
    if !findings.iter().any(|finding| {
        finding.line == candidate.line
            && finding.category == candidate.category
            && finding.current == candidate.current
    }) {
        findings.push(candidate);
    }
}

fn line_number_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn last_path_segment(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

fn count_identifier_occurrences(content: &str, identifier: &str) -> usize {
    let mut count = 0;
    let mut offset = 0;
    while let Some(found_at) = content[offset..].find(identifier) {
        let start = offset + found_at;
        let end = start + identifier.len();
        let before_is_ident = content[..start]
            .chars()
            .next_back()
            .is_some_and(is_identifier_character);
        let after_is_ident = content[end..]
            .chars()
            .next()
            .is_some_and(is_identifier_character);
        if !before_is_ident && !after_is_ident {
            count += 1;
        }
        offset = end;
    }
    count
}

fn is_identifier_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn code_windows(content: &str) -> Vec<(usize, String)> {
    let comment_or_blank = static_regex!(r"^\s*(?://|#|/\*|\*|\*/|$)");
    let whitespace = static_regex!(r"\s+");
    let lines: Vec<_> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| !comment_or_blank.is_match(line))
        .map(|(index, line)| {
            (
                index + 1,
                whitespace.replace_all(line.trim(), " ").into_owned(),
            )
        })
        .collect();

    lines
        .windows(3)
        .filter_map(|window| {
            let text = format!("{}\n{}\n{}", window[0].1, window[1].1, window[2].1);
            if text.len() >= 30 {
                Some((window[0].0, text))
            } else {
                None
            }
        })
        .collect()
}

fn normalized_target_path(path: &str, project_root: &Path) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn project_source_files(project_root: &Path, target_path: &Path) -> Vec<PathBuf> {
    const MAX_FILES: usize = 256;
    const MAX_FILE_BYTES: u64 = 1_048_576;
    let mut sources = Vec::new();
    let mut directories = vec![project_root.to_path_buf()];

    while let Some(directory) = directories.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            if sources.len() >= MAX_FILES {
                return sources;
            }
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if !should_skip_directory(&path) {
                    directories.push(path);
                }
                continue;
            }
            if !file_type.is_file() || path == target_path || !is_source_file(&path) {
                continue;
            }
            if entry
                .metadata()
                .map_or(true, |metadata| metadata.len() > MAX_FILE_BYTES)
            {
                continue;
            }
            sources.push(path);
        }
    }
    sources.sort();
    sources
}

fn should_skip_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "target" | "node_modules" | "vendor" | ".venv" | "dist" | "build")
    )
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some(
            "rs" | "c"
                | "cc"
                | "cpp"
                | "h"
                | "hpp"
                | "go"
                | "java"
                | "js"
                | "jsx"
                | "kt"
                | "py"
                | "rb"
                | "swift"
                | "ts"
                | "tsx"
        )
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        analyze_file, detect_oneline_candidates, detect_single_impl_traits,
        detect_stdlib_alternatives, detect_yagni, format_report_json, format_report_text,
    };

    #[test]
    fn finds_trait_with_one_implementation() {
        let content = "trait Store { fn get(&self); }\nstruct Memory;\nimpl Store for Memory { fn get(&self) {} }\n";
        let findings = detect_single_impl_traits(content);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "single-impl-trait");
        assert_eq!(findings[0].line, 1);
    }

    #[test]
    fn finds_standard_library_alternatives() {
        let content = "struct UserMap { entries: Vec<(String, String)> }\n\nfn render() {\n    let mut out = String::new();\n    out.push_str(\"a\");\n    out.push_str(\"b\");\n}\n";
        let findings = detect_stdlib_alternatives(content);

        assert!(
            findings
                .iter()
                .any(|finding| finding.current.contains("UserMap"))
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.current.contains("push_str"))
        );
    }

    #[test]
    fn finds_yagni_and_one_line_candidates() {
        let content = "trait Marker {}\n\nfn options(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) {}\n\nfn answer() -> u8 {\n    42\n}\n";

        assert_eq!(detect_yagni(content).len(), 2);
        assert_eq!(detect_oneline_candidates(content).len(), 1);
    }

    #[test]
    fn report_formats_text_and_json() {
        let content = "trait Store { fn get(&self); }\nstruct Memory;\nimpl Store for Memory { fn get(&self) {} }\n";
        let report = analyze_file(content, "src/store.rs", Path::new("."));

        assert!(format_report_text(&report, "src/store.rs").contains("Solution Intelligence"));
        let json = format_report_json(&report, "src/store.rs");
        assert_eq!(json["path"], "src/store.rs");
        assert_eq!(json["summary"]["total_findings"], 1);
    }
}
