# lmd Phase 0 — Necessity-Audit & rushdown Spike (Gate) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce the Phase-0 gate artifacts for the native `lmd` engine — an executable R/H/E necessity-audit and a rushdown extension spike — that together decide the final v1 directive scope before any parser/engine code is written.

**Architecture:** `lmd` extends the existing `lean-ctx` Rust crate. Phase 0 adds a thin `src/lmd/` module containing only the audit data (no engine logic yet), plus an integration test that exercises the real `rushdown` extension API (one custom `@`-block + one `{{ }}`-inline directive). The audit is "executable": a test asserts every Router/Hook directive's backing symbol file actually exists, so CI fails if the verified code-base anchors (`structured_read`, `graph_index`, `session::state`, …) are renamed or removed. The spike's render assertion is the go/no-go criterion for the rushdown extension path; a documented abort criterion routes to the preprocessor fallback.

**Tech Stack:** Rust 2024 edition (`lean-ctx` crate; bumped from 2021 in the Pre-Phase), `rushdown` (CommonMark 0.31.2 + GFM, extension API), `cargo nextest` for tests. No `evalexpr` yet (deferred to Phase 3 — YAGNI).

**Scope note:** This is the gate plan only (spec §3.1 Phase 0, §6 row 0). Phases 1–8 get their own plans, written *after* this gate passes, because the rushdown 0.17 API ergonomics (open point R-1) and the audit outcome determine their concrete content. Writing code-complete parser tasks before the spike would be speculation. See the "Gate Outcome → Next Plans" section at the end.

**Project rules baked into every command below:**
- Tests: always `cargo nextest run`, never `cargo test`.
- Before every `git add`: run `mcp__jetbrains__reformat_file` on each changed file.
- No `&&` chains in Bash — run each command separately.
- Working directory for all `cargo`/`git` commands: `rust/` (the crate root).
- Code and code comments: English.

---

## File Structure

Files created or modified in this plan, each with one responsibility:

- Create `rust/.cargo/config.toml` — project build-toolchain config: `mold` linker via `clang`, Cranelift codegen backend for fast dev builds, incremental builds, parallel jobs (Pre-Phase).
- Create `rust/rust-toolchain.toml` — pin the crate to `nightly` + the Cranelift component, because `.cargo/config.toml`'s `[unstable]` table only works on nightly (Pre-Phase).
- Create `rust/src/lmd/mod.rs` — `lmd` module root. Phase 0: declares `pub mod audit;` only. No engine logic.
- Create `rust/src/lmd/audit.rs` — the executable R/H/E necessity-audit: `DirectiveClass` enum, `DirectiveAudit` struct, `directive_audit()` returning the full directive table with backing-symbol paths and bridge-line estimates. Unit tests live inline.
- Modify `rust/src/lib.rs:52` — register `pub mod lmd;`.
- Modify `rust/Cargo.toml` — bump `edition` from `2021` to `2024` (Pre-Phase) and add the `rushdown` dependency (version pinned to whatever resolves; recorded in the decision doc).
- Create `rust/tests/lmd_rushdown_spike.rs` — integration test: register one custom `@`-block parser and one `{{ }}`-inline parser via the real rushdown extension API, render a fixture, assert output. This is the Phase-0 acceptance test (spec §8.1).
- Create `docs/lean-md/decisions/2026-05-31-phase-0-gate-outcome.md` — the gate's structured output: resolved rushdown version, extension-path viable (yes/no), G-1 decision (`@graph recent-neighbors`), and the final v1 directive set carried forward from the audit. Feeds the Phase-1 plan.

---

## Pre-Phase: Build-toolchain config (mold + Cranelift)

Set up the project's fast-build toolchain before any code. This config uses the `mold` linker (via `clang`) and the Cranelift codegen backend for dev builds. **`[unstable] codegen-backend` and `codegen-backend = "cranelift"` require the nightly toolchain** — so this pre-phase also pins the crate to nightly. Every `cargo build` / `cargo nextest run` in the later tasks runs under this toolchain.

**Files:**
- Create: `rust/.cargo/config.toml`
- Create: `rust/rust-toolchain.toml`

- [ ] **Step 1: Verify host prerequisites are installed**

