---
lib_version: 0.1.1
mdai-pack:
  mode: import-only
  exports: [ lean_context_audit ]
---

@markdownai v1.0

@define lean_context_audit(spec_path)

# Lean-Context Audit für {{ spec_path }}

Search `{{ spec_path }}` for each anchor below. For each:

- if found AND adjacent `@note visible consumer="human"` block exists → OK
- if found WITHOUT justification → FLAG as lean-context violation

## 6 Anchors

<!-- drift: mode="full" rule canonical in core/lean-context.md (Defaults/Exceptions table, ctx_read rows) -->
@include ${MDAI_LIBRARY_ROOT}/core/_fragments/lean-context-anchors.md /

Use `mcp__lean-ctx__ctx_search(pattern="<anchor>", path="{{ spec_path }}")` for each.
@define-end
