# Subagent Multi-Agent lean-ctx Rules Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make lean-ctx multi-agent + memory coordination mandatory for subagent-driven plan execution, via a new hand-written rules file + a copyable dispatch contract, mirrored for non-Claude agents.

**Architecture:** Three deliverables, no Rust code. (1) A new `.claude/rules/subagent-multi-agent.md` carrying three role contracts (controller / implementer / reviewer) plus a copyable dispatch contract. (2) A `@`-import + short pointer section in project `CLAUDE.md`. (3) A mirrored section in `AGENTS.md` placed *outside* the managed `<!-- lean-ctx -->` markers. The rules carry only the **behavioral** contract; tool params/signatures stay sourced from the CI-drift-tested `docs/reference/generated/mcp-tools.md`.

**Tech Stack:** Markdown docs only. Verification via `lean-ctx rules diff`, `ctx_search` grep checks, and manual marker-boundary inspection.

---

## Decisions locked in (from spec review + user)

The spec's §3.4 example block and §3.1.3 contain two factual errors vs. `docs/reference/generated/mcp-tools.md` (the spec's own §2 single-source-of-truth). User decisions:

- **Dispatch contract (§3.4):** keep concrete example calls, but **corrected** — use `message=` (not `msg=`), and **remove** the non-existent `ctx_agent action=share_knowledge`.
- **Controller knowledge-share (§3.1.3):** replace `share_knowledge` with `ctx_knowledge action=remember` (persistent) **+** `ctx_agent action=post category=…` (team broadcast). Both verified to exist.

Verified-real tool vocabulary (from `mcp-tools.md`, do **not** widen beyond this):
- `ctx_agent` actions: `register, post, read, status, handoff, sync, diary, recall_diary, diaries, list, info` — params: `action, agent_type, category, message, role, status, to_agent`
- `ctx_share` actions: `push, pull, list, clear` — params: `action, message, paths, to_agent`
- `ctx_session` action `task` — params: `action, session_id, value`
- `ctx_knowledge` action `remember` — params include `action, category, value`
- `ctx_overview` — params: `path, task`

---

## File Structure

| File | Responsibility | Action |
|------|----------------|--------|
| `.claude/rules/subagent-multi-agent.md` | Single home for the three role contracts + dispatch contract. Hand-written, references `mcp-tools.md` for params. | **Create** |
| `CLAUDE.md` (project) | Add a 2–3 sentence "## Subagent-Driven Execution" pointer + `@rules/subagent-multi-agent.md` import. | Modify |
| `AGENTS.md` | Mirror the contract for non-Claude agents, placed **before** the `<!-- lean-ctx -->` marker. | Modify |

No `lean-ctx rules sync/init` is invoked — content is hand-written and lives outside the managed blocks; no central `rules.toml` exists yet (spec §5).

---

## Task 1: Create the subagent multi-agent rules file

**Files:**
- Create: `/home/tholo/Scripts/lean-ctx/.claude/rules/subagent-multi-agent.md`

- [ ] **Step 1: Write the rules file**

Write this exact content to `/home/tholo/Scripts/lean-ctx/.claude/rules/subagent-multi-agent.md`:

