---
name: crate-quality-check
description: >-
  Run and interpret production Rust crate quality checks. Use when the user says
  check this crate, Rust quality gate, cargo clippy, cargo test, cargo fmt,
  rustfmt check, TODO or FIXME audit, verify Rust docs, cargo doc, pre-commit
  Rust checks, or make a crate CI-clean.
---

# Check a Rust crate

## Gather

1. Resolve the crate/workspace manifest and read applicable `AGENTS.md`, Cargo
   features, CI workflow, and repository-specific quality commands.
2. Inspect `git status` and preserve unrelated changes.
3. Determine whether the target is a package inside a workspace. Prefer the
   repository's stricter gate when it exists; for lean-ctx itself, run from
   `rust/` and follow root `AGENTS.md`.
4. Identify changed public APIs and their affected tests/docs before running the
   full gate.

## Act

Run `scripts/check-crate.sh <crate-or-workspace-dir>`. It executes formatting,
Clippy with warnings denied, tests, rustdoc with warnings denied, and a
TODO/FIXME inventory. Set `FAIL_ON_TODOS=1` only when policy treats all markers
as blockers.

When fixing failures:

- Fix code rather than weakening lints, tests, features, or doc warnings.
- Do not delete TODO/FIXME markers without resolving the work or preserving it
  in an issue reference.
- Add docs for changed public behavior and examples where misuse is plausible.
- Run targeted affected tests first, then rerun the complete script.
- For lean-ctx, do not stop the installed runtime; build in the worktree and use
  `lean-ctx dev-install` only when installation is explicitly requested.

## Verify

- Require zero failures from fmt, Clippy, tests, and rustdoc.
- Review every TODO/FIXME hit and report whether it is pre-existing, resolved,
  issue-linked, or a release blocker.
- Confirm no generated files, secrets, or compressed omission markers entered
  the diff.
- Report exact commands, package/features checked, test totals, doc status, and
  any justified exclusions.
