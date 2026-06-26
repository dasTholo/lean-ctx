//! #5: auto-render delegation for `.lmd.md` sources.
//!
//! WITHOUT the lean-md addon, a `.lmd.md` read must return raw text — never an
//! error, never a half-rendered document. The hook-level decision
//! (`try_lmd_addon_render` → `None` → raw fallthrough) is gated hermetically by
//! the unit test `lmd_md_without_addon_returns_raw_text` in
//! `rust/src/tools/registered/ctx_read.rs`; this file adds the user-facing CLI
//! boundary check end-to-end against the freshly built binary.
//!
//! WITH the addon, a `.lmd.md` read delegates to the lean-md `ctx_md_render` and
//! equals a direct `lean-md render`. The headless `lean-ctx call` path cannot
//! drive this end-to-end: the delegation hook needs a Tokio runtime
//! (`Handle::try_current`, ctx_read.rs) which `call` lacks, so the hook returns
//! `None`; and the raw-read fallback that then runs needs a session, which
//! `call` also lacks (hard error "session not available"). Delegation therefore
//! only fires inside the live lean-ctx MCP server (runtime + session). The check
//! below instead drives the lean-md addon's own stdio MCP server directly — the
//! exact surface lean-ctx delegates to — and asserts its `ctx_md_render` equals
//! `lean-md render`; it is `#[ignore]` because it spawns the `lean-md` binary.
use std::io::Write;
use std::process::{Command, Stdio};

/// The lean-ctx binary under test — the freshly built one, never the (possibly
/// stale) `lean-ctx` on PATH.
const LEAN_CTX_BIN: &str = env!("CARGO_BIN_EXE_lean-ctx");

#[test]
fn ctx_read_lmd_without_addon_is_raw() {
    // No addon installed (default CI state) → a read of a `.lmd.md` must surface
    // the raw source, never an error or a half-rendered document. We assert the
    // marker survives the read; the precise hook-level raw-vs-delegate decision
    // is unit-tested in ctx_read.rs::lmd_md_without_addon_returns_raw_text.
    let dir = std::env::temp_dir().join("delegation_raw_e2e");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("d.lmd.md");
    std::fs::write(&f, "@date\nRAW_DELEGATION_MARKER\n").unwrap();

    let out = Command::new(LEAN_CTX_BIN)
        .args(["read", f.to_str().unwrap(), "--mode", "full"])
        .output()
        .expect("lean-ctx read");
    let text = String::from_utf8_lossy(&out.stdout);

    assert!(
        text.contains("RAW_DELEGATION_MARKER"),
        "without an addon a .lmd.md read must return raw text (no error, no half-render): {text}"
    );
}

#[test]
#[ignore = "spawns the lean-md binary (`lean-md mcp`) — needs lean-md on PATH"]
fn ctx_read_lmd_with_addon_equals_direct_render() {
    // #5: the addon surface lean-ctx delegates to (`ctx_md_render`, served by
    // `lean-md mcp`) must be byte-identical to a direct `lean-md render`. We
    // drive that stdio MCP server directly — the same server lean-ctx spawns via
    // the gateway after `addon add` — and compare. The lean-ctx → addon
    // forwarding itself is the manual roundtrip gate (Task 6 §3 / tests in the
    // lean-md repo, #4).
    let dir = std::env::temp_dir().join("delegation_addon");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("d.lmd.md");
    std::fs::write(&f, "@date\nmarker\n").unwrap();
    let path = f.to_str().unwrap();

    let direct = {
        let out = Command::new("lean-md")
            .args(["render", path])
            .output()
            .expect("lean-md render");
        String::from_utf8_lossy(&out.stdout).into_owned()
    };

    // Drive `lean-md mcp` (line-delimited JSON-RPC 2.0 over stdio) and call
    // ctx_md_render for the same path. Dropping stdin signals EOF so the server
    // loop ends and flushes its single response line.
    let mut child = Command::new("lean-md")
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn lean-md mcp");
    let req = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"ctx_md_render","arguments":{{"path":"{path}"}}}}}}"#
    );
    writeln!(child.stdin.as_mut().expect("mcp stdin"), "{req}").expect("write request");
    let out = child.wait_with_output().expect("wait lean-md mcp");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.lines().next().expect("one JSON-RPC response line");
    let resp: serde_json::Value = serde_json::from_str(line).expect("parse JSON-RPC response");
    let via = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("ctx_md_render result text")
        .to_owned();

    assert_eq!(
        via, direct,
        "addon ctx_md_render must equal direct lean-md render (#5)"
    );
}
