@markdownai v1.0

## Hard Rules (from `CLAUDE.md`, always-on)

- Tests: **always** `cargo nextest run`, never `cargo test`.
- Before `git add`: `@call step_reformat_commit(file=<path>, message=<msg>)` (loads `tooling/jetbrains.md`).
- **No** `&&` bash chains — issue each command separately.
- **No** worktrees.
- Rust edits: prefer `@call replace_symbol_body(name=..., path=..., body=...)` / `insert_*_symbol` from
  `tooling/serena.md`.
- Prefer lean-ctx tools: `@call ctx_shell`, `@call ctx_tree`, `@call ctx_edit` (
  from `core/ctx-tools.md`).
- **Lean-context defaults** (`core/lean-context.md` — `@include` to render the rules table inline): bounded reads by
  default — `@call ctx_read_map(path)` / `ctx_read_signatures(path)` for scans, `@call ctx_read_lines(path, start, end)`
  after `ctx_search` / `find_symbol`. `mode="full"` is the exception (justify with `@note visible consumer="human"`).
  `ctx_shell raw=true` and `ctx_read fresh=true` are also exceptions — `fresh=true` ONLY immediately after a write/edit
  to the same path (cache auto-invalidates via mtime).

## v2 Directive Syntax (markdownai 1.3.0)

- **Closers:** block directives close with `@<name>-end` (`@if-end`, `@foreach-end`, `@switch-end`, `@constraint-end`, `@render-template-end`); argument-less directives self-close with a trailing ` /` (e.g. `@include … /`, `@set x = … /`).
- **Predicate call-form:** `file.exists("Cargo.toml")`, `file.containsLine("file", "anchor")` (parens + comma). A pure interpolation arg is **unquoted**: `file.exists({{ path }})`; a string literal (even with embedded `{{ }}`) stays **quoted**: `file.exists("docs/{{ slug }}.md")`. `matches` stays **infix**: `@if @result.stdout matches "…"`.
- **`@foreach` source must be interpolated:** `@foreach x in {{ list }}`. For an **object list** the value must be JSON wrapped in `{{ }}` so it parses and dot-access works: `@set packs = {{ [{"name":"a","flag":"F"}] }} /` then `{{ pack.name }}`. A bare `[{name=…}]` is stored as a string and split on every comma.
- **Dates:** `@date` is a directive and is **NOT** available inside `{{ }}` interpolation (inline `{{ @date }}` renders empty). Capture it via a directive-valued `@set`: `@set d = @date format='YYYY-MM-DD' /` then use `{{ d }}`. `now_iso()` is an interpolation builtin but is CLI-only (MCP `@eval` is blocked) and returns a full timestamp.
- **Cross-pack `@include`:** use `@include ${MDAI_LIBRARY_ROOT}/<pack>.md /` — see `core/mcp-markdownai.md` for the full resolution rule. MCP calls run with `cwd` = repo root.
- **Re-confirmed conventions:** `@render-template from=… to=… force` + key=value body + `@render-template-end` (§6); `mode: include` fragments carry no YAML frontmatter, the parser leaks it as text (§8); tooling/lang packs are `mode: import-only`, loaded via `@include`/`@import` (§11); library wrappers are synthesized and smoke-tested via `call_macro` (§12).
