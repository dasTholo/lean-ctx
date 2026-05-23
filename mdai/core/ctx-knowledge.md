---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [remember_plan, recall_plan]
---

@markdownai v1.0

@define remember_plan(plan_id, body)
@query mcp lean-ctx ctx_knowledge action="remember" key="plan:{{ plan_id }}" body="{{ body }}"
@end

@define recall_plan(plan_id)
@query mcp lean-ctx ctx_knowledge action="recall" key="plan:{{ plan_id }}"
@end
