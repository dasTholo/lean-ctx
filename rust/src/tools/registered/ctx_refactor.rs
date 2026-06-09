use rmcp::model::Tool;
use rmcp::ErrorData;
use serde_json::{json, Map, Value};

use crate::server::tool_trait::{get_str, require_resolved_path, McpTool, ToolContext, ToolOutput};
use crate::tool_defs::tool_def;

pub struct CtxRefactorTool;

impl McpTool for CtxRefactorTool {
    fn name(&self) -> &'static str {
        "ctx_refactor"
    }

    fn tool_def(&self) -> Tool {
        tool_def(
            "ctx_refactor",
            "LSP-powered refactoring. Actions: rename, references, definition, implementations, \
             declaration, type_hierarchy, symbols_overview, inspections. Requires a running \
             language server (rust-analyzer, typescript-language-server, pylsp, gopls) or the \
             JetBrains backend (declaration, type_hierarchy, symbols_overview, inspections are \
             JetBrains-only). Symbol-body edits (replace_symbol_body, insert_before_symbol, \
             insert_after_symbol) are name_path-addressed and work IDE-first with a lossless \
             headless fallback. The Two-Phase rename ops (rename_preview, rename_apply) are \
             name_path-addressed, require a running JetBrains IDE (BACKEND_REQUIRED otherwise), \
             use a stateless plan_hash guard, and block on refactoring conflicts unless force=true.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["rename", "references", "definition", "implementations",
                                 "declaration", "type_hierarchy", "symbols_overview", "inspections",
                                 "replace_symbol_body", "insert_before_symbol", "insert_after_symbol",
                                 "rename_preview", "rename_apply"],
                        "description": "Refactoring action"
                    },
                    "path": { "type": "string", "description": "File path" },
                    "line": { "type": "integer", "description": "1-indexed line number" },
                    "column": { "type": "integer", "description": "0-indexed character offset" },
                    "new_name": { "type": "string", "description": "New name (only for rename action)" },
                    "scope": {
                        "type": "string",
                        "enum": ["project", "all"],
                        "description": "Search scope for references/implementations/type_hierarchy (JetBrains backend). 'project' = project sources only (default); 'all' = include libraries/SDK."
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["supertypes", "subtypes"],
                        "description": "type_hierarchy direction (JetBrains backend). 'supertypes' (default) = parents; 'subtypes' = children/implementors."
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["run", "list"],
                        "description": "inspections mode (JetBrains backend). 'run' (default) = diagnostics for the given file; 'list' = enabled inspections of the current project profile."
                    },
                    "name_path": { "type": "string", "description": "Symbol path for body edits: 'Class/method' (qualified) or bare 'name'. Resolved via the symbol index; ambiguous → AMBIGUOUS_SYMBOL with candidates." },
                    "new_body": { "type": "string", "description": "Full replacement declaration text (replace_symbol_body)." },
                    "text": { "type": "string", "description": "Sibling text to insert (insert_before_symbol/insert_after_symbol); indentation is applied automatically." },
                    "end_line": { "type": "integer", "description": "1-based last line of the symbol (only for the path+line fallback when name_path is omitted)." },
                    "expected_hash": { "type": "string", "description": "Optional BLAKE3-hex of the current range content; mismatch → CONFLICT (no blind overwrite)." },
                    "plan_hash": { "type": "string", "description": "Required for rename_apply: the BLAKE3 plan hash returned by rename_preview (stateless TOCTOU guard; mismatch → CONFLICT)." },
                    "force": { "type": "boolean", "description": "rename_apply only: override blocking refactoring conflicts (default false → CONFLICT when conflicts exist)." },
                    "search_comments": { "type": "boolean", "description": "rename: also rename matches inside comments/strings (default false)." },
                    "search_text_occurrences": { "type": "boolean", "description": "rename: also rename non-code text occurrences (default false)." }
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
        // name_path edits resolve their own path; only require/resolve `path`
        // when actually provided (read actions + position-fallback edits).
        let has_path = args.get("path").and_then(Value::as_str).is_some();
        let abs_path = if has_path {
            require_resolved_path(ctx, args, "path")?
        } else {
            String::new()
        };

        let args_value = Value::Object(args.clone());
        let result = crate::tools::ctx_refactor::handle(&args_value, &ctx.project_root, &abs_path);

        let action = get_str(args, "action").unwrap_or_default();
        Ok(ToolOutput {
            text: result,
            original_tokens: 0,
            saved_tokens: 0,
            mode: Some(action.clone()),
            path: get_str(args, "path"),
            changed: matches!(
                action.as_str(),
                "replace_symbol_body"
                    | "insert_before_symbol"
                    | "insert_after_symbol"
                    | "rename_apply"
            ),
        })
    }
}

#[cfg(test)]
mod schema_tests {
    use super::*;
    use crate::server::tool_trait::McpTool;

    #[test]
    fn schema_advertises_declaration_and_scope() {
        let tool = CtxRefactorTool;
        let def = tool.tool_def();
        let schema = serde_json::to_string(&def).unwrap();
        for needle in [
            "declaration",
            "\"scope\"",
            "type_hierarchy",
            "symbols_overview",
            "\"direction\"",
            "supertypes",
            "subtypes",
            "inspections",
            "\"mode\"",
            "replace_symbol_body",
            "insert_before_symbol",
            "insert_after_symbol",
            "name_path",
            "new_body",
            "expected_hash",
            "rename_preview",
            "rename_apply",
            "plan_hash",
            "force",
            "search_comments",
            "search_text_occurrences",
        ] {
            assert!(schema.contains(needle), "schema missing {needle}: {schema}");
        }
    }
}
