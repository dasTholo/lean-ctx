---
name: mdai-brainstorm
description: Use when starting creative work that will produce a versioned design
  spec under docs/mdai/specs/. After spec approval, the next step is to invoke
  the writing-plans skill (superpowers:writing-plans, or mdai-writing-plans
  when available) — this skill does not write plans.
---

# mdai-brainstorm — pointer

DO NOT read this file's body for the workflow. The live workflow is in
`body.mdai.md` and MUST be loaded phase-by-phase via:

mcp__markdownai__read_file(path="<...>/body.mdai.md", phase="<phase-id>", format="ai")

Start with phase `pre-context`. Then `dialog-rules`, `dialog-process`,
`write-outputs`, `handoff` in order. Never call `read_file` without
a `phase=` argument.

Phases: pre-context | dialog-rules | dialog-process | write-outputs | handoff
