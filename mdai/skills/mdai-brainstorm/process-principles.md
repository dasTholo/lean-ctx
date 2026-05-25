---
mode: include
lib_version: 0.1.1
---

@markdownai v1.0

# The Process — Details (L3 — dialog-process phase)

[hand-ported from superpowers:brainstorming/SKILL.md, lines 70-104]

- **Understanding the idea:** scope check, decomposition for large projects,
  one question at a time, no batched "tell me everything" prompts.
- **Exploring approaches:** 2–3 alternatives with explicit trade-offs, lead
  with recommendation but make alternatives real (not strawmen).
- **Presenting the design:** scaled to complexity, approval-per-section,
  user reads each section before next is drafted.
- **Design for isolation and clarity:** small units, clear interfaces, no
  premature abstractions.
- **Working in existing codebases:** follow existing patterns, no unrelated
  refactoring, no scope-creep into adjacent files.

## Key Principles

[hand-ported from superpowers:brainstorming/SKILL.md, lines 140-145]

- One question at a time.
- Multiple choice preferred over open-ended where it fits.
- YAGNI ruthlessly.
- Explore alternatives before settling.
- Incremental validation.
- Be flexible — go back when something doesn't make sense.
