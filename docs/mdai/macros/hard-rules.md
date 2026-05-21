@markdownai v1.0

## Hard Rules (aus `CLAUDE.md`, immer-an)

- Tests: **immer** `cargo nextest run`, nie `cargo test`.
- Vor `git add`: `mcp__jetbrains__reformat_file` auf jede geänderte Datei.
- **Keine** `&&`-Bash-Chains — jeden Befehl einzeln.
- **Keine** Worktrees.
- Rust-Edits: bevorzugt `mcp__serena__replace_symbol_body` / `insert_*_symbol`.
- File-Moves (`mv`, `cp`) auf `.md`: über `mcp__lean-ctx__ctx_shell(cmd="...", raw=true)`.