```markdown
# Subagent-Driven Multi-Agent Execution — lean-ctx Contract

CRITICAL: This applies whenever a plan is executed via
`superpowers:subagent-driven-development` (controller dispatches one fresh
subagent per task with a self-crafted, isolated prompt).

Native-tool redirection (`Read`/`Grep`/`Bash` → `ctx_*`) is already enforced
elsewhere; this file adds only the proactive coordination + memory behaviors
hooks cannot inject.

> **Single source of truth for tool params/signatures:**
> `docs/reference/generated/mcp-tools.md` (auto-generated from
> `rust/src/core/reference_docs.rs`, CI-drift-tested). Read it on demand via
> `ctx_read(path, mode=map|signatures)`. This file carries only the *behavioral*
> contract — never rely on memorized signatures.

## Controller contract (main agent, drives the plan)

1. **Plan start:** `ctx_overview "<plan-topic>"`; check session restore.
2. Once: `ctx_agent action=register agent_type=claude role=plan`.
3. Persist plan facts twice — durable + team:
   - `ctx_knowledge action=remember category=decision …`
   - `ctx_agent action=post category=decision message="key=val;…"`
4. **Per task, BEFORE dispatch:** warm-read the relevant source files via
   `ctx_read`, then `ctx_share action=push to_agent=<sub-id> paths=[…]`
   (warm cache handoff — lets the subagent pull without `fresh`).
5. Prepend the **Dispatch Contract** (below) to every subagent prompt.
6. **After each task:** `ctx_session action=task value="<task> [N%]"`; durable
   facts via `ctx_knowledge action=remember`.
7. Team status via `ctx_agent action=sync` (not manual polling).

## Implementer subagent contract

1. **Start:** `ctx_agent action=register agent_type=subagent role=dev` +
   `ctx_share action=pull` (pull controller's warm cache) → **never `fresh`**.
2. Reads/search/shell explicitly as `ctx_read`/`ctx_search`/`ctx_shell`, never
   `fresh`, never `raw` (hooks redirect natives anyway; explicit keeps the cache
   consistent).
3. **Rust (`*.rs`) edits via Serena only** (`replace_symbol_body`, `insert_*`,
   `rename`/`move`/`safe_delete`) — never native `Edit`/`ctx_edit` on Rust.
4. **During work:** `ctx_agent action=diary category=<discovery|decision|blocker|progress|insight>`
   at significant steps.
5. **On finish:** `ctx_agent action=post category=status message="…"` with status
   token (see below) + `ctx_agent action=handoff to_agent=<controller-id>` as baton.
6. Durable gotchas/facts: `ctx_knowledge action=remember`.

## Reviewer subagent contract (spec-reviewer + code-quality-reviewer)

1. **Start:** `ctx_agent action=register agent_type=subagent role=review` +
   `ctx_share action=pull`.
2. Post findings via `ctx_agent action=post category=finding` (in addition to the
   text return to the controller).
3. `ctx_agent action=diary` for non-trivial judgments.

## Dispatch Contract (prepend to EVERY subagent prompt)

```text
## lean-ctx Subagent Contract (MANDATORY)
You run in an isolated context. Before any other action:
1. ctx_agent action=register agent_type=subagent role=<dev|review>
2. ctx_share action=pull          # warm cache from controller — DO NOT use fresh=true
Tool discipline:
- Reads/search/shell → ctx_read / ctx_search / ctx_shell (never fresh, never raw)
- Rust (*.rs) edits → Serena tools only (never native Edit / ctx_edit)
- Tool params/signatures → authoritative in docs/reference/generated/mcp-tools.md
  (ctx_read it on demand; do NOT rely on memory)
During work: ctx_agent action=diary category=<discovery|decision|blocker|progress>
On finish:
- ctx_agent action=post category=<status|finding> message="<summary>"
- ctx_agent action=handoff to_agent=<controller-id> message="<baton>"
- ctx_knowledge action=remember for any durable fact/gotcha
Report final status: DONE | DONE_WITH_CONCERNS | NEEDS_CONTEXT | BLOCKED
```
```

- [ ] **Step 2: Verify the file references only real tool actions**

Run:
```
ctx_search pattern="share_knowledge|msg=|action=msg" path=".claude/rules/subagent-multi-agent.md"
```
Expected: **0 matches** (no non-existent `share_knowledge`, no wrong `msg=` param).

- [ ] **Step 3: Verify the dispatch contract block is present and self-contained**

Run:
```
ctx_search pattern="lean-ctx Subagent Contract \(MANDATORY\)|ctx_share action=pull|ctx_agent action=handoff" path=".claude/rules/subagent-multi-agent.md"
```
Expected: at least 3 matches (heading + pull + handoff lines).

- [ ] **Step 4: Commit**

```bash
git add .claude/rules/subagent-multi-agent.md
git commit -m "docs(rules): add mandatory subagent multi-agent lean-ctx contract"
```

---

## Task 2: Wire the rules file into project CLAUDE.md

