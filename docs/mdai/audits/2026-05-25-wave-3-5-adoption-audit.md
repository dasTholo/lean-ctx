---
title: Wave 3-5 Direktiven-Adoption-Audit (Phase 4 / mdai-Skill v0.1.1)
slug: wave-3-5-adoption-audit
date: 2026-05-25
status: ready-for-review
authors: claude
plan: docs/mdai/plans/2026-05-25-mdai-brainstorm-v0.1.1-refactor.md
spec: docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md
scope: alle Files in mdai/core/ + mdai/skills/mdai-brainstorm/ + 2 antizipierte P6-Files
audit_method: Spec §6.3 (search-then-targeted-read)
output_format: Spec §6.4
hotspot_source: Spec §6.5
---

# Wave 3-5 Adoption-Audit

## Übersicht

| File                                                      | Status        | Adoptions | Skipped | Notiz                                                        |
|-----------------------------------------------------------|---------------|-----------|---------|--------------------------------------------------------------|
| mdai/core/ctx-knowledge.md                                | keep-as-is    | 0         | 4       | Pure MCP-Wrapper, kein conditional/iter Pattern              |
| mdai/core/ctx-tools.md                                    | keep-as-is    | 0         | 4       | Pure MCP-Wrapper, keine Logic                                |
| mdai/core/file-utils.md                                   | keep-as-is    | 0         | 1       | `file_check` ist nur Status-Renderer (1-Zeilen-`@if`)        |
| mdai/core/hard-rules.md                                   | keep-as-is    | 0         | 1       | Mode `include` (no frontmatter, no @define) — reine Doku     |
| mdai/core/lean-context.md                                 | keep-as-is    | 0         | 1       | Mode `include` — reine Rules-Tabelle                         |
| mdai/core/mcp-markdownai.md                               | keep-as-is    | 0         | 3       | Pure MCP-Wrapper, kein Logic-Spielraum                       |
| mdai/core/startup-check.md                                | adopt-major   | 5         | 2       | Hotspot — shell-heuristics ersetzbar; `detect_tooling` foreach |
| mdai/core/tool-quick-ref.md                               | keep-as-is    | 0         | 1       | Mode `include` (Tabelle) — reine Doku                        |
| mdai/skills/mdai-brainstorm/README.md                     | keep-as-is    | 0         | 0       | Reines Prose-Readme, kein Direktiven-Body                    |
| mdai/skills/mdai-brainstorm/SKILL.md                      | keep-as-is    | 0         | 0       | Pointer-File, frontmatter-only                               |
| mdai/skills/mdai-brainstorm/body.mdai.md                  | adopt-major   | 6         | 2       | Hotspot + Carry-Over B1 (`@call render_spec` nested default) |
| mdai/skills/mdai-brainstorm/spec-reviewer.md              | adopt-minor   | 3         | 4       | Hotspot — §5 anti-pattern-checks via `file.containsLine`     |
| mdai/skills/mdai-brainstorm/visual-companion-offer.md     | keep-as-is    | 0         | 1       | Fragment unter conditional `@include` — keep flat            |
| mdai/skills/mdai-brainstorm/write-spec.md                 | refactor      | 5         | 1       | Hotspot — shell `mkdir + cat > heredoc`; `@switch` für target|
| mdai/core/lean-context-audit.md (geplant P6)              | planned       | 3         | 0       | Antizipiert: `@set` + `@foreach` + `@if file.containsLine`   |
| mdai/core/library-spec-audit.md (geplant P6)              | planned       | 3         | 0       | Antizipiert: `@foreach` + `@switch check.type`               |

Summen: **adopt-major = 3**, **adopt-minor = 1**, **refactor = 1**, **planned = 2**, **keep-as-is = 9**. Adoptions
gesamt **= 25** (über 16 Files). Skipped gesamt = 25.

---

## Per-File-Audit

### mdai/core/ctx-knowledge.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- `@foreach`: nur 6 atomare `@define`-Wrapper, jeder mit einem einzigen `@query`. Kein Iterations-Pattern.
- `@if file.exists`: keine Filesystem-Tests im Body.
- `@switch`: kein Verzweigungs-Spielraum (jedes Macro ist 1-Liner).
- `@render-template`: kein Template-Compose-Bedarf.

---

