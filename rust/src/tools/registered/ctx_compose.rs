use std::collections::HashSet;
use std::path::Path;

use crate::core::ocla::cache_types::{CacheKeyBuilder, ComposedContextKey};
use rmcp::ErrorData;
use rmcp::model::Tool;
use serde_json::{Map, Value, json};

use crate::server::tool_trait::{McpTool, ToolContext, ToolOutput, get_str};
use crate::tool_defs::tool_def;

pub struct CtxComposeTool;

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
            "PRIMARY TOOL — call FIRST for understanding code (before editing/debugging/'how does X work').\n\
             Returns ranked files with relevant symbol source inline grouped by file.\n\
             Combines BM25 lexical+semantic+associative retrieval+submodular optimization.\n\
             ANTIPATTERN: Do NOT chain search→read→symbol — one compose replaces the whole chain.\n\
             ANTIPATTERN: Do NOT Read files whose source compose already returned — it IS the source.\n\
             WORKFLOW: Fire parallel ctx_read or ctx_compose for different areas.",
            json!({
                "type": "object",
                "properties": {
                    "task": { "type": "string", "description": "Short English task/question or symbol names" },
                    "path": { "type": "string", "description": "Project root" }
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
        let (text, sent) = if let Some(text) = cached {
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