**Files:**
- Modify: `/home/tholo/Scripts/lean-ctx/CLAUDE.md` (insert a new section after "## Project Hard Rules", before "## Language")

Context — current CLAUDE.md ends "## Project Hard Rules" with the bullet
`- **No worktrees** — work directly on the current branch`, immediately followed
by `## Language`. The new section goes between them.

- [ ] **Step 1: Insert the new section**

Use `ctx_edit` (native Read is hook-blocked; this is a `.md` file, so Serena is not required).

old_string:
```
- **No worktrees** — work directly on the current branch

## Language
```

new_string:
```
- **No worktrees** — work directly on the current branch

## Subagent-Driven Execution

When executing a plan via `superpowers:subagent-driven-development` (one fresh
subagent dispatched per task), the lean-ctx multi-agent + memory contract is
**mandatory** — for the controller and for every dispatched subagent. The
controller MUST prepend the Dispatch Contract to each subagent prompt.

@rules/subagent-multi-agent.md

## Language
```

- [ ] **Step 2: Verify the import and section landed**

Run:
```
ctx_search pattern="@rules/subagent-multi-agent.md|## Subagent-Driven Execution" path="CLAUDE.md"
```
Expected: 2 matches (the heading + the `@`-import line).

- [ ] **Step 3: Verify ordering is unbroken (section sits before Language)**

Run:
```
ctx_read path="/home/tholo/Scripts/lean-ctx/CLAUDE.md" mode=full
```
Expected: `## Subagent-Driven Execution` appears after the `No worktrees` bullet and before `## Language`; the `@rules/subagent-multi-agent.md` line is the last line of the new section.

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md
git commit -m "docs(claude): import subagent multi-agent rules + execution pointer"
```

---

## Task 3: Mirror the contract into AGENTS.md (outside managed markers)

**Files:**
- Modify: `/home/tholo/Scripts/lean-ctx/AGENTS.md` (insert after "## Quality Bar", before the `<!-- lean-ctx -->` marker)

Context — AGENTS.md ends its hand-written body with the "## Quality Bar" bullets,
then an empty line, then `<!-- lean-ctx -->`. The managed blocks (`<!-- lean-ctx -->`
and `<!-- lean-ctx-compression -->`) are owned by `lean-ctx rules sync` and MUST NOT
be touched. The new section goes **between** the Quality Bar block and the first
`<!-- lean-ctx -->` marker.

- [ ] **Step 1: Insert the mirrored section**

Use `ctx_edit`.

old_string:
```
- No mock data, no placeholders, no stubs

<!-- lean-ctx -->
```

new_string:
```
- No mock data, no placeholders, no stubs

## Subagent-Driven Multi-Agent Execution

When a controller agent executes a plan by dispatching one fresh subagent per
task, every agent MUST use the lean-ctx coordination + memory tools. Tool
params/signatures are authoritative in `docs/reference/generated/mcp-tools.md`
(read on demand) — the rules below carry only the behavioral contract.

- **Controller:** `ctx_overview` at start; `ctx_agent action=register role=plan`
  once; persist plan facts via `ctx_knowledge action=remember` + broadcast via
  `ctx_agent action=post`; per task, warm-read sources then
  `ctx_share action=push to_agent=<sub-id>`; `ctx_session action=task` after each
  task; `ctx_agent action=sync` for team status.
- **Subagent (dev/review):** before anything,
  `ctx_agent action=register agent_type=subagent role=<dev|review>` +
  `ctx_share action=pull` (warm cache — never `fresh`). Reads/search/shell via
  `ctx_read`/`ctx_search`/`ctx_shell` (never `fresh`/`raw`). Rust `*.rs` edits via
  Serena only. Log progress via `ctx_agent action=diary`. On finish:
  `ctx_agent action=post category=<status|finding> message="…"` +
  `ctx_agent action=handoff to_agent=<controller-id>`; durable facts via
  `ctx_knowledge action=remember`. Report status:
  DONE | DONE_WITH_CONCERNS | NEEDS_CONTEXT | BLOCKED.

