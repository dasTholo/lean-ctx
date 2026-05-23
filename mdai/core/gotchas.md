---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [add_gotcha, list_gotchas]
---

@markdownai v1.0

@define add_gotcha(tag, title, body)
@query mcp lean-ctx ctx_shell command="cat >> docs/mdai/GOTCHAS.md <<'GOTCHA'

### [{{ tag }}] {{ title }}

{{ body }}
GOTCHA
"
@end

@define list_gotchas(tag)
@query mcp lean-ctx ctx_search pattern="^### \\[{{ tag }}\\]" path="docs/mdai/GOTCHAS.md"
@end