Run each separately (no `&&` chains):

```bash
mold --version
clang --version
rustc +nightly --version
```

Expected: `mold` and `clang` print versions; a `nightly` toolchain is available.
If `mold` is missing: install it (e.g. `apt install mold` or from the mold release).
If `clang` is missing: install it (e.g. `apt install clang`).
If nightly is missing: `rustup toolchain install nightly`.

- [ ] **Step 2: Add the Cranelift codegen component to nightly**

Run: `rustup component add rustc-codegen-cranelift-preview --toolchain nightly`
Expected: component installed (or "already installed"). This is what makes `codegen-backend = "cranelift"` resolvable.

- [ ] **Step 3: Write the cargo config**

Create `rust/.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[build]
incremental = true
jobs = 15

[unstable]
codegen-backend = true

# Cranelift for fast dev builds, LLVM (default) for optimized release builds
[profile.dev]
codegen-backend = "cranelift"
opt-level = 0

# Release uses LLVM (default) for optimized binaries
[profile.release]
# codegen-backend = "llvm"
opt-level = 3
```

**Caveat to be aware of (no action required):** `rust/Cargo.toml` already defines `[profile.release] opt-level = "z"`. Manifest profiles take precedence over `.cargo/config.toml` profiles, so the effective release `opt-level` stays `"z"`; the `opt-level = 3` above is shadowed. Leave it — changing release behavior is out of scope for this plan. If a true `opt-level = 3` release is ever wanted, edit `rust/Cargo.toml`, not this config.

- [ ] **Step 4: Pin the toolchain to nightly**

Create `rust/rust-toolchain.toml`:

```toml
[toolchain]
channel = "nightly"
components = ["rustc-codegen-cranelift-preview"]
```

This makes every `cargo` invocation in `rust/` use nightly automatically (required because `.cargo/config.toml` contains an `[unstable]` table, which stable cargo rejects). **Team/CI note:** if CI must stay on stable for the release build, do *not* commit this file and instead invoke dev builds explicitly with `cargo +nightly ...`; otherwise commit it so the toolchain is reproducible.

- [ ] **Step 5: Bump the crate to Rust edition 2024**

In `rust/Cargo.toml`, change the `[package]` edition field:

```toml
edition = "2024"
```

(It is currently `edition = "2021"`.) Edition 2024 needs rustc ≥ 1.85; the nightly pinned in Step 4 satisfies this.

- [ ] **Step 6: Apply edition migrations and verify the build**

Edition 2024 has breaking changes (e.g. stricter `unsafe` in `extern`, `gen` keyword reservation, RPIT lifetime capture, `Future`/`IntoIterator` prelude additions) that can affect a crate this large. Run the migration helper, then build:

```bash
cargo fix --edition --lib --allow-dirty --allow-staged
cargo build
```

Expected: `cargo fix` rewrites any 2024-incompatible spots; `cargo build` then succeeds on nightly + Cranelift + mold. If `cargo build` surfaces residual edition errors that `cargo fix` could not auto-resolve, fix each reported `*.rs` site **using Serena tools** (`mcp__serena__jet_brains_find_symbol` + `replace_symbol_body`/`replace_content`) — never native edits on Rust files (project Hard Rule) — until the build is clean. Also confirm no `unstable`-table rejection error appears (that would mean stable cargo is still active — re-check Step 4).

- [ ] **Step 7: Sanity-check the test runner under the new toolchain**

Run (from `rust/`): `cargo nextest run --no-run`
Expected: the test harness compiles cleanly under edition 2024 + nightly (no execution yet — this just proves the migration did not break test compilation).

- [ ] **Step 8: Commit**

```bash
# reformat via mcp__jetbrains__reformat_file on any *.rs files cargo fix touched
git add .cargo/config.toml rust-toolchain.toml Cargo.toml Cargo.lock
# stage any *.rs files modified by `cargo fix --edition`, by name
git commit -m "build: edition 2024, mold linker + Cranelift dev backend, pin nightly"
```

---

## Task 1: Scaffold the `lmd` module

**Files:**
- Create: `rust/src/lmd/mod.rs`
- Create: `rust/src/lmd/audit.rs` (stub for now)
- Modify: `rust/src/lib.rs:52`

