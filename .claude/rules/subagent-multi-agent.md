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
