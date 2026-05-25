@markdownai v1.0

<!--
  body.mdai.md — Skill A v3 live workflow (mdai-brainstorm)
  Spec: docs/mdai/specs/2026-05-24-mdai-brainstorm-design.mdai.md
  No global @import here — packs are lazy-loaded per phase (Spec §5.1).
-->

@phase pre-context

@call mdai_bootstrap()

@include mdai/core/hard-rules.md
@include mdai/core/tool-quick-ref.md

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
@end

Constraints for the dialog phase:

- Spec target: docs/mdai/specs/ (NOT docs/superpowers/specs/)
- NO plan target — plan-write is a separate skill invocation (handoff phase)
- Hard rules: see @include above
  @end

@phase dialog-rules

@constraint id="hard-gate" severity="high"
Do NOT invoke any implementation skill, write any code, scaffold any project,
or take any implementation action until the user has approved a design.
Applies to EVERY project regardless of perceived simplicity.

This skill writes a SPEC ONLY. Do NOT write a plan in this skill — the plan
is produced by a separate skill invocation after this one ends (see handoff
phase).
@end

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

@end

@phase dialog-process

## Process Checklist

**Each item MUST become a `TaskCreate` entry and be completed in order**
(Upstream §Checklist mandate).

1. Explore project context (already done in pre-context phase).
2. Offer visual companion (if visual) — own message (see Visual-Companion section).
3. Ask clarifying questions — one at a time.
4. Propose 2–3 approaches with trade-offs.
5. Present design sections, get approval after each.
6. Write design doc to `docs/mdai/specs/` (NOT `docs/superpowers/specs/`).
7. Spec Self-Review (5 checks — see "Spec Self-Review" below).
   7.5 OPTIONAL: dispatch reviewer-subagent via `@call spec_reviewer_prompt` (mdai-Augmentation).
