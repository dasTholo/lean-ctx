# CLAUDE.md

## Startup

## Project Hard Rules

> lean-ctx tool-discipline (ctx_read/ctx_shell/ctx_search/ctx_tree mapping, read
> modes, CEP, dense output) is loaded globally via `~/.claude/CLAUDE.md`
> (+ `rules/lean-ctx.md`). Not repeated here — only project deltas below.

- **`ctx_read` — `auto` only; `diff` for re-reads; never `fresh`/`raw`** (project
  delta — overrides the global mode-selection table in `~/.claude/rules/lean-ctx.md`):
  - Default to plain `ctx_read(path)` and let mode **`auto`** pick the optimal
    compression. Do **not** pass any explicit mode (`signatures`/`map`/`full`/
    `lines:N-M`/`aggressive`/`entropy`/…). Session cache + mtime auto-validation +
    auto-delta keep re-reads current & cheap (~13 tok).
  - **Never `fresh`/`raw`.** The only non-`auto` read allowed is the lean-ctx
    incremental diff — **`ctx_read(path, mode="diff")`** (the `ctx_read` `diff` mode),
    or equivalently the dedicated tool **`ctx_delta(path)`** ("only changed lines since
    last read"). This is **not** the Unix `diff` command and there is **no** `ctx_diff`
    tool. Use it to verify your own post-edit changes — wherever you would otherwise
    have reached for `fresh`. (The session cache already tracks each file's state, so a
    `diff`/`delta` re-read is always current and costs only the changed lines.)
- **Tests**: always `cargo nextest run`, never `cargo test`
- **Editing `*.rs` files**: always use Serena tools (`mcp__serena__jet_brains_find_symbol`,
  `replace_symbol_body`, `insert_before_symbol`/`insert_after_symbol`, `replace_content`,
  `rename`/`move`/`safe_delete`) — never native `Edit`/`ctx_edit` on Rust files
- **Deferred-tool reflex:** see `~/.claude/CLAUDE.md` Hard Rules — always
  `ToolSearch(query="select:...")` before any Bash workaround.
- **Before `git add`**: run `mcp__jetbrains__reformat_file` on every changed file
- **No worktrees** — work directly on the current branch
- **`ctx_shell` — bare command + `cwd=`, never `cd <path> &&`** (defeats output
  compression): the pattern router matches on the command **prefix**
  (`rust/src/core/patterns/mod.rs:140-145` → `c.starts_with("git ")`, same for
  `cargo `/`npm `/…). A `cd <path> && git …` wrapper makes the command start with
  `cd`, so `git::compress()` never runs and only weak generic fallbacks apply — the
  git-status / diff / log savings are lost. Therefore:
  - Run the **bare** command (`git diff --name-only HEAD`, `cargo nextest run`, …)
    and pass the directory via the `ctx_shell` **`cwd`** parameter (it persists
    across calls). **Never** prefix with `cd <path> &&`.
  - **No `2>&1`** — `ctx_shell` already captures stderr; the redirect only pollutes
    the pattern input and can break matching.
  - **Test runners (`cargo nextest`/`cargo test`/`pytest`/…): bare command, no
    `| tail`/`| grep`/`| head`.** Test output is kept **verbatim** by design
    (`rust/src/shell/compress/engine.rs:49-55`, `is_test_runner_command` Z. 292-330)
    — only head/tail-truncated when huge, with failure/summary lines preserved.
    A `cd … &&` prefix makes `is_test_runner_command` miss (it splits only on `|`,
    strips only `ENV=` prefixes), and an external `| tail`/`| grep` discards the
    `Summary […] N tests run: …` line *before* lean-ctx sees it. To shrink large
    green runs, do it at the source: `cargo nextest run --status-level fail`.

## Subagent-Driven Execution

When executing a plan via `superpowers:subagent-driven-development` (one fresh
subagent dispatched per task), the lean-ctx multi-agent + memory contract is
**mandatory** — for the controller and for every dispatched subagent. The
controller MUST prepend the Dispatch Contract to each subagent prompt.

@rules/subagent-multi-agent.md

## Language

- Interaction, chat, plans, specs: **German** with proper umlauts (ä ö ü ß) — never ae / oe / ue / ss
- Code and code comments: **English**
