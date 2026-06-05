# Subagent-Driven Multi-Agent Execution — lean-ctx Contract

CRITICAL: This applies whenever a plan is executed via
`superpowers:subagent-driven-development` (controller dispatches one fresh
subagent per task with a self-crafted, isolated prompt).

Native-tool redirection (`Read`/`Grep`/`Bash` → `ctx_*`) is already enforced
elsewhere; this file adds only the proactive coordination + memory behaviors
hooks cannot inject.

> **Single source of truth for tool params/signatures:**
> `docs/reference/appendix-mcp-tools.md` (human tool map; authoritative schemas in
> `rust/src/tools/registered/<tool>.rs`). Also valid: the auto-generated
> `docs/reference/generated/mcp-tools.md` — but ONLY when freshly generated
> (CI-drift-tested); if in doubt, trust the appendix. Read on demand via
> `ctx_read(path, mode=map|signatures)`. This file carries only the *behavioral*
> contract — never rely on memorized signatures.

## lean-ctx tool set (3.7.x — use these proactively)

Requires `tool_profile = "standard"`+ (`lean-ctx tools standard`). Standard tools
below are direct. **Power-profile** tools (`ctx_share`, `ctx_task`, `ctx_handoff`,
`ctx_workflow`) are NOT exposed as direct tools under `standard` — reach them via
the `ctx_call` gateway: `ctx_call name=ctx_share arguments={action:…}`. (Alt:
`lean-ctx tools power` exposes them directly but bloats the tool catalog.)

| Need                              | Tool                                      | Note                                                  |
|-----------------------------------|-------------------------------------------|-------------------------------------------------------|
| Orient at start                   | `ctx_overview` + `ctx_repomap`            | repomap = PageRank top symbols                        |
| Warm-read N files before dispatch | `ctx_multi_read paths=[…]`                | one call, not N× `ctx_read`                           |
| Re-read after an edit             | `ctx_delta path=…`                        | only changed lines (cheaper than diff)                |
| Checkpoint at phase boundary      | `ctx_compress`                            | long-conversation context save                        |
| Warm-cache handoff to a subagent  | `ctx_call name=ctx_share {action:push,…}` | power tool → via gateway                              |
| Team coordination / diaries       | `ctx_agent`                               | register/post/read/diary/sync/handoff/share_knowledge |
| Blast radius (risk gate)          | `ctx_impact`, `ctx_callgraph`             | standard — direct                                     |

## Controller contract (main agent, drives the plan)

1. **Plan start:** `ctx_overview "<plan-topic>"` + `ctx_repomap` (PageRank top
   symbols); check session restore.
2. Once: `ctx_agent action=register agent_type=claude role=plan`.
3. Persist plan facts twice — durable + team:
    - `ctx_knowledge action=remember category=decision …`
    - `ctx_agent action=post category=decision message="key=val;…"`
4. **Per task, BEFORE dispatch:** warm-read the relevant source files in one call
   via `ctx_multi_read paths=[…]`, then push the warm cache with
   `ctx_call name=ctx_share arguments={action:push, to_agent:<sub-id>, paths:[…]}`
   (lets the subagent pull without `fresh`).
5. Prepend the **Dispatch Contract** (below) to every subagent prompt.
6. **After each task:** `ctx_session action=task value="<task> [N%]"`; durable
   facts via `ctx_knowledge action=remember`.
7. Team status via `ctx_agent action=sync` (not manual polling).
8. **At phase boundaries:** `ctx_compress` to checkpoint the long conversation.

## Implementer subagent contract

1. **Start:** `ctx_agent action=register agent_type=subagent role=dev` +
   `ctx_call name=ctx_share arguments={action:pull}` (pull controller's warm
   cache) → **never `fresh`**.
2. Reads/search/shell explicitly as `ctx_read`/`ctx_search`/`ctx_shell`, never
   `fresh`, never `raw` (hooks redirect natives anyway; explicit keeps the cache
   consistent). Batch multiple files with `ctx_multi_read`; re-read after your own
   edits with `ctx_delta` (changed lines only).
3. **Rust (`*.rs`) edits via Serena only** (`replace_symbol_body`, `insert_*`,
   `rename`/`move`/`safe_delete`) — never native `Edit`/`ctx_edit` on Rust.
4. **During work:** `ctx_agent action=diary category=<discovery|decision|blocker|progress|insight>`
   at significant steps.
5. **On finish:** `ctx_agent action=post category=status message="…"` with status
   token (see below) + `ctx_agent action=handoff to_agent=<controller-id>` as baton.
6. Durable gotchas/facts: `ctx_knowledge action=remember`.

## Reviewer subagent contract (spec-reviewer + code-quality-reviewer)

1. **Start:** `ctx_agent action=register agent_type=subagent role=review` +
   `ctx_call name=ctx_share arguments={action:pull}`.
2. Post findings via `ctx_agent action=post category=finding` (in addition to the
   text return to the controller).
3. `ctx_agent action=diary` for non-trivial judgments.

## Dispatch Contract (prepend to EVERY subagent prompt)

```text
## lean-ctx Subagent Contract (MANDATORY)
You run in an isolated context. Before any other action:
1. ctx_agent action=register agent_type=subagent role=<dev|review>
2. ctx_call name=ctx_share arguments={action:pull}   # warm cache from controller — DO NOT use fresh=true
Tool discipline:
- Reads/search/shell → ctx_read / ctx_search / ctx_shell (never fresh, never raw)
- Batch reads → ctx_multi_read ; re-read after your edit → ctx_delta (changed lines)
- Rust (*.rs) edits → Serena tools only (never native Edit / ctx_edit)
- Power tools (ctx_share, ctx_task, ctx_handoff) → via ctx_call name=<tool>
- Tool params/signatures → authoritative in docs/reference/appendix-mcp-tools.md
  (generated/mcp-tools.md also valid IF freshly generated; ctx_read on demand, not memory)
During work: ctx_agent action=diary category=<discovery|decision|blocker|progress>
On finish:
- ctx_agent action=post category=<status|finding> message="<summary>"
- ctx_agent action=handoff to_agent=<controller-id> message="<baton>"
- ctx_knowledge action=remember for any durable fact/gotcha
Report final status: DONE | DONE_WITH_CONCERNS | NEEDS_CONTEXT | BLOCKED
```
