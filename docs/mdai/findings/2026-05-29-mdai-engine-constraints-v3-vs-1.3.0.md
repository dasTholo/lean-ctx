---
title: mdai Engine-Constraints v3 — re-validated against markdownai 1.3.0
date: 2026-05-29
status: ready-for-review
authors: claude
engine_target: markdownai 1.3.0
supersedes: docs/mdai/findings/2026-05-26-mdai-engine-constraints-v2-vs-v1.0.md
related_plans:
  - docs/mdai/plans/2026-05-29-mdai-v0.1.3-v2-adoption.md
related_specs:
  - docs/mdai/specs/2026-05-29-mdai-v0.1.3-v2-adoption-design.mdai.md
verified_via:
  - MCP call_macro / resolve_phase against the live 1.3.0 server
  - CLI npx mai validate / render from repo root
---

# mdai Engine-Constraints v3 — markdownai 1.3.0

Dieses Dokument supersedet `2026-05-26-mdai-engine-constraints-v2-vs-v1.0.md` (v2, gegen v1.0.0). Die v2-Findings
wurden gegen die **live 1.3.0-Engine** re-validiert. Mehrere v1.0-Constraints sind in 1.3.0 **behoben** (teils durch
zwei in dieser Session eingebrachte Engine-Fixes, siehe Anhang A). Zusätzlich dokumentiert v3 die drei vom
Migrations-Tool nicht abgedeckten v2-Syntax-Lücken, die Auflösungsregeln für `@include`/`@date` und den
Dedup-Mechanismus-Entscheid.

## Status-Übersicht (1.3.0)

| §  | Topic                                     | 1.3.0-Status                        |
|----|-------------------------------------------|-------------------------------------|
| 1  | Closer-Syntax                             | GEÄNDERT → `@<name>-end`            |
| 2  | @set-Liste als @foreach-Source            | TEILS-GEFIXT (Objektliste via JSON) |
| 3  | {{var}} in file.containsLine in @define   | GEFIXT (engine-fix f16b4c2)         |
| 4  | file.* Cross-CWD / Absolutpfad            | GEFIXT (löst auf)                   |
| 5  | @date in @set / Interpolation             | GEÄNDERT → directive-valued @set    |
| 6  | @render-template from/to/force            | still-valid (re-confirmed)          |
| 7  | ${MDAI_LIBRARY_ROOT} + @include-Auflösung | still-valid + erweitert             |
| 8  | Frontmatter leakt in mode:include         | still-valid (re-confirmed)          |
| 9  | write_enabled Pflicht                     | still-valid (nicht re-getestet)     |
| 10 | npm-prefix PATH                           | still-valid (User-Env)              |
| 11 | Modus A vs B (Prosa vs @call)             | still-valid (Konvention)            |
| 12 | per-call Wrapper-Synthesis                | still-valid (re-confirmed)          |

---

## §1 — Closer-Syntax: `@<name>-end`

**1.3.0-Status:** GEÄNDERT. 1.3.0 verwendet **named closers** `@<name>-end`: `@if-end`, `@foreach-end`,
`@switch-end`, `@constraint-end`, `@render-template-end`, `@define-end`. Das v2-Modell (`@end` universell,
`@endif`/`@endswitch`) ist obsolet. Argumentlose Directives self-closen mit trailing ` /` (z.B. `@include … /`,
`@set x = … /`).

**Migration:** mechanisch erledigt (Commit `9dc9e46d`); diese Session hat verbliebene v1-Closer in der
Conventions-Doku auf `@<name>-end` umgestellt. `validate` clean über alle migrierten Files.

---

## §2 — `@set`-Liste als `@foreach`-Source

**1.3.0-Status:** TEILS-GEFIXT.

- **Objektlisten (vormals Failure-Mode A):** funktionieren jetzt, ABER nur in der korrekten Form —
  `@set xs = {{ [{"name":"a","flag":"F"}] }} /` (JSON, quoted keys, in `{{ }}` gewrappt) gefolgt von
  `@foreach x in {{ xs }}` mit Dot-Access `{{ x.name }}`. Eine bare `[{name=a}]`-Form (ohne `{{ }}`, unquoted keys)
  wird als String gespeichert und an jedem Komma gesplittet (n Fragmente, leerer Dot-Access). Ermöglicht durch
  Engine-Fix `ede9793` (Anhang A). Verifiziert via `call_macro load_tooling_packs` (2 Iterationen, Dot-Access OK).
- **`@set` als Pipe-Source (Failure-Mode B):** WEITERHIN ungültig. `@set x = y | default("none") /` wirft
  `"@set" cannot be used as a pipe source`. Real vorhanden in `body.mdai.md:185`
  (`@set render_target_resolved = render_target | default("none") /`) — vorbestehend, außerhalb des v0.1.3-Scopes,
  hier als offener Punkt getrackt.

---

## §3 — `{{var}}` in `file.containsLine` (interpolierter Predicate-Arg)

