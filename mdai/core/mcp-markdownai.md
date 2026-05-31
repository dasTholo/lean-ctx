---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [read_phase, list_phases, get_constraints]
---

@markdownai v1.0

@define read_phase(plan, phase_id)
@query mcp markdownai read_file file="{{ plan }}" phase="{{ phase_id }}" /
@define-end

@define list_phases(plan)
@query mcp markdownai list_phases file="{{ plan }}" /
@define-end

@define get_constraints(plan)
@query mcp markdownai get_constraints file="{{ plan }}" /
@define-end

## Engine resolution notes (markdownai 1.3.0)

Verified against the live 1.3.0 engine. They matter for every agent/session authoring or running mdai packs.

> **Dist-Abhängigkeit:** Library benötigt den Dist aus Branch `feat-mdai` (= `origin/main@aac0825` + 2 lokale Fixes `f16b4c2`+`ede9793`). Nach Pull `npm --prefix markdownai run build`; verifizieren via findings-v3 Anhang A (Repro-Smokes). Fixes sind bewusst nicht gepusht.

### @include path resolution differs by entrypoint

- **MCP** (`call_macro` / `resolve_phase`, `cwd` = repo root): `@include` resolves relative to the **repo root**, and `${MDAI_LIBRARY_ROOT}` is expanded. Use `@include ${MDAI_LIBRARY_ROOT}/core/…/file.md /` for cross-pack includes — this is the runtime path and the library convention (`body.mdai.md`, `startup-check.md`).
- **CLI** (`npx mai render` / `validate`): `@include` resolves relative to the **including document's own directory**, `${MDAI_LIBRARY_ROOT}` is **NOT** expanded, and the parser rejects any `..` path segment.
- Consequence: an unconditional `${MDAI_LIBRARY_ROOT}/…` `@include` in a macro body works at MCP runtime, but `mai validate` reports `@include: file not found` for it (no env expansion; eager doc-relative resolution). This is a `validate` tooling limitation, not a runtime defect — verify such files via MCP `call_macro` / `resolve_phase`. (Existing library includes avoid the validate error only because they sit inside `@if` / `@switch`, which validate skips.)
- An included fragment's own `@markdownai` header is NOT leaked into output.

### @date is not an interpolation builtin

`{{ @date … }}` renders empty (CLI and MCP). Capture via `@set d = @date format='YYYY-MM-DD' /`, then reference `{{ d }}`. `now_iso()` works only via CLI (MCP `@eval` is blocked) and returns a full ISO timestamp.

### @foreach over object lists

`@set xs = {{ [{"key":"val"}] }} /` (JSON in `{{ }}`, quoted keys) is required for `{{ x.key }}` dot-access; a bare `[{key=val}]` is stringified and comma-split.

### CLI must run from repo root

`npx --prefix markdownai mai render/validate <path>` from the repo root. Running from inside `markdownai/` triggers "Path traversal above document root".
