---
lib_version: 0.1.1
mdai-pack:
  mode: import-only
  exports: [ library_spec_audit ]
---

@markdownai v1.0

@define library_spec_audit(spec_path)

# Library-Spec Audit für {{ spec_path }}

Each check verifies the spec covers a key library-pack concern. For each:

- run `mcp__lean-ctx__ctx_search(pattern="<anchor>", path="{{ spec_path }}")` (or frontmatter-field check)
- if PRESENT → ✓
- if MISSING → flag with guidance

## 7 Checks

- [ ] **#1 MCP-Signatur-Verifikation** — search anchor: `MCP-Signatur-Verifikation`
  Guidance: Add a section enumerating MCP call signatures used by exported macros.

- [ ] **#2 mode: import-only declaration** — search anchor: `mode: import-only`
  Guidance: Each new pack must declare its mdai-pack mode in frontmatter.

- [ ] **#3 Render-Flow-Tests** — search anchor: `Render-Flow-Tests`
  Guidance: Spec should list call_macro / resolve_phase smoke-tests per exported macro.

- [ ] **#4 @constraint id=** — search anchor: `@constraint id=`
  Guidance: Use `@constraint id=... severity=...` for machine-readable rules.

- [ ] **#5 lib_version** (frontmatter field) — verify `lib_version:` is set in spec frontmatter
  Guidance: Bump `lib_version` when pack contents change. Spec should mention target version.

- [ ] **#6 Discipline §10.4** — search anchor: `Discipline §10.4`
  Guidance: Library specs should map back to brainstorm-Discipline mismatches if any.

- [ ] **#7 Drift-Tracking** — search anchor: `Drift-Tracking`
  Guidance: Hand-ported blocks must carry Drift-Tracking comment with source provenance.

Use `{{ spec_path }}` in your `ctx_search` invocations.
@define-end
