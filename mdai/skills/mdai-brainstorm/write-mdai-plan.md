---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [plan_frontmatter, plan_phase, plan_step, write_mdai_plan]
---

@markdownai v1.0

# Skill-A Pack: write_mdai_plan + helpers

@define plan_frontmatter(id, spec)
---
id: {{ id }}
plan_for: {{ spec }}
created: $(date -u +%Y-%m-%d)
---
@end

@define plan_step(check, body)

- [{{ check | default(' ') }}] {{ body }}
  @end

@define plan_phase(id, title, files, steps)

## Phase {{ id }}: {{ title }}

**Files:**
{{ files }}

**Steps:**
{{ steps }}
@end

@define write_mdai_plan(slug, phases)
@query mcp lean-ctx ctx_shell command="
mkdir -p docs/mdai/plans &&
DATE=$(date -u +%Y-%m-%d) &&
PLAN_PATH=docs/mdai/plans/${DATE}-{{ slug }}.mdai.md &&
cat > \"$PLAN_PATH\" <<'PLAN_EOF'
{{ phases }}
PLAN_EOF
echo \"wrote $PLAN_PATH\"
"
@end
