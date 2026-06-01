bash# CLAUDE.md

## Startup

- **Always activate Serena** at the start of every conversation: `mcp__serena__activate_project`.

## Hard Rules (always-on)
ALWAYS use lean-ctx MCP tools instead of native equivalents.

Tool mapping (MANDATORY):
• Read/cat/head/tail -> ctx_read(path, mode)
• Shell/bash -> ctx_shell(command)
• Grep/rg -> ctx_search(pattern, path)
• ls/find -> ctx_tree(path, depth)
• Edit/StrReplace -> native (lean-ctx=READ only). If Edit needs Read and Read is unavailable, use ctx_edit.
• Write, Delete, Glob -> normal. NEVER loop on Edit failures — use ctx_edit.

ctx_read modes: full|map|signatures|diff|task|reference|aggressive|entropy|lines:N-M
Auto-selects mode. Re-reads ~13 tok. File refs F1,F2.. persist.
Cache auto-validates via file mtime. Use fresh=true (or start_line / lines:N-M) to force a disk re-read.

Auto: ctx_overview, ctx_preload, ctx_dedup, ctx_compress behind the scenes.
Multi-agent: ctx_agent(action=handoff|sync|diary).
ctx_semantic_search for meaning search. ctx_session for memory.
ctx_knowledge: remember|recall|timeline|rooms|search|wakeup.
ctx_shell raw=true for uncompressed.

CEP: 1.ACT FIRST 2.DELTA ONLY 3.STRUCTURED(+/-/~) 4.ONE LINE 5.QUALITY
Prefer: ctx_read>Read | ctx_shell>Shell | ctx_search>Grep | ctx_tree>ls
Edit: native Edit/StrReplace preferred, ctx_edit if Edit unavailable.
Never echo tool output. Never narrate. Show only changed code.
Full instructions at ~/.claude/CLAUDE.md (imports rules/lean-ctx.md)

- **Tests**: always `cargo nextest run`, never `cargo test`
- **Editing `*.rs` files**: always use Serena tools (`mcp__serena__jet_brains_find_symbol`,
  `replace_symbol_body`, `insert_before_symbol`/`insert_after_symbol`, `replace_content`,
  `rename`/`move`/`safe_delete`) — never native `Edit`/`ctx_edit` on Rust files
- **Deferred-tool reflex:** see `~/.claude/CLAUDE.md` Hard Rules — always
  `ToolSearch(query="select:...")` before any Bash workaround.
- **Before `git add`**: run `mcp__jetbrains__reformat_file` on every changed file
- **No worktrees** — work directly on the current branch
- **No `&&` chains** in Bash — run each command separately

## Language

- Interaction, chat, plans, specs: **German** with proper umlauts (ä ö ü ß) — never ae / oe / ue / ss
- Code and code comments: **English**
