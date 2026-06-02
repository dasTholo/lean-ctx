# lmd F-1/F-2 Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the two deferred Phase-1 follow-ups from the lmd design spec §9 — F-1 (Read→Delta observability) and F-2 (HTML-comment injection in the render fallback) — and then update the spec to record the resolution.

**Architecture:** Evidence-based, minimal scope. A live measurement (2026-06-02, real `ctx_read` runtime, cold cache) proved the `[unchanged]` cache-hit stub is a **`mode=full` feature** that already works correctly; auto-mode re-reads are already compact (large files) or trivially small (small files). Therefore **F-1 needs NO production `ctx_read`/`cache` change** — it collapses to correcting the wrongly-ignored lmd engine test (which expected `auto` 2-reads to stub) plus a proof-line-safe fixture. F-2 is a genuine (minor) lmd-local fix: restrict the inline directive-name charset to match the block grammar and HTML-comment-sanitize the render fallback. `@read` keeps its `auto` default by design.

**Tech Stack:** Rust, rushdown 0.18 parser/renderer extensions, `cargo nextest`. Rust edits via Serena tools only. `mcp__jetbrains__reformat_file` on every changed file before `git add`.

**Measurement evidence (decision basis — do not re-litigate):**

| File | Mode | Read 1 | Read 2 | R2 saving |
|---|---|---|---|---|
| small (3L, ~12 tok) | `auto` | full ~12 tok | **full re-dump** ~12 tok | ~0 % |
| small | `full` | full ~12 tok | stub `[unchanged 3L \| "fn main() {"]` ~15 tok | none (stub ≥ content) |
| large (401L, ~5131 tok) | `auto` | full 5123 tok | **compact** ~50 tok | ~99 % (already, no stub) |
| large | `full` | full 5131 tok | stub `[unchanged 401L \| "//! …"]` ~20 tok | ~99.6 % |

Conclusion: the stub works; forcing `mode=full` would defeat auto-compression; extending the stub to `auto` buys ~nothing. F-1 = test correctness only.

---

## File Structure

- **Modify** `rust/src/lmd/parser/inline.rs` — add a directive-name charset guard to `parse_inline_body`; add private `is_valid_directive_name`; add a rejection unit test. (F-2 vector A)
- **Modify** `rust/src/lmd/render.rs` — add `sanitize_comment` helper; use it on `name` and the error-debug string in `dispatch`; add a unit test. (F-2 vector B)
- **Modify** `rust/src/lmd/engine.rs` — replace the `#[ignore]`'d `reread_same_path_is_cache_hit_not_full` test (comment + attribute + body) with a `mode=full`, multi-line-fixture, proof-line-safe version; add an F-2 e2e injection test. (F-1)
- **Modify** `docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md` — record F-1/F-2 resolution at the exact lines listed in Task 6. (Last task, only after code tasks are green.)

**No changes** to `rust/src/tools/ctx_read.rs` or `rust/src/core/cache.rs` (the measurement proved there is no production bug to fix).

---

## Task 1: F-2 — Inline directive-name charset guard

Closes the primary injection vector: today `parse_inline_body` splits on whitespace and accepts ANY first token as the directive name, so `{{ -->x }}` reaches the render fallback and breaks out of the `<!-- … -->` wrapper. The block parser (`block.rs::parse_directive_line`) already restricts names to `[a-z0-9-]` with an ascii-alpha start; we mirror that. An invalid inline name now returns `None`, so the `{{ … }}` passes through as ordinary (HTML-escaped) text instead of dispatching.

**Files:**
- Modify: `rust/src/lmd/parser/inline.rs` (function `parse_inline_body`, ~line 14; tests mod at bottom)

- [ ] **Step 1: Write the failing unit test**

Use `mcp__serena__jet_brains_find_symbol` to locate the `tests` mod in `rust/src/lmd/parser/inline.rs`, then `mcp__serena__insert_after_symbol` after the existing `rejects_empty` test function to add:

