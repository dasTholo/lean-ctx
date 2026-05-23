---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [write_spec, render_spec]
---

@markdownai v1.0

# Skill-A Pack: write_spec / render_spec

@define write_spec(slug, body)
@query mcp lean-ctx ctx_shell command="
mkdir -p docs/mdai/specs &&
DATE=$(date -u +%Y-%m-%d) &&
SPEC_PATH=docs/mdai/specs/${DATE}-{{ slug }}-design.mdai.md &&
cat > \"$SPEC_PATH\" <<'SPEC_EOF'
{{ body }}
SPEC_EOF
echo \"wrote $SPEC_PATH\"
"
@end

@define render_spec(slug, target)
@if {{ target }} == "none"

# no-op

@elseif {{ target }} == "chat"
@query mcp markdownai read_file file="docs/mdai/specs/$(date -u +%Y-%m-%d)-{{ slug }}-design.mdai.md"
@elseif {{ target }} == "file"
@query mcp lean-ctx ctx_shell command="mkdir -p docs/mdai/specs/rendered && (cd /home/tholo/Scripts/lean-ctx/markdownai && npx mai render \"../docs/mdai/specs/$(
date -u +%Y-%m-%d)-{{ slug }}-design.mdai.md\" > \"../docs/mdai/specs/rendered/$(date -u +%Y-%m-%d)-{{ slug
}}.rendered.md\")"
@endif
@end
