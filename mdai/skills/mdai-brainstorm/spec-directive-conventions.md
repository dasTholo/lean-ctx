---
mode: include
lib_version: 0.1.1
---

@markdownai v1.0

# Spec Body Directive Conventions (L1 — write-outputs phase)

Operationalizes Discipline §10.4 #9. Mandatory at the "Write design doc" step.

| Use-Case                               | Best Practice                                                                | Anti-Pattern                        | v1.0 native equivalent                    |
|----------------------------------------|------------------------------------------------------------------------------|-------------------------------------|-------------------------------------------|
| Date in file paths                     | `@set d = @date format='YYYY-MM-DD' /` then `{{ d }}`                        | hard-coded `2026-05-24`; inline `{{ @date }}` (resolves empty) | —                |
| Directory listing                      | `@tree mdai/ depth=2`                                                        | manually typed-out tree             | —                                         |
| File-system status (report)            | `@call file_check(path="...")`                                               | `ls -la` output copy                | —                                         |
| Branching on file existence            | inline `@if file.exists("...")` + `@else`                                    | `@call file_check` for branching    | —                                         |
| Structured data                        | `@list <file.yaml> \| @render type="table"`                                  | plain Markdown table at >50 rows    | —                                         |
| Counts / Statistics                    | `{{ @count ./src "*.ts" }}`                                                  | hard-coded numbers                  | —                                         |
| Cross-File-Content                     | `@include ./CHANGELOG.md` or lines=N-M                                       | copy-paste between specs            | —                                         |
| Machine-Readable Constraints           | `@constraint id="..." severity="high"` + body + `@constraint-end`            | prosaic "Important:" hints          | —                                         |
| Project-Context (live)                 | `@call ctx_overview(task="...")`                                             | manually copied project description | —                                         |
| Filesystem writes (NEW v1.0)           | `@mkdir <path>` / `@copy src dst`                                            | `@query ctx_shell "mkdir -p ..."`   | `@mkdir` / `@copy` / `@append-if-missing` |
| YAML-frontmatter mutate (NEW v1.0)     | `@update-frontmatter file=... key=... value=...`                             | shell `sed -i` über `---`-Block     | `@update-frontmatter`                     |
| Sub-Render mit args (NEW v1.0)         | `@render-template from="<src>" to="<dst>" [force]` + key=value body + `@render-template-end` | manual string-concat in @query      | `@render-template`    |
| Conditional anchor-check (NEW v1.0)    | `@if file.containsLine("<file>", "<anchor>")`                                | grep-output check                   | `file.containsLine`                       |
| Iteration über list/anchors (NEW v1.0) | `@foreach item in {{ items }}` + body + `@foreach-end`                       | repeated `@if`-Blöcke               | `@foreach` + `@set`                       |
| Multi-branch (NEW v1.0)                | `@switch <var>` + `@case "..."` + `@default` + `@switch-end`                 | repeated `@if`/`@elseif`-chain      | `@switch`/`@case`/`@default`              |

**Anti-pattern: `file_check` is not branching.** Use it as a status-renderer only.

For branching ALWAYS inline at the call site:

```
@if file.exists("x.md")
- do this when exists
@else
- do that when missing
@if-end
```

**Exception** (per §10.4 #9): specs for purely algorithmic topics without
file/tool/data dependencies may stay plain Markdown — then set
`markdownai_directives_omitted: <reason>` in the frontmatter.

<!-- Drift-Tracking: hand-ported from body.mdai.md dialog-process phase,
     consolidated with v1.0 Wave-3–5 native equivalents column. -->
