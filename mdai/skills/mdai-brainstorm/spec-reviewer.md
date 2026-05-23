---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [spec_reviewer_prompt]
---

@markdownai v1.0

# Skill-A Pack: spec_reviewer_prompt

@define spec_reviewer_prompt(spec_path)
You are spec reviewer for `{{ spec_path }}`. Your task:

1. **Read the spec in full** using `@call ctx_read(path="{{ spec_path }}", mode="full")`.
2. **Check systematically**:
    - Is the objective sharp (success criteria measurable)?
    - Are assumptions explicitly marked as verifiable?
    - Are risks listed with mitigations?
    - Are non-goals (explicit scope cuts) documented?
    - Are cross-spec consequences captured?
    - Is a RED/GREEN verification setup specified?
3. **Anti-pattern checks (v0.1.0 lessons learned)**: every spec MUST clear these before `ready-to-implement`. If any check fails, downgrade to `needs-revision` and list the missing item under "Gaps".
    1. **MCP signatures verified against source.** If the spec references MCP tools (e.g. `mcp__lean-ctx__ctx_*`, `mcp__serena__*`, `mcp__markdownai__*`), confirm each signature against the Rust/TS source. Use `@call ctx_search(pattern="match action|fn handle|\"<action>\" =>", path="rust/src/tools/<tool>.rs")` and read the matched lines. Spec must NOT lock parameter names that don't exist in source. Counter-example: v0.1.0 assumed `ctx_session action=set/get key=...` — source only has `task/finding/decision/status/load/save/reset/list/cleanup/configure/snapshot/restore`. Also: `serena_info(topic="project")` doesn't exist; only `"jet_brains_debug_repl"` is valid.
    2. **First-class category check.** Before designing a new wrapper that persists state to a file or custom store, search `rust/src/tools/ctx_knowledge.rs` for `salience_score` categories. ctx_knowledge already has first-class categories (`decision`, `gotcha`, `architecture`, `security`, `testing`, `conventions`, `finding`). Counter-example: v0.1.0 had a separate `gotchas.md` that file-appended to `docs/mdai/GOTCHAS.md` — duplicated `ctx_knowledge category="gotcha"` (salience 75). Merged in `2310f908`.
    3. **mai-CLI does not execute `@query`.** The `markdownai` CLI parses but blocks `@query` directives at render time (`engine-include.ts` security policy). Spec MUST NOT propose live-MCP tests via `npx mai render`. Live MCP behavior is observable only from a Claude Code session with active MCP servers. Static render verifies syntax and `@include`/`@import` plumbing only. Counter-example: v0.1.0 spec §12.2/§12.3/§12.4 (cache-hit, service-fail, lang-detection tests) were infeasible — redefined in commit `ab95daa9` + `f3a07c18`.
    4. **Frontmatter convention for `mode: include`.** Files referenced via `@include` must NOT carry YAML frontmatter — `executeInclude` walks all AST nodes including text, so frontmatter leaks as plain text. Spec MUST state this constraint for any `mode: include` pack file. `mode: import-only` files are safe (frontmatter is dropped because `@import` only takes define/env/connect/import nodes). Counter-example: v0.1.0 `core/hard-rules.md` + `core/tool-quick-ref.md` had frontmatter that leaked; fixed in `ab95daa9`.
    5. **Repo-relative paths only.** No absolute `/home/<user>/...` or `~/...` paths in library code. Consumers may live in a different workspace. Spec MUST require this and reviewers must `@call ctx_search(pattern="^/home|^/Users|/Scripts/", path="<lib-dir>")` to verify. Counter-example: v0.1.0 `write-spec.md:32` had `/home/tholo/Scripts/lean-ctx/markdownai` — fixed in `9a9fe827`.
    6. **Language convention enforced.** Per project `CLAUDE.md`: code and library files MUST be English (only chat/plans/specs interaction is German). Spec MUST state this and reviewers must run `@call ctx_search(pattern="[ÄÖÜäöüß]", path="<lib-dir>")` for residual German content. Counter-example: v0.1.0 library was authored in German — translated in `781fb62a`.
    7. **Parameter names match MCP source.** Library macros use full parameter names from the source schema, NOT lean-ctx display-compression aliases. The `ctx_read` display layer compresses `command=` → `cmd=` and `value=` → `val=` in its OUTPUT; the underlying call accepts only `command` / `value`. Reviewers verify by `@call ctx_search(pattern="get_str.*\"<param>\"", path="rust/src/tools/registered/<tool>.rs")`. Counter-example: a Task-19c implementer reported `val=` as a possible bug; ctx_search confirmed file actually had `value=` — display vs. file content must be distinguished in review.
    8. **Smoke-render-test mandatory pre-release.** Every library/pack spec must specify a `npx mai render` smoke test against a fixture that imports every public macro file. Pass-kriteria: `exit 0`, `mode: include` text appears, `mode: import-only` source-text does NOT appear in output, no `unknown directive` errors. Spec must list these criteria; reviewer must confirm they are testable. Counter-example: without this, v0.1.0 would have shipped with leaking frontmatter (caught only after smoke test in Task 17).
4. **Report format**:
    - **Strengths (>=3)**: what is solid.
    - **Gaps (>=3 or "none")**: what is missing or unclear. List any failed anti-pattern checks here with the check number.
    - **Concrete patches**: file-line precise, with diff suggestions.
    - **Block verdict**: `ready-to-implement` | `needs-revision` | `needs-clarification`. Any failed anti-pattern check forces at least `needs-revision`.
5. **Output**: write to `docs/mdai/reviews/$(basename {{ spec_path }} .mdai.md)-review.md`.

Tools: lean-ctx only (`@call ctx_read` / `@call ctx_search` / `@call ctx_shell` / `@call ctx_edit`). No native reads. No `&&` bash chains.
@end
