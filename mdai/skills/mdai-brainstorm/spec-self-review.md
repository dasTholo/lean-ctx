---
lib_version: 0.1.1
mdai-pack:
  mode: import-only
  exports: [spec_self_review]
---

@markdownai v1.0

@define spec_self_review(spec_path)

# Spec Self-Review — {{ spec_path }}

After the spec source is written, look at it with fresh eyes:

## Check #1 — Placeholder scan

Any "TBD", "TODO", incomplete sections, vague requirements? Fix inline.

## Check #2 — Internal consistency

Sections contradict each other? Architecture matches feature descriptions?

## Check #3 — Scope check

Focused enough for a single plan? Or needs decomposition into sub-projects?

## Check #4 — Ambiguity

Any requirement interpretable two different ways? Pick one, make it explicit.

## Check #5 — mdai directive usage (Discipline §10.4 #9)

Does the spec body include markdownai directives for live content where
semantically appropriate? If pure plain Markdown: justified with
`markdownai_directives_omitted: <reason>` in the frontmatter?

## Check #6 — Lean-Context Anchors

Search `{{ spec_path }}` for each anchor. Flag any hit with an adjacent
`@note visible consumer="human"` justification, or remove it.

- [ ] `mode="full"` — only allowed for the one spec-source read (§0); flag all others.
- [ ] `raw=true` — every `ctx_shell raw=true` needs a `@note visible consumer="human"` block.
- [ ] `fresh=true` — only valid immediately after a write/edit to the same path.
- [ ] `Grep` / `rg ` — lean-ctx violation; replace with `@call ctx_search(...)`.
- [ ] `cat ` / `head ` / `tail ` — lean-ctx violation; replace with `@call ctx_read(...)`.
- [ ] `bash ` / `sh ` — lean-ctx violation; replace with `@call ctx_shell(...)`.

## Reviewer Dispatch (optional)

Trigger: spec touches MCP signatures, Library packs, or render flow.
Invoke via:
mcp__markdownai__call_macro(
file="mdai/skills/mdai-brainstorm/spec-reviewer.md",
macro="spec_reviewer_prompt",
args={"spec_path": "{{ spec_path }}"},
cwd="<repo>"
)

Fix issues inline. No re-review loop — fix and move on.
@define-end
