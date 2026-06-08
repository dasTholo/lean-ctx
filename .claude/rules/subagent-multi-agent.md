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

## lean-ctx tool set (use these proactively)

Requires `tool_profile = power` (`lean-ctx tools power` → all 72 MCP tools
exposed). Under `power` **every** lean-ctx tool is direct — **call it directly**
(`ctx_read`, `ctx_search`, `ctx_shell`, `ctx_tree`, `ctx_multi_read`, `ctx_delta`,
`ctx_task`, `ctx_handoff`, `ctx_workflow`, `ctx_share`, `ctx_rules`, …). If a tool
shows up **deferred** in an isolated subagent catalog, run
`ToolSearch(query="select:<tool>")` FIRST, then call it directly. **NEVER wrap a
tool in `ctx_call`** (no `ctx_call name=ctx_read`, no `ctx_call name=ctx_task` —
that is pure overhead).

`ctx_call` is now only a **fallback**: use it solely if a tool stays deferred
after `ToolSearch`. (Profiles for reference — `minimal` = 6 tools, `standard` = 22,
`power` = all 72; this contract assumes `power`.)

> **No `ctx_share`:** the lean-ctx file cache is shared across all agents in the
> session (one MCP process). A subagent's first `ctx_read` is already warm, so
> warm-cache push/pull via `ctx_share` is redundant ceremony and is intentionally
> NOT part of this contract. Subagents just `ctx_read` — **never `fresh`**
> (mtime auto-validation keeps cached entries current), **never `raw`**.

| Need                              | Tool                           | Note                                                                                   |
|-----------------------------------|--------------------------------|----------------------------------------------------------------------------------------|
| Orient at start                   | `ctx_overview` + `ctx_repomap` | repomap = PageRank top symbols                                                         |
| Warm-read N files before dispatch | `ctx_multi_read paths=[…]`     | one call, not N× `ctx_read`                                                            |
| Re-read after an edit             | `ctx_delta path=…`             | only changed lines (cheaper than diff)                                                 |
| Checkpoint at phase boundary      | `ctx_compress`                 | long-conversation context save                                                         |
| Warm cache for a subagent         | (automatic — shared MCP cache) | no `ctx_share`; subagent just `ctx_read`, never `fresh`                                |
| Team coordination / diaries       | `ctx_agent`                    | register/post/read/diary/sync/handoff/share_knowledge                                  |
| Blast radius (risk gate)          | `ctx_impact`, `ctx_callgraph`  | standard — direct                                                                      |
| Task Liste                        | `ctx_task`                     | task create need: to_agent, States: "working,input-required,completed,failed,canceled" | 

## Controller contract (main agent, drives the plan)

1. **Plan start:** `ctx_overview "<plan-topic>"` + `ctx_repomap` (PageRank top
   symbols); check session restore.
2. Once: `ctx_agent action=register agent_type=claude role=plan`.
3. Persist plan facts twice — durable + team:
    - `ctx_knowledge action=remember category=decision …`
    - `ctx_agent action=post category=decision message="key=val;…"`
4. **Per task, BEFORE dispatch:** warm-read the relevant source files in one call
   via `ctx_multi_read paths=[…]`. The cache is shared across all session agents
   (one MCP process) — the subagent's first `ctx_read` hits these warm entries
   automatically. No `ctx_share` push, no `fresh` needed.
5. Prepend the **Dispatch Contract** (below) to every subagent prompt.
6. **After each task:** `ctx_session action=task value="<task> [N%]"`; durable
   facts via `ctx_knowledge action=remember`.
7. Team status via `ctx_agent action=sync` (not manual polling).
8. **At phase boundaries:** `ctx_compress` to checkpoint the long conversation.

## Implementer subagent contract

1. **Start:** `ctx_agent action=register agent_type=subagent role=dev`. The
   controller's warm cache is already shared (one MCP process) — just `ctx_read`,
   **never `fresh`**, **no `ctx_share` pull**.
