---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [ spec_reviewer_prompt ]
---

@markdownai v1.0

# Skill-A Pack: spec_reviewer_prompt

@define spec_reviewer_prompt(spec_path)
You are spec reviewer for `{{ spec_path }}`. Your task: verify this spec is complete, consistent, and ready for plan
writing (next skill: `superpowers:writing-plans` or `mdai-writing-plans` once available).

## 0. Lean-Context-Discipline (always-on, rendered inline)

@include mdai/core/lean-context.md

You are bound by the table above. The ONLY legitimate `mode="full"` call in this entire review is the spec-source
read in §1. Every cross-file read uses `@call ctx_read_map(path)` / `@call ctx_read_signatures(path)` /
`@call ctx_read_lines(path, start, end)`. Every `ctx_shell raw=true` requires a `@note visible consumer="human"`
justification in the same block. `fresh=true` on `ctx_read` is only valid IMMEDIATELY after a write/edit to the
same path. Violations are downgraded to needs-revision via Check #10.

## 1. Read the spec

Use `@call ctx_read(path="{{ spec_path }}", mode="full")`. This is the one allowed `mode="full"` per §0 (spec-source
read is the review job itself).

## 2. What to Check (Quick-Scan, merged from upstream)

| Category     | What to Look For                                                                |
|--------------|---------------------------------------------------------------------------------|
| Completeness | TODOs, placeholders, "TBD", incomplete sections                                 |
| Consistency  | Internal contradictions, conflicting requirements                               |
| Clarity      | Requirements ambiguous enough to cause someone to build the wrong thing         |
| Scope        | Focused enough for a single plan — not covering multiple independent subsystems |
| YAGNI        | Unrequested features, over-engineering                                          |

## 3. Systematic Deep-Checks (mdai-specific)

- Is the objective sharp (success criteria measurable)?
- Are assumptions explicitly marked as verifiable?
- Are risks listed with mitigations?
- Are non-goals (explicit scope cuts) documented?
- Are cross-spec consequences captured?
- Is a RED/GREEN verification setup specified (or explicitly justified as skipped)?

## 4. Calibration (verbatim from upstream — Anti-Pedantry-Bremse)

**Only flag issues that would cause real problems during impl planning.**
A missing section, a contradiction, or a requirement so ambiguous it could be
interpreted two different ways — those are issues. Minor wording improvements,
stylistic preferences, and "sections less detailed than others" are not.
Approve unless there are serious gaps that would lead to a flawed plan.

## 5. mdai-specific Anti-Pattern-Checks (forced needs-revision if any fail)

Every spec MUST clear these before `ready-to-implement`. If any check fails, downgrade to `needs-revision` and list
the missing item under "Gaps" with the check number.

1. **MCP signatures verified against source.** If the spec references MCP tools (e.g. `mcp__lean-ctx__ctx_*`,
   `mcp__serena__*`, `mcp__markdownai__*`), confirm each signature against the Rust/TS source. Use
   `@call ctx_search(pattern="match action|fn handle|\"<action>\" =>", path="rust/src/tools/<tool>.rs")` and read
   the matched lines via `@call ctx_read_lines(path, start, end)` (per §0). Spec must NOT lock parameter names that
   don't exist in source. Failure pattern: assuming `ctx_session` has `action=set/get key=...` when the real
   signature only accepts `task/finding/decision/status/load/save/reset/list/cleanup/configure/snapshot/restore`;
   or assuming `serena_info(topic="project")` exists when only `"jet_brains_debug_repl"` is valid.
2. **Existing-store check.** Before designing a new wrapper that persists state, enumerate existing stores: (a)
   `@call ctx_search(pattern="salience_score|=> &\\[", path="rust/src/tools/ctx_knowledge.rs")` for first-class
   ctx_knowledge categories; (b) `@call list_gotchas(query="")` for user-curated mdai-gotchas; (c)
   `@call list_auto_gotchas()` for lean-ctx's internal auto-tracking GotchaStore. Spec MUST state which store the
   new state belongs to AND why an existing store does not suffice. Failure pattern: inventing a new file-append
   wrapper or a new ctx_knowledge category when the user convention (`category="mdai-gotcha"`) already covers it.
