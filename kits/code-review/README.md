# Code Review Context Kit

Load the kit for the current user:

```bash
lean-ctx kit load code-review
```

The command validates the kit and persists `active_kit = "code-review"` in the
lean-ctx configuration. It then makes automatic reads use the first compatible
mode: Rust source uses the outline (`map`) stage, TOML is read in full, and
tests use signatures. The complete Rust review plan remains available through
the kit so a reviewing agent can request its follow-up `signatures` pass.

Useful companion commands:

```bash
lean-ctx kit show code-review
lean-ctx kit list
lean-ctx kit unload
```

The kit prioritizes files changed in the current branch and records the
compressed `cargo test` and `cargo clippy` evidence a review should use.
