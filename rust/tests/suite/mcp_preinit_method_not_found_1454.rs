//! Pre-init MethodNotFound contract (GH #1454).
//!
//! Modern MCP clients (MCP Go SDK >= 1.7 — Antigravity) probe a legacy stdio
//! server with `server/discover` BEFORE the `initialize` handshake. The
//! vendored rmcp handshake parses that fine (its ClientRequest union has a
//! CustomRequest catch-all, so the #1434 deser-path handler in mcp_stdio.rs
//! never fires) and rejects it with `ExpectedInitializeRequest`. Before the
//! fix the server treated that like a client disconnect and exited with a
//! bare EOF — the client then saw `connection closed: calling "initialize"`
//! and the MCP server failed to start.
//!
//! This test drives the REAL binary over REAL stdio JSON-RPC in an isolated
//! HOME and asserts the full Antigravity-style sequence:
//!   1. `server/discover` → -32601 with the request id echoed, and the
//!      PROCESS MUST STAY ALIVE (the regression this guards),
//!   2. the fallback `initialize` succeeds on the SAME connection,
//!   3. a post-init tools/list works,
//!   4. the same -32601 contract holds for Content-Length framing.

use std::io::{BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const RESPONSE_DEADLINE: Duration = Duration::from_secs(15);

struct TestEnv {
    _tmp: tempfile::TempDir,
    home: std::path::PathBuf,
    data: std::path::PathBuf,
    project: std::path::PathBuf,
}

fn test_env() -> TestEnv {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let data = tmp.path().join("data");
    let project = tmp.path().join("project");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&data).unwrap();
    std::fs::create_dir_all(project.join(".git")).unwrap();
    std::fs::write(project.join("hello.txt"), "hello 1454\n").unwrap();
    TestEnv {
        _tmp: tmp,
        home,
        data,
        project,
    }
}

fn spawn_server(env: &TestEnv) -> Child {
    let bin = env!("CARGO_BIN_EXE_lean-ctx");
    Command::new(bin)
        .arg("mcp")
        .current_dir(&env.project)
        .env("HOME", &env.home)
        .env("LEAN_CTX_DATA_DIR", &env.data)
        .env("CODEX_HOME", env.home.join(".codex"))
        .env("LEAN_CTX_HEADLESS", "1")
        // Root detection must derive from the temp project's cwd. When the
        // suite itself runs inside an IDE/agent session these carry the HOST
        // workspace and would hijack the project root (→ path-jail rejects
        // the temp file).
        .env_remove("LEAN_CTX_PROJECT_ROOT")
        .env_remove("CLAUDE_PROJECT_DIR")
        .env_remove("WORKSPACE_FOLDER_PATHS")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("mcp server spawn")
}

/// Accumulates raw stdout bytes and extracts complete JSON-RPC messages from
/// either wire protocol the server may use (JSON-lines or Content-Length).
struct FrameAccumulator {
    buf: Vec<u8>,
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

impl FrameAccumulator {
    fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Push a chunk and drain any complete frames, parsed as JSON values.
    fn push(&mut self, chunk: &[u8]) -> Vec<serde_json::Value> {
        self.buf.extend_from_slice(chunk);
        let mut out = Vec::new();
        loop {
            if self.buf.starts_with(b"Content-Length:") || self.buf.starts_with(b"content-length:")
            {
                let Some(end) = find_subsequence(&self.buf, b"\r\n\r\n") else {
                    break;
                };
                let header = String::from_utf8_lossy(&self.buf[..end]);
                let Some(len) = header
                    .split_once(':')
                    .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                else {
                    break;
                };
                let body_start = end + 4;
                if self.buf.len() < body_start + len {
                    break;
                }
                let body = &self.buf[body_start..body_start + len];
                if let Ok(value) = serde_json::from_slice(body) {
                    out.push(value);
                }
                self.buf.drain(..body_start + len);
            } else if let Some(nl) = self.buf.iter().position(|byte| *byte == b'\n') {
                let line = &self.buf[..nl];
                let line = line.strip_suffix(b"\r").unwrap_or(line);
                if let Ok(value) = serde_json::from_slice(line) {
                    out.push(value);
                }
                self.buf.drain(..=nl);
            } else {
                break;
            }
        }
        out
    }
}

/// Spawn the server and a reader thread feeding parsed responses into `rx`.
struct ServerSession {
    child: Child,
    stdin: std::process::ChildStdin,
    rx: mpsc::Receiver<serde_json::Value>,
}

fn start_session(env: &TestEnv) -> ServerSession {
    let mut child = spawn_server(env);
    let stdin = child.stdin.take().expect("child stdin");
    let stdout = child.stdout.take().expect("child stdout");
    let (tx, rx) = mpsc::channel::<serde_json::Value>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut frames = FrameAccumulator::new();
        loop {
            let mut chunk = [0u8; 8192];
            match reader.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    for value in frames.push(&chunk[..n]) {
                        if tx.send(value).is_err() {
                            return;
                        }
                    }
                }
            }
        }
    });
    ServerSession { child, stdin, rx }
}

