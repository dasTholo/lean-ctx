# mdai-macro-library — Changelog

## v0.1.0 — 2026-05-24

Initial release.

- **Cross-skill core (6 files):** startup-check, hard-rules, tool-quick-ref, ctx-tools, mcp-markdownai, ctx-knowledge.
- **Refactor during v0.1.0:** the former `gotchas` pack was merged into `ctx-knowledge` (`category="gotcha"` in `ctx_knowledge` is a first-class salience category, score 75; the file-append wrapper was deleted).
- **Opt-in lang/tooling (3 files):** rust, jetbrains, serena.
- **Skill A pack (3 files):** write-spec, write-mdai-plan, spec-reviewer (migrated from inline `@define`s in skill-A spec §6.1).

### Known limitations v0.1.0

- `mai` CLI does not execute `@query` directives — live MCP probes (cache hit / service-fail / lang-detection) are observable only from a Claude Code session with active MCP servers, not from the CLI render. Details: `docs/mdai/green-verification/library/v0.1.0-bootstrap-findings.md`.
- No bootstrap cache: `ctx_session` does not provide an arbitrary key-value store (only structured task/finding/decision actions). Each render re-runs the bootstrap probes (acceptable per library spec §14).
- `mode: include` files must not carry YAML frontmatter (markdownai parser leaks frontmatter as text via `@include`).

### Skill A status

The skill-A spec (`docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md`) MUST be updated in a separate patch session BEFORE skill-A A1 (implementation start) runs — see library spec §10. Skill A is render-broken after this release until the patch session completes (intentional, A9 cleanup decision).