### mdai/core/ctx-tools.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- `@foreach`: 8 atomare `@define`-Wrapper für `ctx_read/ctx_search/ctx_tree/ctx_shell/ctx_edit/ctx_read_lines/ctx_read_map/ctx_read_signatures`. Pure 1-Liner — kein Iter-Bedarf.
- `@if file.*`: keine Filesystem-Tests.
- `@switch`: kein Branching im Body.
- `@mkdir`/`@copy`: keine Filesystem-Writes (nur MCP-Reads).

---

### mdai/core/file-utils.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- `@if file.exists`: bereits korrekt im einzigen Macro `file_check` verwendet (Zeile 11). Ist genau die Wave-1-Direktive.
  Keine weitere Adoption nötig — Macro ist intentional minimal (Status-Renderer, kein Control-Flow).

---

### mdai/core/hard-rules.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- Komplettes File ist `mode: include`-Text (keine YAML-frontmatter, keine `@define`). Reines Rules-Listing.
  Kein @-Direktiven-Body, der Wave 3-5 brauchen könnte. Adoption hier wäre Over-Engineering.

---

### mdai/core/lean-context.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- Wie `hard-rules.md` — `mode: include`-Text (Rules-Tabelle + Naming-Conventions). Kein executable Body.
  Adoption nicht anwendbar.

---

### mdai/core/mcp-markdownai.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- `@foreach`/`@switch`/`@if`: 3 atomare 1-Liner-Wrapper (`read_phase`, `list_phases`, `get_constraints`).
  Keine Verzweigung, keine Iteration, keine Filesystem-Writes.

---

### mdai/core/startup-check.md

Status: adopt-major

#### Adoptions

- `@switch` + `@case`: Zeile 36-46 (`detect_project_lang`)
  Before: 2-stufige Detection: `ctx_overview` → falls leer → `ctx_shell` mit verschachteltem `if/elif/else`-Heredoc
  ("if [ -f Cargo.toml ]; then echo rust; elif ...").
  After: ersten Branch behalten; den Shell-Heuristik-Fallback durch `@if file.exists "Cargo.toml"` / `@elseif file.exists "pyproject.toml"` / `@elseif file.exists "package.json"` ersetzen — nutzt Wave-1 `file.exists` + reine v1.0-Direktiven statt Shell-Heredoc.
  Benefit: kein Bash-Subshell mehr, deterministisch, parser-sichtbar (heredoc-Strings sind Parser-Black-Box).

- `@foreach` + `@set`: Zeile 62-79 (`detect_tooling`)
  Before: `claude mcp list | grep -E 'jetbrains|serena'` + zwei repeated `@if @result.stdout matches "..."`-Blocks.
  After: `@set tools = ["jetbrains", "serena"]` + `@foreach tool in tools` + ein einziges `@if @result.stdout matches "{{ tool }}"` + `@endforeach`.
  Benefit: DRY — neue Tools (z.B. `claude_ai_Gmail`) per Listen-Append statt copy-paste-Block. Pattern matched §6.5 ("foreach über tool-list statt repeated @if").

- `@foreach` + `@switch`: Zeile 81-89 (`load_lang_pack`)
  Before: explizite `@if/@elseif/@elseif`-Kette über 3 lang-strings ("rust"/"python"/"node") mit je einem `@include`.
  After: `@switch @env MDAI_PROJECT_LANG` + `@case "rust"` / `@case "python"` / `@case "node"` mit jeweils dem
  `@include ${MDAI_LIBRARY_ROOT}/lang/{{ @case.value }}.md`.
  Benefit: deklarativ statt prozedural; ergänzbar ohne `@elseif`-Tail-Editing. Spec §6.2 Wave-5+-Direktive.

- `@foreach` über tooling-packs: Zeile 91-97 (`load_tooling_packs`)
  Before: zwei sequentielle `@if @env MDAI_HAS_<TOOL>` + `@include`-Blocks für jetbrains/serena.
  After: `@set tooling_packs = [{name="jetbrains", flag="MDAI_HAS_JETBRAINS"}, {name="serena", flag="MDAI_HAS_SERENA"}]`
  + `@foreach pack in tooling_packs` + `@if @env {{ pack.flag }} == "true"` + `@include ${MDAI_LIBRARY_ROOT}/tooling/{{ pack.name }}.md` + `@endforeach`.
  Benefit: konsistent mit `detect_tooling`-Pattern; neue tooling-packs append-only.

