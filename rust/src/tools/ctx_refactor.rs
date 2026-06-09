use lsp_types::{Location, Position};
use serde_json::Value;

use crate::lsp::client::uri_to_file_path;

pub fn handle(args: &Value, project_root: &str, abs_path: &str) -> String {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("references");

    if matches!(
        action,
        "replace_symbol_body" | "insert_before_symbol" | "insert_after_symbol"
    ) {
        return handle_symbol_edit(action, args, project_root);
    }

    let line = args.get("line").and_then(Value::as_u64).unwrap_or(1) as u32;
    let column = args.get("column").and_then(Value::as_u64).unwrap_or(0) as u32;
    let scope = args
        .get("scope")
        .and_then(Value::as_str)
        .unwrap_or("project");

    let uri = match crate::lsp::router::open_file(abs_path, project_root) {
        Ok(u) => u,
        Err(e) => return format!("ERROR: {e}"),
    };

    let position = Position::new(line.saturating_sub(1), column);

    match action {
        "rename" => handle_rename(args, abs_path, project_root, &uri, position),
        "references" => handle_references(abs_path, project_root, &uri, position, scope),
        "definition" => handle_definition(abs_path, project_root, &uri, position),
        "implementations" => handle_implementations(abs_path, project_root, &uri, position, scope),
        "declaration" => handle_declaration(abs_path, project_root, &uri, position),
        "type_hierarchy" => handle_type_hierarchy(args, abs_path, project_root, &uri, position),
        "symbols_overview" => handle_symbols_overview(abs_path, project_root, &uri),
        "inspections" => handle_inspections(args, abs_path, project_root, &uri),
        _ => format!(
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, \
             implementations, declaration, type_hierarchy, symbols_overview, inspections, \
             replace_symbol_body, insert_before_symbol, insert_after_symbol."
        ),
    }
}

fn handle_rename(
    args: &Value,
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
) -> String {
    let Some(new_name) = args.get("new_name").and_then(Value::as_str) else {
        return "ERROR: 'new_name' parameter is required for rename.".to_string();
    };

    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.rename(uri, position, new_name)
    });

    match result {
        Ok(Some(edit)) => format_workspace_edit(&edit, project_root),
        Ok(None) => "No rename edits returned by language server.".to_string(),
        Err(e) => format!("ERROR: {e}"),
    }
}

fn handle_references(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
    scope: &str,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        let locs = backend.references(uri, position, scope)?;
        Ok((locs, backend.last_truncation()))
    });

    match result {
        Ok((locations, meta)) => {
            let mut out = format_locations(&locations, project_root);
            out.push_str(&truncation_note(locations.len(), meta));
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}

fn handle_definition(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.definition(uri, position)
    });

    match result {
        Ok(resp) => {
            let locations = match resp {
                lsp_types::GotoDefinitionResponse::Scalar(loc) => vec![loc],
                lsp_types::GotoDefinitionResponse::Array(locs) => locs,
                lsp_types::GotoDefinitionResponse::Link(links) => links
                    .into_iter()
                    .map(|l| Location {
                        uri: l.target_uri,
                        range: l.target_selection_range,
                    })
                    .collect(),
            };
            format_locations(&locations, project_root)
        }
        Err(e) => format!("ERROR: {e}"),
    }
}

fn handle_implementations(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
    scope: &str,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        let locs = backend.implementations(uri, position, scope)?;
        Ok((locs, backend.last_truncation()))
    });

    match result {
        Ok((locations, meta)) => {
            let mut out = format_locations(&locations, project_root);
            out.push_str(&truncation_note(locations.len(), meta));
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}

fn handle_declaration(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.declaration(uri, position)
    });

    match result {
        Ok(locations) => format_locations(&locations, project_root),
        Err(e) => format!("ERROR: {e}"),
    }
}

use crate::lsp::backend::{
    HierarchyDirection, InspectionDiag, InspectionInfo, SymbolOverviewItem, TypeHierarchyNode,
};

