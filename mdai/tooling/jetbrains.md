---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [reformat_file, step_reformat_commit]
---

@markdownai v1.0

# JetBrains Pack (opt-in via MDAI_HAS_JETBRAINS=true)

@define reformat_file(file)
@query mcp jetbrains reformat_file path="{{ file }}"
@end

@define step_reformat_commit(file, message)
@call reformat_file(file="{{ file }}")
@call ctx_shell(cmd="git add {{ file }}")
@call ctx_shell(cmd="git commit -m '{{ message }}'")
@end
