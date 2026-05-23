---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [cargo_nextest, cargo_clippy, cargo_fmt, rustfmt_file]
---

@markdownai v1.0

# Rust Pack (opt-in via MDAI_PROJECT_LANG=rust)

Mandates from `~/.claude/CLAUDE.md` + project `CLAUDE.md`: nextest instead of `cargo test`, clippy with `-D warnings`, and `@call step_reformat_commit(file, message)` (from `mdai/tooling/jetbrains.md`) for the reformat + `git add` + `git commit` sequence on every changed file. `@call cargo_fmt()` here is workspace-wide; per-file formatting before staging goes through `step_reformat_commit` when JetBrains is available, otherwise through `@call rustfmt_file(file)`.

@define cargo_nextest()
@query mcp lean-ctx ctx_shell command="cargo nextest run"
@end

@define cargo_clippy()
@query mcp lean-ctx ctx_shell command="cargo clippy --workspace --all-targets -- -D warnings"
@end

@define cargo_fmt()
@query mcp lean-ctx ctx_shell command="cargo fmt"
@end

# Single-file formatting. Choose between:
#   - `@call rustfmt_file(file=...)`     — rustfmt directly (no IDE required).
#   - `@call reformat_file(file=...)`    — JetBrains reformat from `mdai/tooling/jetbrains.md`
#                                          (preferred when MDAI_HAS_JETBRAINS=true; respects
#                                          IDE-wide code-style settings beyond rustfmt rules).
# For workspace-wide formatting use `@call cargo_fmt()`.
@define rustfmt_file(file)
@query mcp lean-ctx ctx_shell command="rustfmt {{ file }}"
@end