/// A resolved symbol location (project-relative path + 1-based inclusive line span).
#[derive(Debug)]
pub(crate) struct Resolved {
    pub rel_path: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Apply a resolved edit. IDE-first: a live JetBrains backend (port file +
/// liveness, mirroring router::select_backend) handles it via WriteCommandAction;
/// otherwise the headless local_range_write applies the identical bytes.
pub(crate) fn apply_symbol_edit(
    action: &str,
    project_root: &str,
    edit: &crate::lsp::backend::RangeEdit,
) -> Result<crate::lsp::backend::EditResult, String> {
    use crate::lsp::backend::LspBackend;
    use crate::lsp::port_discovery;

    let mut backend: Box<dyn LspBackend> =
        if let Some(pf) = port_discovery::read_port_file(project_root) {
            if port_discovery::pid_alive(pf.pid) && port_discovery::health_ok(&pf) {
                Box::new(crate::lsp::jetbrains_backend::JetBrainsHttpBackend::new(
                    pf.port,
                    pf.token,
                    project_root.to_string(),
                    pf.pid,
                ))
            } else {
                Box::new(crate::lsp::edit_apply::HeadlessBackend)
            }
        } else {
            Box::new(crate::lsp::edit_apply::HeadlessBackend)
        };

    match action {
        "replace_symbol_body" => backend.replace_symbol_body(edit),
        "insert_before_symbol" => backend.insert_before_symbol(edit),
        "insert_after_symbol" => backend.insert_after_symbol(edit),
        other => Err(format!("INTERNAL: not an edit action: {other}")),
    }
}

/// Leading whitespace of the 1-based `line` in `content` (anchor indentation).
pub(crate) fn anchor_indent(content: &str, line: usize) -> String {
    content
        .lines()
        .nth(line.saturating_sub(1))
        .map(|l| l.chars().take_while(|c| *c == ' ' || *c == '\t').collect())
        .unwrap_or_default()
}

/// Prefix `indent` to the first line of `text` iff that line has no leading
/// whitespace of its own (deterministic; the same Rust computes it for both
/// apply paths, so the wire text is byte-identical).
pub(crate) fn reindent_first_line(text: &str, indent: &str) -> String {
    if text.starts_with(' ') || text.starts_with('\t') || indent.is_empty() {
        return text.to_string();
    }
    format!("{indent}{text}")
}

/// Resolve a `name_path` (`Class/method` or bare `name`) to a single symbol via
/// the tree-sitter index (spec v2a §3/§5.3). Disambiguates a qualified path by
/// enclosing-range containment (ancestor symbol's line span contains the leaf's).
pub(crate) fn resolve_name_path(name_path: &str, project_root: &str) -> Result<Resolved, String> {
    use crate::core::graph_provider;
    let open = graph_provider::open_or_build(project_root)
        .ok_or_else(|| "NO_SYMBOL: no symbol index available".to_string())?;
    let gp = &open.provider;

    let segments: Vec<&str> = name_path.split('/').filter(|s| !s.is_empty()).collect();
    let leaf = *segments
        .last()
        .ok_or_else(|| "NO_SYMBOL: empty name_path".to_string())?;

    // Exact-name leaf candidates (case-sensitive — the index may substring-match).
    let mut leaves: Vec<_> = gp
        .find_symbols(leaf, None, None)
        .into_iter()
        .filter(|s| s.name == leaf)
        .collect();

    // Qualify by the immediate ancestor segment, if present.
    if segments.len() >= 2 {
        let ancestor = segments[segments.len() - 2];
        let parents: Vec<_> = gp
            .find_symbols(ancestor, None, None)
            .into_iter()
            .filter(|s| s.name == ancestor)
            .collect();
        leaves.retain(|leaf_sym| {
            parents.iter().any(|p| {
                p.file == leaf_sym.file
                    && p.start_line <= leaf_sym.start_line
                    && leaf_sym.end_line <= p.end_line
            })
        });
    }

    match leaves.len() {
        0 => Err(format!(
            "NO_SYMBOL: '{name_path}' did not resolve to any indexed symbol"
        )),
        1 => Ok(Resolved {
            rel_path: leaves[0].file.clone(),
            start_line: leaves[0].start_line,
            end_line: leaves[0].end_line,
        }),
        _ => {
            let mut msg = format!(
                "AMBIGUOUS_SYMBOL: '{name_path}' matches {} symbols; qualify it:\n",
                leaves.len()
            );
            for s in leaves.iter().take(10) {
                msg.push_str(&format!(
                    "  {}:{} (L{}-{})\n",
                    s.file, s.name, s.start_line, s.end_line
                ));
            }
            Err(msg)
        }
    }
}

fn parse_direction(args: &Value) -> HierarchyDirection {
    match args.get("direction").and_then(Value::as_str) {
        Some("subtypes") => HierarchyDirection::Subtypes,
        _ => HierarchyDirection::Supertypes,
    }
}

fn handle_type_hierarchy(
    args: &Value,
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
) -> String {
    let direction = parse_direction(args);
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        let tree = backend.type_hierarchy(uri, position, direction)?;
        Ok((tree, backend.last_truncation()))
    });
    match result {
        Ok((tree, meta)) => {
            let mut out = format_type_hierarchy(&tree);
            if matches!(meta, Some(m) if m.truncated) {
                out.push_str("\n(truncated — depth/node cap reached)\n");
            }
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}

fn handle_symbols_overview(file_path: &str, project_root: &str, uri: &lsp_types::Uri) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        let items = backend.symbols_overview(uri)?;
        Ok((items, backend.last_truncation()))
    });
    match result {
        Ok((items, meta)) => {
            let mut out = format_symbols_overview(&items);
            out.push_str(&truncation_note(items.len(), meta));
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}

fn handle_symbol_edit(action: &str, args: &Value, project_root: &str) -> String {
    // 1) Resolve target: name_path (primary) or path+line(+column) fallback.
    let (rel_path, start_line, end_line) = if let Some(np) =
        args.get("name_path").and_then(Value::as_str)
    {
        match resolve_name_path(np, project_root) {
            Ok(r) => (r.rel_path, r.start_line, r.end_line),
            Err(e) => return format!("ERROR: {e}"),
        }
    } else {
        let Some(path) = args.get("path").and_then(Value::as_str) else {
            return "ERROR: provide 'name_path' or 'path'+'line' for symbol edits.".to_string();
        };
        let line = args.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
        let end = args
            .get("end_line")
            .and_then(Value::as_u64)
            .unwrap_or(line as u64) as usize;
        if line == 0 {
            return "ERROR: 'line' is required (1-based) when using the path fallback.".to_string();
        }
        (path.to_string(), line, end)
    };

    // 2) PathJail on the resolved path (v1 §4.5 seam — critical before writes).
    let abs_path =
        match crate::core::path_resolve::resolve_tool_path(Some(project_root), None, &rel_path) {
            Ok(p) => p,
            Err(e) => return format!("ERROR: path blocked by jail: {e}"),
        };

    let content = match std::fs::read_to_string(&abs_path) {
        Ok(c) => c,
        Err(e) => return format!("ERROR: FILE_NOT_FOUND: {abs_path}: {e}"),
    };

    // 3) Build the canonical range + final wire text per action.
    let expected_hash = args
        .get("expected_hash")
        .and_then(Value::as_str)
        .map(String::from);
    let (range, text) = match action {
        "replace_symbol_body" => {
            let Some(new_body) = args.get("new_body").and_then(Value::as_str) else {
                return "ERROR: 'new_body' is required for replace_symbol_body.".to_string();
            };
            let end_col = content
                .lines()
                .nth(end_line.saturating_sub(1))
                .map_or(0, str::len) as u32;
            (
                crate::lsp::backend::TextRange0Based {
                    start_line: (start_line - 1) as u32,
                    start_char: 0,
                    end_line: (end_line - 1) as u32,
                    end_char: end_col,
                },
                new_body.to_string(),
            )
        }
        "insert_before_symbol" | "insert_after_symbol" => {
            let Some(t) = args.get("text").and_then(Value::as_str) else {
                return format!("ERROR: 'text' is required for {action}.");
            };
            let indent = anchor_indent(&content, start_line);
            let final_text = format!("{}\n", reindent_first_line(t, &indent));
            let insert_line = if action == "insert_before_symbol" {
                (start_line - 1) as u32
            } else {
                end_line as u32
            };
            (
                crate::lsp::backend::TextRange0Based {
                    start_line: insert_line,
                    start_char: 0,
                    end_line: insert_line,
                    end_char: 0,
                },
                final_text,
            )
        }
        other => return format!("ERROR: INTERNAL: not an edit action: {other}"),
    };

    // CONFLICT guard (BLAKE3, same source as headless local_range_write): verify
    // expected_hash against the current on-disk range BEFORE dispatch. This makes
    // the IDE path enforce CONFLICT identically to the headless path (which also
    // re-checks atomically). hash_hex == blake3::hash(...).to_hex().
    if let Some(exp) = &expected_hash {
        let s =
            match crate::lsp::edit_apply::offset_of(&content, range.start_line, range.start_char) {
                Ok(o) => o,
                Err(e) => return format!("ERROR: {e}"),
            };
        let e = match crate::lsp::edit_apply::offset_of(&content, range.end_line, range.end_char) {
            Ok(o) => o,
            Err(e) => return format!("ERROR: {e}"),
        };
        if e < s {
            return "ERROR: POSITION_OUT_OF_RANGE: end before start".to_string();
        }
        let actual = crate::core::hasher::hash_hex(&content.as_bytes()[s..e]);
        if *exp != actual {
            return format!(
                "ERROR: CONFLICT: range hash mismatch (expected={exp}, actual={actual})"
            );
        }
    }

    let edit = crate::lsp::backend::RangeEdit {
        abs_path,
        rel_path,
        range,
        text,
        expected_hash,
    };

    // 4) Dispatch (IDE-first, headless fallback) + format.
    match apply_symbol_edit(action, project_root, &edit) {
        Ok(res) => format_edit_result(action, &res),
        Err(e) => format!("ERROR: {e}"),
    }
}

fn format_edit_result(action: &str, res: &crate::lsp::backend::EditResult) -> String {
    if !res.applied {
        return format!("{action}: not applied.");
    }
    let r = res.new_range;
    let body = if res.diff.is_empty() {
        res.edited_text.clone()
    } else {
        res.diff.clone()
    };
    format!(
        "{action} applied (L{}:{}-L{}:{}):\n{}",
        r.start_line + 1,
        r.start_char,
        r.end_line + 1,
        r.end_char,
        body
    )
}

fn handle_inspections(
    args: &Value,
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
) -> String {
    let mode = args.get("mode").and_then(Value::as_str).unwrap_or("run");
    match mode {
        "run" => {
            let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
                let diags = backend.inspections(uri)?;
                Ok((diags, backend.last_truncation()))
            });
            match result {
                Ok((diags, meta)) => {
                    let mut out = format_inspections(&diags);
                    out.push_str(&truncation_note(diags.len(), meta));
                    out
                }
                Err(e) => format!("ERROR: {e}"),
            }
        }
        "list" => {
            let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
                let items = backend.list_inspections()?;
                Ok((items, backend.last_truncation()))
            });
            match result {
                Ok((items, meta)) => {
                    let mut out = format_inspection_list(&items);
                    out.push_str(&truncation_note(items.len(), meta));
                    out
                }
                Err(e) => format!("ERROR: {e}"),
            }
        }
        other => format!("ERROR: Unknown mode '{other}' for inspections. Available: run, list."),
    }
}