```rust
    #[test]
    fn rejects_comment_injection_name() {
        // F-2: a name that is not [a-z0-9-]-with-alpha-start must NOT be claimed,
        // so `{{ -->x }}` can never reach the HTML-comment render fallback.
        assert!(parse_inline_body("-->x").is_none());
        assert!(parse_inline_body("a-->b").is_none());
        assert!(parse_inline_body("<script").is_none());
        // valid names still parse
        assert_eq!(parse_inline_body("read").unwrap().0, "read");
        assert_eq!(parse_inline_body("hard-rules x").unwrap().0, "hard-rules");
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo nextest run -p lean-ctx lmd::parser::inline::tests::rejects_comment_injection_name`
Expected: FAIL — `parse_inline_body("-->x")` currently returns `Some(("-->x", ""))`, so `is_none()` assertion fails.

- [ ] **Step 3: Add the `is_valid_directive_name` helper**

Use `mcp__serena__insert_before_symbol` before `parse_inline_body` in `rust/src/lmd/parser/inline.rs` to add:

```rust
/// True if `name` matches the lmd directive-name grammar: an ascii-alphabetic
/// first byte, then only `[a-z0-9-]` (ascii-alphanumeric or `-`). Mirrors the
/// block grammar in `block.rs::parse_directive_line` so inline and block
/// directives share one charset (spec §9 F-2).
fn is_valid_directive_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    match bytes.first() {
        Some(b) if b.is_ascii_alphabetic() => {}
        _ => return false,
    }
    bytes.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'-')
}
```

- [ ] **Step 4: Add the charset guard to `parse_inline_body`**

Use `mcp__serena__replace_symbol_body` on `parse_inline_body` in `rust/src/lmd/parser/inline.rs` with this body:

```rust
    let body = body.trim();
    if body.is_empty() {
        return None;
    }
    let (name, args) = match body.split_once(char::is_whitespace) {
        Some((name, args)) => (name.to_string(), args.trim().to_string()),
        None => (body.to_string(), String::new()),
    };
    if !is_valid_directive_name(&name) {
        return None;
    }
    Some((name, args))
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo nextest run -p lean-ctx lmd::parser::inline`
Expected: PASS — `rejects_comment_injection_name` and the existing `parses_inline_name_and_args` / `parses_inline_name_only` / `rejects_empty` all green.

- [ ] **Step 6: Reformat + commit**

```bash
# reformat via JetBrains MCP first:
#   mcp__jetbrains__reformat_file path=rust/src/lmd/parser/inline.rs
git add rust/src/lmd/parser/inline.rs
git commit -m "fix(lmd): restrict inline directive-name charset to block grammar (F-2)"
```

---

## Task 2: F-2 — HTML-comment-sanitize the render fallback

Belt-and-suspenders for the error path: `dispatch` emits `<!-- lmd:@{name} error: {e:?} -->` on a bridge error and `<!-- lmd: unknown directive @{name} -->` on a miss. After Task 1 `name` is charset-safe, but the `{e:?}` debug string can still embed arbitrary content (paths, args) containing `-->`. We neutralize the comment delimiters in both interpolated fields.

**Files:**
- Modify: `rust/src/lmd/render.rs` (function `dispatch`, ~line 35; tests — add a `#[cfg(test)]` mod if none exists)

- [ ] **Step 1: Write the failing unit test**

First check whether `rust/src/lmd/render.rs` has a `#[cfg(test)] mod tests`. Use `mcp__serena__jet_brains_get_symbols_overview` on the file. If absent, use `mcp__serena__insert_after_symbol` after the last top-level item (`lmd_renderer_extension`) to add the whole mod; if present, `insert_after_symbol` the last test fn. Add:

```rust
#[cfg(test)]
mod tests {
    use super::sanitize_comment;

    #[test]
    fn sanitizes_comment_breakout_sequences() {
        assert_eq!(sanitize_comment("x-->y"), "x--&gt;y");
        assert_eq!(sanitize_comment("<!--z"), "&lt;!--z");
        assert_eq!(sanitize_comment("plain"), "plain");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo nextest run -p lean-ctx lmd::render::tests::sanitizes_comment_breakout_sequences`
