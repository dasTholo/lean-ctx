@markdownai v1.0

<!--
  body.mdai.md — Skill A v3 live workflow (mdai-brainstorm)
  Spec: docs/mdai/specs/2026-05-24-mdai-brainstorm-design.mdai.md
  No global @import here — packs are lazy-loaded per phase (Spec §5.1).
-->

@phase pre-context

@call mdai_bootstrap() /

@call detect_mai_hook_version() /

@include ${MDAI_LIBRARY_ROOT}/core/hard-rules.md /
@include ${MDAI_LIBRARY_ROOT}/core/tool-quick-ref.md /

## Pre-resolved project context

**Branch:** {{ @call ctx_shell(cmd="git branch --show-current") }}
**Recent commits:**
{{ @call ctx_shell(cmd="git log --oneline -10") }}

**Project map (task-scoped):**
{{ @query mcp lean-ctx ctx_overview task="$user_task" }}

**Dependency graph (Mermaid, depth=2):**
{{ @query mcp lean-ctx ctx_graph action="diagram" kind="deps" depth=2 }}

**Task-relevant subgraph:**
{{ @query mcp lean-ctx ctx_graph action="context" }}

**Tree (depth=2):**
{{ @call ctx_tree(path=".", depth=2) }}

**Known gotchas:**
{{ @call list_gotchas(query="") }}

@constraint id="tool-selection" severity="high"
Read file → `@call ctx_read(path, mode)` (not `ctx_shell cmd="cat ..."`).
List directory → `@call ctx_tree(path, depth)` (not `ls`/`find`).
Pattern search → `@call ctx_search(pattern, path)` (not `grep`/`rg`).
File edit without read → `@call ctx_edit(path, old, new)`.
Read plan phase → `@call read_phase(plan, phase_id)`.
`@call ctx_shell` only as a last resort (git ops, shell scripts, tools without a wrapper).
@constraint-end

Constraints for the dialog phase:

- Spec target: docs/mdai/specs/ (NOT docs/superpowers/specs/)
- NO plan target — plan-write is a separate skill invocation (handoff phase)
- Hard rules: see @include above
  @phase-end

@phase dialog-rules

@constraint id="hard-gate" severity="high"
Do NOT invoke any implementation skill, write any code, scaffold any project,
or take any implementation action until the user has approved a design.
Applies to EVERY project regardless of perceived simplicity.

This skill writes a SPEC ONLY. Do NOT write a plan in this skill — the plan
is produced by a separate skill invocation after this one ends (see handoff
phase).
@constraint-end

## Red Flags — STOP and re-enter discipline

If any of these thoughts arise, STOP, re-read the HARD-GATE constraint, and
return to the checklist:

