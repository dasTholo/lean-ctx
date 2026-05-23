@markdownai v1.0

## Tool Quick Reference

Preference: `@call <macro>` from `mdai/core/*.md` and `mdai/tooling/*.md` > native MCP strings > native Bash/Read.

| Task                        | Macro (preferred)                                  | Fallback MCP / native                                               |
|-----------------------------|----------------------------------------------------|---------------------------------------------------------------------|
| Read file                   | `@call ctx_read(path, mode)`                       | `mcp__lean-ctx__ctx_read`                                           |
| Pattern search              | `@call ctx_search(pattern, path)`                  | `mcp__lean-ctx__ctx_search` / `rg`                                  |
| Directory listing           | `@call ctx_tree(path, depth)`                      | `mcp__lean-ctx__ctx_tree` / `ls`                                    |
| Shell                       | `@call ctx_shell(cmd)`                             | `mcp__lean-ctx__ctx_shell`                                          |
| File edit (no read needed)  | `@call ctx_edit(path, old, new)`                   | `mcp__lean-ctx__ctx_edit`                                           |
| Reformat before git add     | `@call reformat_file(file)`                        | `mcp__jetbrains__reformat_file`                                     |
| Composite reformat + commit | `@call step_reformat_commit(file, message)`        | — (library-only)                                                    |
| Read Rust symbol body       | `@call find_symbol(name, path, include_body=true)` | `mcp__serena__jet_brains_find_symbol`                               |
| Replace Rust symbol         | `@call replace_symbol_body(name, path, body)`      | `mcp__serena__replace_symbol_body`                                  |
| Insert Rust symbol          | `@call insert_after_symbol` / `_before_symbol`     | `mcp__serena__insert_*_symbol`                                      |
| File symbol overview        | `@call symbols_overview(path)`                     | `mcp__serena__jet_brains_get_symbols_overview`                      |
| Read plan phase             | `@call read_phase(plan, phase_id)`                 | `mcp__markdownai__read_file file=... phase=...`                     |
| List plan phases            | `@call list_phases(plan)`                          | `mcp__markdownai__list_phases`                                      |
| Plan constraints            | `@call get_constraints(plan)`                      | `mcp__markdownai__get_constraints`                                  |
| Persist plan state          | `@call remember_plan(id, body)`                    | `mcp__lean-ctx__ctx_knowledge action=remember category=plan`        |
| Recall plan state           | `@call recall_plan(id)`                            | `mcp__lean-ctx__ctx_knowledge action=recall category=plan`          |
| Add mdai-gotcha             | `@call add_gotcha(key, symptom, mitigation)`       | `mcp__lean-ctx__ctx_knowledge action=remember category=mdai-gotcha` |
| List mdai-gotchas           | `@call list_gotchas(query)`                        | `mcp__lean-ctx__ctx_knowledge action=recall category=mdai-gotcha`   |
| List auto-tracked gotchas   | `@call list_auto_gotchas()`                        | `lean-ctx gotchas list`                                             |
| Gotcha bug-memory stats     | `@call gotcha_stats()`                             | `lean-ctx gotchas stats`                                            |
| Cargo tests                 | `@call cargo_nextest()`                            | `cargo nextest run`                                                 |
| Cargo lint                  | `@call cargo_clippy()`                             | `cargo clippy --workspace --all-targets -- -D warnings`             |
| Cargo format                | `@call cargo_fmt()`                                | `cargo fmt`                                                         |
