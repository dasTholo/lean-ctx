---
lib_version: "0.1.0"
released: 2026-05-24
status: pre-stable
requires:
  mcp__lean-ctx: ">=3.6.16"
  mcp__markdownai: ">=0.0.24"
---

# mdai-macro-library — Index v0.1.0

Source of truth for all versioned mdai macros. Consumed by the three mdai skills (`mdai-brainstorm`, `mdai-execution`,
`mdai-memory`) and every generated `.mdai.md` plan via `@call mdai_bootstrap()` + `@import`/`@include`.

## Inventory v0.1.0

| File                                        | Mode        | Opt-In                  | Exports                                                                                                |
|---------------------------------------------|-------------|-------------------------|--------------------------------------------------------------------------------------------------------|
| `core/startup-check.md`                     | import-only | always                  | service_check, detect_project_lang, detect_tooling, load_lang_pack, load_tooling_packs, mdai_bootstrap |
| `core/hard-rules.md`                        | include     | always                  | (text only)                                                                                            |
| `core/tool-quick-ref.md`                    | include     | always                  | (text only)                                                                                            |
| `core/ctx-tools.md`                         | import-only | always                  | ctx_read, ctx_search, ctx_tree, ctx_shell, ctx_edit                                                    |
| `core/mcp-markdownai.md`                    | import-only | always                  | read_phase, list_phases, get_constraints                                                               |
| `core/ctx-knowledge.md`                     | import-only | always                  | remember_plan, recall_plan, add_gotcha, list_gotchas                                                   |
| `lang/rust.md`                              | import-only | MDAI_PROJECT_LANG=rust  | cargo_nextest, cargo_clippy, cargo_fmt, rustfmt_file                                                                 |
| `tooling/jetbrains.md`                      | import-only | MDAI_HAS_JETBRAINS=true | reformat_file, step_reformat_commit, get_file_errors                                                                    |
| `tooling/serena.md`                         | import-only | MDAI_HAS_SERENA=true    | find_symbol, replace_symbol_body, insert_before_symbol, insert_after_symbol, symbols_overview          |
| `skills/mdai-brainstorm/write-spec.md`      | import-only | skill A only            | write_spec, render_spec                                                                                |
| `skills/mdai-brainstorm/write-mdai-plan.md` | import-only | skill A only            | plan_frontmatter, plan_phase, plan_step, write_mdai_plan                                               |
| `skills/mdai-brainstorm/spec-reviewer.md`   | import-only | skill A only            | spec_reviewer_prompt                                                                                   |

## Conventions

- **Per-pack frontmatter:** see library spec §9.2 / Appendix A. Every file has `lib_version` and
  `mdai-pack: { mode, exports }`. Optional `status: experimental` for staging, `deprecated_since: 0.x` for deprecation.
- **`mode: include`** renders inline text + loads `@define`s. Used for rule files (hard-rules, tool-quick-ref). These
  files MUST NOT carry YAML frontmatter — `@include` walks all AST nodes and would leak the frontmatter as text. Mode is
  determined at the use-site via the `@include` directive.
- **`mode: import-only`** loads only `@define`s, no inline output. Default for all macro files. YAML frontmatter is safe
  here (`@import` only processes define/env/connect/import nodes; text is ignored).
- **Naming:** `snake_case` for macro names (`write_spec`, not `writeSpec`). `kebab-case` for filenames (
  `write-spec.md`).
- **Bootstrap:** Every consuming skill calls `@call mdai_bootstrap()` as the first line of its `pre-context`. Per-render
  only in v0.1.0 (no cache — see changelog).

