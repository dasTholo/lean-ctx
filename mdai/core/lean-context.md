@markdownai v1.0

# Lean-Context-Discipline (Core)

Single source of truth for bounded-read and lean-shell defaults across all mdai
skills and generated specs. Consumed via `@include mdai/core/lean-context.md` —
the rules table renders inline. This file is text-only (no YAML frontmatter,
no `@define` blocks) so it can be safely `@include`'d anywhere.

<!-- canonical lean-context source of truth (incl. the Grep/cat/bash anti-pattern semantics); derived operational checklist: core/_fragments/lean-context-anchors.md (Cluster 3) -->

## Defaults / Exceptions

| Tool                                                | Default (always)                                                                                  | Exception (requires `@note visible consumer="human"` justification)                                                       |
|-----------------------------------------------------|---------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------|
| `ctx_read` (cross-file scan)                        | `@call ctx_read_map(path)` (deps + cross-refs) or `@call ctx_read_signatures(path)` (API surface) | `mode="full"`                                                                                                             |
| `ctx_read` (after `ctx_search` / `find_symbol`)     | `@call ctx_read_lines(path, start, end)`                                                          | `mode="full"`                                                                                                             |
| `ctx_read` (spec-review target)                     | `mode="full"` exactly once on the spec source itself                                              | (n/a — that IS the review job)                                                                                            |
| `ctx_shell`                                         | compressed (library default)                                                                      | `raw=true`                                                                                                                |
| `@include` (cross-file content)                     | `lines=N-M` when a specific block is meant                                                        | full-file `@include`                                                                                                      |
| `find_symbol` (Serena, when `MDAI_HAS_SERENA=true`) | `body=false` (symbol header only)                                                                 | `body=true` for targeted inspection                                                                                       |
| `ctx_read fresh=true` (cache bypass)                | NEVER pass `fresh=true` on a regular re-read — the cache auto-invalidates via file mtime          | only IMMEDIATELY after a write/edit to that path (e.g., when running as a subagent that may not share the parent's cache) |

## Why bounded by default

Bounded modes are token-cheaper than `mode="full"`; the search-then-targeted-read
pattern (`ctx_search` → `ctx_read_lines`) is the cheapest path for known-location
lookups. Wrappers implementing these defaults are defined in `core/ctx-tools.md`
and referenced in `core/tool-quick-ref.md` — this file is the rules doc only.

## Conventions

- **Naming:** `snake_case` for macro names (`write_spec`, not `writeSpec`).
  `kebab-case` for filenames (`write-spec.md`, not `write_spec.md`).
- **Markdown header italic**: never wrap header lines (first ~5 lines, especially date/metadata) with `_..._` —
  triggers a reproducible hang in `ctx_read mode="lines:N-M"`. Use `*...*` for italic, or omit entirely.
