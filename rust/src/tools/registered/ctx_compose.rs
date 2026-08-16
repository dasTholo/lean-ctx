use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use crate::core::ocla::cache_types::{CacheKeyBuilder, ComposedContextKey};
use rmcp::ErrorData;
use rmcp::model::Tool;
use serde_json::{Map, Value, json};

use crate::server::tool_trait::{McpTool, ToolContext, ToolOutput, get_bool, get_str};
use crate::tool_defs::tool_def;

pub struct CtxComposeTool;

const MIN_TASK_AWARE_COVERAGE: f32 = 0.20;
const TASK_AWARE_TOP_K: usize = 2;
const TASK_AWARE_DIVERSE_SECTIONS: usize = 1;
const PROFILE_KEYWORD_WEIGHT: f32 = 0.45;
const STOP_WORD_WEIGHT: f32 = 0.20;
const QUERY_STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "how", "in", "is", "it", "of",
    "on", "or", "that", "the", "this", "to", "use", "what", "with",
];

#[derive(Debug, Clone)]
struct QueryKeyword {
    term: String,
    weight: f32,
}

#[derive(Debug)]
struct ScoredSection<'a> {
    index: usize,
    text: &'a str,
    score: f32,
    matched_terms: HashSet<String>,
}

fn normalized_keywords(input: &str) -> Vec<String> {
    let mut keywords = input
        .split(|character: char| !character.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|keyword| keyword.chars().count() > 1)
        .collect::<Vec<_>>();
    keywords.sort_unstable();
    keywords.dedup();
    keywords
}

fn add_keywords(weights: &mut BTreeMap<String, f32>, input: &str, source_weight: f32) {
    for keyword in normalized_keywords(input) {
        let weight = if QUERY_STOP_WORDS.contains(&keyword.as_str()) {
            source_weight * STOP_WORD_WEIGHT
        } else {
            source_weight
        };
        weights
            .entry(keyword)
            .and_modify(|existing| *existing = (*existing).max(weight))
            .or_insert(weight);
    }
}

fn task_aware_keywords(
    task: &str,
    profile: &crate::core::triage::profile::TaskProfileLocal,
) -> Vec<QueryKeyword> {
    let mut weights = BTreeMap::new();
    // The caller's task is the retrieval query. Profile fields only provide a
    // lower-weight session hint so broad classes such as `bug_fix` cannot win.
    add_keywords(&mut weights, task, 1.0);
    add_keywords(&mut weights, &profile.task_class, PROFILE_KEYWORD_WEIGHT);
    add_keywords(&mut weights, &profile.intent, PROFILE_KEYWORD_WEIGHT);
    weights
        .into_iter()
        .map(|(term, weight)| QueryKeyword { term, weight })
        .collect()
}

fn score_section(chunk: &str, keywords: &[QueryKeyword]) -> (f32, HashSet<String>) {
    let terms = normalized_keywords(chunk)
        .into_iter()
        .collect::<HashSet<_>>();
    let mut matched_terms = HashSet::new();
    let mut matched_weight = 0.0;
    let mut total_weight = 0.0;

    for keyword in keywords {
        total_weight += keyword.weight;
        if terms.contains(&keyword.term) {
            matched_weight += keyword.weight;
            matched_terms.insert(keyword.term.clone());
        }
    }

    let score = if total_weight > 0.0 {
        matched_weight / total_weight
    } else {
        0.0
    };
    (score, matched_terms)
}

