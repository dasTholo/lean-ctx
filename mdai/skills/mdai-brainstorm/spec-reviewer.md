---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [ spec_reviewer_prompt ]
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
3. **Anti-pattern checks**: every spec MUST clear these before `ready-to-implement`. If any check fails, downgrade to `needs-revision` and list the missing item under "Gaps".
    1. **MCP signatures verified against source.** If the spec references MCP tools (e.g. `mcp__lean-ctx__ctx_*`, `mcp__serena__*`, `mcp__markdownai__*`), confirm each signature against the Rust/TS source. Use `@call ctx_search(pattern="match action|fn handle|\"<action>\" =>", path="rust/src/tools/<tool>.rs")` and read the matched lines. Spec must NOT lock parameter names that don't exist in source. Failure pattern: assuming `ctx_session` has `action=set/get key=...` when the real signature only accepts `task/finding/decision/status/load/save/reset/list/cleanup/configure/snapshot/restore`; or assuming `serena_info(topic="project")` exists when only `"jet_brains_debug_repl"` is valid.
    2. **Existing-store check.** Before designing a new wrapper that persists state, enumerate existing stores: (a) `@call ctx_search(pattern="salience_score|=> &\\[", path="rust/src/tools/ctx_knowledge.rs")` for first-class ctx_knowledge categories; (b) `@call list_gotchas(query="")` for user-curated mdai-gotchas; (c) `@call list_auto_gotchas()` for lean-ctx's internal auto-tracking GotchaStore. Spec MUST state which store the new state belongs to AND why an existing store does not suffice. Failure pattern: inventing a new file-append wrapper or a new ctx_knowledge category when the user convention (`category="mdai-gotcha"`) already covers it.
    3. **mai-CLI does not execute `@query`.** The `markdownai` CLI parses but blocks `@query` directives at render time (`engine-include.ts` security policy). Spec MUST NOT propose live-MCP tests via `npx mai render`. Live MCP behavior is observable only from a Claude Code session with active MCP servers. Static render verifies syntax and `@include`/`@import` plumbing only. Failure pattern: writing cache-hit / service-fail / lang-detection tests that depend on `@query` execution from the CLI.
    4. **Frontmatter convention for `mode: include`.** Files referenced via `@include` must NOT carry YAML frontmatter — `executeInclude` walks all AST nodes including text, so frontmatter leaks as plain text. Spec MUST state this constraint for any `mode: include` pack file. `mode: import-only` files are safe (frontmatter is dropped because `@import` only takes define/env/connect/import nodes). Failure pattern: a `mode: include` rule file with `---lib_version: ...---` at the top whose YAML block surfaces verbatim in the rendered output.
    5. **Repo-relative paths only.** No absolute `/home/<user>/...` or `~/...` paths in library code. Consumers may live in a different workspace. Spec MUST require this and reviewers must `@call ctx_search(pattern="^/home|^/Users|/Scripts/", path="<lib-dir>")` to verify. Failure pattern: an inline `cd /home/<user>/<repo>/markdownai` in a `ctx_shell command="..."` macro body.
    6. **Language convention enforced.** Per project `CLAUDE.md`: code and library files MUST be English (only chat/plans/specs interaction is German). Spec MUST state this and reviewers must run `@call ctx_search(pattern="[ÄÖÜäöüß]", path="<lib-dir>")` for residual German content. Failure pattern: macro bodies authored in German that ship in a library file under `mdai/`.
    7. **Parameter names match MCP source.** Library macros use full parameter names from the source schema, NOT lean-ctx display-compression aliases. The `ctx_read` display layer compresses `command=` → `cmd=` and `value=` → `val=` in its OUTPUT; the underlying call accepts only `command` / `value`. Reviewers verify by `@call ctx_search(pattern="get_str.*\"<param>\"", path="rust/src/tools/registered/<tool>.rs")` against the source. When a `ctx_read` output appears to show truncated parameter names, re-verify via `@call ctx_search` for the literal token in the file — display compression must not be mistaken for an actual file bug.
    8. **Smoke-render-test mandatory pre-release.** Every library/pack spec must specify a `npx mai render` smoke test against a fixture that imports every public macro file. Pass criteria: `exit 0`, `mode: include` text appears, `mode: import-only` source-text does NOT appear in output, no `unknown directive` errors. Spec must list these criteria; reviewer must confirm they are testable. Failure pattern: shipping a pack without smoke render — bugs like leaking frontmatter or unresolved directives surface only at consumer site.
4. **Report format**:
    - **Strengths (>=3)**: what is solid.
    - **Gaps (>=3 or "none")**: what is missing or unclear. List any failed anti-pattern checks here with the check number.
    - **Concrete patches**: file-line precise, with diff suggestions.
    - **Block verdict**: `ready-to-implement` | `needs-revision` | `needs-clarification`. Any failed anti-pattern check forces at least `needs-revision`.
5. **Output**: write to `docs/mdai/reviews/$(basename {{ spec_path }} .mdai.md)-review.md`.

Tools: prefer library wrappers from the mdai pack — `@call ctx_read` / `@call ctx_search` / `@call ctx_shell` / `@call ctx_edit` (`mdai/core/ctx-tools.md`), `@call find_symbol` / `@call replace_symbol_body` / `@call insert_before_symbol` / `@call insert_after_symbol` / `@call symbols_overview` (`mdai/tooling/serena.md` when MDAI_HAS_SERENA=true), `@call reformat_file` / `@call step_reformat_commit` (`mdai/tooling/jetbrains.md` when MDAI_HAS_JETBRAINS=true), `@call read_phase` / `@call list_phases` / `@call get_constraints` (`mdai/core/mcp-markdownai.md`), `@call list_gotchas` / `@call list_auto_gotchas` / `@call gotcha_stats` (`mdai/core/ctx-knowledge.md`). Fall back to native MCP only if no wrapper exists. No native filesystem reads. No `&&` bash chains.
@end
