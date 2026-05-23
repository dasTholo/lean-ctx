---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports:
    - find_symbol
    - replace_symbol_body
    - insert_before_symbol
    - insert_after_symbol
    - symbols_overview
---

@markdownai v1.0

# Serena Pack (opt-in via MDAI_HAS_SERENA=true)

@define find_symbol(name, path, include_body)
@query mcp serena jet_brains_find_symbol name_path="{{ name }}" relative_path="{{ path }}" include_body="{{ include_body | default('false') }}"
@end

@define replace_symbol_body(name, path, body)
@query mcp serena replace_symbol_body name_path="{{ name }}" relative_path="{{ path }}" body="{{ body }}"
@end

@define insert_before_symbol(name, path, body)
@query mcp serena insert_before_symbol name_path="{{ name }}" relative_path="{{ path }}" body="{{ body }}"
@end

@define insert_after_symbol(name, path, body)
@query mcp serena insert_after_symbol name_path="{{ name }}" relative_path="{{ path }}" body="{{ body }}"
@end

@define symbols_overview(path)
@query mcp serena jet_brains_get_symbols_overview relative_path="{{ path }}"
@end