fn select_task_aware_sections<'a>(
    mut sections: Vec<ScoredSection<'a>>,
    keywords: &[QueryKeyword],
    task_terms: &HashSet<String>,
) -> Vec<ScoredSection<'a>> {
    sections.retain(|section| section.score >= MIN_TASK_AWARE_COVERAGE);
    sections.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.index.cmp(&right.index))
    });

    let top_k = sections.len().min(TASK_AWARE_TOP_K);
    let mut selected = sections.drain(..top_k).collect::<Vec<_>>();
    let mut covered_terms = selected
        .iter()
        .flat_map(|section| section.matched_terms.iter().cloned())
        .collect::<HashSet<_>>();

    for _ in 0..TASK_AWARE_DIVERSE_SECTIONS {
        let Some((position, _)) = sections
            .iter()
            .enumerate()
            .filter_map(|(position, section)| {
                let novel_weight = keywords
                    .iter()
                    .filter(|keyword| {
                        task_terms.contains(&keyword.term)
                            && section.matched_terms.contains(&keyword.term)
                            && !covered_terms.contains(&keyword.term)
                    })
                    .map(|keyword| keyword.weight)
                    .sum::<f32>();
                (novel_weight > 0.0).then_some((position, novel_weight))
            })
            .max_by(
                |(left_position, left_weight), (right_position, right_weight)| {
                    left_weight
                        .total_cmp(right_weight)
                        .then_with(|| {
                            sections[*left_position]
                                .score
                                .total_cmp(&sections[*right_position].score)
                        })
                        .then_with(|| {
                            sections[*right_position]
                                .index
                                .cmp(&sections[*left_position].index)
                        })
                },
            )
        else {
            break;
        };

        let section = sections.remove(position);
        covered_terms.extend(section.matched_terms.iter().cloned());
        selected.push(section);
    }

    selected
}

fn apply_task_aware_filter(
    output: &str,
    task: &str,
    profile: &crate::core::triage::profile::TaskProfileLocal,
    enabled: bool,
) -> String {
    if !enabled {
        return output.to_owned();
    }

    let task_terms = normalized_keywords(task)
        .into_iter()
        .collect::<HashSet<_>>();
    if task_terms.is_empty() {
        return output.to_owned();
    }
    let keywords = task_aware_keywords(task, profile);
    if keywords.is_empty() {
        return output.to_owned();
    }

    let mut chunks = output.split("\n## ");
    let prefix = chunks.next().unwrap_or_default();
    let sections = chunks
        .enumerate()
        .map(|(index, chunk)| {
            let (score, matched_terms) = score_section(chunk, &keywords);
            ScoredSection {
                index,
                text: chunk,
                score,
                matched_terms,
            }
        })
        .collect::<Vec<_>>();
    let selected = select_task_aware_sections(sections, &keywords, &task_terms);
    if selected.is_empty() {
        return output.to_owned();
    }

    // Keep selected source byte-for-byte. The global dispatch pipeline owns
    // line triage and the final turn budget, so compose is never compressed twice.
    let mut filtered = prefix.to_owned();
    for section in selected {
        filtered.push_str("\n## ");
        filtered.push_str(section.text);
    }
    filtered
}

fn current_task_profile(
    ctx: &ToolContext,
) -> Option<crate::core::triage::profile::TaskProfileLocal> {
    let session = ctx.session.as_ref()?.try_read().ok()?;
    crate::core::decision_loop_runtime::DecisionLoopRuntime::get_or_init()
        .profile_for_session(&session.id)
const SOLUTION_HINT_ITEM_LIMIT: usize = 20;
const UTILITY_DIRECTORIES: &[&str] = &["utils", "helpers", "common"];

/// Collect declared runtime dependencies from manifest files at the project root.
fn project_dependencies(project_root: &Path) -> Vec<String> {
    let mut dependencies = Vec::new();
    cargo_dependencies(&project_root.join("Cargo.toml"), &mut dependencies);
    package_dependencies(&project_root.join("package.json"), &mut dependencies);
    requirements_dependencies(&project_root.join("requirements.txt"), &mut dependencies);
    dependencies
}

fn cargo_dependencies(manifest: &Path, dependencies: &mut Vec<String>) {
    let Ok(content) = std::fs::read_to_string(manifest) else {
        return;
    };

    let mut in_dependencies = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(section) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_dependencies = section == "dependencies"
                || section == "workspace.dependencies"
                || section.ends_with(".dependencies");
            continue;
        }
        if !in_dependencies || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = trimmed.split_once('=') {
            add_dependency(dependencies, name.trim().trim_matches(['\"', '\'']));
        }
    }
}

