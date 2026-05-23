---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [read_phase, list_phases, get_constraints]
---

@markdownai v1.0

@define read_phase(plan, phase_id)
@query mcp markdownai read_file file="{{ plan }}" phase="{{ phase_id }}"
@end

@define list_phases(plan)
@query mcp markdownai list_phases file="{{ plan }}"
@end

@define get_constraints(plan)
@query mcp markdownai get_constraints file="{{ plan }}"
@end
