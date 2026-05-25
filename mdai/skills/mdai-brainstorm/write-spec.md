---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [write_spec, render_spec]
---

@markdownai v1.0

# Skill-A Pack: write_spec / render_spec

@define write_spec(slug, body)
@if file.exists "docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"
- ABORT: Spec file already exists at docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md
- Choose a different slug, delete the existing file first, or amend the body in place.
- Not overwriting to prevent silent data loss.
@else
@query mcp lean-ctx ctx_shell command="
mkdir -p docs/mdai/specs &&
SPEC_PATH=docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md &&
cat > \"$SPEC_PATH\" <<'SPEC_EOF'
{{ body }}
SPEC_EOF
echo \"wrote $SPEC_PATH\"
"
@endif
@end

@define render_spec(slug, target)
@if {{ target }} == "none"

# no-op

@elseif {{ target }} == "chat"
@if file.exists "docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"
@query mcp markdownai read_file file="docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"
@else
- ERROR: Cannot render — spec file does not exist at docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md
- Call write_spec(slug, body) first.
@endif
@elseif {{ target }} == "file"
@if file.exists "docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"
@query mcp lean-ctx ctx_shell command="mkdir -p docs/mdai/specs/rendered && (cd markdownai && npx mai render \"../docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md\" > \"../docs/mdai/specs/rendered/{{ @date format='YYYY-MM-DD' }}-{{ slug }}.rendered.md\")"
@else
- ERROR: Cannot render — spec file does not exist at docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md
- Call write_spec(slug, body) first.
@endif
@endif
@end
