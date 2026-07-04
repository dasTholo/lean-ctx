//! `.lmd.md` reads are raw — like any other file.
//!
//! After the lmd reverse-cut, lean-ctx has no `.lmd.md`-specific code path: a
//! read returns the raw source verbatim (never an error, never a half-rendered
//! document). Rendering `.lmd.md` is owned entirely by the external lean-md
//! addon (`ctx_md_render` / CLI `lean-md render`) and is out of scope here.
//! This end-to-end check drives the freshly built `lean-ctx` binary and asserts
//! a `.lmd.md` read surfaces the raw marker.
use std::process::Command;

/// The lean-ctx binary under test — the freshly built one, never the (possibly
/// stale) `lean-ctx` on PATH.
const LEAN_CTX_BIN: &str = env!("CARGO_BIN_EXE_lean-ctx");

#[test]
fn ctx_read_lmd_md_returns_raw_source() {
    // No addon installed (default CI state) → a read of a `.lmd.md` must surface
    // the raw source, never an error or a half-rendered document. We assert the
    // marker survives the read.
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
