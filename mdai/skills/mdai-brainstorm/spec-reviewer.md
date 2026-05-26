---
lib_version: 0.1.1
mdai-pack:
  mode: import-only
  exports: [spec_reviewer_prompt]
---

@markdownai v1.0

@define spec_reviewer_prompt(spec_path)
You are a spec doc reviewer. Verify {{ spec_path }} is complete and ready.

## 1. Read the spec

mcp__markdownai__read_file(path="{{ spec_path }}", cwd="<repo>")

## 2. What to Check

| Category     | What to Look For                                    |
|--------------|-----------------------------------------------------|
| Completeness | TODOs, placeholders, "TBD", incomplete sections     |
| Consistency  | Internal contradictions, conflicting requirements   |
| Clarity      | Requirements ambiguous enough to cause wrong builds |
| Scope        | Focused enough for a single plan                    |
| YAGNI        | Unrequested features, over-engineering              |

## 3. Calibration

Only flag issues that would cause real problems during impl planning.
Approve unless there are serious gaps.

## 4. mdai-Augmentations (universal)

a. Language convention (CLAUDE.md): spec body German, code/snippets English.
b. mdai directives in body (Discipline §10.4 #9): ≥3 distinct directive types
in body, OR frontmatter has `markdownai_directives_omitted: <reason>`.
c. Lean-context audit: invoke via
mcp__markdownai__call_macro(file="mdai/core/lean-context-audit.md",
macro="lean_context_audit",
args={"spec_path": "{{ spec_path }}"},
cwd="<repo>")

## 5. Heavy library-spec checks (conditional)

@if file.containsLine "{{ spec_path }}" "target_library:"
Invoke via mcp__markdownai__call_macro(
file="mdai/core/library-spec-audit.md",
macro="library_spec_audit",
args={"spec_path": "{{ spec_path }}"},
cwd="<repo>")
@endif

## 6. Output

Invoke via mcp__markdownai__call_macro(
file="mdai/skills/mdai-brainstorm/write-spec.md",
macro="write_review_report",
args={"spec_path": "{{ spec_path }}", "status": "...", ...},
cwd="<repo>")
@end