3. **mai-CLI does not execute `@query`.** The `markdownai` CLI parses but blocks `@query` directives at render
   time (`engine-include.ts` security policy). Spec MUST NOT propose live-MCP tests via `npx mai render`. Live MCP
   behavior is observable only from a Claude Code session with active MCP servers. Static render verifies syntax
   and `@include`/`@import` plumbing only. Failure pattern: writing cache-hit / service-fail / lang-detection tests
   that depend on `@query` execution from the CLI.
4. **Frontmatter convention for `mode: include`.** Files referenced via `@include` must NOT carry YAML frontmatter —
   `executeInclude` walks all AST nodes including text, so frontmatter leaks as plain text. Spec MUST state this
   constraint for any `mode: include` pack file. `mode: import-only` files are safe (frontmatter is dropped because
   `@import` only takes define/env/connect/import nodes). Failure pattern: a `mode: include` rule file with
   `---lib_version: ...---` at the top whose YAML block surfaces verbatim in the rendered output.
5. **Repo-relative paths only.** No absolute `/home/<user>/...` or `~/...` paths in library code. Consumers may live
   in a different workspace. Spec MUST require this and reviewers must
   `@call ctx_search(pattern="^/home|^/Users|/Scripts/", path="<lib-dir>")` to verify. Failure pattern: an inline
   `cd /home/<user>/<repo>/markdownai` in a `ctx_shell command="..."` macro body.
6. **Language convention enforced.** Per project `CLAUDE.md`: code and library files MUST be English (only
   chat/plans/specs interaction is German). Spec MUST state this and reviewers must run
   `@call ctx_search(pattern="[ÄÖÜäöüß]", path="<lib-dir>")` for residual German content. Failure pattern: macro
   bodies authored in German that ship in a library file under `mdai/`.
7. **Parameter names match MCP source.** Library macros use full parameter names from the source schema, NOT
   lean-ctx display-compression aliases. The `ctx_read` display layer compresses `command=` → `cmd=` and `value=` →
   `val=` in its OUTPUT; the underlying call accepts only `command` / `value`. Reviewers verify by
   `@call ctx_search(pattern="get_str.*\"<param>\"", path="rust/src/tools/registered/<tool>.rs")` against the
   source. When a `ctx_read` output appears to show truncated parameter names, re-verify via `@call ctx_search`
   for the literal token in the file — display compression must not be mistaken for an actual file bug.
8. **Smoke-render-test mandatory pre-release.** Every library/pack spec must specify a `npx mai render` smoke test
   against a fixture that imports every public macro file. Pass criteria: `exit 0`, `mode: include` text appears,
   `mode: import-only` source-text does NOT appear in output, no `unknown directive` errors. Spec must list these
   criteria; reviewer must confirm they are testable. Failure pattern: shipping a pack without smoke render — bugs
   like leaking frontmatter or unresolved directives surface only at consumer site.
