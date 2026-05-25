## Visual companion — post-accept setup

The user accepted the offer (outer `@if` in body.mdai.md `dialog-process` gated
this include). Apply the per-question discipline and start the companion server.

Per-question decision: use the browser **only** when content IS visual
(mockups, wireframes, layout comparisons, architecture diagrams, side-by-side
visual designs). Conceptual/text questions stay in the terminal.

Read the upstream guide for HTML-fragment patterns:

@note visible consumer="human"
visual-companion.md (upstream) has no map/signatures path — full-read is the only sensible variant.
Version pinned to 5.1.0 (Spec §5.3 version pin); update when upstream bumps. Reviewer check #10 passes
without further note.
@end

{{ @call ctx_read(path="~/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/visual-companion.md", mode="full") }}

Start the companion server (persistent mockups under `.superpowers/brainstorm/`):
@call ctx_shell(cmd="~/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/start-server.sh --project-dir \"$PWD\"")

Capture `screen_dir` and `state_dir` from the server-info JSON for subsequent
screen pushes.