fn format_inspections(diags: &[InspectionDiag]) -> String {
    if diags.is_empty() {
        return "No inspection findings.".to_string();
    }
    let mut out = format!("{} finding(s):\n", diags.len());
    for d in diags {
        out.push_str(&format!(
            "  {}:{}  {}  {}\n",
            d.path, d.line, d.severity, d.message
        ));
    }
    out
}

fn format_inspection_list(items: &[InspectionInfo]) -> String {
    if items.is_empty() {
        return "No inspections enabled.".to_string();
    }
    let mut out = format!("{} inspection(s):\n", items.len());
    for i in items {
        out.push_str(&format!("  {}  {}  {}\n", i.id, i.name, i.severity));
    }
    out
}

fn truncation_note(shown: usize, meta: Option<crate::lsp::backend::Truncation>) -> String {
    match meta {
        Some(m) if m.truncated => {
            format!("\n(truncated — showing {shown} of {})\n", m.total)
        }
        _ => String::new(),
    }
}

fn format_type_hierarchy(root: &TypeHierarchyNode) -> String {
    fn walk(node: &TypeHierarchyNode, depth: usize, out: &mut String) {
        let indent = "  ".repeat(depth);
        out.push_str(&format!(
            "{indent}{} ({}:{})\n",
            node.name, node.path, node.line
        ));
        for child in &node.children {
            walk(child, depth + 1, out);
        }
    }
    let mut out = String::new();
    walk(root, 0, &mut out);
    out
}