fn package_dependencies(manifest: &Path, dependencies: &mut Vec<String>) {
    let Ok(content) = std::fs::read_to_string(manifest) else {
        return;
    };
    let Ok(package) = serde_json::from_str::<Value>(&content) else {
        return;
    };
    let Some(dependencies_object) = package.get("dependencies").and_then(Value::as_object) else {
        return;
    };

    for dependency in dependencies_object.keys() {
        add_dependency(dependencies, dependency);
    }
}

fn requirements_dependencies(manifest: &Path, dependencies: &mut Vec<String>) {
    let Ok(content) = std::fs::read_to_string(manifest) else {
        return;
    };

    for line in content.lines() {
        let requirement = line.split('#').next().unwrap_or_default().trim();
        if requirement.is_empty() || requirement.starts_with('-') {
            continue;
        }
        let name = requirement
            .split(|c: char| matches!(c, '=' | '<' | '>' | '!' | '~' | ';' | '[' | ' '))
            .next()
            .unwrap_or_default()
            .trim();
        add_dependency(dependencies, name);
    }
}

fn add_dependency(dependencies: &mut Vec<String>, dependency: &str) {
    if !dependency.is_empty() && !dependencies.iter().any(|known| known == dependency) {
        dependencies.push(dependency.to_string());
    }
}