- "I'll just dash off the plan while I'm at it, the user will be happy" → STOP. This skill writes NO plan.
  Plan-write is the job of `/superpowers:writing-plans` post-spec. (Discipline §10.4 #1, Scope-Drift)
- "This spec is so small, a plain Markdown table is enough" → STOP. Main goal §1 requires
  markdownai directives for live content (`@tree`, `@call ctx_overview`, `@constraint`). Plain Markdown only with
  `markdownai_directives_omitted: <reason>` in frontmatter. (Discipline §10.4 #9)
- "I'll just save the spec under `docs/superpowers/specs/`, that's the standard" → STOP. mdai specs
  belong in `docs/mdai/specs/`, file extension `.mdai.md`. (Discipline §10.4 #2 + #3)
- "The user wants a quick approach listing, I'll present the design directly" → STOP. First 2–3
  approach alternatives with trade-offs, then design sections. One-question-at-a-time applies here too.
  (Discipline §10.4 #4 + #5, time-pressure)
- "She already approved after I showed Section 1, I'll write the spec now" → STOP.
  Per-section approval at the design walkthrough — one section approval is not spec approval.
  (Discipline §10.4 #6, authority-pressure)
- "I can skip Self-Review, I was careful while writing" → STOP. Self-Review §7 is
  MANDATORY before the User-Review-Gate. Four checks (Placeholders / Consistency / Scope / Ambiguity) plus #5
  mdai directive usage. (Discipline §10.4 #7)
- "I'll just load body.mdai.md full for a moment, it's more efficient" → STOP. The pointer instruction in SKILL.md is a
  hard constraint ("MUST"). Phase-by-phase via `mcp__markdownai__read_file(phase=..., format=ai)`.
  (Discipline §10.4 #8, cold-start)

## Anti-Pattern: "This Is Too Simple To Need A Design"

[hand-ported from superpowers:brainstorming/SKILL.md, lines 16-20]

Even a one-paragraph feature deserves a brainstorm pass. The discipline is
about *not skipping the dialog*, not about scaling the output. A 100-line
spec for a 5-line change is fine. Skipping the dialog and going straight to
code is the failure mode.

## Rationalization-Table

| Excuse                                                                                                         | Reality                                                                                                                                                                                                                     |
|----------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| "I'll write the plan along with it, it's the same context anyway" [reasoned-counter]                           | Skill A v3 writes NO plan. Plan-write is a separate skill invocation after spec approval (`/superpowers:writing-plans`). (§10.4 #1)                                                                                         |
| "Other specs live under `docs/superpowers/specs/`, I'll follow that convention" [reasoned-counter]             | mdai specs belong in `docs/mdai/specs/` with extension `.mdai.md` — deliberately separated from upstream specs. (§10.4 #2 + #3)                                                                                             |
| "One question is enough, the user already knows their project" [reasoned-counter]                              | One-question-at-a-time is discipline, not inefficiency. Batched questions lead to half-answers and re-loops. (§10.4 #4)                                                                                                     |
| "I already have a clear solution in mind, an approach comparison would be theatre" [reasoned-counter]          | Approach comparison (2–3 alternatives) is mandatory BEFORE presenting the design. Without alternatives the solution is ungrounded. (§10.4 #5)                                                                               |
| "Section-by-section approval is cumbersome, I'll send the whole spec at once" [reasoned-counter]               | Per-section approval prevents late vetoes on already-settled sections. Incremental validation beats big-bang review. (§10.4 #6)                                                                                             |
| "I implicitly did Self-Review while writing, I'll skip it" [reasoned-counter]                                  | Self-Review §7 has 5 explicit checks (Placeholders / Consistency / Scope / Ambiguity / mdai directives). Implicit reviews miss at least one check. (§10.4 #7)                                                               |
| "`body.mdai.md` is only 100 lines, a full read is more efficient than 4 phase reads" [reasoned-counter]        | Phase isolation keeps context small (each phase <580 words). Full read pollutes context for subsequent steps. The pointer instruction is MUST. (§10.4 #8)                                                                   |
| "Plain Markdown is more readable than a spec with `@call`/`@tree`/`@constraint` directives" [reasoned-counter] | Main goal §1: specs actively use markdownai for live content. Static tables and tree listings go stale immediately. Directives always deliver current state. (§10.4 #9)                                                     |
| "`mode='full'` makes sense everywhere, I can see the whole context" [reasoned-counter]                         | Lean-context defaults from `mdai/core/lean-context.md`: cross-file scan → `ctx_read_map`/`signatures`; after-search → `ctx_read_lines`. `mode='full'` only with `@note visible consumer="human"` justification. (§10.4 #9b) |

@phase-end

@phase dialog-process

## Process Checklist

**Each item MUST become a `TaskCreate` entry and be completed in order**
(Upstream §Checklist mandate).

1. Explore project context (already done in pre-context phase).
2. Offer visual companion (if visual) — own message (see Visual-Companion section).
3. Ask clarifying questions — one at a time.
4. Propose 2–3 approaches with trade-offs.
5. Present design sections, get approval after each.
6. Switch to write-outputs phase:
   `mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="write-outputs", cwd="<repo>")`
    - Apply spec-directive-conventions (rendered via @include) while finalizing design_content.
    - Invoke write_spec via call_macro pointer there.
7. Switch to handoff phase:
   `mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="handoff", cwd="<repo>")`
   7a. Invoke spec_self_review via call_macro.
   7b. Apply review findings inline.
   7c. opt: dispatch spec_reviewer_prompt via call_macro.
8. User-Review-Gate (in handoff phase, exact wording).
9. Transition: invoke writing-plans skill.

## Visual companion offer (step 2, conditional)

@prompt
Step 2 of the process checklist (conditional — only when upcoming questions
involve visual content: mockups, layouts, diagrams).

Ask the user with this exact wording (own message — never combined with
clarifying questions or context summaries):

> "Some of this is easier to show in the browser than to describe. I can build
> mockups, diagrams, comparisons, and other visuals to go with it. The feature
> is still new and token-intensive. Want to give it a try? (Opens a local URL.)"

Wait for response. If the user declines → text-only path, skip the include
below and proceed to step 3. If the user accepts → persist the choice via
`mcp__lean-ctx__ctx_session action="finding" val="[mdai-brainstorm] visual=true"`,
then re-load this phase so the include below fires.
@prompt-end

@query mcp lean-ctx ctx_session action="status" /

@if @result.stdout matches "\[mdai-brainstorm\] visual=true"
@include ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/visual-companion-offer.md /
@if-end

@include ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/process-principles.md /
@phase-end

@phase write-outputs

@include ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/spec-directive-conventions.md /

Apply the conventions above when finalizing design_content. Then invoke write_spec
via call_macro:

mcp__markdownai__call_macro(
file="mdai/skills/mdai-brainstorm/write-spec.md",
macro="write_spec",
args={ "slug": "{{ slug }}", "body": "{{ design_content }}" },
cwd="<repo>"
)

Optional inline-render (only when explicitly requested):

Wird kein render_target gesetzt, "none" übergeben (kein Inline-Render).

mcp__markdownai__call_macro(
file="mdai/skills/mdai-brainstorm/write-spec.md",
macro="render_spec",
args={ "slug": "{{ slug }}", "target": "{{ render_target }}" },
cwd="<repo>"
)

Default output (one file staged in working tree):

- `docs/mdai/specs/<date>-<slug>-design.mdai.md` (spec source, consumer="ai")

Verification:
@call ctx_tree(path="docs/mdai/specs/", depth=1) /

Note: commit is left to the user (per CLAUDE.md — never auto-commit).
Note: NO plan file is written here. Plan-write is a separate skill invocation.
@phase-end

@phase handoff

Spec Self-Review (5+1 checks). Invoke library-pack:

mcp__markdownai__call_macro(
file="mdai/skills/mdai-brainstorm/spec-self-review.md",
macro="spec_self_review",
args={ "spec_path": "{{ spec_path }}" },
cwd="<repo>"
)

Apply review findings inline.

Optional: dispatch full reviewer subagent:

mcp__markdownai__call_macro(
file="mdai/skills/mdai-brainstorm/spec-reviewer.md",
macro="spec_reviewer_prompt",
args={ "spec_path": "{{ spec_path }}" },
cwd="<repo>"
)

## User-Review-Gate (exact wording, MANDATORY)

> "Spec written and committed to `<path>`. Please review and give feedback on
> whether you want changes, before invoking `/superpowers:writing-plans <path>`
> as the next step (or `/mdai-writing-plans` once that skill exists)."

Wait for explicit response. If user requests changes → patch inline → re-run
spec_self_review via call_macro. Only proceed once user explicitly approves.

Next: invoke writing-plans skill.
@phase-end
