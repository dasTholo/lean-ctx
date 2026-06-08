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
             JetBrains-only).",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["rename", "references", "definition", "implementations",
                                 "declaration", "type_hierarchy", "symbols_overview", "inspections"],
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
                    }
                },
                "required": ["action", "path"]
            }),
        )
    }

    fn handle(
        &self,
        args: &Map<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, ErrorData> {
        // §4.5: PathJail runs in the dispatcher BEFORE this handle. require_resolved_path
        // surfaces a jail rejection / missing / non-string `path` as an MCP error here,
        // so no relative/escaping path is ever rebuilt or sent to a backend.
        let abs_path = require_resolved_path(ctx, args, "path")?;

        let args_value = Value::Object(args.clone());
        let result = crate::tools::ctx_refactor::handle(&args_value, &ctx.project_root, &abs_path);

        let action = get_str(args, "action").unwrap_or_default();
        Ok(ToolOutput {
            text: result,
            original_tokens: 0,
            saved_tokens: 0,
            mode: Some(action),
            path: get_str(args, "path"),
            changed: false,
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
        ] {
            assert!(schema.contains(needle), "schema missing {needle}: {schema}");
        }
    }
}
