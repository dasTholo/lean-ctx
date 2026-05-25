---
lib_version: 0.1.1
mdai-pack:
  mode: import-only
  exports: [lean_context_audit]
---
@markdownai v1.0

@define lean_context_audit(spec_path)
# Lean-Context Audit für {{ spec_path }}

Search `{{ spec_path }}` for each anchor below. For each:
- if found AND adjacent `@note visible consumer="human"` block exists → OK
- if found WITHOUT justification → FLAG as lean-context violation

## 6 Anchors
- [ ] `mode="full"` — only allowed for the one spec-source read; flag all others.
- [ ] `raw=true` — every `ctx_shell raw=true` needs `@note visible consumer="human"`.
- [ ] `fresh=true` — only valid immediately after a write/edit to the same path.
- [ ] `Grep` / `rg ` — lean-ctx violation; replace with `@call ctx_search(...)`.
- [ ] `cat ` / `head ` / `tail ` — lean-ctx violation; replace with `@call ctx_read(...)`.
- [ ] `bash ` / `sh ` — lean-ctx violation; replace with `@call ctx_shell(...)`.

Use `mcp__lean-ctx__ctx_search(pattern="<anchor>", path="{{ spec_path }}")` for each.
@end