- `@if file.exists`: ergänzend in `detect_project_lang()` (s.o.) — explizit als separater Adoption-Eintrag, weil
  Wave-1-Direktive `file.exists` direkt das Shell-Heredoc ersetzt (nicht nur als Switch-Argument):
  Before: `if [ -f Cargo.toml ]; then echo rust` (Shell-Heredoc).
  After: `@if file.exists "Cargo.toml"` (native Wave-1).
  Benefit: keine subshell-roundtrip, kein `@result.stdout`-Parsing.

#### Skipped

- `@mkdir`/`@copy`/`@append-if-missing`: keine Filesystem-Writes im File (nur Reads/Probes).
- `@render-template`: alle Outputs sind reine Status-Strings (`[mdai-bootstrap OK] ...`). Kein Template-Compose
  rechtfertigt eigene Template-Datei.

---

### mdai/core/tool-quick-ref.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- `mode: include`-Text (Tabelle Tool→Macro→MCP). Kein executable Body. Wartung über Edits, nicht Direktiven.

---

### mdai/skills/mdai-brainstorm/README.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- Reines Markdown-Readme — keine `.mdai.md`-Direktiven. Außerhalb des v1.0-Scope.

---

### mdai/skills/mdai-brainstorm/SKILL.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- Pointer-File für `/mdai-brainstorm` (Frontmatter + 15 Zeilen Hinweis). Kein Logic-Body.

---

### mdai/skills/mdai-brainstorm/body.mdai.md

Status: adopt-major

#### Adoptions

- **Carry-Over B1 (Parser-Fix, MUST):** Zeile 290 `@call render_spec(slug={{ slug }}, target={{ render_target | default("none") }})`
  Before: nested `{{ ... | default("none") }}` direkt im `@call`-Argument — markdownai v1.0 ParseError (nested `{{ }}` in directive-args).
  After: `@set render_target_resolved = render_target | default("none")` davor + `@call render_spec(slug={{ slug }}, target={{ render_target_resolved }})`.
  Benefit: parser-konform; Wave-5-`@set`-Direktive löst auf einen Schlag das Carry-Over auf.

- `@set` + Phase-Vars: pre-context phase (Zeile 17-32)
  Before: 5 separate inline `{{ @call ... }}` für Branch / Recent-Commits / Project-Map / Dep-Graph / Tree.
  After: `@set ctx_branch = @call ctx_shell(cmd="git branch --show-current")` etc. — vorab binden, dann referenzieren.
  Benefit: pre-computed, einmalige Evaluation, leichter zu cachen / zu refactoren.

- `@switch current_phase` (Spec §6.5): Process-Checklist Zeile 145-154 (phase-transitions)
  Before: die 9-Punkt-Liste ist statischer Prose-Text — phase-übergänge sind nur als Hinweise dokumentiert.
  After: optional `@switch @env MDAI_CURRENT_PHASE` mit `@case "dialog-process"` / `@case "write-outputs"` / `@case "handoff"` —
  jeweils render des aktuellen Checklist-Status (welche items done / pending).
  Benefit: deklarativ + machine-readable; reduziert "wo bin ich gerade"-Klärungsschleifen.
  Note: optional, weil current code statisch funktioniert; nur wenn Phase-Tracking gewünscht.

- `@if file.exists` statt visual-companion `ctx_session` matches: Zeile 180-184
  Before: `@query mcp lean-ctx ctx_session action="status"` + `@if @result.stdout matches "\[mdai-brainstorm\] visual=true"` + `@include …/visual-companion-offer.md`.
  After: zusätzlich `@if file.exists ".superpowers/brainstorm/server.json"` als zweite Bedingung (file-based persistence statt session-only).
  Benefit: session-restart-resilient; visual=true bleibt cross-session erkennbar.

- `@call detect_mai_hook_version()`: pre-context (geplant in P2/P5)
  Before: keine `mai`-CLI-Version-Detection — Skill nimmt v1.0 an.
  After: `@call detect_mai_hook_version()` aus `startup-check.md` early in pre-context (laut Spec §6.5).
  Benefit: Skill kann auf v0.0.24 vs v1.0 brench-react; verhindert silent Wave-3-5-failures auf alten Engines.

