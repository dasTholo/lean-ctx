# Cursor skill index

| Skill | Triggers | Purpose |
|---|---|---|
| `agent-orchestration` | multi-agent round, Codex sub-agents, worktrees, `codex exec`, agents starten, Runde | Split work safely across isolated Codex CLI agents, review results, and merge them. |
| `crate-quality-check` | Rust quality gate, Cargo test, Clippy, rustfmt, rustdoc, TODO/FIXME audit | Run and interpret a production Rust crate quality gate. |
| `enterprise-deploy` | enterprise deploy, production rollout, Docker Compose, health check, rollback | Deploy or roll back the enterprise suite with explicit safety confirmations. |
| `frontend-dev` | dashboard, admin UI, portal, Next.js, React, TypeScript, ESLint, frontend build | Develop and verify the enterprise frontends with deterministic installs and full UI checks. |
| `lean-ctx-dev` | add `ctx_*` tool, ToolRegistry, core module, PathJail, bounded lock, preflight, dev-install | Develop lean-ctx using current wiring, security boundaries, and repository quality gates. |
| `new-api-endpoint` | API endpoint, Axum route, handler, dashboard proxy, Suite API docs | Add an authenticated enterprise Suite endpoint with routing, proxy, tests, and documentation. |
| `release-checklist` | release, version bump, changelog, Git tag, Homebrew, AUR | Prepare, publish, verify, or recover a lean-ctx release without moving published tags. |
