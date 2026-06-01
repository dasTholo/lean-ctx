# CLAUDE.md

## Startup

- **Always activate Serena** at the start of every conversation: `mcp__serena__activate_project`.

## Project Hard Rules

> lean-ctx tool-discipline (ctx_read/ctx_shell/ctx_search/ctx_tree mapping, read
> modes, CEP, dense output) is loaded globally via `~/.claude/CLAUDE.md`
> (+ `rules/lean-ctx.md`). Not repeated here — only project deltas below.

- **`@read`/`ctx_read` — no `fresh`/`raw`**: always read without `fresh`/`raw`.
  Session cache + mtime auto-validation + auto-delta keep re-reads current & cheap
  (~13 tok). `fresh`/`lines:N-M` only as a justified exception; **never `fresh`
  right after a cache read** (lmd spec §4.2a: shared EngineContext cache → read→delta).
- **Tests**: always `cargo nextest run`, never `cargo test`
- **Editing `*.rs` files**: always use Serena tools (`mcp__serena__jet_brains_find_symbol`,
  `replace_symbol_body`, `insert_before_symbol`/`insert_after_symbol`, `replace_content`,
  `rename`/`move`/`safe_delete`) — never native `Edit`/`ctx_edit` on Rust files
- **Deferred-tool reflex:** see `~/.claude/CLAUDE.md` Hard Rules — always
  `ToolSearch(query="select:...")` before any Bash workaround.
- **Before `git add`**: run `mcp__jetbrains__reformat_file` on every changed file
- **No worktrees** — work directly on the current branch

## Subagent-Driven Execution

When executing a plan via `superpowers:subagent-driven-development` (one fresh
subagent dispatched per task), the lean-ctx multi-agent + memory contract is
**mandatory** — for the controller and for every dispatched subagent. The
controller MUST prepend the Dispatch Contract to each subagent prompt.

@rules/subagent-multi-agent.md

## Language

- Interaction, chat, plans, specs: **German** with proper umlauts (ä ö ü ß) — never ae / oe / ue / ss
- Code and code comments: **English**