Expected: FAIL to COMPILE — `sanitize_comment` does not exist yet.

- [ ] **Step 3: Add the `sanitize_comment` helper**

Use `mcp__serena__insert_before_symbol` before `dispatch` in `rust/src/lmd/render.rs` to add:

```rust
/// Neutralize HTML-comment delimiters so an untrusted directive name or a
/// bridge error string cannot break out of the fallback `<!-- … -->` wrapper
/// (spec §9 F-2). Phase-1 target is the AI context, not a browser DOM, so a
/// minimal delimiter-escape is sufficient.
fn sanitize_comment(s: &str) -> String {
    s.replace("-->", "--&gt;").replace("<!--", "&lt;!--")
}
```

- [ ] **Step 4: Apply sanitization in `dispatch`**

Use `mcp__serena__replace_symbol_body` on `dispatch` in `rust/src/lmd/render.rs` with this body:

```rust
    let args = DirectiveArgs::parse(raw_args);
    match ctx.registry.get(name) {
        Some(bridge) => match bridge.execute(ctx, &args) {
            Ok(out) => out,
            Err(e) => format!(
                "<!-- lmd:@{} error: {} -->",
                sanitize_comment(name),
                sanitize_comment(&format!("{e:?}"))
            ),
        },
        None => format!(
            "<!-- lmd: unknown directive @{} -->",
            sanitize_comment(name)
        ),
    }
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo nextest run -p lean-ctx lmd::render`
Expected: PASS.

- [ ] **Step 6: Reformat + commit**

```bash
#   mcp__jetbrains__reformat_file path=rust/src/lmd/render.rs
git add rust/src/lmd/render.rs
git commit -m "fix(lmd): HTML-comment-sanitize render fallback name + error (F-2)"
```

---

## Task 3: F-1 — Correct the wrongly-ignored Read→Delta engine test

The existing `reread_same_path_is_cache_hit_not_full` is `#[ignore]`'d because it expected `auto` 2-reads to produce an `[unchanged]` stub. The measurement proves the stub is a **`mode=full`** feature; `auto` deliberately compresses. Two further problems: a single-line sentinel fixture lets the cache-hit proof-line (first file line) leak the sentinel into the stub, and a 2-read render hits a trailing-directive off-by-one. Fix: use `mode=full` (where the guarantee lives — matches spec §6 "3-Read/mode=full" gate), a multi-line fixture with the sentinel on line 2 (proof-line shows line 1 only), and 3 reads. `@read`'s production default stays `auto` — this test simply demonstrates the guarantee in its real home.

**Files:**
- Modify: `rust/src/lmd/engine.rs` (test `reread_same_path_is_cache_hit_not_full`, lines 95–121; add F-2 e2e test in the same `tests` mod)

- [ ] **Step 1: Replace the ignored test (remove comment + `#[ignore]`, rewrite body)**

The replacement removes the stale 14-line CONCERN comment (lines 95–108) and the `#[ignore = …]` attribute (line 110). Use `mcp__serena__replace_content` on `rust/src/lmd/engine.rs`, matching this exact existing block:

