@markdownai v1.0

## Hard Rules (from `CLAUDE.md`, always-on)

- Tests: **always** `cargo nextest run`, never `cargo test`.
- Before `git add`: `@call step_reformat_commit(file=<path>, message=<msg>)` (loads `tooling/jetbrains.md`).
- **No** `&&` bash chains — issue each command separately.
- **No** worktrees.
- Rust edits: prefer `@call replace_symbol_body(name=..., path=..., body=...)` / `insert_*_symbol` from
  `tooling/serena.md`.
- Prefer lean-ctx tools: `@call ctx_shell`, `@call ctx_tree`, `@call ctx_edit` (
  from `core/ctx-tools.md`).
- **Lean-context defaults** (`core/lean-context.md` — `@include` to render the rules table inline): bounded reads by
  default — `@call ctx_read_map(path)` / `ctx_read_signatures(path)` for scans, `@call ctx_read_lines(path, start, end)`
  after `ctx_search` / `find_symbol`. `mode="full"` is the exception (justify with `@note visible consumer="human"`).
  `ctx_shell raw=true` and `ctx_read fresh=true` are also exceptions — `fresh=true` ONLY immediately after a write/edit
  to the same path (cache auto-invalidates via mtime).