<!-- lean-ctx -->
```

- [ ] **Step 2: Verify the section is outside the managed markers**

Run:
```
ctx_read path="/home/tholo/Scripts/lean-ctx/AGENTS.md" mode=full
```
Expected: `## Subagent-Driven Multi-Agent Execution` appears **before** the first `<!-- lean-ctx -->` line; the `<!-- lean-ctx -->` … `<!-- /lean-ctx -->` and `<!-- lean-ctx-compression -->` … `<!-- /lean-ctx-compression -->` blocks are unchanged.

- [ ] **Step 3: Verify no wrong tool vocabulary leaked in**

Run:
```
ctx_search pattern="share_knowledge|msg=" path="AGENTS.md"
```
Expected: **0 matches**.

- [ ] **Step 4: Verify rules sync sees no drift (content lives outside markers)**

Run:
```
ctx_shell command="lean-ctx rules diff"
```
Expected: no drift reported for the managed `<!-- lean-ctx -->` blocks (our hand-written section is outside them, so it is not compared).

- [ ] **Step 5: Reformat changed files before staging (project rule)**

Run `mcp__jetbrains__reformat_file` on `AGENTS.md` (and any other file touched this task).

- [ ] **Step 6: Commit**

```bash
git add AGENTS.md
git commit -m "docs(agents): mirror subagent multi-agent contract outside lean-ctx markers"
```

---

## Task 4: Final cross-file consistency check

**Files:** none modified — verification only.

- [ ] **Step 1: Confirm all three deliverables exist and agree**

Run:
```
ctx_search pattern="Dispatch Contract|Subagent-Driven|ctx_share action=pull" paths=[".claude/rules/subagent-multi-agent.md","CLAUDE.md","AGENTS.md"]
```
Expected: matches in all three files (rules file has the full contract; CLAUDE.md has the pointer + import; AGENTS.md has the mirror).

- [ ] **Step 2: Confirm tool-name consistency across files**

Run:
```
ctx_search pattern="ctx_agent action=(register|post|diary|handoff|sync)|ctx_share action=(push|pull)|ctx_knowledge action=remember|ctx_session action=task" paths=[".claude/rules/subagent-multi-agent.md","AGENTS.md"]
```
Expected: only these real actions appear; cross-check that no file uses `share_knowledge` or `msg=`.

- [ ] **Step 3: Confirm `lean-ctx rules diff` is still clean**

Run:
```
ctx_shell command="lean-ctx rules diff"
```
Expected: no drift on managed blocks.

---

## Self-Review (run against spec before handoff)

**Spec coverage:**
- §3.1 controller contract → Task 1 rules file "Controller contract" + Task 3 mirror. ✓
- §3.2 implementer contract → Task 1 "Implementer subagent contract". ✓
- §3.3 reviewer contract → Task 1 "Reviewer subagent contract". ✓
- §3.4 dispatch contract → Task 1 "Dispatch Contract" block (corrected `message`, no `share_knowledge`). ✓
- §2 single-source principle (no verbatim params) → rules link `mcp-tools.md`; only behavioral contract + minimal corrected examples (per user decision). ✓
- §5.1 new rules file → Task 1. ✓
- §5.2 CLAUDE.md section + import → Task 2. ✓
- §5.3 AGENTS.md mirror outside markers → Task 3. ✓
- §4 conflict: content outside `<!-- lean-ctx -->` markers → Task 3 Step 1/2/4. ✓
- §6 success criteria → verification steps in Tasks 1–4 (`rules diff`, grep for real actions, no `fresh`). ✓
- §7 non-goals → no `ctx_task` board, no `ctx_handoff` bundles, no `rules init`, no global CLAUDE.md change, no plugin-cache patches. Plan touches none of these. ✓

**Placeholder scan:** no TBD/TODO; every step has exact content or exact command + expected output.

**Type/name consistency:** all tool actions used (`register/post/diary/handoff/sync`, `push/pull`, `remember`, `task`) match `mcp-tools.md` verified vocabulary; `message` param used consistently (never `msg`); `share_knowledge` never appears.
