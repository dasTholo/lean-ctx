---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [remember_plan, recall_plan, add_gotcha, list_gotchas, list_auto_gotchas, gotcha_stats]
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

# User-curated gotchas: category="mdai-gotcha". Single source of truth — the
# former docs/mdai/GOTCHAS.md was migrated into this store on 2026-05-24.
# Schema: key=g-<slug>, symptom=<short>, mitigation=<actionable>.

@define add_gotcha(key, symptom, mitigation)
@query mcp lean-ctx ctx_knowledge action="remember" category="mdai-gotcha" key="{{ key }}" value="symptom: {{ symptom }} | mitigation: {{ mitigation }}"
@end

@define list_gotchas(query)
@query mcp lean-ctx ctx_knowledge action="recall" category="mdai-gotcha" query="{{ query | default('') }}"
@end

# lean-ctx auto-tracked gotchas: separate GotchaStore in core::gotcha_tracker.
# Surfaces errors_detected, fixes_correlated, bugs_prevented, promoted_to_knowledge.
# Independent from the user-curated mdai-gotcha store above.

@define list_auto_gotchas()
@query mcp lean-ctx ctx_shell command="lean-ctx gotchas list"
@end

@define gotcha_stats()
@query mcp lean-ctx ctx_shell command="lean-ctx gotchas stats"
@end
