# mdai-macro-library — Changelog

## v0.1.4 — 2026-05-31 — hardening (v0.1.3 follow-ups)

- **TG1 — Engine-Fix-Verifikation & Dependency-Doku:** zwei lokale Engine-Fixes (`f16b4c2`, `ede9793`) leben weiterhin nur auf `feat-mdai` (nicht in `origin/main`). Repro-Smokes ergänzt in findings-v3 Anhang A; Dist-Pointer in `core/mcp-markdownai.md`. Dist = `feat-mdai` (`origin/main@aac0825` + die 2 Fixes); nach Pull `npm --prefix markdownai run build`, verifizieren via Anhang A.
- **TG2 — `@set`-Pipe-Fix (`body.mdai.md`):** `@set render_target_resolved = render_target | default("none") /` entfernt (`@set` kann keine Pipe-Source sein); Use-Site auf inline-Interpolation umgestellt.
- **TG3 — Directive-Indentation:** alle echten `@`-Directives in `write-spec.md` und `body.mdai.md` auf Spalte 0 normalisiert (Verhalten unverändert).

## v0.1.3 — 2026-05-29 — markdownai-v2 adoption & library fix (engine 1.3.0)

- **Engine adoption:** library targets markdownai 1.3.0 (v2 directive syntax, plugin system).
- **P0 — closed the 3 migration-tool gaps manually:** predicate call-form `name(a, b)` across `core/file-utils.md`,
  `core/startup-check.md`, `skills/mdai-brainstorm/spec-reviewer.md`, `skills/mdai-brainstorm/write-spec.md`;
  `@foreach x in {{ list }}` in `startup-check.md` (`detect_tooling`, `load_tooling_packs`); unquoted interpolated
  predicate args (`file.exists({{ var }})`, `file.containsLine({{ p }}, "…")`).
- **Object-list dot-access:** `load_tooling_packs` uses `@set tooling_packs = {{ [{"name":..,"flag":..}] }} /` (JSON in
  `{{ }}`) so `{{ pack.name }}` / `{{ pack.flag }}` resolve; a bare `[{name=..}]` is stringified and comma-split.
- **A — §5 date workaround replaced:** `write-spec.md` drops the broken inline `{{ @date }}` (resolves empty) for the
  directive-valued `@set spec_date = @date format='YYYY-MM-DD' /` + `{{ spec_date }}`.
- **B — content dedup:** 6-anchor lean-context list extracted to canonical fragment
  `core/_fragments/lean-context-anchors.md`, consumed via `@include ${MDAI_LIBRARY_ROOT}/core/_fragments/...` by
  `lean-context-audit.md` + `spec-self-review.md` (Cluster 1); `lean-context.md` marked canonical anti-pattern source
  (Cluster 3); `mode="full"` drift comments added (Cluster 2).
- **C — docs:** `spec-directive-conventions.md` migrated to v2 (incl. corrected date guidance); `hard-rules.md` gains a
  v2 directive-syntax section; `mcp-markdownai.md` gains engine resolution notes; findings-v3 supersedes v2.
- **Two engine bugs fixed in markdownai (branch `feat-mdai`):** `f16b4c2` propagate macro named-args into skillContext
  for `@if file.exists({{ param }})`; `ede9793` bind `@foreach` object items into `ctx.data` for dot-access.

### Known limitations v0.1.3

- `@date` is NOT available inside `{{ }}` interpolation; use `@set d = @date format='YYYY-MM-DD' /` then `{{ d }}`.
  `now_iso()` is CLI-only (MCP `@eval` blocked) and returns a full timestamp.
- `@include` resolves repo-root-relative and expands `${MDAI_LIBRARY_ROOT}` under MCP, but is document-relative with no
  env-expansion under `mai validate` / `render` (and `..` is parser-rejected). Unconditional `${MDAI_LIBRARY_ROOT}/...`
  includes therefore fail `mai validate` — verify via MCP. See `core/mcp-markdownai.md`.
- `body.mdai.md:185` `@set render_target_resolved = render_target | default("none") /` — `@set` cannot be a pipe source
  in 1.3.0 (pre-existing; out of this scope, tracked in findings-v3).

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