/// List files in conventional utility directories rooted at the project.
fn project_utility_files(project_root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for directory in UTILITY_DIRECTORIES {
        let utility_root = project_root.join(directory);
        if !utility_root.is_dir() {
            continue;
        }
        for entry in ignore::WalkBuilder::new(&utility_root)
            .hidden(false)
            .follow_links(false)
            .build()
            .flatten()
        {
            if !entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
            {
                continue;
            }
            if let Ok(relative_path) = entry.path().strip_prefix(project_root) {
                files.push(relative_path.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

fn utility_hint(utility_files: &[String]) -> Option<String> {
    (!utility_files.is_empty()).then(|| {
        let listed = utility_files
            .iter()
            .take(SOLUTION_HINT_ITEM_LIMIT)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let remaining = utility_files.len().saturating_sub(listed.len());
        let suffix = (remaining > 0).then(|| format!(", … (+{remaining} more)"));
        format!(
            "Existing utilities: {}{}",
            listed.join(", "),
            suffix.unwrap_or_default()
        )
    })
}

/// Extract unique file paths from compose output and sum their raw byte sizes
/// to compute what the agent would have read without compose.
fn estimate_raw_input_tokens(compose_output: &str, project_root: &str) -> usize {
    let mut seen = HashSet::new();
    let mut raw_bytes: u64 = 0;
    let root = Path::new(project_root);

    for line in compose_output.lines() {
        let trimmed = line.trim();
        let candidate = if let Some(rest) = trimmed.strip_prefix("// ") {
            rest.split(':').next().map(str::trim)
        } else if trimmed.bytes().next().is_some_and(|b| b.is_ascii_digit()) {
            trimmed
                .split_once(". ")
                .map(|x| x.1)
                .and_then(|s| s.split(" (").next())
                .map(str::trim)
        } else if trimmed.contains(':') && !trimmed.starts_with('#') && !trimmed.starts_with("TASK")
        {
            let part = trimmed.split(':').next().unwrap_or("").trim();
            if part.contains('.') && !part.contains(' ') {
                Some(part)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(rel) = candidate {
            if rel.is_empty() || rel.len() > 256 {
                continue;
            }
            let full = root.join(rel);
            if seen.insert(full.clone()) {
                if let Ok(meta) = std::fs::metadata(&full) {
                    if meta.is_file() {
                        raw_bytes += meta.len();
                    }
                }
            }
        }
    }

    (raw_bytes / 4) as usize
}

impl McpTool for CtxComposeTool {
    fn name(&self) -> &'static str {
        "ctx_compose"
    }

    fn tool_def(&self) -> Tool {
        tool_def(
            "ctx_compose",
            "First-pass context for one task; returns ranked files and inline source — use instead of search→read chains.",
            json!({
                "type": "object",
                "properties": {
                    "task": { "type": "string", "description": "Short English task/question or symbol names" },
                    "path": { "type": "string", "description": "Project root" },
                    "task_aware": { "type": "boolean", "default": true, "description": "Rank compose sections against the task (default: true)" }
                },
                "required": ["task"]
            }),
        )
    }

    fn handle(
        &self,
        args: &Map<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, ErrorData> {
        let task = get_str(args, "task")
            .ok_or_else(|| ErrorData::invalid_params("task is required", None))?;
        let task_aware = get_bool(args, "task_aware").unwrap_or(true);
        let path = if let Some(p) = ctx.resolved_path("path") {
            p.to_string()
        } else if let Some(err) = ctx.path_error("path") {
            return Err(ErrorData::invalid_params(format!("path: {err}"), None));
        } else {
            ctx.project_root.clone()
        };

        // Share the resident BM25 cache with the composed semantic search.
        if let Some(ref cache) = ctx.bm25_cache {
            crate::tools::ctx_semantic_search::set_thread_cache(cache.clone());
        }

        let cache_enabled = crate::core::config::Config::load()
            .cache
            .compose_cache_enabled;
        let cached = cache_enabled
            .then(|| crate::core::ocla::compose_cache::global().check(&task, &path))
            .flatten();
        let (text, _) = if let Some(text) = cached {
            let sent = crate::core::tokens::count_tokens(&text);
            (text, sent)
        } else {
            // Cross-process delivery check before expensive computation
            let compose_builder = ComposedContextKey {
                task: task.clone(),
                path: path.clone(),
                source_digests: Vec::new(),
            };
            let ck = compose_builder.cache_key();
            let cv = compose_builder.validator();
            if let Some(entry) = crate::core::ocla::cache_delivery::check(&ck, &cv, "ctx_compose") {
                let stub = crate::core::ocla::cache_delivery::stub(&entry, "compose");
                let sent = crate::core::tokens::count_tokens(&stub);
                (stub, sent)
            } else {
                let (text, sent) = tokio::task::block_in_place(|| {
                    crate::tools::ctx_compose::handle(&task, &path, ctx.crp_mode)
                });
                if cache_enabled && !text.starts_with("ERROR") {
                    crate::core::ocla::compose_cache::global().record(&task, &path, text.clone());
                    crate::core::ocla::cache_delivery::record(
                        ck,
                        crate::core::ocla::cache_types::DeliveryKind::ComposedContext,
                        cv,
                        Some(path.clone()),
                        &text,
                        "ctx_compose",
                    );
                }
                (text, sent)
            }
        };

        if text.starts_with("ERROR") {
            return Err(ErrorData::invalid_params(text, None));
        }

        let text = current_task_profile(ctx).map_or_else(
            || text.clone(),
            |profile| apply_task_aware_filter(&text, &task, &profile, task_aware),
        );
        let text = {
            let cfg = crate::core::config::Config::load();
            if cfg.solution.enabled && cfg.solution.inject_in_compose {
                let project_root = Path::new(&path);
                let project_deps = project_dependencies(project_root);
                let mut hints =
                    crate::core::solution_rules::solution_compose_hints(true, &project_deps);
                if let Some(utility_hint) = utility_hint(&project_utility_files(project_root)) {
                    hints.push('\n');
                    hints.push_str(&utility_hint);
                }
                if hints.is_empty() {
                    text
                } else {
                    format!("{text}\n\n{hints}")
                }
            } else {
                text
            }
        };
        let sent = crate::core::tokens::count_tokens(&text);

        let raw_tokens = estimate_raw_input_tokens(&text, &path);
        let original = if raw_tokens > sent { raw_tokens } else { sent };
        let saved = original.saturating_sub(sent);

        Ok(ToolOutput {
            text,
            original_tokens: original,
            saved_tokens: saved,
            mode: Some("compose".to_string()),
            path: Some(path),
            changed: false,
            shell_outcome: None,
            content_blocks: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::triage::profile::{TaskProfileLocal, TaskScopeLocal};

    fn profile(task_class: &str, intent: &str, context_need_milli: u16) -> TaskProfileLocal {
        TaskProfileLocal {
            task_class: task_class.into(),
            intent: intent.into(),
            complexity: "low".into(),
            scope: TaskScopeLocal::SingleFile,
            context_need_milli,
            reasoning_need_milli: 0,
            risk_signal_milli: 0,
            confidence_milli: 500,
        }
    }

    #[test]
    fn task_aware_filter_can_be_disabled() {
        let output = "TASK: test\n\n## Unrelated\nwidget catalog\n";
        assert_eq!(
            apply_task_aware_filter(
                output,
                "target query",
                &profile("bug_fix", "fix filtering", 700),
                false
            ),
            output
    use super::{project_dependencies, project_utility_files};

    #[test]
    fn detects_root_manifest_dependencies() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[dependencies]\ntokio = \"1\"\nserde = { version = \"1\" }\n",
        )
        .expect("Cargo.toml");
        std::fs::write(
            root.path().join("package.json"),
            r#"{"dependencies":{"react":"1","zod":"1"}}"#,
        )
        .expect("package.json");
        std::fs::write(
            root.path().join("requirements.txt"),
            "requests>=2\nrich[markdown]==1\n# comment\n",
        )
        .expect("requirements.txt");

        assert_eq!(
            project_dependencies(root.path()),
            vec![
                "tokio".to_string(),
                "serde".to_string(),
                "react".to_string(),
                "zod".to_string(),
                "requests".to_string(),
                "rich".to_string(),
            ]
        );
    }

    #[test]
    fn task_aware_filter_uses_the_call_task_over_session_profile() {
        let output = "TASK: test\n\n## Query A\nrenew oauth token\n\n## Query B\noauth renewal flow\n\n## Session profile\nmaintenance routine\n";
        let filtered = apply_task_aware_filter(
            output,
            "renew oauth token",
            &profile("maintenance", "routine", 700),
            true,
        );

        assert!(filtered.contains("## Query A"));
        assert!(filtered.contains("## Query B"));
        assert!(!filtered.contains("## Session profile"));
    }

    #[test]
    fn task_aware_filter_downweights_stop_words() {
        let output =
            "TASK: test\n\n## Generic\nhow to do this\n\n## Parser\nupdate parser properly\n";
        let filtered = apply_task_aware_filter(
            output,
            "how to update the parser properly",
            &profile("maintenance", "routine", 700),
            true,
        );

        assert!(filtered.contains("## Parser"));
        assert!(!filtered.contains("## Generic"));
    }

    #[test]
    fn task_aware_filter_uses_top_k_then_diverse_coverage() {
        let output = "TASK: test\n\n## A\ncache storage retries\n\n## B\ncache storage retries duplicate\n\n## C\nmetrics\n\n## D\ncache\n";
        let filtered = apply_task_aware_filter(
            output,
            "cache storage retries metrics",
            &profile("cache", "storage retries metrics", 700),
            true,
        );

        assert!(filtered.contains("## A"));
        assert!(filtered.contains("## B"));
        assert!(filtered.contains("## C"));
        assert!(!filtered.contains("## D"));
    }

    #[test]
    fn task_aware_filter_leaves_triage_to_the_global_pipeline() {
        let output = format!(
            "TASK: test\n\n## Relevant\nfix context gate\n{}// TODO: keep this\n",
            "// boilerplate\n".repeat(40)
        );
        let filtered = apply_task_aware_filter(
            &output,
            "fix context gate",
            &profile("bug_fix", "fix context gate filtering", 450),
            true,
        );
        assert!(filtered.contains("## Relevant"));
        assert!(filtered.contains("// TODO: keep this"));
        assert!(filtered.contains("// boilerplate"));
        assert!(!filtered.contains("filtered by triage"));
    fn finds_files_in_root_utility_directories() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("utils/nested")).expect("utils directory");
        std::fs::create_dir_all(root.path().join("helpers")).expect("helpers directory");
        std::fs::create_dir_all(root.path().join("common")).expect("common directory");
        for file in [
            "utils/nested/time.rs",
            "helpers/format.ts",
            "common/model.py",
        ] {
            std::fs::write(root.path().join(file), "").expect("utility file");
        }

        assert_eq!(
            project_utility_files(root.path()),
            vec![
                "common/model.py".to_string(),
                "helpers/format.ts".to_string(),
                "utils/nested/time.rs".to_string(),
            ]
        );
    }
}
