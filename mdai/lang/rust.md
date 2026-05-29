---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [ cargo_nextest, cargo_clippy, cargo_fmt, rustfmt_file, format_file ]
---

@markdownai v1.0

# Rust Pack (opt-in via MDAI_PROJECT_LANG=rust)

# Tests: nextest only — `cargo test` is forbidden per CLAUDE.md.

@define cargo_nextest()
@query mcp lean-ctx ctx_shell command="cargo nextest run" /
@define-end

# Lint: clippy with `-D warnings` per CLAUDE.md.

@define cargo_clippy()
@query mcp lean-ctx ctx_shell command="cargo clippy --workspace --all-targets -- -D warnings" /
@define-end

# Workspace-wide cargo fmt.

@define cargo_fmt()
@query mcp lean-ctx ctx_shell command="cargo fmt" /
@define-end

# Single-file formatting via rustfmt (no IDE required).

@define rustfmt_file(file)
@query mcp lean-ctx ctx_shell command="rustfmt {{ file }}" /
@define-end

# Single-file formatting dispatcher: JetBrains reformat (IDE-aware) when

# MDAI_HAS_JETBRAINS=true, else rustfmt. Use this in the reformat step

# before `git add` instead of choosing the backend manually. The JetBrains

# branch requires `mdai/tooling/jetbrains.md` to be loaded (via

# `@call load_tooling_packs()` from `mdai/core/startup-check.md`).

@define format_file(file)
@if @env MDAI_HAS_JETBRAINS == "true"
@call reformat_file(file="{{ file }}") /
@else
@call rustfmt_file(file="{{ file }}") /
@if-end
@define-end