impl ServerSession {
    fn write_line(&mut self, body: &serde_json::Value) {
        writeln!(self.stdin, "{body}").expect("write JSON-line");
    }

    fn write_content_length(&mut self, body: &serde_json::Value) {
        let serialized = body.to_string();
        write!(
            self.stdin,
            "Content-Length: {}\r\n\r\n{}",
            serialized.len(),
            serialized
        )
        .expect("write Content-Length frame");
        self.stdin.flush().expect("flush stdin");
    }

    fn wait_for(
        &self,
        predicate: impl Fn(&serde_json::Value) -> bool,
        label: &str,
    ) -> serde_json::Value {
        let until = Instant::now() + RESPONSE_DEADLINE;
        loop {
            let remaining = until
                .checked_duration_since(Instant::now())
                .unwrap_or_else(|| panic!("timeout waiting for {label}"));
            let value = self
                .rx
                .recv_timeout(remaining)
                .unwrap_or_else(|e| panic!("no {label} within deadline: {e}"));
            if predicate(&value) {
                return value;
            }
        }
    }

    fn assert_alive(&mut self, context: &str) {
        match self.child.try_wait().expect("try_wait") {
            None => {}
            Some(status) => {
                panic!("server EXITED after {context} (status {status}) — must stay alive")
            }
        }
    }
}

fn discover_request(id: i64) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "server/discover",
        "params": {}
    })
}

fn initialize_request(id: i64) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "antigravity-1454-test", "version": "2.5.5" }
        }
    })
}

/// The exact Antigravity startup sequence, JSON-line framing: server/discover
/// must answer -32601 (not EOF) and the process must stay alive so the
/// fallback initialize lands on the same connection.
#[test]
#[cfg_attr(
    windows,
    ignore = "HOME-override isolation is Unix-only (dirs::home_dir uses the Win32 API)"
)]
fn discover_gets_32601_then_initialize_succeeds_json_line() {
    let env = test_env();
    let mut session = start_session(&env);

    session.write_line(&discover_request(1));
    let discover = session.wait_for(
        |v| v.get("id").and_then(serde_json::Value::as_i64) == Some(1),
        "server/discover response",
    );
    assert_eq!(
        discover["error"]["code"],
        serde_json::json!(-32601),
        "server/discover must answer -32601; got: {discover}"
    );
    let message = discover["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("legacy initialize handshake"),
        "error message must flag the legacy handshake; got: {message}"
    );
    // THE regression: the process must not die after the -32601 reply.
    session.assert_alive("server/discover -32601");

    session.write_line(&initialize_request(2));
    let init = session.wait_for(
        |v| v.get("id").and_then(serde_json::Value::as_i64) == Some(2),
        "initialize response",
    );
    assert_eq!(
        init["result"]["serverInfo"]["name"],
        serde_json::json!("lean-ctx"),
        "fallback initialize must succeed; got: {init}"
    );

    session.write_line(&serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));
    session.write_line(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list",
        "params": {}
    }));
    let tools = session.wait_for(
        |v| v.get("id").and_then(serde_json::Value::as_i64) == Some(3),
        "tools/list response",
    );
    assert!(
        tools["result"]["tools"]
            .as_array()
            .is_some_and(|t| !t.is_empty()),
        "tools/list after fallback must work; got: {tools}"
    );

    drop(session.stdin); // EOF → clean server shutdown
    let _ = session.child.wait();
}

/// The same contract over Content-Length framing (the modern spec's other
/// wire protocol — both must mirror, cf. GH #1434's dual-framing matrix).
#[test]
#[cfg_attr(
    windows,
    ignore = "HOME-override isolation is Unix-only (dirs::home_dir uses the Win32 API)"
)]
fn discover_gets_32601_then_initialize_succeeds_content_length() {
    let env = test_env();
    let mut session = start_session(&env);

    session.write_content_length(&discover_request(1));
    let discover = session.wait_for(
        |v| v.get("id").and_then(serde_json::Value::as_i64) == Some(1),
        "server/discover response",
    );
    assert_eq!(
        discover["error"]["code"],
        serde_json::json!(-32601),
        "server/discover must answer -32601; got: {discover}"
    );
    session.assert_alive("server/discover -32601");

    session.write_content_length(&initialize_request(2));
    let init = session.wait_for(
        |v| v.get("id").and_then(serde_json::Value::as_i64) == Some(2),
        "initialize response",
    );
    assert_eq!(
        init["result"]["serverInfo"]["name"],
        serde_json::json!("lean-ctx"),
        "fallback initialize must succeed; got: {init}"
    );

    drop(session.stdin); // EOF → clean server shutdown
    let _ = session.child.wait();
}