2. Reads/search/shell via `ctx_read`/`ctx_search`/`ctx_shell` called **directly**
   (if deferred → `ToolSearch(query="select:<tool>")` first; **never** wrap them in
   `ctx_call`). **Never `fresh`** (mtime auto-validates; `fresh` right after a
   cache read is forbidden — lmd spec §4.2a), **never `raw`**. Search with
   `ctx_search`, not `grep`/`rg` inside `ctx_shell`; read files with `ctx_read`,
   not `cat`. Batch files with `ctx_multi_read`; re-read after your own edits with
   `ctx_delta` (changed lines only — that is what `ctx_delta` is for, not a `fresh`
   full re-read). **`ctx_shell`: bare command + `cwd=` — never `cd <path> &&`**
   (the pattern router matches on prefix, `mod.rs:140-145` `starts_with("git ")`/
   `cargo `/`npm `; a `cd … &&` wrapper kills git/cargo/npm compression) and **no
   `2>&1`** (stderr is already captured; the redirect breaks pattern matching).
3. **Rust (`*.rs`) edits via Serena only** (`replace_symbol_body`, `insert_*`,
   `rename`/`move`/`safe_delete`) — never native `Edit`/`ctx_edit` on Rust.
4. **During work:** `ctx_agent action=diary category=<discovery|decision|blocker|progress|insight>`
   at significant steps.
5. **On finish:** `ctx_agent action=post category=status message="…"` with status
   token (see below) + `ctx_agent action=handoff to_agent=<controller-id>` as baton.
6. Durable gotchas/facts: `ctx_knowledge action=remember`.

## Reviewer subagent contract (spec-reviewer + code-quality-reviewer)

1. **Start:** `ctx_agent action=register agent_type=subagent role=review` (warm
   cache already shared — just `ctx_read` directly, never `fresh`, never via
   `ctx_call`).
2. Post findings via `ctx_agent action=post category=finding` (in addition to the
   text return to the controller).
3. `ctx_agent action=diary` for non-trivial judgments.

## Dispatch Contract (prepend to EVERY subagent prompt)

```text
## lean-ctx Subagent Contract (MANDATORY)
You run in an isolated context. Before any other action:
1. ctx_agent action=register agent_type=subagent role=<dev|review>
   (controller's cache is already shared — no ctx_share pull, just ctx_read)
Tool discipline:
- ctx_read / ctx_search / ctx_shell / ctx_tree / ctx_multi_read / ctx_delta are
  DIRECT standard tools — call them DIRECTLY. If one shows up deferred, run
  ToolSearch(query="select:<tool>") FIRST, then call it. NEVER wrap a standard
  tool in ctx_call (no ctx_call name=ctx_read / name=ctx_shell — pure overhead).
- NEVER fresh, NEVER raw (mtime auto-validates; fresh after a cache read is forbidden)
- ctx_shell: bare command + cwd= — NEVER cd <path> && (prefix router, starts_with("git ")
  /cargo /npm ; a cd … && wrapper kills git/cargo/npm compression); and NO 2>&1
- Test runners (cargo nextest/cargo test/pytest/…): bare command, NO | tail/| grep/| head
  (output is kept verbatim w/ failures preserved; cd … && or | tail discards the Summary
  line before lean-ctx sees it). Shrink at source: cargo nextest run --status-level fail
- Search → ctx_search (never grep/rg in ctx_shell); read files → ctx_read (never cat)
- Batch reads → ctx_multi_read ; re-read after your edit → ctx_delta (changed lines)
- Rust (*.rs) edits → Serena tools only (never native Edit / ctx_edit)
- Power tools ONLY (ctx_task, ctx_handoff, ctx_workflow) → via ctx_call name=<tool>
- Tool params/signatures → authoritative in docs/reference/appendix-mcp-tools.md
  (generated/mcp-tools.md also valid IF freshly generated; ctx_read on demand, not memory)
During work: ctx_agent action=diary category=<discovery|decision|blocker|progress>
On finish:
- ctx_agent action=post category=<status|finding> message="<summary>"
- ctx_agent action=handoff to_agent=<controller-id> message="<baton>"
- ctx_knowledge action=remember for any durable fact/gotcha
Report final status: DONE | DONE_WITH_CONCERNS | NEEDS_CONTEXT | BLOCKED
```