fn format_symbols_overview(items: &[SymbolOverviewItem]) -> String {
    if items.is_empty() {
        return "No symbols found.".to_string();
    }
    let mut out = format!("{} symbol(s):\n", items.len());
    for item in items {
        out.push_str(&format!(
            "  {} {} (line {})\n",
            item.kind, item.name, item.line
        ));
    }
    out
}

fn format_locations(locations: &[Location], project_root: &str) -> String {
    if locations.is_empty() {
        return "No results found.".to_string();
    }

    let mut out = format!("{} location(s):\n", locations.len());
    for loc in locations {
        let path = uri_to_file_path(&loc.uri).map_or_else(
            || loc.uri.as_str().to_string(),
            |p| {
                p.strip_prefix(project_root)
                    .map(|s| s.strip_prefix('/').unwrap_or(s).to_string())
                    .unwrap_or(p)
            },
        );

        let line = loc.range.start.line + 1;
        let col = loc.range.start.character;
        out.push_str(&format!("  {path}:{line}:{col}\n"));
    }
    out
}

fn format_workspace_edit(edit: &lsp_types::WorkspaceEdit, project_root: &str) -> String {
    let mut out = String::from("Rename edits:\n");
    let mut file_count = 0;
    let mut edit_count = 0;

    if let Some(ref changes) = edit.changes {
        for (uri, edits) in changes {
            let path = uri_to_file_path(uri).map_or_else(
                || uri.as_str().to_string(),
                |p| {
                    p.strip_prefix(project_root)
                        .map(|s| s.strip_prefix('/').unwrap_or(s).to_string())
                        .unwrap_or(p)
                },
            );

            file_count += 1;
            out.push_str(&format!("  {path}: {} edit(s)\n", edits.len()));
            for e in edits {
                edit_count += 1;
                let line = e.range.start.line + 1;
                out.push_str(&format!("    L{line}: -> \"{}\"\n", e.new_text));
            }
        }
    }

    if let Some(ref doc_changes) = edit.document_changes {
        match doc_changes {
            lsp_types::DocumentChanges::Edits(edits) => {
                for text_edit in edits {
                    let path = uri_to_file_path(&text_edit.text_document.uri)
                        .unwrap_or_else(|| text_edit.text_document.uri.as_str().to_string());
                    file_count += 1;
                    let edits_len = text_edit.edits.len();
                    edit_count += edits_len;
                    out.push_str(&format!("  {path}: {edits_len} edit(s)\n"));
                }
            }
            lsp_types::DocumentChanges::Operations(ops) => {
                for op in ops {
                    if let lsp_types::DocumentChangeOperation::Edit(text_edit) = op {
                        let path = uri_to_file_path(&text_edit.text_document.uri)
                            .unwrap_or_else(|| text_edit.text_document.uri.as_str().to_string());
                        file_count += 1;
                        let edits_len = text_edit.edits.len();
                        edit_count += edits_len;
                        out.push_str(&format!("  {path}: {edits_len} edit(s)\n"));
                    }
                }
            }
        }
    }

    out.push_str(&format!(
        "\nTotal: {edit_count} edit(s) across {file_count} file(s)."
    ));
    out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// §4.5: inner handle MUST use the (already jailed) abs_path it is given,
    /// never re-derive a path from raw args. A raw "../escape.rs" must never
    /// reach the filesystem layer; only the provided abs_path does.
    #[test]
    fn inner_handle_uses_provided_abs_path_not_raw_args() {
        let args = json!({"action": "references", "path": "../escape.rs", "line": 1, "column": 0});
        let out = super::handle(&args, "/proj", "/proj/jailed.rs");
        // open_file fails reading the (nonexistent) jailed file → error names abs_path.
        assert!(out.contains("/proj/jailed.rs"), "abs_path not used: {out}");
        assert!(
            !out.contains("../escape.rs"),
            "raw path leaked to fs layer: {out}"
        );
    }

    /// `declaration` is a known action: the unknown-action arm must not fire for it,
    /// and its help text now advertises `declaration`.
    ///
    /// NOTE (adaptation): the real `handle` opens the file *before* the action
    /// match, so reaching the unknown-action help arm requires a backend. We seed
    /// a no-op stub backend for `rust` and point at a real temp `.rs` file so
    /// dispatch deterministically reaches the help text, offline, without
    /// starting rust-analyzer.
    #[test]
    fn unknown_action_help_lists_declaration() {
        struct StubBackend;
        impl crate::lsp::backend::LspBackend for StubBackend {
            fn open_file(
                &mut self,
                _uri: &lsp_types::Uri,
                _language_id: &str,
                _text: &str,
            ) -> Result<(), String> {
                Ok(())
            }
            fn references(
                &mut self,
                _uri: &lsp_types::Uri,
                _position: lsp_types::Position,
                _scope: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
            }
            fn definition(
                &mut self,
                _uri: &lsp_types::Uri,
                _position: lsp_types::Position,
            ) -> Result<lsp_types::GotoDefinitionResponse, String> {
                Ok(lsp_types::GotoDefinitionResponse::Array(vec![]))
            }
            fn implementations(
                &mut self,
                _uri: &lsp_types::Uri,
                _position: lsp_types::Position,
                _scope: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
            }
            fn rename(
                &mut self,
                _uri: &lsp_types::Uri,
                _position: lsp_types::Position,
                _new_name: &str,
            ) -> Result<Option<lsp_types::WorkspaceEdit>, String> {
                Ok(None)
            }
        }

        let dir = std::env::temp_dir().join(format!("leanctx_r1_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("x.rs");
        std::fs::write(&file, "fn x() {}\n").unwrap();
        let root = dir.to_string_lossy().to_string();
        let abs = file.to_string_lossy().to_string();

        crate::lsp::router::seed_stub_backend("rust", Box::new(StubBackend));

        let args = json!({"action": "definitely_bogus", "path": "x.rs", "line": 1});
        let out = super::handle(&args, &root, &abs);
        assert!(
            out.contains("declaration"),
            "help text missing declaration: {out}"
        );
        assert!(
            out.contains("inspections"),
            "help text missing inspections: {out}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn type_hierarchy_formats_indented_tree() {
        use crate::lsp::backend::{
            HierarchyDirection, LspBackend, SymbolOverviewItem, TypeHierarchyNode,
        };

        struct HierBackend;
        impl LspBackend for HierBackend {
            fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> {
                Ok(())
            }
            fn references(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
            }
            fn definition(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
            ) -> Result<lsp_types::GotoDefinitionResponse, String> {
                Ok(lsp_types::GotoDefinitionResponse::Array(vec![]))
            }
            fn implementations(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
            }
            fn rename(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _n: &str,
            ) -> Result<Option<lsp_types::WorkspaceEdit>, String> {
                Ok(None)
            }
            fn type_hierarchy(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                dir: HierarchyDirection,
            ) -> Result<TypeHierarchyNode, String> {
                assert_eq!(dir, HierarchyDirection::Subtypes);
                Ok(TypeHierarchyNode {
                    name: "Animal".into(),
                    path: "A.kt".into(),
                    line: 1,
                    children: vec![TypeHierarchyNode {
                        name: "Dog".into(),
                        path: "A.kt".into(),
                        line: 2,
                        children: vec![],
                    }],
                })
            }
            fn symbols_overview(
                &mut self,
                _u: &lsp_types::Uri,
            ) -> Result<Vec<SymbolOverviewItem>, String> {
                Ok(vec![SymbolOverviewItem {
                    name: "Animal".into(),
                    kind: "interface".into(),
                    line: 1,
                }])
            }
        }

        let tree = HierBackend
            .type_hierarchy(
                &crate::lsp::client::file_path_to_uri("/p/A.kt").unwrap(),
                lsp_types::Position::new(0, 0),
                HierarchyDirection::Subtypes,
            )
            .unwrap();
        let out = super::format_type_hierarchy(&tree);
        assert!(out.contains("Animal (A.kt:1)"), "{out}");
        assert!(out.contains("  Dog (A.kt:2)"), "{out}"); // child indented

        let items = HierBackend
            .symbols_overview(&crate::lsp::client::file_path_to_uri("/p/A.kt").unwrap())
            .unwrap();
        let out2 = super::format_symbols_overview(&items);
        assert!(out2.contains("interface Animal (line 1)"), "{out2}");
    }

    #[test]
    fn parse_direction_defaults_to_supertypes() {
        use crate::lsp::backend::HierarchyDirection;
        assert_eq!(
            super::parse_direction(&json!({})),
            HierarchyDirection::Supertypes
        );
        assert_eq!(
            super::parse_direction(&json!({"direction": "subtypes"})),
            HierarchyDirection::Subtypes
        );
        assert_eq!(
            super::parse_direction(&json!({"direction": "supertypes"})),
            HierarchyDirection::Supertypes
        );
    }

    #[test]
    fn resolve_name_path_unique_class() {
        let _lock = crate::core::data_dir::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        std::fs::create_dir_all(&data).unwrap();
        std::env::set_var("LEAN_CTX_DATA_DIR", data.to_string_lossy().to_string());

        let proj = tmp.path().join("proj");
        std::fs::create_dir_all(proj.join("src")).unwrap();
        std::fs::write(
            proj.join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.0.0\"\n",
        )
        .unwrap();
        std::fs::write(
            proj.join("src/lib.rs"),
            "pub struct UniqueZqWidget { pub a: u8 }\n",
        )
        .unwrap();
        let root = proj.to_string_lossy().to_string();

        let r = super::resolve_name_path("UniqueZqWidget", &root).expect("unique resolution");
        assert!(r.rel_path.ends_with("lib.rs"), "got: {}", r.rel_path);
        assert!(r.end_line >= r.start_line && r.start_line > 0);

        std::env::remove_var("LEAN_CTX_DATA_DIR");
    }

    #[test]
    fn resolve_name_path_unknown_is_no_symbol() {
        let _lock = crate::core::data_dir::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        std::fs::create_dir_all(&data).unwrap();
        std::env::set_var("LEAN_CTX_DATA_DIR", data.to_string_lossy().to_string());

        let proj = tmp.path().join("proj");
        std::fs::create_dir_all(proj.join("src")).unwrap();
        std::fs::write(
            proj.join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.0.0\"\n",
        )
        .unwrap();
        std::fs::write(
            proj.join("src/lib.rs"),
            "pub struct UniqueZqWidget { pub a: u8 }\n",
        )
        .unwrap();
        let root = proj.to_string_lossy().to_string();

        let err = super::resolve_name_path("ZzzNoSuchSymbol123", &root).unwrap_err();
        assert!(err.starts_with("NO_SYMBOL"), "got: {err}");

        std::env::remove_var("LEAN_CTX_DATA_DIR");
    }

    #[test]
    fn anchor_indent_reads_leading_whitespace() {
        let content = "class A {\n    fun b() {}\n}\n";
        assert_eq!(super::anchor_indent(content, 2), "    "); // line 2 (1-based) → 4 spaces
        assert_eq!(super::anchor_indent(content, 1), ""); // line 1 → none
    }

    #[test]
    fn reindent_prefixes_first_line_only() {
        assert_eq!(
            super::reindent_first_line("fun x() {}", "    "),
            "    fun x() {}"
        );
        // Already-indented text is left untouched.
        assert_eq!(
            super::reindent_first_line("    fun x()", "    "),
            "    fun x()"
        );
    }

    #[test]
    fn apply_symbol_edit_headless_replaces_range() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Foo.txt"), "aaa\nBODY\nccc\n").unwrap();
        let abs = dir.path().join("Foo.txt").to_string_lossy().to_string();
        let edit = crate::lsp::backend::RangeEdit {
            abs_path: abs.clone(),
            rel_path: "Foo.txt".into(),
            range: crate::lsp::backend::TextRange0Based {
                start_line: 1,
                start_char: 0,
                end_line: 1,
                end_char: 4,
            },
            text: "NEW".into(),
            expected_hash: None,
        };
        // No port file under this temp dir → headless apply.
        let res =
            super::apply_symbol_edit("replace_symbol_body", dir.path().to_str().unwrap(), &edit)
                .unwrap();
        assert!(res.applied);
        assert_eq!(std::fs::read_to_string(&abs).unwrap(), "aaa\nNEW\nccc\n");
    }

    #[test]
    fn handle_replace_symbol_body_via_position_fallback() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn old() {\n  1\n}\n").unwrap();
        let args = serde_json::json!({
            "action": "replace_symbol_body",
            "path": "a.rs",
            "line": 1,
            "end_line": 3,
            "new_body": "fn new() {\n  2\n}"
        });
        let out = super::handle(&args, dir.path().to_str().unwrap(), "");
        assert!(out.contains("replace_symbol_body applied"), "got: {out}");
        let after = std::fs::read_to_string(dir.path().join("a.rs")).unwrap();
        assert!(after.contains("fn new()"), "file: {after}");
    }

    #[test]
    fn handle_replace_symbol_body_conflict_on_stale_hash() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn old() {\n  1\n}\n").unwrap();
        // Range = full file lines 1..=3; old content = the whole file text.
        let stale = serde_json::json!({
            "action": "replace_symbol_body",
            "path": "a.rs", "line": 1, "end_line": 3,
            "new_body": "fn new() {\n  2\n}",
            "expected_hash": "deadbeefnotahash"
        });
        let out = super::handle(&stale, dir.path().to_str().unwrap(), "");
        assert!(out.contains("CONFLICT"), "got: {out}");
        // file unchanged
        assert!(std::fs::read_to_string(dir.path().join("a.rs"))
            .unwrap()
            .contains("fn old()"));
    }

    #[test]
    fn references_output_surfaces_truncation_note() {
        use lsp_types::Position;
        struct TruncBackend;
        impl crate::lsp::backend::LspBackend for TruncBackend {
            fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> {
                Ok(())
            }
            fn references(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                let uri = crate::lsp::client::file_path_to_uri("/proj/a.rs").unwrap();
                Ok(vec![lsp_types::Location {
                    uri,
                    range: lsp_types::Range::default(),
                }])
            }
            fn definition(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
            ) -> Result<lsp_types::GotoDefinitionResponse, String> {
                Ok(lsp_types::GotoDefinitionResponse::Array(vec![]))
            }
            fn implementations(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
            }
            fn rename(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _n: &str,
            ) -> Result<Option<lsp_types::WorkspaceEdit>, String> {
                Ok(None)
            }
            fn last_truncation(&self) -> Option<crate::lsp::backend::Truncation> {
                Some(crate::lsp::backend::Truncation {
                    truncated: true,
                    total: 742,
                })
            }
        }
        crate::lsp::router::seed_stub_backend("rust", Box::new(TruncBackend));
        let uri = crate::lsp::client::file_path_to_uri("/proj/a.rs").unwrap();
        let out = super::handle_references(
            "/proj/a.rs",
            "/proj",
            &uri,
            Position {
                line: 0,
                character: 0,
            },
            "project",
        );
        assert!(
            out.contains("truncated"),
            "expected truncation note, got: {out}"
        );
        assert!(out.contains("742"), "expected total in note, got: {out}");
    }

    #[test]
    fn inspections_run_and_list_dispatch_and_truncation() {
        use lsp_types::Position;
        struct InspBackend;
        impl crate::lsp::backend::LspBackend for InspBackend {
            fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> {
                Ok(())
            }
            fn references(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
            }
            fn definition(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
            ) -> Result<lsp_types::GotoDefinitionResponse, String> {
                Ok(lsp_types::GotoDefinitionResponse::Array(vec![]))
            }
            fn implementations(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
            }
            fn rename(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _n: &str,
            ) -> Result<Option<lsp_types::WorkspaceEdit>, String> {
                Ok(None)
            }
            fn inspections(
                &mut self,
                _u: &lsp_types::Uri,
            ) -> Result<Vec<crate::lsp::backend::InspectionDiag>, String> {
                Ok(vec![crate::lsp::backend::InspectionDiag {
                    path: "A.kt".into(),
                    line: 7,
                    severity: "WARNING".into(),
                    message: "unused".into(),
                }])
            }
            fn list_inspections(
                &mut self,
            ) -> Result<Vec<crate::lsp::backend::InspectionInfo>, String> {
                Ok(vec![crate::lsp::backend::InspectionInfo {
                    id: "UnusedSymbol".into(),
                    name: "Unused declaration".into(),
                    severity: "WARNING".into(),
                }])
            }
            fn last_truncation(&self) -> Option<crate::lsp::backend::Truncation> {
                Some(crate::lsp::backend::Truncation {
                    truncated: true,
                    total: 99,
                })
            }
        }
        crate::lsp::router::seed_stub_backend("rust", Box::new(InspBackend));
        let uri = crate::lsp::client::file_path_to_uri("/proj/a.rs").unwrap();

        // run mode (default): formats path:line SEVERITY message + truncation note
        let run_out = super::handle_inspections(
            &json!({"action": "inspections"}),
            "/proj/a.rs",
            "/proj",
            &uri,
        );
        assert!(run_out.contains("A.kt:7"), "run diag missing: {run_out}");
        assert!(
            run_out.contains("WARNING"),
            "run severity missing: {run_out}"
        );
        assert!(run_out.contains("unused"), "run message missing: {run_out}");
        assert!(
            run_out.contains("truncated"),
            "run truncation missing: {run_out}"
        );
        assert!(run_out.contains("99"), "run total missing: {run_out}");

        // list mode: formats id name severity
        let list_out = super::handle_inspections(
            &json!({"action": "inspections", "mode": "list"}),
            "/proj/a.rs",
            "/proj",
            &uri,
        );
        assert!(
            list_out.contains("UnusedSymbol"),
            "list id missing: {list_out}"
        );
        assert!(
            list_out.contains("Unused declaration"),
            "list name missing: {list_out}"
        );

        // unknown mode → defined ERROR
        let bad_out = super::handle_inspections(
            &json!({"action": "inspections", "mode": "bogus"}),
            "/proj/a.rs",
            "/proj",
            &uri,
        );
        assert!(
            bad_out.contains("ERROR"),
            "unknown mode not rejected: {bad_out}"
        );
        let _ = (Position::new(0, 0),); // keep import used if refactored
    }
}
