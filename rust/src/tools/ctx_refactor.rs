use lsp_types::{Location, Position};
use serde_json::Value;

use crate::lsp::client::uri_to_file_path;

pub fn handle(args: &Value, project_root: &str, abs_path: &str) -> String {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("references");

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
        "implementations" => {
            handle_implementations(abs_path, project_root, &uri, position, scope)
        }
        "declaration" => handle_declaration(abs_path, project_root, &uri, position),
        "type_hierarchy" => handle_type_hierarchy(args, abs_path, project_root, &uri, position),
        "symbols_overview" => handle_symbols_overview(abs_path, project_root, &uri),
        _ => format!(
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, \
             implementations, declaration, type_hierarchy, symbols_overview."
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
        backend.references(uri, position, scope)
    });

    match result {
        Ok(locations) => format_locations(&locations, project_root),
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
        backend.implementations(uri, position, scope)
    });

    match result {
        Ok(locations) => format_locations(&locations, project_root),
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

use crate::lsp::backend::{HierarchyDirection, SymbolOverviewItem, TypeHierarchyNode};

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
        backend.type_hierarchy(uri, position, direction)
    });
    match result {
        Ok(tree) => format_type_hierarchy(&tree),
        Err(e) => format!("ERROR: {e}"),
    }
}

fn handle_symbols_overview(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.symbols_overview(uri)
    });
    match result {
        Ok(items) => format_symbols_overview(&items),
        Err(e) => format!("ERROR: {e}"),
    }
}

fn format_type_hierarchy(root: &TypeHierarchyNode) -> String {
    fn walk(node: &TypeHierarchyNode, depth: usize, out: &mut String) {
        let indent = "  ".repeat(depth);
        out.push_str(&format!("{indent}{} ({}:{})\n", node.name, node.path, node.line));
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
        out.push_str(&format!("  {} {} (line {})\n", item.kind, item.name, item.line));
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

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn type_hierarchy_formats_indented_tree() {
        use crate::lsp::backend::{HierarchyDirection, LspBackend, SymbolOverviewItem, TypeHierarchyNode};

        struct HierBackend;
        impl LspBackend for HierBackend {
            fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> { Ok(()) }
            fn references(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
            fn definition(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position) -> Result<lsp_types::GotoDefinitionResponse, String> { Ok(lsp_types::GotoDefinitionResponse::Array(vec![])) }
            fn implementations(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
            fn rename(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _n: &str) -> Result<Option<lsp_types::WorkspaceEdit>, String> { Ok(None) }
            fn type_hierarchy(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, dir: HierarchyDirection) -> Result<TypeHierarchyNode, String> {
                assert_eq!(dir, HierarchyDirection::Subtypes);
                Ok(TypeHierarchyNode {
                    name: "Animal".into(), path: "A.kt".into(), line: 1,
                    children: vec![TypeHierarchyNode { name: "Dog".into(), path: "A.kt".into(), line: 2, children: vec![] }],
                })
            }
            fn symbols_overview(&mut self, _u: &lsp_types::Uri) -> Result<Vec<SymbolOverviewItem>, String> {
                Ok(vec![SymbolOverviewItem { name: "Animal".into(), kind: "interface".into(), line: 1 }])
            }
        }

        let tree = HierBackend.type_hierarchy(
            &crate::lsp::client::file_path_to_uri("/p/A.kt").unwrap(),
            lsp_types::Position::new(0, 0),
            HierarchyDirection::Subtypes,
        ).unwrap();
        let out = super::format_type_hierarchy(&tree);
        assert!(out.contains("Animal (A.kt:1)"), "{out}");
        assert!(out.contains("  Dog (A.kt:2)"), "{out}"); // child indented

        let items = HierBackend.symbols_overview(
            &crate::lsp::client::file_path_to_uri("/p/A.kt").unwrap(),
        ).unwrap();
        let out2 = super::format_symbols_overview(&items);
        assert!(out2.contains("interface Animal (line 1)"), "{out2}");
    }

    #[test]
    fn parse_direction_defaults_to_supertypes() {
        use crate::lsp::backend::HierarchyDirection;
        assert_eq!(super::parse_direction(&json!({})), HierarchyDirection::Supertypes);
        assert_eq!(super::parse_direction(&json!({"direction": "subtypes"})), HierarchyDirection::Subtypes);
        assert_eq!(super::parse_direction(&json!({"direction": "supertypes"})), HierarchyDirection::Supertypes);
    }
}
