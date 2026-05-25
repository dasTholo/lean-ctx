# mdai-brainstorm — Skill Notes

Triggered as `/mdai-brainstorm`. Writes a versioned design spec under
`docs/mdai/specs/<date>-<slug>-design.mdai.md` with `consumer="ai"` default.

## Next step after spec approval

This skill does **not** write plans. After `/mdai-brainstorm` produces a spec
and the user approves it via the User-Review-Gate, invoke the plan-writing
skill manually:

```
/superpowers:writing-plans docs/mdai/specs/<date>-<slug>-design.mdai.md
```

Once `mdai-writing-plans` (Spec §14 Backlog #1) ships, switch to:

```
/mdai-writing-plans docs/mdai/specs/<date>-<slug>-design.mdai.md
```

## Source layout

- `SKILL.md` — pointer (~15 lines, do not put workflow content here).
- `body.mdai.md` — live workflow (5 phases, loaded phase-by-phase via MCP).
- `write-spec.md` — Skill-A pack (`write_spec`, `render_spec`).
- `spec-reviewer.md` — Skill-A pack (`spec_reviewer_prompt`).
- `visual-companion-offer.md` — post-accept setup content (conditional `@include` from body.mdai.md `dialog-process`).