- [ ] **Step 1: Create the module root**

Create `rust/src/lmd/mod.rs`:

```rust
//! `lmd` — native lean-ctx Live-Markdown engine.
//!
//! Phase 0 contains only the executable R/H/E necessity-audit (`audit`).
//! No parser or bridge logic exists yet; that scope is decided by the
//! Phase-0 gate (see docs/lean-md/decisions/2026-05-31-phase-0-gate-outcome.md).

pub mod audit;
```

- [ ] **Step 2: Create the audit stub so the module compiles**

Create `rust/src/lmd/audit.rs`:

```rust
//! Executable R/H/E necessity-audit for lmd directives (spec §3.1).
```

- [ ] **Step 3: Register the module in the crate root**

In `rust/src/lib.rs`, after the existing `pub mod lsp;` line (currently line 32) keep alphabetical-ish ordering by inserting `pub mod lmd;` right before `pub mod lsp;`:

```rust
pub mod lmd;
pub mod lsp;
```

- [ ] **Step 4: Verify the crate still compiles**

Run (from `rust/`): `cargo build`
Expected: builds successfully; `lmd` and `lmd::audit` modules now exist (empty).

- [ ] **Step 5: Commit**

Reformat changed files first, then stage by name:

```bash
# reformat via mcp__jetbrains__reformat_file on: src/lmd/mod.rs, src/lmd/audit.rs, src/lib.rs
git add src/lmd/mod.rs src/lmd/audit.rs src/lib.rs
git commit -m "feat(lmd): scaffold lmd module for Phase-0 audit"
```

---

## Task 2: R/H/E audit data structure

The audit classifies every spec directive as **R** (Router → existing lean-ctx core API), **H** (Hook layer), **E** (rushdown extension), or the mixes **R+H** / **R+E** (spec §3.1). For Router/Hook directives it records the backing source file, so the next task can assert those anchors exist.

