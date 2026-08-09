---
name: lean-ctx-dev
description: >-
  Develop, extend, debug, and verify the lean-ctx Rust codebase. Use when adding
  an MCP or ctx_* tool, registering ToolRegistry entries, changing core modules,
  PathJail, bounded locks, shell execution, graph edges, context compression,
  cargo tests, Clippy, rustfmt, preflight, or installing a local lean-ctx build.
---

# Develop lean-ctx

## Gather

1. Read root `AGENTS.md`, `rust/AGENTS.md`, and applicable `.cursor/rules/`
   before changing code. Preserve unrelated work from `git status`.
2. Run `ctx_impact` for every planned `rust/src/**` change and identify the
   affected tests before editing.
3. Inspect the closest implementation and current public contracts. Key paths:
   - Tool trait: `rust/src/server/tool_trait.rs`
   - Tool modules: `rust/src/tools/registered/`
   - Module exports: `rust/src/tools/registered/mod.rs`
   - Registry: `rust/src/server/registry.rs`
   - Derived tool definitions: `rust/src/tool_defs/granular.rs`
   - Core modules: `rust/src/core/`
4. Check security boundaries before file, lock, shell, network, or credential
   changes: `core/pathjail.rs`, `server/bounded_lock.rs`, and
   `server/execute.rs`.

## Act

### Add an MCP tool

1. Create `rust/src/tools/registered/ctx_<name>.rs` and implement `McpTool`:
   `name`, `tool_def`, and `handle`.
2. Export the module from `rust/src/tools/registered/mod.rs` and register its
   unit struct in `server::registry::build_registry()`.
3. Keep the schema in `McpTool::tool_def()`. `granular_tool_defs()` is derived
   from the registry, so do not add a duplicate granular schema. Edit the
   separate unified definitions only when intentionally changing that surface.
4. Add handler, schema, annotation, dispatch, error, and security tests matching
   the nearest tool.

### Add or change core code

1. Add modules alphabetically in their parent module.
2. Route user-controlled paths through PathJail/path resolution and filesystem
   I/O through the repository boundary helpers.
3. Use bounded lock helpers and adaptive timeouts; do not add unbounded blocking
   lock acquisition.
4. Use `server::execute::execute_command_with_env` or its cancellable variant
   for subprocesses and preserve the shell allowlist.
5. Keep collections bounded, errors visible, outputs deterministic, and secrets
   redacted.

Build in the worktree without stopping the installed runtime:

```bash
cd rust
cargo build --release
```

Use `lean-ctx dev-install` only when installation is explicitly requested; it
performs the short atomic stop/install/restart itself.

## Verify

1. Run the affected tests from `ctx_impact`, then the repository gate from
   `rust/`:

   ```bash
   cargo test --lib
   cargo clippy --all-features -- -D warnings
   cargo fmt --check
   ```

2. Run `scripts/preflight.sh fast` when the changed area is covered by the
   project preflight checks.
3. Confirm registry/schema tests pass for tool changes and regenerate committed
   contracts only when their source types changed.
4. Inspect the diff for secrets, hardcoded machine paths, omission markers,
   placeholders, weakened tests, and unrelated edits.
5. Use `$release-checklist` for version publication; do not tag or push during a
   development-only request.

