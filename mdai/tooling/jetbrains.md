---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [ reformat_file, step_reformat_commit, get_file_errors ]
---

@markdownai v1.0

# JetBrains Pack (opt-in via MDAI_HAS_JETBRAINS=true)

@define reformat_file(file)
@query mcp jetbrains reformat_file path="{{ file }}" /
@define-end

@define step_reformat_commit(file, message)
@call reformat_file(file="{{ file }}") /
@call ctx_shell(cmd="git add {{ file }}") /
@call ctx_shell(cmd="git commit -m '{{ message }}'") /
@define-end

# IDE inspection results for a single file (errors + warnings via IntelliJ

# inspections). errors_only=true suppresses warnings. Returns problems with

# severity, description, and 1-based line/column.

@define get_file_errors(file, errors_only)
@query mcp jetbrains get_file_problems filePath="{{ file }}" errorsOnly="{{ errors_only | default('false') }}" /
@define-end