**1.3.0-Status:** GEFIXT durch Engine-Fix `f16b4c2` (Anhang A). Macro-named-args werden jetzt in
`skillContext.namedArgs` propagiert, sodass `@if file.containsLine({{ spec_path }}, "…")` in einem `@define`-Body
das `{{ spec_path }}` auflöst. Verifiziert: `spec_reviewer_prompt` (spec-reviewer.md) rendert ohne Predicate-Fehler.
Syntax-seitig zusätzlich auf v2-Call-Form migriert: `file.containsLine({{ var }}, "literal")` — reine Interp-Arg
unquoted, Literal quoted.

---

## §4 — `file.*` Cross-CWD / Absolutpfad

**1.3.0-Status:** GEFIXT. `file.exists` mit Absolutpfad löst korrekt auf. Verifiziert via
`call_macro file_check(path="/home/tholo/Scripts/lean-ctx/CLAUDE.md")` → `- … exists`, warnings []. Der v1.0-Block
(`filesystem.ts checkFilePath` blockte `isAbsolute` bedingungslos) greift in 1.3.0 nicht mehr.

---

## §5 — `@date` in `@set` / Interpolation

**1.3.0-Status:** GEÄNDERT — und der v2-Workaround war FALSCH. `@date` ist eine **Directive**, NICHT im
`{{ }}`-Interpolations-Sandbox verfügbar. Inline `{{ @date format='YYYY-MM-DD' }}` rendert **leer** (CLI und MCP) —
der v2-Empfehlung „`@date` direkt am Point-of-Use verwenden" erzeugt denselben leeren Output.

**Korrekter Weg:** directive-valued `@set` — `@set spec_date = @date format='YYYY-MM-DD' /` (ohne Quotes, ohne
`{{ }}`), danach `{{ spec_date }}`. Verifiziert: `render_spec` liefert `docs/mdai/specs/2026-05-29-<slug>-…`
(echtes Datum). `now_iso()` ist ein Interpolations-Builtin, aber CLI-only (MCP `@eval` gesperrt) und liefert einen
vollen ISO-Timestamp (falsches Format für Datei-Pfade). Migration in `write-spec.md` umgesetzt.

---

## §6 — `@render-template from="…" to="…" [force]` + `@render-template-end`

**1.3.0-Status:** still-valid (re-confirmed). `from`/`to`/`force` + Block-Closer `@render-template-end`. Kein
`output=`-Arg. Verifiziert via `write_spec`/`render_spec`-Smokes.

---

## §7 — `${MDAI_LIBRARY_ROOT}` + `@include`-Auflösung (erweitert)

**1.3.0-Status:** still-valid + neue Erkenntnisse. Cross-pack-Includes nutzen
`@include ${MDAI_LIBRARY_ROOT}/<pfad>.md /` (Library-Konvention). Die Auflösung unterscheidet sich nach Entrypoint:

- **MCP** (`call_macro`/`resolve_phase`, `cwd` = Repo-Root): `@include` löst **repo-root-relativ** auf und
  expandiert `${MDAI_LIBRARY_ROOT}`. Das ist der Laufzeit-Pfad.
- **CLI** (`mai render`/`validate`): `@include` löst **dokument-relativ** auf, expandiert `${MDAI_LIBRARY_ROOT}`
  **nicht**, und der Parser lehnt jedes `..`-Segment ab.
- **Folge:** Ein unbedingter `${MDAI_LIBRARY_ROOT}/…`-`@include` im Makro-Body funktioniert zur MCP-Laufzeit, aber
  `mai validate` meldet dafür `@include: file not found` (keine Env-Expansion, eager dokument-relativ). Das ist eine
  **validate-Tooling-Grenze**, kein Laufzeit-Defekt — solche Files via MCP verifizieren. Bestehende Library-Includes
  umgehen den validate-Fehler nur, weil sie in `@if`/`@switch` stehen (validate überspringt bedingte Includes).
- Ein included Fragment-`@markdownai`-Header wird NICHT in die Ausgabe geleakt.

Dokumentiert in `core/mcp-markdownai.md` (Engine resolution notes) + `core/hard-rules.md`.

---

## §8 — Frontmatter leakt als Text in `mode:include`

**1.3.0-Status:** still-valid (re-confirmed). Fragment-Files (`core/_fragments/lean-context-anchors.md`) wurden
daher ohne YAML-Frontmatter angelegt (nur `@markdownai`-Header + Inhalt) — kein Leak im `@include`-Output bestätigt.

---

## §9 — `filesystem.write_enabled: true` Pflicht

**1.3.0-Status:** still-valid (in dieser Session nicht erneut getestet; v2-Status übernommen). `write_enabled` bleibt
das primäre Gate für `@mkdir`/`@render-template`; `write_root` + `allowed_write_paths` als granulare Controls daneben.

---

## §10 — `npm config prefix` + PATH

