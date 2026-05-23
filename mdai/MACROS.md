---
lib_version: "0.1.0"
released: 2026-05-24
status: pre-stable
requires:
  mcp__lean-ctx: ">=3.6.16"
  mcp__markdownai: ">=0.0.24"
---

# mdai-macro-library — Index v0.1.0

Quelle der Wahrheit für alle versionierten mdai-Macros. Konsumiert von den drei mdai-Skills (`mdai-brainstorm`, `mdai-execution`, `mdai-memory`) und jedem generierten `.mdai.md`-Plan via `@call mdai_bootstrap()` + `@import`/`@include`.

## Inventar v0.1.0

| Datei | Mode | Opt-In | Exports |
|---|---|---|---|
| `core/startup-check.md` | import-only | always | service_check, detect_project_lang, detect_tooling, load_lang_pack, load_tooling_packs, mdai_bootstrap |
| `core/hard-rules.md` | include | always | (text only) |
| `core/tool-quick-ref.md` | include | always | (text only) |
| `core/ctx-tools.md` | import-only | always | ctx_read, ctx_search, ctx_tree, ctx_shell, ctx_edit |
| `core/mcp-markdownai.md` | import-only | always | read_phase, list_phases, get_constraints |
| `core/ctx-knowledge.md` | import-only | always | remember_plan, recall_plan |
| `core/gotchas.md` | import-only | always | add_gotcha, list_gotchas |
| `lang/rust.md` | import-only | MDAI_PROJECT_LANG=rust | cargo_nextest, cargo_clippy, cargo_fmt |
| `tooling/jetbrains.md` | import-only | MDAI_HAS_JETBRAINS=true | reformat_file, step_reformat_commit |
| `tooling/serena.md` | import-only | MDAI_HAS_SERENA=true | find_symbol, replace_symbol_body, insert_before_symbol, insert_after_symbol, symbols_overview |
| `skills/mdai-brainstorm/write-spec.md` | import-only | skill A only | write_spec, render_spec |
| `skills/mdai-brainstorm/write-mdai-plan.md` | import-only | skill A only | plan_frontmatter, plan_phase, plan_step, write_mdai_plan |
| `skills/mdai-brainstorm/spec-reviewer.md` | import-only | skill A only | spec_reviewer_prompt |

## Conventions

- **Frontmatter pro Pack-File:** siehe Spec §9.2 / Anhang A. Jedes File hat `lib_version`, `mdai-pack: { mode, exports }`. Optional `status: experimental` für Staging, `deprecated_since: 0.x` für Deprecation.
- **`mode: include`** rendert Inline-Text + lädt `@define`s. Wird genutzt für Regel-Files (hard-rules, tool-quick-ref).
- **`mode: import-only`** lädt nur `@define`s, kein Inline-Output. Default für alle Macro-Files.
- **Naming:** `snake_case` für Macro-Namen (`write_spec`, nicht `writeSpec`). `kebab-case` für Filenames (`write-spec.md`).
- **Bootstrap:** Jeder konsumierende Skill ruft `@call mdai_bootstrap()` als erste Zeile in `pre-context`. Setzt `ctx_session`-Flags für MCP-Liveness + Projekt-Typ + Tooling.

## Changelog

### v0.1.0 — 2026-05-24

Initial release.

- **Cross-skill core (7 Files):** startup-check, hard-rules, tool-quick-ref, ctx-tools, mcp-markdownai, ctx-knowledge, gotchas.
- **Opt-in lang/tooling (3 Files):** rust, jetbrains, serena.
- **Skill A Pack (3 Files):** write-spec, write-mdai-plan, spec-reviewer (migriert aus inline `@define`s im Skill-A-Spec §6.1).

**Hinweis Skill A:** Skill-A-Spec (`docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md`) MUSS in separater Patch-Session aktualisiert werden, BEVOR Skill-A-A1 (Impl-Start) läuft — siehe library-spec §10. Skill A ist nach diesem Release render-broken bis Patch-Session durch ist (beabsichtigt, A9-Cleanup-Entscheidung).
