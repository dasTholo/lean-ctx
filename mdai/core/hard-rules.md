@markdownai v1.0

## Hard Rules (from `CLAUDE.md`, always-on)

- Tests: **always** `cargo nextest run`, never `cargo test`.
- Before `git add`: `@call step_reformat_commit(file=<path>, message=<msg>)` (loads `tooling/jetbrains.md`).
- **No** `&&` bash chains — issue each command separately.
- **No** worktrees.
- Rust edits: prefer `@call replace_symbol_body(name=..., path=..., body=...)` / `insert_*_symbol` from `tooling/serena.md`.
- Prefer lean-ctx tools: `@call ctx_read`, `@call ctx_search`, `@call ctx_shell`, `@call ctx_tree`, `@call ctx_edit` (from `core/ctx-tools.md`).
