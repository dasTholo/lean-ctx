//! Backing B: in-IDE JetBrains PSI backend over HTTP/JSON (127.0.0.1).
//! Synchronous (`ureq`) — matches the synchronous `McpTool::handle` path and does
//! not block the Tokio runtime. Phase 1 implements references/definition/
//! implementations; rename + the degrading ops follow in later phases.

use std::time::Duration;

use lsp_types::{GotoDefinitionResponse, Location, Position, Range, Uri, WorkspaceEdit};
use serde_json::Value;

use crate::lsp::backend::{HierarchyDirection, LspBackend, SymbolOverviewItem, TypeHierarchyNode};
use crate::lsp::client::file_path_to_uri;

const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct JetBrainsHttpBackend {
    base_url: String,
    token: String,
    /// Absolute project root, to rejoin project-relative wire paths.
    project_root: String,
}

impl JetBrainsHttpBackend {
    pub fn new(port: u16, token: String, project_root: String) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            token,
            project_root,
        }
    }

    fn post(&self, endpoint: &str, body: &Value) -> Result<Value, String> {
        let url = format!("{}{endpoint}", self.base_url);
        // ureq 3.x + repo convention (NO `json` feature): serialize via serde_json,
        // send raw bytes, read response body as string, parse. Per-request timeout via
        // `.config().timeout_global(..).build()`. Pattern mirrors port_discovery.rs + llm_enhance.rs.
        let payload = serde_json::to_vec(body).map_err(|e| format!("serialize request: {e}"))?;
        let resp = ureq::post(&url)
            .config()
            .timeout_global(Some(Duration::from_secs(REQUEST_TIMEOUT_SECS)))
            .build()
            .header("X-LeanCtx-Token", &self.token)
            .header("Content-Type", "application/json")
            .send(payload.as_slice())
            .map_err(|e| format!("JetBrains backend request to {endpoint} failed: {e}"))?;
        let text = resp
            .into_body()
            .read_to_string()
            .map_err(|e| format!("JetBrains backend: read response: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("JetBrains backend: parse response: {e}"))
    }

    /// Project-relative path → absolute file URI (Rust rejoins, spec §6).
    fn rel_to_uri(&self, rel: &str) -> Option<Uri> {
        let abs = format!("{}/{}", self.project_root, rel);
        file_path_to_uri(&abs).ok()
    }

    fn parse_position(v: &Value) -> Option<Position> {
        let line = v.get("line")?.as_u64()? as u32;
        let character = v.get("character")?.as_u64()? as u32;
        Some(Position { line, character })
    }

    fn parse_locations(&self, v: &Value) -> Vec<Location> {
        v.get("locations")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|loc| {
                        let rel = loc.get("path")?.as_str()?;
                        let uri = self.rel_to_uri(rel)?;
                        let range = loc.get("range")?;
                        let start = Self::parse_position(range.get("start")?)?;
                        let end = Self::parse_position(range.get("end")?)?;
                        Some(Location {
                            uri,
                            range: Range { start, end },
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_type_hierarchy(v: &Value) -> TypeHierarchyNode {
        fn node(v: &Value) -> TypeHierarchyNode {
            TypeHierarchyNode {
                name: v
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_string(),
                path: v
                    .get("path")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                line: v.get("line").and_then(Value::as_u64).unwrap_or(0) as u32,
                children: v
                    .get("children")
                    .and_then(Value::as_array)
                    .map(|arr| arr.iter().map(node).collect())
                    .unwrap_or_default(),
            }
        }
        v.get("tree").map_or_else(
            || TypeHierarchyNode {
                name: String::new(),
                path: String::new(),
                line: 0,
                children: vec![],
            },
            node,
        )
    }

    fn parse_symbols(v: &Value) -> Vec<SymbolOverviewItem> {
        v.get("symbols")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| {
                        Some(SymbolOverviewItem {
                            name: s.get("name")?.as_str()?.to_string(),
                            kind: s.get("kind")?.as_str()?.to_string(),
                            line: s.get("line")?.as_u64()? as u32,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `{path}` request body (file-level ops, no position).
    fn path_body(&self, uri: &Uri) -> Value {
        let abs = crate::lsp::client::uri_to_file_path(uri).unwrap_or_default();
        let rel = abs
            .strip_prefix(&self.project_root)
            .map(|s| s.strip_prefix('/').unwrap_or(s).to_string())
            .unwrap_or(abs);
        serde_json::json!({ "path": rel })
    }

    /// Build the `{path, line, character}` request body. `position` is already
    /// 0-based (LSP convention) — sent verbatim. `uri` → project-relative path.
    fn position_body(&self, uri: &Uri, position: Position) -> Value {
        let abs = crate::lsp::client::uri_to_file_path(uri).unwrap_or_default();
        let rel = abs
            .strip_prefix(&self.project_root)
            .map(|s| s.strip_prefix('/').unwrap_or(s).to_string())
            .unwrap_or(abs);
        serde_json::json!({
            "path": rel,
            "line": position.line,
            "character": position.character,
        })
    }
}

impl LspBackend for JetBrainsHttpBackend {
    fn open_file(&mut self, _uri: &Uri, _language_id: &str, _text: &str) -> Result<(), String> {
        // The IDE already has the file in its VFS/index — no explicit open needed.
        Ok(())
    }

    fn references(
        &mut self,
        uri: &Uri,
        position: Position,
        scope: &str,
    ) -> Result<Vec<Location>, String> {
        let mut body = self.position_body(uri, position);
        body["scope"] = serde_json::json!(scope);
        let resp = self.post("/references", &body)?;
        Ok(self.parse_locations(&resp))
    }

    fn definition(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<GotoDefinitionResponse, String> {
        let body = self.position_body(uri, position);
        let resp = self.post("/definition", &body)?;
        Ok(GotoDefinitionResponse::Array(self.parse_locations(&resp)))
    }

    fn implementations(
        &mut self,
        uri: &Uri,
        position: Position,
        scope: &str,
    ) -> Result<Vec<Location>, String> {
        let mut body = self.position_body(uri, position);
        body["scope"] = serde_json::json!(scope);
        let resp = self.post("/implementations", &body)?;
        Ok(self.parse_locations(&resp))
    }

    fn declaration(&mut self, uri: &Uri, position: Position) -> Result<Vec<Location>, String> {
        let body = self.position_body(uri, position);
        let resp = self.post("/declaration", &body)?;
        Ok(self.parse_locations(&resp))
    }

    fn type_hierarchy(
        &mut self,
        uri: &Uri,
        position: Position,
        direction: HierarchyDirection,
    ) -> Result<TypeHierarchyNode, String> {
        let mut body = self.position_body(uri, position);
        body["direction"] = serde_json::json!(match direction {
            HierarchyDirection::Supertypes => "supertypes",
            HierarchyDirection::Subtypes => "subtypes",
        });
        let resp = self.post("/type_hierarchy", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(err
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("INTERNAL")
                .to_string());
        }
        Ok(Self::parse_type_hierarchy(&resp))
    }

    fn symbols_overview(&mut self, uri: &Uri) -> Result<Vec<SymbolOverviewItem>, String> {
        let body = self.path_body(uri);
        let resp = self.post("/symbols_overview", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(err
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("INTERNAL")
                .to_string());
        }
        Ok(Self::parse_symbols(&resp))
    }

    fn rename(
        &mut self,
        _uri: &Uri,
        _position: Position,
        _new_name: &str,
    ) -> Result<Option<WorkspaceEdit>, String> {
        // Symbolic edits are v2 (spec §9 v2-Ausblick). Phase 1 skeleton: not yet.
        Err("rename via JetBrains backend is not implemented yet (v2 edit spec)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Spins up a one-shot TCP server returning a canned HTTP/JSON response,
    /// so we can assert the wire→Location mapping without a real IDE.
    fn mock_once(json_body: &'static str) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 2048];
                let _ = stream.read(&mut buf); // drain request
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    json_body.len(),
                    json_body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        port
    }

    #[test]
    fn references_parses_wire_locations() {
        let body = r#"{"locations":[{"path":"src/main.rs","range":{"start":{"line":5,"character":13},"end":{"line":5,"character":18}}}]}"#;
        let port = mock_once(body);
        let mut backend = JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string());
        let uri = file_path_to_uri("/proj/src/main.rs").unwrap();
        let locs = backend
            .references(
                &uri,
                Position {
                    line: 5,
                    character: 13,
                },
                "project",
            )
            .expect("should parse");
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].range.start.line, 5);
        assert_eq!(locs[0].range.start.character, 13);
        assert!(locs[0].uri.as_str().ends_with("/proj/src/main.rs"));
    }

    #[test]
    fn type_hierarchy_parses_wire_tree() {
        use crate::lsp::backend::HierarchyDirection;
        let body = r#"{"tree":{"name":"Animal","path":"A.kt","line":1,"children":[{"name":"Dog","path":"A.kt","line":2,"children":[]}]},"truncated":false}"#;
        let port = mock_once(body);
        let mut backend = JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string());
        let uri = file_path_to_uri("/proj/A.kt").unwrap();
        let tree = backend
            .type_hierarchy(
                &uri,
                Position {
                    line: 0,
                    character: 0,
                },
                HierarchyDirection::Subtypes,
            )
            .expect("should parse");
        assert_eq!(tree.name, "Animal");
        assert_eq!(tree.line, 1);
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].name, "Dog");
        assert_eq!(tree.children[0].path, "A.kt");
    }

    #[test]
    fn symbols_overview_parses_wire_items() {
        let body = r#"{"symbols":[{"name":"Animal","kind":"interface","line":1},{"name":"main","kind":"function","line":9}],"truncated":false,"total":2}"#;
        let port = mock_once(body);
        let mut backend = JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string());
        let uri = file_path_to_uri("/proj/A.kt").unwrap();
        let items = backend.symbols_overview(&uri).expect("should parse");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].kind, "interface");
        assert_eq!(items[1].name, "main");
        assert_eq!(items[1].line, 9);
    }
}
