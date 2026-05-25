---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [ctx_read, ctx_search, ctx_tree, ctx_shell, ctx_edit, ctx_read_lines, ctx_read_map, ctx_read_signatures]
---

@markdownai v1.0

@define ctx_read(path, mode)
@query mcp lean-ctx ctx_read path="{{ path }}" mode="{{ mode | default('auto') }}"
@end

@define ctx_search(pattern, path)
@query mcp lean-ctx ctx_search pattern="{{ pattern }}" path="{{ path | default('.') }}"
@end

@define ctx_tree(path, depth)
@query mcp lean-ctx ctx_tree path="{{ path | default('.') }}" depth="{{ depth | default(3) }}"
@end

@define ctx_shell(cmd)
@query mcp lean-ctx ctx_shell command="{{ cmd }}"
@end

@define ctx_edit(path, old, new)
@query mcp lean-ctx ctx_edit path="{{ path }}" old_string="{{ old }}" new_string="{{ new }}"

@define ctx_read_lines(path, start, end)
@query mcp lean-ctx ctx_read path="{{ path }}" mode="lines:{{ start }}-{{ end }}"

@define ctx_read_map(path)
@query mcp lean-ctx ctx_read path="{{ path }}" mode="map"

@define ctx_read_signatures(path)
@query mcp lean-ctx ctx_read path="{{ path }}" mode="signatures"
@end
