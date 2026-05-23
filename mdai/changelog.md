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