8. User reviews written spec (exact wording — see "User-Review-Gate").
9. Transition: invoke writing-plans skill (currently `superpowers:writing-plans`;
   future: `mdai-writing-plans` once that skill exists per Spec §14 Backlog #1)
   — THIS SKILL DOES NOT WRITE THE PLAN.

## The Process — Details

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
@end

@query mcp lean-ctx ctx_session action="status"

@if @result.stdout matches "\[mdai-brainstorm\] visual=true"
@include mdai/skills/mdai-brainstorm/visual-companion-offer.md
@endif

## Spec Self-Review (step 7, MANDATORY, Claude himself)

After the spec source (`.mdai.md`) is written, look at it with fresh eyes:

1. **Placeholder scan:** any "TBD", "TODO", incomplete sections, vague
   requirements? Fix inline.
2. **Internal consistency:** sections contradict each other? Architecture
   matches feature descriptions?
3. **Scope check:** focused enough for a single plan? Or needs decomposition
   into sub-projects?
4. **Ambiguity check:** any requirement interpretable two different ways?
   Pick one, make it explicit.
5. **mdai directive usage (Discipline §10.4 #9):** Does the spec body include
   markdownai directives for live content where semantically appropriate? If
   the spec is pure plain Markdown: is that justified with
   `markdownai_directives_omitted: <reason>` in the frontmatter? If not:
   extend the spec body with suitable directives (e.g. `@tree mdai/` instead
   of a static directory listing).

Fix issues inline. No re-review loop — fix and move on.

## Spec reviewer dispatch (step 7.5, OPTIONAL, mdai-Augmentation)

**Lazy-load** the reviewer macro just before dispatch:

@import mdai/skills/mdai-brainstorm/spec-reviewer.md

Then dispatch a reviewer subagent with `@call spec_reviewer_prompt(spec_path=<path>)`
as the prompt body. Returns Status (Approved | Needs-Revision | Needs-Clarification)

+ Strengths + Gaps + Concrete patches + Recommendations. Apply issues inline;
  surface recommendations.

Trigger: spec touches MCP signatures, Library packs, or render flow. Skip
for pure-prose specs (Self-Review §7 is sufficient).

## User-Review-Gate (step 8, exact wording, MANDATORY)

After Self-Review (and optional reviewer dispatch), ask the user with this
exact wording:

> "Spec written and committed to `<path>`. Please review and give feedback on
> whether you want changes, before invoking `/superpowers:writing-plans <path>`
> as the next step (or `/mdai-writing-plans` once that skill exists)."

Wait for explicit response. If user requests changes → patch inline → re-run
Self-Review §7. Only proceed to write-outputs phase once user explicitly approves.

Collect for the next phase:

- `slug` — kebab-case topic name (e.g. "user-onboarding-flow").
- `design_content` — full design body as Markdown.

## Spec body mdai directive conventions (mandatory reading for Step 6)

Operationalizes Discipline §10.4 #9. Mandatory at the "Write design doc" step.

| Use-Case | Best Practice | Anti-Pattern |
| Date in file paths | `{{ @date format='YYYY-MM-DD' }}`                                | hard-coded `2026-05-24` in
spec body |
| Directory listing | `@tree mdai/ depth=2`                                            | manually typed-out tree
output |
| File-system status (report)  | `@call file_check(path="...")` (from `core/file-utils.md`)       | `ls -la` output
copied + committed |
| Branching on file existence | inline `@if file.exists "..."` + `@else` + `@endif` at call site | `@call file_check` (
status only, not flow)                |
| Structured data | `@list <file.yaml> \| @render type="table" columns="..."`        | plain Markdown table at >50 rows
or with external SoT |
| Counts / Statistics | `{{ @count ./src "*.ts" }}` (inline)                             | hard-coded numbers that go
stale |
| Cross-File-Content | `@include ./CHANGELOG.md` or `@include <file> lines=N-M`         | copy-paste between specs |
| Machine-Readable Constraints | `@constraint id="..." severity="high"` + body + `@end`           | prosaic "Important:"
hints |
| Project-Context (live)       | `@call ctx_overview(task="...")` or `@call ctx_tree(...)`         | manually copied
project description |

**Anti-pattern: `file_check` is not branching.** `@call file_check(path="x.md")`
renders status only (`- x.md exists` / `- x.md MISSING`) — no control flow.
For branching ALWAYS inline at the call site:

@if file.exists "x.md"

- do this when exists
  @else
- do that when missing
  @endif

**Exception** (per §10.4 #9): specs for purely algorithmic topics without
file/tool/data dependencies may stay plain Markdown — then set
`markdownai_directives_omitted: <reason>` in the frontmatter.

<!--
  Drift-Tracking: hand-ported from superpowers/5.1.0/.../brainstorming/SKILL.md,
  lines 16-20 (anti-pattern), 22-32 (checklist), 70-104 (process details),
  107-136 (after-the-design: documentation/self-review/user-review-gate/
  implementation-transition), 140-145 (key principles).
-->
@end

@phase write-outputs

@import mdai/skills/mdai-brainstorm/write-spec.md

@call write_spec(slug={{ slug }}, body={{ design_content }})
@call render_spec(slug={{ slug }}, target={{ render_target | default("none") }})

Default output (one file staged in working tree):

- `docs/mdai/specs/<date>-<slug>-design.mdai.md` (spec source, consumer="ai")

Opt-in render targets (passed via `render_target` from dialog step 6):

- `target="none"` (default) → no render
- `target="chat"` → render inline via `mcp__markdownai__read_file`
- `target="file"` → adds `docs/mdai/specs/rendered/<date>-<slug>.rendered.md`
  via `npx mai render`

Verification:
@call ctx_tree(path="docs/mdai/specs/", depth=1)

Note: commit is left to the user (per CLAUDE.md — never auto-commit).
Note: NO plan file is written here. Plan-write is a separate skill invocation.
@end

@phase handoff

Spec ready for plan-write. Next step (manual, separate skill invocation):

`/superpowers:writing-plans docs/mdai/specs/<date>-<slug>-design.mdai.md`

This skill does NOT write the plan. Plan-write is the responsibility of a
separate writing-plans skill:

- **Now:** `/superpowers:writing-plans <spec-path>` (upstream)
- **Future:** `/mdai-writing-plans <spec-path>` — once Spec §14 Backlog #1 is
  shipped, use that instead (produces `.mdai.md` plan with `@phase` markers,
  compatible with `mdai-execution`).

Verify spec file is in place:
@call ctx_read(path="docs/mdai/specs/<date>-<slug>-design.mdai.md", mode="map")
@end
