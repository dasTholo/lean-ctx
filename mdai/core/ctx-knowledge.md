---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [remember_plan, recall_plan, add_gotcha, list_gotchas]
---

@markdownai v1.0

# ctx_knowledge wrappers (project-persistent state)

# Plan-state: category="plan", key=plan_id, value=body.
@define remember_plan(plan_id, body)
@query mcp lean-ctx ctx_knowledge action="remember" category="plan" key="{{ plan_id }}" value="{{ body }}"
@end

@define recall_plan(plan_id)
@query mcp lean-ctx ctx_knowledge action="recall" category="plan" query="{{ plan_id }}"
@end

# Gotchas: category="gotcha" (first-class in ctx_knowledge with salience 75).
# Replaces the deleted gotchas.md file-append wrapper.
@define add_gotcha(title, body)
@query mcp lean-ctx ctx_knowledge action="remember" category="gotcha" key="{{ title }}" value="{{ body }}"
@end

@define list_gotchas(query)
@query mcp lean-ctx ctx_knowledge action="recall" category="gotcha" query="{{ query | default('') }}"
@end
