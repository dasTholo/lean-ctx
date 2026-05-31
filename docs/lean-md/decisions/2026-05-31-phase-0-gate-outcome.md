# Phase-0 Gate Outcome — lmd

**Date:** 2026-05-31
**Spec:** docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md
**Plan:** docs/lean-md/plans/2026-05-31-lmd-phase-0-gate.md

## 1. rushdown (R-1)

- Resolved version: `rushdown 0.18.0` — `rust/Cargo.toml` pins `rushdown = "0.18"`. Chosen over the
  plan's original `0.17` pin (0.18 is the latest published line; resolves and compiles cleanly into the crate).
- Extension path viable: **YES**. The Phase-0 spike (`rust/tests/lmd_rushdown_spike.rs`) renders both a custom
  `@upper` block directive (`@upper hello world` → `<p>HELLO WORLD</p>`) and a `{{ shout:TEXT }}` inline
  directive (`value is {{ shout:done }}` → `<p>value is done!</p>`) against the real rushdown 0.18 extension
  API. Both acceptance tests pass; spec-review confirmed the result is genuine (no faking / no API bypass).
- Working API notes (actual trait/method names used in the spike — the plan skeleton's names were hypothetical):
  - **Entry point:** `new_markdown_to_html(parser::Options, html::Options, parser_ext, renderer_ext)` returns a
    closure `Fn(&mut String, &str) -> Result<()>`, called as `render(&mut output, input)`. NOT the skeleton's
    2-arg string-returning form.
  - **Block parser:** trait `BlockParser` with `trigger() -> &[u8]` + `open(arena, parent, reader: &mut
    text::BasicReader, ctx: &mut parser::Context) -> Option<(NodeRef, State)>` (plus `cont`/`close`) — the method
    is `open`, NOT `parse`. `State` is a bitflags type (`NO_CHILDREN` / `HAS_CHILDREN` / `REQUIRE_PARAGRAPH`).
    `can_interrupt_paragraph() -> true` lets a bare directive line fire inside prose. The `open` guard returns
    `None` for non-matching lines so normal text is untouched. Wrap via `From<P> for AnyBlockParser::Extension`;
    register `p.add_block_parser(P::new, NoParserOptions, PRIORITY_ATX_HEADING)`.
  - **Inline parser:** trait `InlineParser` with `trigger() -> &[u8]` + `parse(arena, parent, reader: &mut
    text::BlockReader, ctx) -> Option<NodeRef>`. Gotcha: the inline dispatcher does NOT pre-consume the trigger
    byte — the parser must `reader.advance(full_match_len)`. Trigger `b"{"` does not collide with CommonMark.
    Wrap via `From<P> for AnyInlineParser::Extension`; register `p.add_inline_parser(P::new, NoParserOptions,
    PRIORITY_EMPHASIS + 100)`.
  - **Custom node:** `impl NodeKind` (`typ()` → `NodeType::LeafBlock` for the block / `NodeType::Inline` for
    inline, `kind_name()`), `impl PrettyPrint`, `impl From<X> for KindData::Extension`.
  - **Render hook:** `struct XHtmlRenderer<W: TextWrite>` + `impl RenderNode<W>::render_node(...) -> Result<WalkStatus>`
    (recover the node via `as_extension_data!(arena, node_ref, X)`, write with `html::Writer`), registered through
    `impl NodeRenderer<'cb, W>` → `nrr.register_node_renderer_fn(TypeId::of::<X>(), BoxRenderNode::new(self))`.
  - **Wiring:** `parser_extension(|p| ...)` and `renderer_extension(|r| ...)`, combined with `.and(...)`.

## 2. Necessity-audit (R/H/E) summary

- Source of truth: `src/lmd/audit.rs::directive_audit()` — 22 directives, guarded by three tests:
  `audit_covers_every_spec_directive` (coverage vs. a spec sentinel list, both directions),
  `router_and_hook_directives_have_a_backing`, and `audit_backing_files_exist` (every `src/...` backing path
  resolves on disk — the anchor-drift CI guard).
- Router directives (thin bridges): @read, @search, @list, @query, @graph, @remember, @recall, @env, @date, @count.
- Router+Hook (needed the H-check below before building): @phase, @on complete.
- Router+Extension: tdd-output.
- Extension (real engine work, ~6 primitives): @lean-md, @include, @import, @define, @call, @if, @consumer,
  {{ expr }}, @render.
- **H-check result for @phase / @on complete** (verified against src):
  - **Findings ARE already auto-tracked.** `server/mod.rs:1156` runs `auto_findings::extract(tool, output)` after
    every MCP tool call and, on a hit, calls `session.add_finding(...)` plus `auto_capture::capture_finding`
    (with `DEDUP_WINDOW_SECS` dedup). → An `@on complete` bridge that writes findings would **double-track**; the
    engine must NOT re-emit findings already captured by this hook — defer to it / reuse its dedup window.
  - **Decisions are NOT auto-tracked.** `session.add_decision(...)` is called only explicitly (the `ctx_session`
    tool, handoff/consolidation). → `@phase` writing a decision is genuinely additive; the bridge SHOULD write it
    through the same session API, with no double-tracking risk.

## 3. G-1 — @graph recent-neighbors

- `session.recently_touched_files()` exists: **NO (verified)**. There is no such accessor on `SessionState`;
  touched-files data only flows as an explicit `touched_files: &[String]` parameter into `core::intent_engine`
  (`from_file_patterns` / `from_query_with_session`), never exposed as a session-recent accessor.
- Decision: ~~**drop recent-neighbors from v1**~~. The other six `@graph` ops (via
  `graph_index` / `call_graph` / `graph_context`) carry v1; recent-neighbors can be added with a small session
  API in Phase 4 if real demand appears.

- **Correction (2026-05-31, post-gate — supersedes the decision above):** the "no accessor" finding was too
  literal (it searched only for a method literally named `recently_touched_files()`). The data source **does
  exist** and the exact recent-neighbors computation is **already wired in production**:
  - `SessionState.files_touched` is a public field (loaded via `SessionState::load_latest_for_project_root`).
  - `core/graph_context.rs:263 graph_neighbor_ranks_for_recent_files(root, recent, 40, 120)` exists.
  - `tools/ctx_semantic_search.rs:791-815 graph_rrf_ranks_for_search_root` already builds the recent list
    (`session.files_touched.iter().rev().filter(under_root).take(12).map(|f| f.path)`) and calls that helper.
  → `@graph recent-neighbors` is a **thin R-router** like the other six graph ops — **no new session API**, and
  it **stays in v1** (no Phase-4 deferral). Dashboard `/#compression` recent-files is a separate recency source
  (`core/bounce_tracker.rs` `recent_reads`/`recently_edited`); the `@graph` op reuses `session.files_touched`.

## 4. Final v1 directive scope carried into Phase 1

- BUILD in v1: all 22 audit entries — the 10 Router bridges; @phase + @on complete (per the H-check: @phase
  writes decisions, @on complete defers finding-writes to the existing `auto_findings` hook); tdd-output; and the
  9 Extension constructs.
- DEFER:
  - ~~@graph **recent-neighbors** sub-op — per G-1 (no session accessor); revisit in Phase 4.~~
    **Corrected (see §3 Correction):** now **BUILD in v1** — `session.files_touched` +
    `graph_neighbor_ranks_for_recent_files` exist and are already wired in `ctx_semantic_search`.
  - `@if`'s `evalexpr` backing — Phase 3 (YAGNI; the dep was deliberately not added in Phase 0).
  - finding-writes inside `@on complete` — defer to the existing auto_findings hook rather than re-implement.

## 5. Gate verdict

- **PASS → proceed to the Phase-1 plan.** The rushdown 0.18 extension path is viable (spike green and verified
  genuine), the executable R/H/E audit is in place and guarded by CI tests, and both open points are resolved:
  R-1 (rushdown version/ergonomics) and G-1 (recent-neighbors). No fallback to the preprocessor path is required.
