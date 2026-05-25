# mdai-macro-library — Changelog

## v0.1.0 — 2026-05-24

Initial release.

- **Cross-skill core (6 files):** startup-check, hard-rules, tool-quick-ref, ctx-tools, mcp-markdownai, ctx-knowledge.
- **Refactor during v0.1.0:** the former `gotchas` pack was merged into `ctx-knowledge` and aligned with the user
  convention from the (since-retired) `docs/mdai/GOTCHAS.md`: user-curated gotchas use
  `ctx_knowledge category="mdai-gotcha"` with schema `(key, symptom, mitigation)`. The `docs/mdai/GOTCHAS.md` file was
  deleted after its 5 entries were migrated into the store. Library also exposes wrappers for lean-ctx's internal
  auto-tracking gotcha CLI (`@call list_auto_gotchas` / `@call gotcha_stats` → `lean-ctx gotchas list/stats`). The
  original file-append wrapper was deleted.
- **Opt-in lang/tooling (3 files):** rust, jetbrains, serena.
- **Skill A pack (3 files):** write-spec, write-mdai-plan, spec-reviewer (migrated from inline `@define`s in skill-A
  spec §6.1).
- **Library scope (2026-05-24 clarification):** Library = `core/` + `lang/` + `tooling/` only (shared infrastructure
  consumed by multiple skills). Files under `skills/<skill-name>/` are **skill-owned assets**, not Library packs —
  documented in the owning skill's spec. Co-located under `mdai/` purely for filesystem proximity. Library v0.1.0 stays
  v0.1.0 regardless of skill-asset changes.

- **Additive update 2026-05-24:** `core/file-utils.md` added (exports: `file_check`). Cross-skill filesystem status
  helper — renders `- {{ path }} exists` or `- {{ path }} MISSING` based on `file.exists`. Use for inline status
  reporting in specs/plans (Pre-Flight-Checks, Verification-Logs); **NOT** for control-flow (for branching use inline
  `@if file.exists "..."` at the call site per README Z 1095). Pattern source: `markdownai/README.md` Z 282-293.
  Additive only — no breaking changes, library remains v0.1.0 (pre-stable).

- **Additive update 2026-05-24 (`MACROS.md` removed):** `mdai/MACROS.md` deleted. The inventory was redundant (skills
  discover available macros via `ctx_search` on `core/*` + `tooling/*` + `lang/*` directly) and the conventions block
  was duplicated elsewhere — the only unique bullet (Naming: `snake_case` macros, `kebab-case` files) was migrated to
  `core/lean-context.md` `## Conventions`. Canonical library version is now tracked per-pack via each file's
  `lib_version: 0.1.0` frontmatter, plus the section headers in this changelog. Skill / spec references to
  `mdai/MACROS.md` were removed in the same patch session.

- **Additive update 2026-05-24 (Lean-Context-Discipline):** `core/lean-context.md` added (mode: include, text only —
  no YAML frontmatter, no `@define` blocks; consumed via `@include mdai/core/lean-context.md` to render the rules
  table inline anywhere). Single source of truth for bounded-read + lean-shell + cache-bypass defaults across all
  mdai skills and generated specs. Rules table covers `ctx_read` (cross-file scan → `map`/`signatures`;
  after-search → `lines:N-M`; spec-review target → `full`), `ctx_shell` (compressed default, `raw=true` exception),
  `@include` (`lines=N-M` for targeted blocks), Serena `find_symbol` (`body=false` default), and
  `ctx_read fresh=true` (only IMMEDIATELY after a write/edit to that path; otherwise the cache auto-invalidates
  via mtime). Wrappers implementing the bounded-read modes (`ctx_read_lines`, `ctx_read_map`,
  `ctx_read_signatures`) were added to `core/ctx-tools.md` and referenced in `core/tool-quick-ref.md` — they live
  with the other `ctx_*` wrappers, not in the rules doc. Skill-A `spec-reviewer.md` simultaneously gains
  anti-pattern checks #9 (markdownai-directives-active), #10 (lean-context-defaults) and
  #11 (structured-data-via-@read/@list). Additive only — no breaking changes, library remains v0.1.0 (pre-stable).
  Design: `docs/mdai/specs/2026-05-24-mdai-brainstorm-design.mdai.md` §5.7.

### Known limitations v0.1.0

- `mai` CLI does not execute `@query` directives — live MCP probes (cache hit / service-fail / lang-detection) are
  observable only from a Claude Code session with active MCP servers, not from the CLI render. Details:
  `docs/mdai/green-verification/library/v0.1.0-bootstrap-findings.md`.
- Bootstrap cache uses `ctx_session action="finding" / "status"` (session-scoped). The cache is automatically
  invalidated when the chat session restarts. To force re-detection in the same session, run
  `ctx_session action="reset"` (note: this also clears other session state). Cache marker format:
  `[mdai-bootstrap-cache] tooling=detected lang=<LANG> jetbrains=<bool> serena=<bool>`.
- `mode: include` files must not carry YAML frontmatter (markdownai parser leaks frontmatter as text via `@include`).
- Two coexisting gotcha stores (intentional): (a) `ctx_knowledge category="mdai-gotcha"` for user-curated entries via
  `@call add_gotcha`; (b) `lean-ctx gotchas` CLI for auto-tracked errors / bug-memory stats. They are independent stores
  by design — (a) captures convention/policy, (b) captures observed incidents.

### Skill A status

The skill-A spec (`docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md`) MUST be updated in a separate patch
session BEFORE skill-A A1 (implementation start) runs — see library spec §10. Skill A is render-broken after this
release until the patch session completes (intentional, A9 cleanup decision).