9. **Spec-Body uses markdownai directives actively (Discipline §10.4 #9 + convention §5.6).** Verify via:
   `@call ctx_search(pattern="^@(call|include|import|list|render|tree|constraint|date|count|if|elseif|else|endif)|\\{\\{ @", path="{{ spec_path }}")`.
   Pass: ≥3 distinct directive types in the body (excluding frontmatter and the `@markdownai v1.0` header).
   Fail: 0 directives in body. Allowed exception: frontmatter contains `markdownai_directives_omitted: <reason>` —
   reviewer verifies the reason is genuine (purely algorithmic topics without file/tool/data ties per §10.4 #9).
   Sub-check: `file_check` is NOT used for branching (per `core/file-utils.md` + §5.6 Anti-Pattern) — if
   `@call file_check` appears followed by conditional logic depending on its output, downgrade to needs-revision
   and point to §5.6 Anti-Pattern-Sektion.
10. **Lean-Context-Defaults (Discipline §5.7 + `core/lean-context.md`).** Reviewer verifies the spec body honours
    the bounded-read / lean-shell / cache-bypass defaults from §0. Run all of:
    - `@call ctx_search(pattern="ctx_read\\(", path="{{ spec_path }}")` → flag every match without an explicit
      `mode=` parameter (implicit `auto` resolves unpredictably; default must be explicit per §0).
    - `@call ctx_search(pattern="mode=\"full\"", path="{{ spec_path }}")` → flag every match outside the
      spec-reviewer §1 spec-source read line. Each such match must have a `@note visible consumer="human"`
      justification adjacent (preceding or following block).
    - `@call ctx_search(pattern="raw=true", path="{{ spec_path }}")` → flag every `ctx_shell raw=true` without a
      `@note visible consumer="human"` justification adjacent.
    - `@call ctx_search(pattern="fresh=true", path="{{ spec_path }}")` → flag every `fresh=true` not immediately
      preceded by a write/edit (`@call ctx_edit` / `Write` / `Edit`) to the same path. Cache auto-invalidates via
      mtime; `fresh=true` is otherwise wasteful.
    - `@call ctx_search(pattern="^@include [^\\n]+$", path="{{ spec_path }}")` → for each match, verify the
      included file is ≤50 lines OR the include uses `lines=N-M`. Otherwise flag.
    - Serena: `@call ctx_search(pattern="find_symbol.*body=true", path="{{ spec_path }}")` → flag matches without
      `@note`-justification.
      Failure pattern: `@call ctx_read(path="rust/src/tools/ctx_session.rs", mode="full")` with no preceding
      `ctx_search` anchor. Patch suggestion: `@call ctx_search(pattern="match action", path="...")` +
      `@call ctx_read_lines(path="...", start=42, end=78)`.
11. **Structured data via `@read` / `@list`.** Tables in the spec body that draw from an external single source
    of truth (CHANGELOG, package.json, Cargo.toml, YAML/CSV configs) OR tables with more than ~50 rows MUST be
    generated via `@read` (scalar extract) or `@list … | @render` (rows), not hardcoded prose. Scan via
    `@call ctx_search(pattern="^\\|", path="{{ spec_path }}")` to locate table blocks; for each, ask: does the
    data exist in a versioned SoT elsewhere in the repo? If yes and the table hand-types it: downgrade.
    Failure pattern: a 30-row hand-typed changelog summary that duplicates `CHANGELOG.md`. Patch suggestion:
    `@list ./CHANGELOG.md path=$.entries | @render type="table" columns="date,summary"`.

## 6. Report format (merged: upstream Recommendations-Sektion + mdai Strengths/Gaps/Patches)

- **Status:** Approved | Needs-Revision | Needs-Clarification (any failed anti-pattern check from §5 forces at
  least Needs-Revision).
- **Strengths (≥3):** what is solid.
- **Gaps (≥0; calibrate per §4 — only real planning-blockers):** what is missing or unclear. List any failed
  anti-pattern check here with the check number.
- **Concrete patches (advisory):** file-line precise, with diff suggestions.
- **Recommendations (advisory, do not block approval; from upstream):** suggestions for improvement that did NOT
  reach the bar of "real problems during impl planning" per §4 calibration.

## 7. Output

Write report to `docs/mdai/reviews/$(basename {{ spec_path }} .mdai.md)-review.md`.

## 8. Tools

Lean-Context-Defaults FIRST (from `mdai/core/lean-context.md`, see §0):
`@call ctx_read_map(path)`, `@call ctx_read_signatures(path)`, `@call ctx_read_lines(path, start, end)`.

Then library wrappers from the mdai pack — `@call ctx_read` / `@call ctx_search` / `@call ctx_shell` /
`@call ctx_edit` (`mdai/core/ctx-tools.md`), `@call find_symbol` / `@call replace_symbol_body` /
`@call insert_before_symbol` / `@call insert_after_symbol` / `@call symbols_overview` (`mdai/tooling/serena.md`
when MDAI_HAS_SERENA=true), `@call reformat_file` / `@call step_reformat_commit` (`mdai/tooling/jetbrains.md`
when MDAI_HAS_JETBRAINS=true), `@call read_phase` / `@call list_phases` / `@call get_constraints` (
`mdai/core/mcp-markdownai.md`), `@call list_gotchas` / `@call list_auto_gotchas` / `@call gotcha_stats` (
`mdai/core/ctx-knowledge.md`). Fall back to native MCP only if no wrapper exists. No native filesystem reads.
No `&&` bash chains. **No `mode="full"` calls outside the §1 spec-source read** (per §0 + Check #10).
@end