```rust
    // CONCERN (Read→Delta off-by-one): assertion intentionally NOT weakened.
    // Empirically reproducible finding in `ctx_read::handle`'s cache state machine
    // (rust/src/tools/ctx_read.rs:660 handle_full_with_auto_delta):
    //   * `full_content_delivered` is only set inside the `was_hit` branch, AND
    //   * rushdown renders the *final* block directive such that the 2nd-and-LAST
    //     `@read p` is emitted as a FULL read, whereas a 2nd *non-final* `@read p`
    //     (e.g. with a 3rd directive after it) becomes the expected `[unchanged]`
    //     cache-hit stub.
    // Observed (2 reads): read#1=full+sentinel, read#2=full+sentinel  -> 2 sentinels.
    // Observed (3 reads): read#1=full+sentinel, read#2=stub, read#3=stub -> 1 sentinel.
    // So the shared session cache DOES warm (3rd read proves Read→Delta), but the
    // cache-hit does not deterministically land on the 2nd read of a *trailing*
    // directive. This is a `ctx_read` state-machine / rushdown render-order
    // interaction, NOT an lmd wiring bug. Ignored pending a Phase-1 follow-up.
    #[test]
    #[ignore = "Read->Delta off-by-one: 2nd read of a TRAILING @read still full; see comment + handoff"]
    fn reread_same_path_is_cache_hit_not_full() {
        let f = std::env::temp_dir().join("lmd_reread_cache.txt");
        std::fs::write(&f, "REREAD_SENTINEL_99\n").unwrap();
        let p = f.to_str().unwrap();
        let out = render(&format!("@read {p}\n\n@read {p}\n"));
        let hits = out.matches("REREAD_SENTINEL_99").count();
        assert_eq!(
            hits, 1,
            "2nd read must be a cache-hit, not a full re-dump; got {hits}x in: {out}"
        );
    }
```

Replace it with:

```rust
    // Read→Delta guarantee (spec §4.2a / §6 gate). The `[unchanged]` cache-hit
    // stub is a `mode=full` feature: `auto` deliberately compresses (and auto
    // re-reads are already compact), so the clean single-sentinel stub only lands
    // in full mode — verified empirically 2026-06-02. The fixture is multi-line
    // with the sentinel on line 2 so the cache-hit proof-line (first file line)
    // never leaks the sentinel into a stub. Three reads match the spec's
    // "3-Read/mode=full" gate.
    #[test]
    fn reread_same_path_is_cache_hit_not_full() {
        let f = std::env::temp_dir().join("lmd_reread_cache.txt");
        std::fs::write(&f, "// reread fixture header\nREREAD_SENTINEL_99\n").unwrap();
        let p = f.to_str().unwrap();
        let out = render(&format!(
            "@read {p} mode=full\n\n@read {p} mode=full\n\n@read {p} mode=full\n"
        ));
        let sentinels = out.matches("REREAD_SENTINEL_99").count();
        let stubs = out.matches("[unchanged").count();
        assert_eq!(
            sentinels, 1,
            "only the first full read carries the sentinel; re-reads must be cache-hit stubs; got {sentinels}x in: {out}"
        );
        assert!(
            stubs >= 2,
            "the 2nd and 3rd reads must be [unchanged] cache-hit stubs; got {stubs} in: {out}"
        );
    }
```

- [ ] **Step 2: Run the test to verify it passes**

Run: `cargo nextest run -p lean-ctx lmd::engine::tests::reread_same_path_is_cache_hit_not_full`
Expected: PASS (no longer ignored). Read 1 emits the full file (1 sentinel); reads 2 & 3 emit `[unchanged …]` stubs (≥2), and the proof-line — if present in this runtime — shows `// reread fixture header`, never the sentinel.

> If this step instead shows a trailing read still full (`sentinels == 2`), that is a real rushdown render-order finding: STOP and report it — do not weaken the assertion silently. Per the measurement, `mode=full` stubs are gated on `is_full_delivered` + hash (not render position), so this is not expected.

- [ ] **Step 3: Add an F-2 e2e injection test (same tests mod)**

Use `mcp__serena__insert_after_symbol` after `reread_same_path_is_cache_hit_not_full` in `rust/src/lmd/engine.rs` to add:

```rust
    #[test]
    fn inline_comment_injection_is_inert() {
        // F-2 e2e: `{{ -->x }}` must NOT be claimed as a directive (invalid name
        // charset) and must NOT inject an HTML comment into the render.
        let out = render("pre {{ -->x }} post\n");
        assert!(
            !out.contains("<!-- lmd"),
            "injection must not reach the comment fallback; got: {out}"
        );
        assert!(out.contains("pre") && out.contains("post"), "got: {out}");
    }
```

- [ ] **Step 4: Run the e2e test to verify it passes**

Run: `cargo nextest run -p lean-ctx lmd::engine::tests::inline_comment_injection_is_inert`
Expected: PASS — the inline is rejected by the charset guard (Task 1) and renders as escaped text, so no `<!-- lmd` appears.