**Files:**
- Modify: `rust/src/lmd/audit.rs`
- Test: `rust/src/lmd/audit.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write the failing test for the audit shape**

Append to `rust/src/lmd/audit.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// The exact set of directives the v0.7 spec (§3.1) enumerates. This
    /// sentinel list is the contract: if the audit drifts from the spec,
    /// this test fails.
    const SPEC_DIRECTIVES: &[&str] = &[
        "@read",
        "@search",
        "@list",
        "@query",
        "@graph",
        "@remember",
        "@recall",
        "@env",
        "@date",
        "@count",
        "@phase",
        "@on complete",
        "@lean-md",
        "@include",
        "@import",
        "@define",
        "@call",
        "@if",
        "@consumer",
        "{{ expr }}",
        "@render",
        "tdd-output",
    ];

    #[test]
    fn audit_covers_every_spec_directive() {
        let audit = directive_audit();
        let names: Vec<&str> = audit.iter().map(|d| d.directive).collect();
        for expected in SPEC_DIRECTIVES {
            assert!(
                names.contains(expected),
                "audit is missing spec directive `{expected}`"
            );
        }
        assert_eq!(
            audit.len(),
            SPEC_DIRECTIVES.len(),
            "audit has entries not present in the spec sentinel list"
        );
    }

    #[test]
    fn router_and_hook_directives_have_a_backing() {
        for entry in directive_audit() {
            if matches!(
                entry.class,
                DirectiveClass::Router | DirectiveClass::Hook | DirectiveClass::RouterHook
            ) {
                assert!(
                    !entry.backing.is_empty(),
                    "directive `{}` is Router/Hook but has no backing",
                    entry.directive
                );
            }
        }
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run (from `rust/`): `cargo nextest run --lib lmd::audit`
Expected: FAIL — `directive_audit`, `DirectiveAudit`, `DirectiveClass` are not defined.

- [ ] **Step 3: Implement the audit types and table**

Insert at the top of `rust/src/lmd/audit.rs`, after the module doc comment and before the `#[cfg(test)]` block:

```rust
/// Necessity classification for an lmd directive (spec §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveClass {
    /// Thin alias over an existing lean-ctx core API. No new logic.
    Router,
    /// Already handled (better) by the hook layer; engine must not double-track.
    Hook,
    /// A genuine rushdown engine construct with no lean-ctx equivalent.
    Extension,
    /// Router behavior plus a hook double-tracking check.
    RouterHook,
    /// Router data plus a render-side extension hook.
    RouterExtension,
}

/// One row of the executable necessity-audit.
#[derive(Debug, Clone)]
pub struct DirectiveAudit {
    /// Directive token as written in an `.lmd.md` source.
    pub directive: &'static str,
    /// R / H / E classification.
    pub class: DirectiveClass,
    /// For Router/Hook directives: crate-relative source file of the backing
    /// API (e.g. `src/core/structured_read.rs`). `std` / `rushdown` / `chrono`
    /// for non-source backings. Empty only for pure-Extension entries with no
    /// stable anchor yet.
    pub backing: &'static str,
    /// Rough bridge-size estimate in lines (spec §3.1 "Bridge-Zeilenschätzung").
    pub est_bridge_lines: u32,
    /// Free-form note: H-checks, deferred ops, fallbacks.
    pub note: &'static str,
}

/// The full v1 necessity-audit table (spec §3.1). This is the executable
/// artifact: `audit_backing_files_exist` (Task 3) asserts every `src/...`
/// backing actually resolves on disk, turning the audit into a CI guard
/// against anchor drift.
#[must_use]
pub fn directive_audit() -> Vec<DirectiveAudit> {
    use DirectiveClass::{Extension, Hook, Router, RouterExtension, RouterHook};
    vec![
        DirectiveAudit {
            directive: "@read",
            class: Router,
            backing: "src/core/structured_read.rs",
            est_bridge_lines: 20,
            note: "routes to core::structured_read / ctx_read",
        },
        DirectiveAudit {
            directive: "@search",
            class: Router,
            backing: "src/tools/ctx_search.rs",
            est_bridge_lines: 20,
            note: "routes to ctx_search",
        },
        DirectiveAudit {
            directive: "@list",
            class: Router,
            backing: "src/tools/ctx_tree.rs",
            est_bridge_lines: 20,
            note: "routes to ctx_tree",
        },
        DirectiveAudit {
            directive: "@query",
            class: Router,
            backing: "src/shell/exec.rs",
            est_bridge_lines: 30,
            note: "shell/exec + compress; same allowlist/redaction as ctx_shell (security gate §7)",
        },
        DirectiveAudit {
            directive: "@graph",
            class: Router,
            backing: "src/core/graph_index.rs",
            est_bridge_lines: 80,
            note: "7 ops via graph_index/call_graph/graph_context; recent-neighbors gated on G-1",
        },
        DirectiveAudit {
            directive: "@remember",
            class: Router,
            backing: "src/core/knowledge/core.rs",
            est_bridge_lines: 15,
            note: "ctx_knowledge remember; profile=skill only (§7)",
        },
        DirectiveAudit {
            directive: "@recall",
            class: Router,
            backing: "src/core/knowledge/query.rs",
            est_bridge_lines: 15,
            note: "ctx_knowledge recall_for_output, no_track",
        },
        DirectiveAudit {
            directive: "@env",
            class: Router,
            backing: "std",
            est_bridge_lines: 8,
            note: "std::env",
        },
        DirectiveAudit {
            directive: "@date",
            class: Router,
            backing: "chrono",
            est_bridge_lines: 8,
            note: "chrono (already a dep)",
        },
        DirectiveAudit {
            directive: "@count",
            class: Router,
            backing: "glob",
            est_bridge_lines: 10,
            note: "glob (already a dep)",
        },
        DirectiveAudit {
            directive: "@phase",
            class: RouterHook,
            backing: "src/core/session/state.rs",
            est_bridge_lines: 25,
            note: "session add_decision/add_finding; H-check: does a hook already track this?",
        },
        DirectiveAudit {
            directive: "@on complete",
            class: RouterHook,
            backing: "src/core/session/state.rs",
            est_bridge_lines: 15,
            note: "session add_finding; same H-check as @phase",
        },
        DirectiveAudit {
            directive: "@lean-md",
            class: Extension,
            backing: "",
            est_bridge_lines: 40,
            note: "header config parser",
        },
        DirectiveAudit {
            directive: "@include",
            class: Extension,
            backing: "",
            est_bridge_lines: 60,
            note: "file inline (content visible) + jail (§7)",
        },
        DirectiveAudit {
            directive: "@import",
            class: Extension,
            backing: "",
            est_bridge_lines: 40,
            note: "definitions-only scope + jail",
        },
        DirectiveAudit {
            directive: "@define",
            class: Extension,
            backing: "",
            est_bridge_lines: 70,
            note: "macro engine; no lean-ctx equivalent",
        },
        DirectiveAudit {
            directive: "@call",
            class: Extension,
            backing: "",
            est_bridge_lines: 50,
            note: "macro invocation with param substitution",
        },
        DirectiveAudit {
            directive: "@if",
            class: Extension,
            backing: "",
            est_bridge_lines: 60,
            note: "container transformer + evalexpr (Phase 3 dep)",
        },
        DirectiveAudit {
            directive: "@consumer",
            class: Extension,
            backing: "",
            est_bridge_lines: 30,
            note: "ai/human audience transformer only (§10)",
        },
        DirectiveAudit {
            directive: "{{ expr }}",
            class: Extension,
            backing: "",
            est_bridge_lines: 40,
            note: "inline eval / AstTransformer",
        },
        DirectiveAudit {
            directive: "@render",
            class: Extension,
            backing: "",
            est_bridge_lines: 40,
            note: "postfix pipe AstTransformer (| @render type=table)",
        },
        DirectiveAudit {
            directive: "tdd-output",
            class: RouterExtension,
            backing: "src/core/tdd_schema.rs",
            est_bridge_lines: 35,
            note: "tdd_schema (R) + render hook (E); modes tdd/compact/off",
        },
    ]
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run (from `rust/`): `cargo nextest run --lib lmd::audit`
Expected: PASS — both `audit_covers_every_spec_directive` and `router_and_hook_directives_have_a_backing` green.

- [ ] **Step 5: Commit**

```bash
# reformat via mcp__jetbrains__reformat_file on: src/lmd/audit.rs
git add src/lmd/audit.rs
git commit -m "feat(lmd): executable R/H/E necessity-audit table"
```

---

## Task 3: Make the audit executable — assert backing files exist

This is what turns the audit from documentation into a CI guard: every `src/...` backing path must resolve to a real file under the crate. If `structured_read.rs`, `graph_index.rs`, `session/state.rs`, `tdd_schema.rs`, etc. are ever renamed, this test fails and forces the audit to be updated.

**Files:**
- Modify: `rust/src/lmd/audit.rs` (extend the `#[cfg(test)]` block)

- [ ] **Step 1: Write the failing test**

Add this test inside the existing `mod tests` block in `rust/src/lmd/audit.rs`:

```rust
    #[test]
    fn audit_backing_files_exist() {
        // CARGO_MANIFEST_DIR is the `rust/` crate root at compile time.
        let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for entry in directive_audit() {
            if entry.backing.starts_with("src/") {
                let path = crate_root.join(entry.backing);
                assert!(
                    path.exists(),
                    "backing file `{}` for directive `{}` does not exist — \
                     the verified code anchor moved; update the audit",
                    entry.backing,
                    entry.directive
                );
            }
        }
    }
```

- [ ] **Step 2: Run the test to verify it fails (then confirms anchors)**

Run (from `rust/`): `cargo nextest run --lib lmd::audit::tests::audit_backing_files_exist`
Expected: At first run it compiles and then PASSES if all anchors are correct. If any `src/...` path is wrong, it FAILS naming the bad path. Fix any path in the audit table to match the real file location (use `mcp__serena__jet_brains_find_symbol` to locate the moved symbol), then re-run until PASS.

- [ ] **Step 3: Verify the full audit module is green**

Run (from `rust/`): `cargo nextest run --lib lmd::audit`
Expected: PASS — all three audit tests green.

- [ ] **Step 4: Commit**

```bash
# reformat via mcp__jetbrains__reformat_file on: src/lmd/audit.rs
git add src/lmd/audit.rs
git commit -m "test(lmd): assert audit backing files exist (anchor-drift guard)"
```

---

## Task 4: Add the rushdown dependency and pin the version

The spike needs `rushdown`. The exact version is open point R-1 — pin whatever resolves and record it in the decision doc (Task 6). `evalexpr` is **not** added here (it backs `@if` in Phase 3 — YAGNI).

**Files:**
- Modify: `rust/Cargo.toml`

- [ ] **Step 1: Add the dependency**

Add to the `[dependencies]` section of `rust/Cargo.toml` (near the other parser/text deps such as `similar` and `regex`):

```toml
rushdown = "0.17"
```

- [ ] **Step 2: Resolve and lock**

Run (from `rust/`): `cargo update -p rushdown --precise 0.17.0`
If that exact version is unavailable, instead run: `cargo fetch`
Then capture the resolved version (from `rust/`): `cargo tree -p rushdown --depth 0`
Expected: prints the resolved `rushdown vX.Y.Z`. **Record this exact version string** — it goes into the decision doc in Task 6.

- [ ] **Step 3: Verify it compiles into the crate**

Run (from `rust/`): `cargo build`
Expected: builds; `rushdown` is now linkable.

**Abort criterion (R-1):** If `rushdown` is not publishable/fetchable from the configured registry, or no 0.1x line exists, STOP this task and record in the decision doc (Task 6) that the dependency is unavailable. The gate then fails toward the preprocessor fallback (see Task 5 abort criterion). Do not fabricate a version.

- [ ] **Step 4: Commit**

```bash
# reformat is not applicable to Cargo.toml/Cargo.lock; stage directly
git add Cargo.toml Cargo.lock
git commit -m "build(lmd): add rushdown dependency for Phase-0 spike"
```

---

## Task 5: rushdown extension spike — one custom block + one inline directive

The acceptance test for the gate (spec §8.1): prove the rushdown extension API can express (a) a custom `@`-prefixed block directive and (b) a `{{ }}` inline directive, and that both render. The code below follows the API shape documented in spec §4.3 (`parser_extension`, `add_block_parser`, `add_inline_parser`, `trigger()`, `parse()`, `NodeKind`, `RenderNode`, `.and()` → `new_markdown_to_html`). Because the precise 0.17 ergonomics are R-1, **the exact symbol names may differ** — the spike's job is to reconcile this code with the real API and record the working form.

**Files:**
- Create: `rust/tests/lmd_rushdown_spike.rs`

- [ ] **Step 1: Write the spike acceptance test**

Create `rust/tests/lmd_rushdown_spike.rs`:

```rust
//! Phase-0 rushdown spike (spec §8.1).
//!
//! Acceptance: a custom `@upper { ... }` block directive and a `{{ shout }}`
//! inline directive both parse via the rushdown extension API and render to
//! the expected HTML/text. This is the go/no-go for the extension path.
//!
//! If the rushdown 0.17 API cannot express these cleanly, see the abort
//! criterion in the Phase-0 plan (Task 5) — the fallback is a preprocessor
//! stage in front of rushdown, recorded in the gate decision doc.

use rushdown::*;

/// Minimal block parser: turns a line `@upper TEXT` into uppercased text.
/// NOTE: adjust the trait/method names to the resolved rushdown 0.17 API.
struct UpperBlock;

/// Minimal inline parser: turns `{{ shout:TEXT }}` into `TEXT!`.
struct ShoutInline;

fn lmd_spike_extension() -> impl ParserExtension {
    parser_extension(|p| {
        p.add_block_parser(UpperBlock::new, NoParserOptions, PRIORITY_BLOCK_HIGH);
        p.add_inline_parser(ShoutInline::new, NoParserOptions, PRIORITY_EMPHASIS + 100);
    })
}

#[test]
fn custom_block_directive_renders() {
    let md = "@upper hello world\n";
    let html = new_markdown_to_html(md, lmd_spike_extension());
    assert!(
        html.contains("HELLO WORLD"),
        "custom @upper block did not render uppercased; got: {html}"
    );
}

#[test]
fn inline_directive_renders() {
    let md = "value is {{ shout:done }}\n";
    let html = new_markdown_to_html(md, lmd_spike_extension());
    assert!(
        html.contains("done!"),
        "inline {{{{ shout }}}} directive did not render; got: {html}"
    );
}
```

- [ ] **Step 2: Reconcile the skeleton with the real rushdown API**

Open the resolved rushdown source/docs and fill in the `impl` bodies for `UpperBlock` and `ShoutInline` against the actual trait surface. Use these commands to find the real API (from `rust/`):

```
# locate the rushdown crate source in the cargo registry cache
cargo doc -p rushdown --no-deps
```

Then read the rushdown extension example it ships (the spec cites `tests/extension.rs` as the template). Implement:
- `UpperBlock::new` constructor, `trigger() -> &[u8]` returning `b"@"`, `parse() -> Option<NodeRef>` that consumes `@upper <text>` and produces a `NodeKind` node carrying the uppercased text.
- `ShoutInline::new`, `trigger()` for `{{`, `parse()` producing the `TEXT!` node.
- A `RenderNode` impl (or the 0.17 equivalent) for each node kind that writes the rendered output.

Keep the two `#[test]` assertions from Step 1 unchanged — they define success.

- [ ] **Step 3: Run the spike**

Run (from `rust/`): `cargo nextest run --test lmd_rushdown_spike`
Expected (success path): both `custom_block_directive_renders` and `inline_directive_renders` PASS.

**Abort criterion (gate fail → fallback):** If the rushdown extension API cannot express a `@`-block and a `{{ }}`-inline parser after a genuine attempt (e.g. block parsers cannot trigger on `@`, or inline `{{` collides irreconcilably with CommonMark), STOP. Record in the decision doc (Task 6): extension-path = NOT viable, and that Phase 1+ must use the **preprocessor fallback** (a pre-rushdown text pass that rewrites `@`/`{{ }}` directives into placeholder nodes). Do not force a broken extension.

- [ ] **Step 4: Commit**

```bash
# reformat via mcp__jetbrains__reformat_file on: tests/lmd_rushdown_spike.rs
git add tests/lmd_rushdown_spike.rs
git commit -m "test(lmd): rushdown extension spike — custom block + inline directive"
```

---

## Task 6: Record the gate decision

The gate's output is a structured decision doc that the Phase-1 plan consumes. It pins the rushdown version, states whether the extension path is viable, resolves G-1, and freezes the v1 directive set from the audit.

**Files:**
- Create: `docs/lean-md/decisions/2026-05-31-phase-0-gate-outcome.md`

- [ ] **Step 1: Write the decision document**

Create `docs/lean-md/decisions/2026-05-31-phase-0-gate-outcome.md` and fill every bracketed value from the actual results of Tasks 2–5:

```markdown
# Phase-0 Gate Outcome — lmd

**Date:** 2026-05-31
**Spec:** docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md
**Plan:** docs/lean-md/plans/2026-05-31-lmd-phase-0-gate.md

## 1. rushdown (R-1)
- Resolved version: `rushdown <X.Y.Z>`  <!-- from Task 4 Step 2 -->
- Extension path viable: **<YES | NO>**  <!-- from Task 5 Step 3 -->
- If NO: Phase 1+ uses the preprocessor fallback (rewrite @/{{ }} before rushdown).
- Working API notes (actual trait/method names used in the spike):
  - block parser: <...>
  - inline parser: <...>
  - render hook: <...>

## 2. Necessity-audit (R/H/E) summary
- Source of truth: `src/lmd/audit.rs::directive_audit()` (22 directives, guarded by tests).
- Router directives (thin bridges): @read, @search, @list, @query, @graph,
  @remember, @recall, @env, @date, @count.
- Router+Hook (need H-check before building): @phase, @on complete.
- Router+Extension: tdd-output.
- Extension (real engine work, ~6 primitives): @lean-md, @include, @import,
  @define, @call, @if, @consumer, {{ expr }}, @render.
- H-check result for @phase / @on complete: <does a hook already track session
  decisions/findings? YES → engine must not double-track / NO → bridge writes>.

## 3. G-1 — @graph recent-neighbors
- `session.recently_touched_files()` exists: **<NO (verified) | NOW ADDED>**.
- Decision: **<drop recent-neighbors from v1 | add small session API in Phase 4>**.

## 4. Final v1 directive scope carried into Phase 1
- BUILD in v1: <list, default = all audit entries except deferred ops>.
- DEFER: <e.g. @graph recent-neighbors per G-1; anything the spike blocked>.

## 5. Gate verdict
- **<PASS → proceed to Phase-1 plan | FAIL → preprocessor-fallback re-plan>**.
```

- [ ] **Step 2: Verify the whole Phase-0 surface is green**

Run (from `rust/`): `cargo nextest run --lib lmd::audit`
Run (from `rust/`): `cargo nextest run --test lmd_rushdown_spike`
Expected: all audit tests PASS; spike tests PASS (or the abort criterion is recorded in the decision doc with verdict FAIL).

- [ ] **Step 3: Commit**

```bash
git add docs/lean-md/decisions/2026-05-31-phase-0-gate-outcome.md
git commit -m "docs(lmd): record Phase-0 gate outcome (rushdown version, audit, G-1)"
```

---

## Self-Review (plan author checklist — already applied)

**Spec coverage (Phase 0 only, spec §3.1 + §6 row 0):**
- R/H/E necessity-audit as executable artifact → Tasks 2 & 3. ✅
- rushdown 0.17 spike (1 block + 1 inline, template `tests/extension.rs`) → Task 5. ✅
- Audit decides final v1 scope → Task 6 decision doc §4. ✅
- Abort criterion if rushdown API unsuitable → Task 5 abort criterion + Task 6 fallback. ✅
- G-1 (`session.recently_touched_files()` missing) resolved in Phase 0 → Task 6 §3. ✅
- R-1 (rushdown version/ergonomics) fixed in Phase 0 → Tasks 4 & 6 §1. ✅
- Q-05 (`@phase` error behavior) — correctly **out of scope** here; spec §9 defers it to the executing-plans migration, not the engine spec. Not a gap.

**Infra (user-requested, not spec):** Pre-Phase adds the `mold` + Cranelift build config and pins nightly; all later `cargo` commands run under it. ✅

**Type consistency:** `DirectiveClass` / `DirectiveAudit` / `directive_audit()` used identically across Tasks 2, 3, and the decision doc. Test names stable (`audit_covers_every_spec_directive`, `router_and_hook_directives_have_a_backing`, `audit_backing_files_exist`).

**Placeholder scan:** The only bracketed `<...>` values live in the Task-6 decision *template*, which is intentionally fill-in-from-results — that is the artifact's purpose, not a plan placeholder. All code steps carry complete code.

---

## Gate Outcome → Next Plans (written after this gate passes)

Once `docs/lean-md/decisions/2026-05-31-phase-0-gate-outcome.md` records **PASS**, write the subsequent plans (one per spec §6 phase group), each grounded in the now-known rushdown API:

1. **Phase 1 plan** — header parser (`@lean-md`) + block/inline parser + `DirectiveBridge` registry + built-in-first fragment resolver + first `@read` router rendering end-to-end (spec §4, §6 row 1).
2. **Phase 2 plan** — the R-bridges `@read`/`@search`/`@list`/`@query`/`@graph`/`@env`/`@date`/`@count` (spec §6 row 2), each a unit test against its audited backing API (§8.5).
3. **Phase 3 plan** — E-constructs: `@define`/`@call`, `@import`, `@if`/`@consumer`, `{{ }}`, pipe/`@render`; adds `evalexpr` (spec §6 row 3).
4. **Phase 4 plan** — `@phase`/`@on complete` (with the H-check from the decision doc) + `@remember`/`@recall` (spec §6 row 4).
5. **Phase 5 plan** — `@dispatch` + tool-discipline constraint injection + hook-gap closure (spec §3.5, §6 row 5) — the core anti-drift work.
6. **Phase 6 plan** — TDD render hook (`tdd_schema`) (spec §6 row 6).
7. **Phase 7 plan** — `ctx_md_*` MCP tools + `lean-ctx md` CLI (spec §4.4, §6 row 7).
8. **Phase 8 plan** — pilot migration `mdai-brainstorm` + golden-parity & phase-isolation token tests against benchmark targets (spec §5.1, §8.2–8.3, §6 row 8).

If the gate records **FAIL** (extension path not viable), the Phase-1 plan is instead written around the preprocessor fallback, and the directive surface stays identical (only the parsing mechanism changes).
