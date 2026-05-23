@markdownai v1.0

## Hard Rules (aus `CLAUDE.md`, immer-an)

- Tests: **immer** `cargo nextest run`, nie `cargo test`.
- Vor `git add`: `@call step_reformat_commit(file=<path>, message=<msg>)` (lädt `tooling/jetbrains.md`).
- **Keine** `&&`-Bash-Chains — jeden Befehl einzeln.
- **Keine** Worktrees.
- Rust-Edits: bevorzugt `@call replace_symbol_body(name=..., path=..., body=...)` / `insert_*_symbol` aus `tooling/serena.md`.
- lean-ctx-Tools bevorzugen: `@call ctx_read`, `@call ctx_search`, `@call ctx_shell`, `@call ctx_tree`, `@call ctx_edit` (aus `core/ctx-tools.md`).
