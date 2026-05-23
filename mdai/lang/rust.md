---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [cargo_nextest, cargo_clippy, cargo_fmt]
---

@markdownai v1.0

# Rust Pack (opt-in via MDAI_PROJECT_LANG=rust)

Mandates from `~/.claude/CLAUDE.md` + project `CLAUDE.md`: nextest instead of `cargo test`, clippy with `-D warnings`, `cargo fmt` before `git add`.

@define cargo_nextest()
@query mcp lean-ctx ctx_shell command="cargo nextest run"
@end

@define cargo_clippy()
@query mcp lean-ctx ctx_shell command="cargo clippy --workspace --all-targets -- -D warnings"
@end

@define cargo_fmt()
@query mcp lean-ctx ctx_shell command="cargo fmt"
@end
