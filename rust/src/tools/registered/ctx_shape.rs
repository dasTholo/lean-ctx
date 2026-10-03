use rmcp::ErrorData;
use rmcp::model::Tool;
use serde_json::{Map, Value, json};

use crate::server::tool_trait::{McpTool, ToolContext, ToolOutput};
use crate::tool_defs::tool_def;

/// Shapes the output of a host-native tool *after* it ran (concept K2,
/// "shape, don't redirect"): a host mod (Claude Code `tool.call`) forwards the
/// native Bash/Grep/Glob result here instead of denying the call and forcing a
/// round trip to `ctx_shell`/`ctx_search`. The result flows through the same
/// dispatch pipeline as every `ctx_*` output (sensitivity floor, policy-pack
/// redaction, PII/injection filters), so native output gets the same guards.
///
/// Internal host surface: callable (`tools/call`, a mod's `$.mcp.call`) but
/// never advertised — see `dynamic_tools::INTERNAL_HOST_TOOLS`.
pub struct CtxShapeTool;

/// Below this the shaping gain cannot pay for the call; the host keeps the
/// native bytes (the mod applies the same floor before calling).
const MIN_SHAPE_CHARS: usize = 2_000;

impl McpTool for CtxShapeTool {
    fn name(&self) -> &'static str {
        "ctx_shape"
    }

    /// The body replaces a native tool's output verbatim, so the pipeline must
    /// hand it back without hints, nudges or checkpoints — after its redaction,
    /// sensitivity and filter passes, which machine-readable calls keep.
    fn produces_machine_readable(&self, _args: Option<&Map<String, Value>>) -> bool {
        true
    }

    fn tool_def(&self) -> Tool {
        tool_def(
            "ctx_shape",
            "Internal host hook: shape a native tool's output after it ran. Not for agents.",
            json!({
                "type": "object",
                "properties": {
                    "tool": { "type": "string", "description": "Native tool name: Bash | Grep | Glob" },
                    "command": { "type": "string", "description": "Exact command line (Bash)" },
                    "output": { "type": "string", "description": "The native tool's output as the model would read it" }
                },
                "required": ["tool", "output"]
            }),
        )
    }

    fn handle(
        &self,
        args: &Map<String, Value>,
        _ctx: &ToolContext,
    ) -> Result<ToolOutput, ErrorData> {
        let tool = args
            .get("tool")
            .and_then(Value::as_str)
            .ok_or_else(|| ErrorData::invalid_params("tool is required", None))?;
        let output = args
            .get("output")
            .and_then(Value::as_str)
            .ok_or_else(|| ErrorData::invalid_params("output is required", None))?;
        let command = args.get("command").and_then(Value::as_str);

        let shaped = shape(tool, command, output);
        let original_tokens = crate::core::tokens::count_tokens(output);
        let shaped_tokens = crate::core::tokens::count_tokens(&shaped);
        Ok(ToolOutput {
            text: shaped,
            original_tokens,
            saved_tokens: original_tokens.saturating_sub(shaped_tokens),
            mode: Some("shape".to_string()),
            path: None,
            changed: false,
            shell_outcome: None,
            content_blocks: None,
        })
    }
}

/// Pure shaping decision, separated from MCP plumbing for testing.
fn shape(tool: &str, command: Option<&str>, output: &str) -> String {
    if output.chars().count() < MIN_SHAPE_CHARS {
        return output.to_string();
    }
    let redacted = super::ctx_shell_background::redact_shell_output_secrets(output);
    let shaped = match (tool, command) {
        ("Bash", Some(cmd)) if !cmd.trim().is_empty() => {
            crate::proxy::compress::shape_command_output(cmd, &redacted)
        }
        _ => crate::proxy::compress::compress_tool_result(&redacted, Some(tool)),
    };
    // Never hand back more than the host already has.
    if shaped.len() >= redacted.len() {
        redacted
    } else {
        shaped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_output_is_returned_verbatim() {
        assert_eq!(shape("Bash", Some("ls"), "a\nb\n"), "a\nb\n");
    }

    /// The contract the mod relies on: never larger than the input, and a
    /// lossy shrink of a verbose log stays recoverable.
    #[test]
    fn verbose_output_shrinks_and_never_grows() {
        use std::fmt::Write as _;
        let mut noisy = String::new();
        for i in 0..400 {
            let _ = writeln!(noisy, "   Compiling crate-{i} v0.1.{i} (/tmp/x)");
        }
        let shaped = shape("Bash", Some("cargo build"), &noisy);
        assert!(
            shaped.len() < noisy.len(),
            "verbose cargo output must shrink"
        );
        assert!(
            shaped.contains("full original"),
            "a lossy shrink must carry its recovery handle: {shaped}"
        );

        let mut opaque = String::new();
        for i in 0..300 {
            let _ = writeln!(opaque, "{i:x}{}", i * 7919);
        }
        assert!(shape("Grep", None, &opaque).len() <= opaque.len());
    }
}
