@markdownai v1.0

## Anhang — Tool-Quick-Reference

| Rust-Symbol-Body lesen | `mcp__serena__jet_brains_find_symbol <name> <path> include_body=true` |
| Rust-Symbol ersetzen | `mcp__serena__replace_symbol_body <name> <path>`                      |
| Rust-Symbol einfügen | `mcp__serena__insert_after_symbol` / `_before_symbol`                 |
| Datei-Inventar (alle Top-Level) | `mcp__serena__jet_brains_get_symbols_overview <path>`                 |
| Markdown-Edit | `mcp__lean-ctx__ctx_edit <path> <old> <new>`                          |
| Plain-Text-Lookup | `mcp__lean-ctx__ctx_search <pattern> <path>`                          |
| Datei-Range lesen | `mcp__lean-ctx__ctx_read <path> mode=lines:N-M`                       |
| Reformat (vor `git add`)        | `mcp__jetbrains__reformat_file <path>`                                |
| Shell | `mcp__lean-ctx__ctx_shell <cmd>` (oder `raw=true` für mv/cp)          |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings`               |