- `@constraint id="b1-fix" severity="high"`: oberhalb der `render_spec`-Call-Site (write-outputs phase)
  Before: keiner.
  After: `@constraint id="render-target-binding" severity="medium"` — dokumentiert die `@set`-Pflicht und verlinkt auf B1-Carry-Over.
  Benefit: machine-readable Reminder; künftige Refactors brechen B1-Fix nicht versehentlich.

#### Skipped

- `@mkdir`/`@copy`/`@append-if-missing`: keine direkten Filesystem-Writes im body (delegated an `write-spec.md`-Pack).
- `@render-template`: bereits über `@import write-spec.md` + `@call write_spec` abstrahiert — kein redundant Template-Compose.

---

### mdai/skills/mdai-brainstorm/spec-reviewer.md

Status: adopt-minor

#### Adoptions

- `@foreach anti_pattern_check in checks`: §5 (Zeile 90-150, anti-pattern-checks #1-#11)
  Before: 11 prose-formatted Check-Bullets, jeder mit eigenem `ctx_search`-Pattern.
  After: `@set checks = [{id="mcp-signatures", pattern="match action|fn handle", ...}, {id="existing-store", ...}, ...]`
  + `@foreach check in checks` + `@call ctx_search(pattern={{ check.pattern }}, path={{ spec_path }})` + bewerten.
  Benefit: Check-Liste wird machine-readable; neue Checks per List-Append; reviewer-prompt bleibt schlank.
  Note: bewahrt die Calibration-Logik aus §4 (jeder Check kann needs-revision triggern).

- `@switch status` für Output-Variation (Spec §6.5): §6 Report-Format
  Before: einheitlicher Report-Bullet-Block für jeden Status.
  After: `@switch status` mit `@case "Approved"` / `@case "Needs-Revision"` / `@case "Needs-Clarification"` —
  jeweils template-spezifischer Patches/Recommendations-Block (Approved = nur Strengths, Needs-Revision = full).
  Benefit: kürzerer Output bei Approved; klarere Differenzierung; weniger Cognitive Load.

- `@if file.containsLine`: Check #6 Language-Convention (Zeile 132-135)
  Before: `ctx_search(pattern="[ÄÖÜäöüß]", path="<lib-dir>")` als Shell-pattern-search.
  After: für einzelne library-files zusätzlich `@if file.containsLine "ß"` etc. als Pre-Gate; wenn Pre-Gate false,
  Skip des teureren `ctx_search` über das gesamte lib-dir.
  Benefit: schneller bei großen lib-trees; explizite Line-Anchors.

#### Skipped

- `@mkdir`: Output-Write nach `docs/mdai/reviews/` wird über `write_review_report` in P5-T9 geplant — nicht direkter
  spec-reviewer-Adoption.
- `@render-template`: Reviewer-Prompt selbst ist ein `@define`-Block; Template-Mechanik wäre Over-Engineering.
- `@update-frontmatter`: Reviewer schreibt keinen Frontmatter — nur den Review-Report-Body.
- `@copy`/`@append-if-missing`: nicht anwendbar (kein File-Sync).

---

### mdai/skills/mdai-brainstorm/visual-companion-offer.md

Status: keep-as-is

#### Adoptions

(keine)

#### Skipped

- File ist ein conditional-included Fragment (24 Zeilen, single `@call ctx_read` + single `@call ctx_shell`). Keine
  Iterations- oder Verzweigungs-Logic. Flat-bleiben ist intentional.

---

### mdai/skills/mdai-brainstorm/write-spec.md

Status: refactor

#### Adoptions

- `@mkdir docs/mdai/specs` + `@append-if-missing` (oder vergleichbar) für `write_spec` (Zeile 12-22)
  Before:
  ```
  @query mcp lean-ctx ctx_shell cmd="
  mkdir -p docs/mdai/specs &&
  SPEC_PATH=docs/mdai/specs/... &&
  cat > \"$SPEC_PATH\" <<'SPEC_EOF'
  SPEC_EOF
  echo \"wrote $SPEC_PATH\""
  ```
  After:
  ```
  @mkdir docs/mdai/specs
  @render-template ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/spec-template.md
    args={slug="{{ slug }}", body="{{ body }}", date="{{ @date format='YYYY-MM-DD' }}"}
    output="docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"
  ```
  Benefit: ❶ kein Shell-Heredoc mehr (Parser-opak), ❷ Template-File ist editierbar/lintbar, ❸ atomarer Write
  statt 2-Step (mkdir+cat).

- `@render-template`: render_spec target="chat" (Zeile 22-26)
  Before: `@query mcp markdownai read_file file="..."` — direkter MCP-Call.
  After: kein Change nötig; aber: prüfen ob `@render-template` mit `output="chat"` der natürlichere v1.0-Idiom ist.
  Benefit: einheitliches Mental-Model `render-template` für alle render-Pfade. **Note: optional, bewahrt Backward-Compat falls `output="chat"` nicht supported ist.**

- `@switch target`: render_spec (Zeile 29-46)
  Before: `@if {{ target }} == "none"` + `@elseif {{ target }} == "chat"` + `@elseif {{ target }} == "file"` + `@endif`.
  After: `@switch target` + `@case "none"` / `@case "chat"` / `@case "file"` / `@default`.
  Benefit: explizit dispatch, leichter neue Targets ergänzen (z.B. `"pdf"`), `@default`-fallback statt silent no-op.

- `@if file.exists` (already used) wiederholt für jedes target-branch (Zeile 35, 42)
  Before: jedes `@case` wiederholt die `@if file.exists "..."`-Prüfung.
  After: top-level `@set spec_path = "docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"`
  + `@if file.exists "{{ spec_path }}"` als outer Guard + inner `@switch target`.
  Benefit: DRY — Path nur einmal komputiert; outer-Guard reduziert nested checks; `@set` ist Wave-5-Direktive.

- `@mkdir docs/mdai/specs/rendered` (target="file"-Branch, Zeile 43)
  Before: `mkdir -p docs/mdai/specs/rendered && (cd markdownai && npx mai render ...)` — Shell-Chain.
  After: `@mkdir docs/mdai/specs/rendered` + separate `@call ctx_shell(cmd="cd markdownai && npx mai render ...")`.
  Benefit: filesystem-mkdir via native Wave-3-Direktive; `cd`-Side-Effect bleibt in `ctx_shell` (npx-Call hat keine Wave-3-Entsprechung).
  Note: der `npx mai render`-Call selbst bleibt `ctx_shell` — `@call ctx_shell` ist der primäre v1.0-Workaround dafür.

#### Skipped

- `@update-frontmatter`: spec-Frontmatter wird beim ersten Write komplett via Template gesetzt — kein Post-Write-Mutate-Bedarf.
- `@foreach`: zwei Macros, jedes mit linearem Flow — keine Iter.
- `@if file.containsLine`/`file.frontmatterField`: kein Content-Predicate-Bedarf (Existenz reicht).
- `@constraint`: ist Skill-Pack, keine Spec — `@constraint`-Use für Konsumenten, nicht für Pack-Wrapper.

---

### mdai/core/lean-context-audit.md (geplant in P6 — antizipiert)

Status: planned (Datei existiert noch nicht; Empfehlung für P6-T13)

#### Adoptions

- `@set anchors = ["mode_eq_full", "raw_eq_true", "fresh_eq_true", "find_symbol_body_true", "include_no_lines", "ctx_read_no_mode"]`
  + `@foreach anchor in anchors` + `@if file.containsLine "{{ anchor.pattern }}"`
  Before: in der Vorgänger-Spec ein 6-zeiliger flacher `ctx_search`-Block pro Anchor (copy-paste).
  After: `@set` der Anchor-Liste + `@foreach` über die 6 Anchors + `@call ctx_search` + Result-Aggregation.
  Benefit: DRY; neue Anchors per List-Append; Output ist tabellarisch reportbar.

- `@switch severity`: report-builder
  Before: prose-only Findings.
  After: `@switch finding.severity` → `@case "blocker"` / `@case "warn"` / `@case "info"` mit jeweils unterschiedlicher
  Render-Variante.
  Benefit: differenziertes Reporting; calibration-bewusst.

- `@render-template ${MDAI_LIBRARY_ROOT}/core/lean-context-audit-template.md`: Report-Output
  Before: hand-crafted Markdown-Bullets.
  After: Template-File mit `{{ findings }}` als Platzhalter; konsistent über mehrere Audits.
  Benefit: konsistente Audit-Reports; Template editierbar ohne Macro-Touch.

#### Skipped

(keine — File ist noch nicht erstellt; alle Wave-3-5-Optionen sind antizipiert.)

---

### mdai/core/library-spec-audit.md (geplant in P6 — antizipiert)

Status: planned (Datei existiert noch nicht; Empfehlung für P6-T14)

#### Adoptions

- `@set checks = [{type="mcp-signatures", ...}, {type="store-enum", ...}, ...]` + `@foreach check in checks` + `@switch check.type`
  Before: in der Vorgänger-Spec 7 separate hand-gewartete Check-Blöcke (copy-paste-pattern wie in `spec-reviewer.md`).
  After: machine-readable Check-List + `@switch check.type` für dispatch zu jeweils passendem `@call ctx_search`-Pattern.
  Benefit: 7 Checks deklarativ; neue Checks per List-Append; Reviewer-Logic bleibt schlank.

- `@if file.containsLine` für Inline-Pre-Gates (jeder Check)
  Before: jeder Check würde `ctx_search` über das gesamte lib-dir laufen lassen.
  After: pro Check zuerst `@if file.containsLine "<anchor>"` als Pre-Gate — wenn false, ganzen Check skippen.
  Benefit: schneller bei großen Library-Trees; explizite File-Anchors.

- `@render-template ${MDAI_LIBRARY_ROOT}/core/library-spec-audit-template.md`: Report-Output
  Before: hand-crafted Markdown.
  After: Template + `{{ checks_results }}`-Aggregation.
  Benefit: konsistente Reports; gleicher Mechanismus wie `lean-context-audit.md`.

#### Skipped

(keine — File ist noch nicht erstellt; alle Wave-3-5-Optionen sind antizipiert.)

---

## Anhang A: Carry-Over B1 (Parser-Fix MUST in T10)

**Pfad:** `mdai/skills/mdai-brainstorm/body.mdai.md` Zeile 290.

**Aktueller Code:**

```
@call render_spec(slug={{ slug }}, target={{ render_target | default("none") }})
```

**Problem:** markdownai v1.0 Parser akzeptiert kein nested `{{ ... | default(...) }}` direkt in `@call`-Args. Pipe-Default
muss VOR dem `@call` gebunden werden.

**Fix (in T10, body.mdai.md Phase-Refactor):**

```
@set render_target_resolved = render_target | default("none")
@call render_spec(slug={{ slug }}, target={{ render_target_resolved }})
```

**Verifikation (post-fix):** `mcp__markdownai__resolve_phase(path="<body>", phase="write-outputs")` → 0 ParseError.

---

## Anhang B: Audit-Methodik-Telemetry

- Searches ausgeführt (`@query|@call ctx_shell|bash|if.*then`, `frontmatter|@define|@constraint`, `for each|...|@foreach|@if|@switch|@set`, `mkdir|cat >|sed |@copy|@render-template|@update-frontmatter|@append-if-missing`): 4 batched über `mdai/` (insgesamt 179 matches, 18 Files).
- Targeted reads (mode=full): 14 Files (alle in Scope).
- Spec-Referenz-Reads: §6.3-6.5 (lines:507-650).
- Audit-Output: dieses File.

---

## Anhang C: Empfehlung für User-Review-Gate

**Decision-Matrix:** der User entscheidet pro Adoption-Item adopt | skip | defer.

**Empfehlung des Audits:**

| Priorität     | Items                                                                                              |
|---------------|-----------------------------------------------------------------------------------------------------|
| **MUST**      | body.mdai.md Carry-Over B1 (Parser-Fix). Blockt sonst write-outputs phase auf v1.0.                |
| **SHOULD**    | write-spec.md `@mkdir + @render-template` + `@switch target` (Hotspot-Refactor; spart Shell-Heredoc). |
| **SHOULD**    | startup-check.md `@switch lang` + `@foreach tooling_packs` (DRY für lang/tooling-extensions).      |
| **NICE**      | spec-reviewer.md §5 `@foreach checks` (lesbarer aber funktional äquivalent).                       |
| **NICE**      | body.mdai.md `@set ctx_*` pre-context vars (mikrooptimierung).                                     |
| **DEFER**     | `@switch current_phase` in body (nur sinnvoll mit echtem Phase-Tracking — separate Spec).          |
| **PLANNED**   | lean-context-audit.md + library-spec-audit.md — Adoptions in P6-Implementation einarbeiten.        |
