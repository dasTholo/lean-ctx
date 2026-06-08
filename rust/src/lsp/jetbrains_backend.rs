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
    /// IDE process id from the discovered port file — for cheap staleness checks.
    pid: u32,
    /// IDE listen port — re-compared against the port file to detect restarts.
    port: u16,
    /// Truncation meta of the most recent capped call (references/implementations/
    /// type_hierarchy/symbols_overview), surfaced by ctx_refactor.
    last_meta: Option<crate::lsp::backend::Truncation>,
}

impl JetBrainsHttpBackend {
    /// Canonicalize the project root ONCE so project-relative wire paths rejoin
    /// byte-identically with the Kotlin side (port-file key = sha256(realpath)[..16]).
    /// Mirrors `port_discovery::project_hash` canonicalization. On error (e.g. path
    /// does not exist), fall back to the raw root with a trailing-slash trim.
    fn canonical_root(project_root: &str) -> String {
        let canonical = std::fs::canonicalize(project_root).map_or_else(
            |_| project_root.to_string(),
            |p| p.to_string_lossy().to_string(),
        );
        canonical
            .strip_suffix('/')
            .unwrap_or(&canonical)
            .to_string()
    }

    #[allow(clippy::needless_pass_by_value)] // public ctor; callers own String
    pub fn new(port: u16, token: String, project_root: String, pid: u32) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            token,
            project_root: Self::canonical_root(&project_root),
            pid,
            port,
            last_meta: None,
        }
    }

    #[cfg(test)]
    fn project_root_for_test(&self) -> &str {
        &self.project_root
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

    fn parse_truncation(v: &Value, shown: u32) -> Option<crate::lsp::backend::Truncation> {
        let truncated = v.get("truncated").and_then(Value::as_bool)?;
        let total = v
            .get("total")
            .and_then(Value::as_u64)
            .map_or(shown, |n| n as u32);
        Some(crate::lsp::backend::Truncation { truncated, total })
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
        let locs = self.parse_locations(&resp);
        self.last_meta = Self::parse_truncation(&resp, locs.len() as u32);
        Ok(locs)
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
        let locs = self.parse_locations(&resp);
        self.last_meta = Self::parse_truncation(&resp, locs.len() as u32);
        Ok(locs)
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
        self.last_meta = Self::parse_truncation(&resp, 0);
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
        let items = Self::parse_symbols(&resp);
        self.last_meta = Self::parse_truncation(&resp, items.len() as u32);
        Ok(items)
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

    fn is_stale(&self, project_root: &str) -> bool {
        // Cheap re-check: port file gone, or pid/port changed (IDE restarted),
        // or our cached pid is dead → stale. NO HTTP (health is not pinged per call).
        match crate::lsp::port_discovery::read_port_file(project_root) {
            Some(pf) => {
                pf.pid != self.pid
                    || pf.port != self.port
                    || !crate::lsp::port_discovery::pid_alive(self.pid)
            }
            None => true,
        }
    }

    fn last_truncation(&self) -> Option<crate::lsp::backend::Truncation> {
        self.last_meta
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
        let mut backend = JetBrainsHttpBackend::new(
            port,
            "tok".to_string(),
            "/proj".to_string(),
            std::process::id(),
        );
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
        let mut backend = JetBrainsHttpBackend::new(
            port,
            "tok".to_string(),
            "/proj".to_string(),
            std::process::id(),
        );
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
        let mut backend = JetBrainsHttpBackend::new(
            port,
            "tok".to_string(),
            "/proj".to_string(),
            std::process::id(),
        );
        let uri = file_path_to_uri("/proj/A.kt").unwrap();
        let items = backend.symbols_overview(&uri).expect("should parse");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].kind, "interface");
        assert_eq!(items[1].name, "main");
        assert_eq!(items[1].line, 9);
    }

    #[test]
    fn references_records_truncation_meta() {
        let body = r#"{"locations":[{"path":"a.rs","range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}}}],"truncated":true,"total":742}"#;
        let port = mock_once(body);
        let mut backend =
            JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string(), std::process::id());
        let uri = file_path_to_uri("/proj/a.rs").unwrap();
        let _ = backend
            .references(&uri, Position { line: 0, character: 0 }, "project")
            .unwrap();
        let meta = backend.last_truncation().expect("meta recorded");
        assert!(meta.truncated);
        assert_eq!(meta.total, 742);
    }

    #[test]
    fn is_stale_true_when_no_port_file() {
        // Unlikely root → no port file → cached backend is stale.
        let backend = JetBrainsHttpBackend::new(
            12345,
            "tok".to_string(),
            "/nonexistent/leanctx/proj/xyz".to_string(),
            999_999_999,
        );
        assert!(backend.is_stale("/nonexistent/leanctx/proj/xyz"));
    }

    #[test]
    fn is_stale_false_for_matching_live_pid() {
        let _lock = crate::core::data_dir::test_env_lock();
        // A port file describing THIS process (pid alive) + matching port/token
        // must be considered fresh. We stage a port file via the data-dir env.
        let tmp = std::env::temp_dir().join(format!("leanctx-stale-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_string_lossy().to_string();
        // Write a port file at the discovery path for `root`.
        std::env::set_var("LEAN_CTX_DATA_DIR", &tmp);
        let pf_path = crate::lsp::port_discovery::port_file_path(&root).unwrap();
        let pid = std::process::id();
        std::fs::write(
            &pf_path,
            format!(
                r#"{{"port":4567,"token":"tok","pid":{pid},"project_root":"{root}","ide_version":"x"}}"#
            ),
        )
        .unwrap();
        let backend = JetBrainsHttpBackend::new(4567, "tok".to_string(), root.clone(), pid);
        assert!(
            !backend.is_stale(&root),
            "matching live pid+port must be fresh"
        );
        // Different cached pid → stale even though the file is live.
        let other = JetBrainsHttpBackend::new(4567, "tok".to_string(), root.clone(), pid + 1);
        assert!(other.is_stale(&root), "pid mismatch must be stale");
        std::env::remove_var("LEAN_CTX_DATA_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn canonical_root_strips_trailing_slash_and_resolves_realpath() {
        // Existing dir with a trailing slash → canonical form has no trailing slash
        // and matches sha2's canonicalize (port_discovery::project_hash parity).
        let tmp = std::env::temp_dir();
        let with_slash = format!("{}/", tmp.to_string_lossy());
        let backend =
            JetBrainsHttpBackend::new(1, "t".to_string(), with_slash.clone(), std::process::id());
        let expected = std::fs::canonicalize(&tmp)
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert_eq!(backend.project_root_for_test(), expected);
        assert!(!backend.project_root_for_test().ends_with('/'));
    }

    #[test]
    fn canonical_root_falls_back_to_raw_for_nonexistent() {
        let raw = "/nonexistent/leanctx/xyz";
        let backend =
            JetBrainsHttpBackend::new(1, "t".to_string(), raw.to_string(), std::process::id());
        assert_eq!(backend.project_root_for_test(), raw);
    }
}