- [ ] **Step 5: Reformat + commit**

```bash
#   mcp__jetbrains__reformat_file path=rust/src/lmd/engine.rs
git add rust/src/lmd/engine.rs
git commit -m "fix(lmd): un-ignore Read->Delta test via mode=full + proof-safe fixture (F-1); add F-2 e2e"
```

---

## Task 4: Full verification

**Files:** none (verification only)

- [ ] **Step 1: Run the full lmd test module**

Run: `cargo nextest run -p lean-ctx lmd`
Expected: PASS — all lmd parser/render/engine/bridge tests, including the three new/changed tests, green. Zero `#[ignore]` skips for `reread_same_path_is_cache_hit_not_full`.

- [ ] **Step 2: Run the whole crate test suite (regression guard)**

Run: `cargo nextest run -p lean-ctx`
Expected: PASS — confirms no `ctx_read`/`cache` behavior was touched and nothing else regressed.

- [ ] **Step 3: Confirm no production read-path files changed**

Run: `git diff --name-only main -- rust/src/tools/ctx_read.rs rust/src/core/cache.rs`
Expected: empty output — F-1 was resolved without any production `ctx_read`/`cache` change.

---

## Task 5: Clean up measurement fixtures

**Files:** delete `rust/target/lmd_measure/` (gitignored throwaway from the 2026-06-02 measurement)

- [ ] **Step 1: Remove the fixture directory**

Use the native Delete/Glob path or `mcp__lean-ctx__ctx_shell` is not for writes — delete the directory directly:

```bash
rm -rf rust/target/lmd_measure
```

Expected: directory gone; nothing in git status (it was under the gitignored `target/`).

---

## Task 6: Update the design spec (ONLY after Tasks 1–5 are green)

Record the resolution in `docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md`. The spec is Markdown — use native `Edit` (or `mcp__lean-ctx__ctx_edit`), NOT Serena. Match each old string exactly. Line numbers are as of 2026-06-02 and may drift as you edit top-to-bottom; anchor on the quoted text.

- [ ] **Step 1: §4.2a Phase-1 follow-up note (lines ~328–334)**

Replace the existing block-quote that begins `> **Phase-1-Implementierungs-Befund (Follow-up F-1, §9):**` and ends `…der Engine-Unit-Test der 2-Read-Form ist bis zum ctx_read-Fix \`#[ignore]\`'d.` with:

```markdown
> **Phase-1-Befund F-1 — gelöst (2026-06-02, empirisch).** Messung (echte
> `ctx_read`-Runtime, kalter Cache) zeigt: der `[unchanged]`-Stub ist ein
> **`mode=full`-Feature** und funktioniert korrekt (klein **und** groß: ~99 %
> Ersparnis). `auto` komprimiert bewusst — Auto-Re-Reads sind bereits kompakt
> (große Datei ~50 Tok) bzw. trivial klein (kleine Datei). **Kein produktiver
> `ctx_read`/`cache`-Bug**: der Auto-Pfad prüft das `full_content_delivered`-Flag
> nie und ist eh kompakt; der Full-Pfad markiert via `:508`/`:743` korrekt. F-1
> war ein **Test-Korrektheits-Problem** — der Engine-Test erwartete fälschlich
> einen `auto`-2-Read-Stub. Behoben: `reread_same_path_is_cache_hit_not_full`
> läuft jetzt (kein `#[ignore]`) über `mode=full` + mehrzeilige Fixture
> (Sentinel nicht in Zeile 1, damit die Proof-Line ihn nie leakt) + 3 Reads.
> `@read` behält `auto` als Default.
```

- [ ] **Step 2: §9 F-1 table row (line ~507)**

Replace the entire `| F-1  | …` row (the row whose Frage is `Read→Delta-Cache-Hit über lmd nicht sauber 2-Read-beobachtbar (Phase-1-Befund)`) with:

```markdown
| F-1  | Read→Delta-Cache-Hit über lmd nicht sauber 2-Read-beobachtbar (Phase-1-Befund) | **gelöst (2026-06-02, empirisch):** kein produktiver ctx_read-Bug — der `[unchanged]`-Stub ist ein `mode=full`-Feature und funktioniert (klein+groß ~99 %); `auto` komprimiert by-design (Auto-Re-Read groß ~50 Tok, klein trivial). F-1 war Test-Korrektheit: `reread_same_path_is_cache_hit_not_full` erwartete fälschlich einen `auto`-2-Read-Stub. Fix: Test über `mode=full` + mehrzeilige Fixture (Sentinel nicht Zeile 1 → Proof-Line leakt nicht) + 3 Reads, un-`#[ignore]`'d. `@read` bleibt `auto`. Keine `ctx_read.rs`/`cache.rs`-Änderung. |
```

- [ ] **Step 3: §9 F-2 table row (line ~508)**

Replace the entire `| F-2  | …` row (Frage `HTML-Kommentar-Injection im Render-Fallback (Phase-1-Befund)`) with:

```markdown
| F-2  | HTML-Kommentar-Injection im Render-Fallback (Phase-1-Befund)                   | **gelöst (2026-06-02):** beide Vektoren zu — Inline-Name-Charset in `parser/inline.rs::parse_inline_body` an die Block-Grammatik `[a-z0-9-]` (ascii-alpha-Start) angeglichen (invalider Name → pass-through statt Dispatch); `render.rs::dispatch` sanitisiert `name` **und** `{e:?}` via `sanitize_comment` (`-->`/`<!--` neutralisiert). Tests: `rejects_comment_injection_name`, `sanitizes_comment_breakout_sequences`, e2e `inline_comment_injection_is_inert`. |
```

- [ ] **Step 4: §6 Phase-1 gate row clarification (line ~422, optional but recommended)**

In the Phase **1** row, the Gate cell currently reads `… \`@read\`-Re-Read = Cache-Hit/Delta **ohne \`fresh\`** (§4.2a)`. Append a clarifying clause so the gate matches the empirical finding:

Find: `\`@read\`-Re-Read = Cache-Hit/Delta **ohne \`fresh\`** (§4.2a)`
Replace with: `\`@read\`-Re-Read = Cache-Hit/Delta **ohne \`fresh\`** (§4.2a; `[unchanged]`-Stub ist `mode=full`-Feature, `auto` ist by-design kompakt — F-1 2026-06-02)`

- [ ] **Step 5: §1 status footer (line ~526)**

Update the status line `*Status: v0.9 — …*` to note the follow-ups are resolved. Find the substring `R-1/G-1 gelöst.` and replace with `R-1/G-1 gelöst; Phase-1-Follow-ups F-1/F-2 gelöst (2026-06-02, siehe docs/lean-md/plans/2026-06-02-lmd-f1-f2-hardening.md).`

- [ ] **Step 6: Commit the spec update**

```bash
git add docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md docs/lean-md/plans/2026-06-02-lmd-f1-f2-hardening.md
git commit -m "docs(lmd): record F-1/F-2 resolution in design spec §4.2a/§6/§9"
```

---

## Self-Review

- **Spec coverage:** F-1 (§9) → Task 3 (test) + Task 6 §1/§2/§4 (spec). F-2 (§9) → Task 1 (inline charset) + Task 2 (render sanitize) + Task 6 §3. "No production ctx_read change" claim → Task 4 Step 3 guard. Spec line refs requested by the user → Task 6 (exact lines + old/new text).
- **Placeholder scan:** none — every code/edit step carries full code and exact match text; every run step has an exact command + expected output.
- **Type consistency:** `is_valid_directive_name(&str) -> bool`, `sanitize_comment(&str) -> String`, `parse_inline_body(&str) -> Option<(String, String)>` used consistently across tasks; test names referenced in run commands match the defined `#[test]` fns.
- **Scope:** single focused subsystem (lmd Phase-1 follow-ups); no Phase-2 work; `@read` default unchanged.
