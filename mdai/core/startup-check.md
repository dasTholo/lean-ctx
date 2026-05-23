---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports:
    - service_check
    - detect_project_lang
    - detect_tooling
    - load_lang_pack
    - load_tooling_packs
    - mdai_bootstrap
    - mdai_bootstrap_check_cache
---

@markdownai v1.0

@define service_check(service, mcp_tool, required)
@query mcp {{ service }} {{ mcp_tool }}
@if @result.success
[mdai-bootstrap OK] {{ service }} MCP reachable
@else
@if {{ required }} == "true"
[mdai-bootstrap FAIL] required service '{{ service }}' MCP unreachable.
Reason: {{ @result.error | default("no response") }}
Action: run `/mcp` to inspect connection, reconnect, then re-trigger skill.
Blocking: skill cannot continue without '{{ service }}'.
@query mcp lean-ctx ctx_shell command="exit 1"
@else
[mdai-bootstrap WARN] optional service '{{ service }}' MCP unreachable — skipping {{ service }} pack.
Reason: {{ @result.error | default("no response") }}
Impact: any later @call to {{ service }}-pack macros will be a no-op.
@endif
@endif
@end

@define detect_project_lang()
@query mcp lean-ctx ctx_overview task="lang detect"

# Result contains WAKEUP block with "architecture/languages_top=<LANG>:<N>,...".

# Parse the first token before the first ":" as primary language.

@query mcp lean-ctx ctx_shell command="echo '{{ @result.stdout | default('') }}' | grep -oE '
architecture/languages_top=[a-z]+' | head -1 | cut -d= -f2"
@if @result.stdout != ""
[mdai-bootstrap] project lang detected via ctx_overview: {{ @result.stdout }}
@else

# last-resort shell heuristic

@query mcp lean-ctx ctx_shell command="
if [ -f Cargo.toml ]; then echo rust
elif [ -f pyproject.toml ] || [ -f setup.py ]; then echo python
elif [ -f package.json ]; then echo node
else echo unknown
fi
"
[mdai-bootstrap] project lang detected via shell heuristic: {{ @result.stdout }}
@endif
@end

@define detect_tooling()
@query mcp lean-ctx ctx_shell command="claude mcp list | grep -E 'jetbrains|serena' || true"

# Flags: MDAI_HAS_JETBRAINS, MDAI_HAS_SERENA.

# @if-logic matches @result.stdout directly (no ctx_session key-store in v0.1.0).

@if @result.stdout matches "jetbrains"
[mdai-bootstrap] jetbrains MCP available
@else
[mdai-bootstrap] jetbrains MCP NOT available
@endif
@if @result.stdout matches "serena"
[mdai-bootstrap] serena MCP available
@else
[mdai-bootstrap] serena MCP NOT available
@endif
@end

@define load_lang_pack()
@if @env MDAI_PROJECT_LANG == "rust"
@include mdai/lang/rust.md
@elseif @env MDAI_PROJECT_LANG == "python"
@include mdai/lang/python.md
@elseif @env MDAI_PROJECT_LANG == "node"
@include mdai/lang/node.md
@endif
@end

@define load_tooling_packs()
@if @env MDAI_HAS_JETBRAINS == "true"
@include mdai/tooling/jetbrains.md
@endif
@if @env MDAI_HAS_SERENA == "true"
@include mdai/tooling/serena.md
@endif
@end

# Session-scoped cache helper: probes ctx_session status for an existing
# [mdai-bootstrap-cache] finding. Sets @result.cache_hit (truthy on hit).
@define mdai_bootstrap_check_cache()
@query mcp lean-ctx ctx_session action="status"
@end

# Top-level orchestrator. Session-scoped cache via ctx_session findings:
#   - First call per chat-session: runs full detection + writes a finding
#     with prefix `[mdai-bootstrap-cache]` capturing tooling/lang flags.
#   - Subsequent calls: read ctx_session status, match the prefix, skip
#     detection. session_id changes on session restart -> natural invalidation.
# To force re-detection in the same session: `ctx_session action="reset"`
# (note: this also clears other session state).
@define mdai_bootstrap()
@call mdai_bootstrap_check_cache()
@if @result.stdout matches "\[mdai-bootstrap-cache\]"
  [mdai-bootstrap CACHED] session-scoped cache hit; skipping detection.
  # Cache line (verbatim from ctx_session status):
  {{ @result.stdout }}
@else
  @call service_check(service="lean_ctx",   mcp_tool="ctx_session action=status", required="true")
  @call service_check(service="markdownai", mcp_tool="list_phases file=.",        required="true")
  @call detect_tooling()
  @call detect_project_lang()
  # Persist cache marker for the rest of this session.
  @query mcp lean-ctx ctx_session action="finding" value="[mdai-bootstrap-cache] tooling=detected lang={{ @env MDAI_PROJECT_LANG | default('unknown') }} jetbrains={{ @env MDAI_HAS_JETBRAINS | default('false') }} serena={{ @env MDAI_HAS_SERENA | default('false') }}"
@endif
@end
