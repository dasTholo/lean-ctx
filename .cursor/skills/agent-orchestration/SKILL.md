---
name: agent-orchestration
description: >-
  Plan, launch, monitor, review, and merge parallel Codex CLI agents in isolated
  Git worktrees. Use for multi-agent refactoring rounds, Codex sub-agents,
  parallel implementation, worktree agents, codex exec, merge-round, or requests
  such as "nächste Phase", "agents starten", "Runde", and "R14".
---

# Orchestrate Codex agents

Keep each agent's file ownership disjoint. The helper scripts target macOS
Terminal and the current `codex exec` CLI; use an equivalent process supervisor
on other platforms.

## Gather

1. Read root `AGENTS.md`, the files in scope, and
   `scripts/preamble-template.md` before drafting tasks.
2. Confirm the installed CLI supports the required command and sandbox:

   ```bash
   codex --version
   codex exec --help
   ```

3. Split the round into two to five independently verifiable tasks with no file
   overlap. Give every shared type or contract a single owner.
4. Create `/tmp/codex-goals-<round>/preamble.md` and one
   `agent-<name>.md` goal per agent. Include exact paths, constraints, affected
   tests, quality commands, and explicit exclusions.
5. Inspect `git status`; do not launch from an unreviewed or ambiguous base.

## Act

1. Preview the branch/worktree names and goal files. Then launch:

   ```bash
   scripts/launch-round.sh <round> <agent-name> [agent-name...]
   ```

   The script creates `.worktrees/<round>-agent-<name>` and runs
   `codex exec -s workspace-write` in a separate Terminal session. This syntax
   is supported by Codex CLI 0.147.0; recheck `codex exec --help` after upgrades.

2. Monitor each worktree's process, commit, and status. Do not edit, commit, or
   merge an agent's work while its Codex process is still running.
3. Review each finished diff independently. Require the agent's affected tests
   and repository quality gate to pass before merge.
4. Merge only after every result is reviewed:

   ```bash
   scripts/merge-round.sh <round> <agent-name> [agent-name...]
   ```

   This script can commit uncommitted agent work, merge branches, run the Rust
   gate, and force-remove worktrees. Run it only with explicit authorization for
   those mutations and after confirming the exact arguments.
5. For a current Codex-native review of the merged result, use:

   ```bash
   codex exec review --base <base-branch> "Review architecture and regressions"
   ```

## Verify

- Confirm every requested task is present and no agent changed files outside its
  ownership.
- Run affected tests first, then from `rust/`: `cargo test --lib`,
  `cargo clippy --all-features -- -D warnings`, and `cargo fmt --check`.
- Confirm `scripts/loc-gate.sh` and any contract/schema checks relevant to the
  merged files pass.
- Inspect final `git status`, branch ancestry, and worktree state before any
  push or cleanup.
- Report agent outcomes, merged commits, conflicts, test results, remaining
  worktrees, and any skipped checks. Never push unless separately authorized.

