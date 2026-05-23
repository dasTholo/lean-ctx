@markdownai v1.0

## Tool-Quick-Reference

Bevorzugung: `@call <macro>` aus `mdai/core/*.md` und `mdai/tooling/*.md` > native MCP-Strings > native Bash/Read.

| Aufgabe                      | Macro (bevorzugt)                                  | Fallback MCP / native                                   |
|------------------------------|----------------------------------------------------|---------------------------------------------------------|
| Datei lesen                  | `@call ctx_read(path, mode)`                       | `mcp__lean-ctx__ctx_read`                               |
| Pattern-Suche                | `@call ctx_search(pattern, path)`                  | `mcp__lean-ctx__ctx_search` / `rg`                      |
| Verzeichnis-Listing          | `@call ctx_tree(path, depth)`                      | `mcp__lean-ctx__ctx_tree` / `ls`                        |
| Shell                        | `@call ctx_shell(cmd)`                             | `mcp__lean-ctx__ctx_shell`                              |
| Datei-Edit (kein Read nötig) | `@call ctx_edit(path, old, new)`                   | `mcp__lean-ctx__ctx_edit`                               |
| Reformat vor git add         | `@call reformat_file(file)`                        | `mcp__jetbrains__reformat_file`                         |
| Composite Reformat + Commit  | `@call step_reformat_commit(file, message)`        | — (Library-only)                                        |
| Rust-Symbol-Body lesen       | `@call find_symbol(name, path, include_body=true)` | `mcp__serena__jet_brains_find_symbol`                   |
| Rust-Symbol ersetzen         | `@call replace_symbol_body(name, path, body)`      | `mcp__serena__replace_symbol_body`                      |
| Rust-Symbol einfügen         | `@call insert_after_symbol` / `_before_symbol`     | `mcp__serena__insert_*_symbol`                          |
| Datei-Inventar               | `@call symbols_overview(path)`                     | `mcp__serena__jet_brains_get_symbols_overview`          |
| Plan-Phase lesen             | `@call read_phase(plan, phase_id)`                 | `mcp__markdownai__read_file file=... phase=...`         |
| Plan-Phasen listen           | `@call list_phases(plan)`                          | `mcp__markdownai__list_phases`                          |
| Plan-Constraints             | `@call get_constraints(plan)`                      | `mcp__markdownai__get_constraints`                      |
| Plan-State persist           | `@call remember_plan(id, body)`                    | `mcp__lean-ctx__ctx_knowledge action=remember`          |
| Plan-State recall            | `@call recall_plan(id)`                            | `mcp__lean-ctx__ctx_knowledge action=recall`            |
| Gotcha hinzufügen            | `@call add_gotcha(tag, title, body)`               | edit `docs/mdai/GOTCHAS.md`                             |
| Gotcha listen                | `@call list_gotchas(tag)`                          | grep `docs/mdai/GOTCHAS.md`                             |
| Cargo Tests                  | `@call cargo_nextest()`                            | `cargo nextest run`                                     |
| Cargo Lint                   | `@call cargo_clippy()`                             | `cargo clippy --workspace --all-targets -- -D warnings` |
| Cargo Format                 | `@call cargo_fmt()`                                | `cargo fmt`                                             |
