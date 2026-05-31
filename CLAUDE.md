bash# CLAUDE.md

## Startup

- **Always activate Serena** at the start of every conversation: `mcp__serena__activate_project`.

## Workflow

- **No worktrees**: Work directly on the current branch, do not use git worktrees.
- **Before `git add`**: Always run `mcp__jetbrains__reformat_file` on changed files before staging them.
- **Keine verketteten Bash-Befehle**: Keine `&&`-Ketten verwenden (z.B. `cargo fmt && git add ... && git commit`).
  Jeden Befehl einzeln ausführen — verkettete Befehle müssen einzeln bestätigt werden, einzelne werden per
  `settings.local.json` automatisch erlaubt.

## Hard Rules (always-on)

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