**1.3.0-Status:** still-valid (User-Env-Side-Effect). In dieser Session wurde `mai` via
`npx --prefix markdownai mai …` aus dem Repo-Root aufgerufen — der Workspace-Symlink
`node_modules/.bin/mai → packages/core/dist/cli.js` zeigt auf den frisch gebauten Dist. CLI immer aus Repo-Root
aufrufen (nicht aus `markdownai/` — sonst „Path traversal above document root").

---

## §11 — Modus A (Prosa `call_macro`) vs. Modus B (`@import` + `@call`)

**1.3.0-Status:** still-valid (Konvention). Modus A für Library-Packs außerhalb `body.mdai.md`, Modus B für
Bootstrap in `body.mdai.md`. Pro Phase nicht mischen.

---

## §12 — per-call Wrapper-Synthesis

**1.3.0-Status:** still-valid (re-confirmed). `call_macro` synthetisiert pro Aufruf einen Wrapper
(`@markdownai\n@import ./<file>\n@call <macro>(args)`) — daher kein expliziter `@import` in aufrufenden Files nötig.
Bestätigt: alle Macro-Smokes dieser Session liefen via `call_macro` ohne expliziten `@import`.

---

## Anhang A — Engine-Fixes (markdownai, Branch `feat-mdai`)

Zwei echte Engine-Bugs blockierten die v2-Adoption real und wurden (vom User autorisiert) im markdownai-Repo
gefixt. Beide sind im neu gebauten Dist enthalten (der laufende Dist war zu Session-Beginn älter als die Fixes —
Rebuild war nötig).

1. **`f16b4c2`** — `fix(engine): propagate macro named-args into skillContext for @if conditions`. Ohne den Fix
   löst `@if file.exists({{ param }})` in `@define`-Bodies das `{{ param }}` nicht auf (immer `MISSING`). Blockierte
   §3 + alle predicate-call-Migrationen mit interp-Args.
2. **`ede9793`** — `fix(engine): bind @foreach object items into ctx.data for dot-access`. `splitItems` zerstörte
   Objekt-Elemente via `String(v)`; `executeForeach` band Items nur als String in `ctx.envFiles`. Fix: Objekte
   erhalten + an `ctx.data` binden (Dot-Access `{{ x.field }}`). +1 vitest-File (3 Tests). Ermöglicht §2-Objektlisten.

---

## Anhang B — Migrations-Tool-Lücken (manuell geschlossen)

Das v1→v2-Migrations-Tool deckte drei Syntax-Klassen nicht ab; manuell geschlossen in P0:

1. **Predicate-Call-Form:** `file.exists "x"` → `file.exists("x")`; `file.containsLine "a" "b"` →
   `file.containsLine("a", "b")`. Quoting: reine Interp-Variable unquoted (`file.exists({{ var }})`); String-Literal
   (auch mit Interp) quoted (`file.exists("docs/{{ slug }}.md")`). `matches` bleibt infix.
2. **`@foreach`-Source-Interpolation:** `@foreach x in liste` → `@foreach x in {{ liste }}`. Objektlisten zusätzlich
   als JSON-in-`{{ }}` (siehe §2).
3. **Objektlisten-`@set` (Sonderfall):** `@set xs = [{name=…}]` → `@set xs = {{ [{"name":…}] }} /` (in der
   Session-Notiz als „cda7ef4bc" referenziert, aber nie committet — in dieser Session korrekt nachgeholt).

---

## Anhang C — Dedup-Mechanismus-Entscheid (Workstream B)

Mechanismus = **`@include`-Fragment** (Default des Plans). Smoke bestätigte: Fragment rendert inline, kein
Header-Leak, kein `lines=N-M` nötig. Cross-Directory-Dedup ist möglich, aber NUR über
`@include ${MDAI_LIBRARY_ROOT}/…` (repo-root-relativ, via MCP) — die dokument-relative `../../`-Form scheitert am
Parser (`..` verboten) und am Security-Jail (`docDir`). Cluster 1 (6-Anker-Liste) ist als canonical Fragment
`core/_fragments/lean-context-anchors.md` zentralisiert; Cluster 2 (`mode="full"`) + Cluster 3 (Anti-Pattern-Quelle)
via Drift-Tracking-Kommentare auf `core/lean-context.md`.

---

## Was in v0.1.3 umgesetzt wurde

- §1/§2/§3/§4/§5 als P0 + A migriert/verifiziert (file-utils, startup-check, spec-reviewer, write-spec).
- §6–§12 re-validiert; relevante Konventionen in `core/hard-rules.md` + `core/mcp-markdownai.md` dokumentiert.
- Content-Dedup (Cluster 1/2/3) umgesetzt; `spec-directive-conventions.md` auf v2 migriert (inkl. korrigierter
  Datums-Guidance).
- Offen (out-of-scope, getrackt): `body.mdai.md:185` `@set`-Pipe-Source (§2-B).
