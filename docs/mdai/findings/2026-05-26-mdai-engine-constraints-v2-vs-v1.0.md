---
title: mdai Engine-Constraints v2 — re-validated against markdownai v1.0.0
date: 2026-05-26
status: ready-for-review
authors: claude
supersedes: docs/mdai/findings/2026-05-25-mdai-v0.1.1-engine-constraints.md
related_audits:
  - docs/mdai/audits/2026-05-26-mdai-library-root-audit.md
  - docs/mdai/audits/2026-05-26-mdai-v1.0-adoption-audit.md
related_specs:
  - docs/mdai/specs/2026-05-26-mdai-v0.1.2-refit-v1.0-audit-design.mdai.md
related_plans:
  - docs/mdai/plans/2026-05-26-mdai-v0.1.2-part-b-findings-v2-and-dedup.md
sourced_from:
  - ctx_session [t0.2-§1] .. [t0.2-§12]
---

# mdai Engine-Constraints v2 — markdownai v1.0.0

Dieses Dokument supersedes `2026-05-25-mdai-v0.1.1-engine-constraints.md` (v1). Pro Finding wird der v1.0-Status
festgehalten, der empfohlene Workaround unter v1.0 angegeben, und der Migration-Pfad für bestehende Library-Files
markiert.

## Status-Übersicht

| §  | Topic                                       | v1.0-Status                            | Migration |
| --- | --- | --- | --- |
|----|---------------------------------------------|----------------------------------------|-----------|
| 1  | Closer-Syntax                               | still-valid                            | no        |
| 2  | @set+@foreach+{{var}} in @define            | still-valid                            | yes       |
| 3  | {{var}} in file.containsLine in @define     | still-valid                            | yes       |
| 4  | file.containsLine Cross-CWD                 | still-valid                            | no        |
| 5  | @set var="{{@date}}" Parser-Bug             | still-valid                            | yes       |
| 6  | @render-template from/to/force              | still-valid                            | no        |
| 7  | ${MDAI_LIBRARY_ROOT} nicht expandiert       | still-valid                            | optional  |
| 8  | Frontmatter leakt in mode:include           | still-valid                            | no        |
| 9  | write_enabled Pflicht                       | still-valid-with-new-mechanism         | optional  |
| 10 | npm-prefix PATH                             | still-valid                            | no        |
| 11 | Modus A vs B (Prosa vs @call)               | still-valid                            | no        |
| 12 | Bootstrap-Macros via global scope           | still-valid                            | no        |

---

## §1 — Closer-Syntax (`@end` für alle Block-Directives)

**Symptom (v0.1.1):** `@foreach`-, `@define`- und `@render-template`-Blöcke werden ohne den korrekten Closer
vom Parser nicht geschlossen und erzeugen `ParseError: unexpected EOF` oder stille Fehlinterpretation des
nachfolgenden Inhalts.

**Korrektur aus Part-A T2:** Closer ist universell `@end` — NICHT `@endforeach` / `@endswitch` / `@endif` wie
v1-Doc-Symptom-Text vermutet. Source-Evidence: `markdownai/packages/parser/src/parser-blocks.ts:113`
(`walkBody(state, 'end')`). Im v1.0-Quellcode bestätigt: `foreach.ts:23` `closeTag='end'`; `if.ts:6`
`closeTag='endif'`; Switch verwendet `@endswitch` (parser.ts:111). Das v0.1.1-Doc §1 Workaround-Text war damit
für `@foreach` falsch — korrekter Closer ist `@end`, nicht `@endforeach`.

**v1.0-Status:** still-valid (evidence: parser-blocks.ts:113 `walkBody(state,'end')`; foreach.ts:23
`closeTag='end'`; if.ts:6 `closeTag='endif'`; switch closer=`@endswitch` per parser.ts:111; all closers
confirmed in v1.0 source, no implicit block-end by blank line).

**Empfohlener Workaround in v1.0:** Strikte Closer-Disziplin einhalten: `@define … @end`,
`@foreach … @end`, `@switch … @endswitch`, `@if … @endif`, `@render-template`-Block mit `@end`.
Kein implizites Block-Ende durch Leerzeile oder Einrückung.

**Migration-Pfad für Library-Files:** no — bestehende Library-Files, die bereits `@end` als Closer verwenden,
sind korrekt. Files, die fälschlicherweise `@endforeach` verwenden (per v0.1.1-Workaround-Text), müssen auf
`@end` korrigiert werden.

---

## §2 — `@set`+`@foreach`+`{{var}}` inside `@define` via `call_macro`

**Symptom (v0.1.1):** `@set`-Variablen, die innerhalb eines `@define`-Blocks gesetzt werden, sind in
nachgelagerten `@foreach`-Iterationen via `{{ var }}`-Interpolation beim Aufruf über
`mcp__markdownai__call_macro` nicht erreichbar — der Engine-Scope isoliert `@set`-Bindings in der falschen
Layer beim Macro-Dispatch.

**HINWEIS aus Part-A T2:** Finding hat **2 distinkte Failure-Modes**, beide dokumentiert:

- **(A)** `@set items = ["a","b"]` + `@foreach item in items` → Output `Item: items` (literal var name, keine
  Iteration). `@set`-list-literal NICHT als `@foreach`-source nutzbar. Repro: `.tmp-bt/s2.md`.
- **(B)** Engine-Error `"@set" cannot be used as a pipe source` in mdai-brainstorm `write-outputs`-Phase
  (Cross-Ref zu T0.1 Smoke-3 Variante B finding — separater Failure-Mode von (A)).

**v1.0-Status:** still-valid (evidence: call_macro repro `.tmp-bt/s2.md`: `@set items=["a","b"]` +
`@foreach item in items` → Output `"Item: items"` (literal var name, keine Iteration); T0.1 side-finding:
write-outputs Phase emittiert `"@set cannot be used as a pipe source"` error — zwei distinkte Failure-Modes
bestätigt in v1.0).

**Empfohlener Workaround in v1.0:** `@set`-Listenliterale nicht als `@foreach`-Source verwenden — stattdessen
`@list`-Direktive oder hardkodierte kommagetrennte Literale als `@foreach`-Source einsetzen. Komplexe
Objekt-Listen (`@set checks=[{…}]`) mit `@switch check.type` innerhalb `@foreach` als etabliertes Pattern
verwenden (library-spec-audit). Jedes `@set`+`@foreach`-Pattern vor Deployment testen.

**Migration-Pfad für Library-Files:** yes — bestehende Macros, die `@set`-Listenliterale als `@foreach`-Source
verwenden, müssen auf `@list`-Direktive oder hardkodierte Literale umgestellt werden. Betrifft v0.1.3
Macro-Author-Guidance.

---

## §3 — `{{var}}` in `file.containsLine` inside `@define`

**Symptom (v0.1.1):** Beim Aufruf eines `@define`-Macros via `call_macro` wird `{{ spec_path }}` innerhalb
des `file.containsLine`-Pfad-Arguments nicht interpoliert — die Engine übergibt den Literal-String
`{{spec_path}}` an das Filesystem-Predicate, das damit keine Datei findet und immer `false` zurückgibt.

**v1.0-Status:** still-valid (evidence: call_macro repro `.tmp-bt/s3.md`: `@set path="/…/s3.md"` +
`@if file.containsLine "{{ path }}" "test_macro"` → warning `"Unresolvable expression: file.containsLine
\"\"…\"\" \"test_macro\""` (double-quoted path, kein match). `{{var}}` in `file.containsLine`-First-Arg nicht
interpoliert in `@define`-Scope. NOTE: T0.1 `detect_mai_hook_version` verwendete literal path (kein `{{var}}`),
das ist ein separater §4-class Issue).

**Empfohlener Workaround in v1.0:** Keine `{{var}}`-Interpolation als `file.containsLine`-Pfad-Argument in
v0.1.3-Macros verwenden — literal Pfad-Strings übergeben oder path-basierte Checks an den aufrufenden Agent
delegieren.

**Migration-Pfad für Library-Files:** yes — Macros, die `{{var}}` als `file.containsLine`-Pfad-Arg verwenden,
müssen auf literal Strings oder Agent-seitige Checks umgestellt werden. Betrifft v0.1.3 Macro-Author-Guidance.

---

## §4 — `file.containsLine` Cross-CWD blockiert

**Symptom (v0.1.1):** `@if file.containsLine "~/.markdownai/hooks/preToolUse.mjs" "isMarkdownAIDocument"`
schlägt mit einem Zugriffsversagungs-Fehler fehl — die Engine-Sandbox erlaubt `file.*`-Predicates nur für
Pfade innerhalb des Projekt-CWD.

**v1.0-Status:** still-valid (evidence: `filesystem.ts:30` `checkFilePath` blockiert `isAbsolute(filePath)`
bedingungslos — noch vor dem `allowed_data_paths`-Check. Repro `.tmp-bt/s4.md` mit `/home/**` in
`allowed_data_paths` liefert trotzdem `"Unresolvable expression: file.containsLine"` warning. Absolut-Pfad wird
auf Source-Read-Layer geblockt, nicht auf Data-Path-Layer. `security.json`-Revert verifiziert: diff leer).

**Empfohlener Workaround in v1.0:** Absolut-Pfade außerhalb CWD aus `@if file.*`-Predicates heraushalten —
unabhängig von `security.json`-Config; dies ist ein hartes Engine-Constraint. Für Home-Dir-Checks stattdessen
`ctx_shell` mit dediziertem Shell-Command verwenden, oder den Check als Prosa-Instruktion an den aufrufenden
Agent delegieren. §4 wird als upstream markdownai-Engine-Issue getrackt (keine config-basierte Workaround-Option).

**Migration-Pfad für Library-Files:** no — keine `file.*`-Predicates mit Absolut-Pfaden in bestehenden
Library-Files vorhanden (bereits via v0.1.1-Workaround vermieden).

---

## §5 — Parser-Bug: `@set var = "{{@date}}"` + `@if file.exists "{{var}}"` at `@define` top-level

**Symptom (v0.1.1):** Ein `@set`-Statement mit `@date`-Interpolation
(`@set today = "{{ @date format='YYYY-MM-DD' }}"`) gefolgt von `@if file.exists "{{ today }}-..."` direkt im
Top-Level eines `@define`-Blocks erzeugt einen `ParseError` oder falsche Auflösung — die Engine interpoliert
`@date` nicht innerhalb von `@set`-String-Literalen zum Zeitpunkt des Macro-Dispatches.

**v1.0-Status:** still-valid (evidence: call_macro repro `.tmp-bt/s5.md`: `@set today="{{ @date format='YYYY-MM-DD' }}"` +
`@if file.exists "{{ today }}-foo.txt"` → warnings `"Unresolvable expression: @date format='YYYY-MM-DD'"` und
`"Unresolvable expression: file.exists \"\"\"…\"-foo.txt\""` — `@date` nicht innerhalb `@set`-String-Literal
interpoliert, und das kaskadierte `{{ today }}` nicht in `file.exists`-Arg aufgelöst. Cross-ref T0.1:
`"@set cannot be used as a pipe source"` ist ein distinkt zusätzlicher Failure-Mode).

**Empfohlener Workaround in v1.0:** `@date`-Direktiven nicht in `@set`-String-Literale einbetten. Stattdessen
`{{ @date format='YYYY-MM-DD' }}` direkt am Verwendungsort einsetzen (z.B. im `@render-template`-Block oder als
direktes Argument), ohne Zwischenbindung via `@set`.

**Migration-Pfad für Library-Files:** yes — Macros, die `@date`-Interpolation in `@set`-Strings verwenden,
müssen auf direkte Verwendung am Point-of-Use umgestellt werden. Betrifft v0.1.3 Macro-Author-Guidance.

---

## §6 — `@render-template` korrekt: `from="…" to="…" [force]` Block + `@end`

**Symptom (v0.1.1):** Der ursprüngliche Plan beschrieb `@render-template` mit einem `output=`-Argument
(z.B. `@render-template <path> args={…} output="{{ report_path }}"`). Die tatsächliche Engine-API erfordert
stattdessen `from="…" to="…"` (Quell- und Zielpfad als Named-Args) in einem Block-Format, abgeschlossen mit
`@end`. Das `output=`-Argument existiert nicht.

**v1.0-Status:** still-valid (evidence: `render-template.ts:4-7` Kommentar dokumentiert `"from=… to=… [force]"`
Block + `@end`; Parser bestätigt `from`/`to` als Named-Args (lines 22-23), `force` als positional flag
(line 26); kein `output=`-Arg im Source. `render-template.ts:19` `closeTag='end'`).

**Empfohlener Workaround in v1.0:** Keine Änderung nötig — `@render-template from=… to=… force @end` ist die
korrekte Form. Plan-Autoren dürfen `output=`-Syntax nicht verwenden.

**Migration-Pfad für Library-Files:** no — bestehende Library-Files verwenden bereits die korrekte
`from`/`to`/`force`-Form (via v0.1.1-Korrektur).

---

## §7 — `${MDAI_LIBRARY_ROOT}` nicht expandiert bei standalone `resolve_phase`

**Symptom (v0.1.1):** Beim Aufruf von `mcp__markdownai__resolve_phase` ohne explizit gesetztes
`MDAI_LIBRARY_ROOT` im Subagent-Environment wird `${MDAI_LIBRARY_ROOT}` nicht zu
`/home/tholo/Scripts/lean-ctx/mdai` expandiert — `@include ${MDAI_LIBRARY_ROOT}/...`-Statements resultieren in
ENOENT.

**Evidence-Hinweis:** Kommt aus Part-A T1 Smokes `[t0.1-A-smoke2]` (ENOENT) + `[t0.1-B-smoke2]` (PASS). MCP
Tool-Schema hat keinen `env=`- oder `cwd=`-Parameter; Engine erbt `MDAI_LIBRARY_ROOT` ausschließlich aus dem
Spawn-Process des MCP-Servers.

**v1.0-Status:** still-valid (evidence: audit-doc + `[t0.1-A-smoke2]` (ENOENT auf `hard-rules.md` +
`tool-quick-ref.md`) + `[t0.1-B-smoke2]` (PASS); MCP-Tool-Schema hat keinen `env=`- oder `cwd=`-Parameter;
Engine erbt `MDAI_LIBRARY_ROOT` ausschließlich vom Spawn-Prozess).

**Empfohlener Workaround in v1.0:** `cwd`-Parameter in allen MCP-Calls explizit auf das Repo-Root setzen
(`cwd="/home/tholo/Scripts/lean-ctx"`). Explizite Warnung in `mdai/core/hard-rules.md` hinzufügen: MCP-Server
erbt `MDAI_LIBRARY_ROOT` vom Spawn-Prozess — isolierte CI/Subagent-Spawns müssen die Env-Var explizit setzen,
bevor Claude Code oder der MCP-Server gestartet wird. Kein Code-Change nötig.

**Migration-Pfad für Library-Files:** optional — doc-only Update in `mdai/core/hard-rules.md` empfohlen;
bestehende Library-Files müssen nicht geändert werden (Variante B Status-quo beibehalten per User-Decision T0.3).

---

## §8 — Frontmatter leakt als Text in `mode:include` bei standalone `read_file`

**Symptom (v0.1.1):** Dateien mit `mode: include` + YAML-Frontmatter-Block (`---`) zeigen beim Aufruf via
`mcp__markdownai__read_file` den Frontmatter-Block als sichtbaren Plaintext im Output — der Parser rendert
`---`-Delimiters + Frontmatter-Felder als Fließtext statt sie zu parsen.

**v1.0-Status:** still-valid (evidence: `parser.ts:249+273` frontmatter-Zeilen werden als `makeMarkdown`-Nodes
in `frontmatterNodes[]` gepusht, werden in `nodes:[...frontmatterNodes,...bodyNodes]` zurückgegeben;
`executeInclude`/`walkNodesFn` verarbeitet alle Nodes → Frontmatter rendert als Text; kein Stripping-Pfad
existiert in `read_file.ts` oder `executeInclude`).

**Empfohlener Workaround in v1.0:** Konvention aus `hard-rules.md` (keine Frontmatter in Include-Files) bleibt
die einzige Mitigation. Zwei akzeptierte Strategien: (a) Include-Files ohne Frontmatter schreiben — kein
Leakage; (b) Frontmatter bewusst akzeptieren, wenn `lib_version`/`mode`-Tracking gewünscht ist (dann ist
Leakage-Text im `read_file`-Output für AI-Consumer harmlos). Keine Mischstrategie — pro Library-Pack einheitlich
entscheiden.

**Migration-Pfad für Library-Files:** no — bestehende Include-Files folgen bereits der No-Frontmatter-Konvention
(oder haben Frontmatter bewusst akzeptiert).

---

## §9 — `filesystem.write_enabled: true` in `~/.markdownai/security.json` Pflicht für `@mkdir`/`@render-template`

**Symptom (v0.1.1):** `@mkdir`- und `@render-template`-Direktiven schlagen mit einem Security-Policy-Fehler
fehl (`Error: filesystem writes are disabled`) wenn `filesystem.write_enabled` in
`~/.markdownai/security.json` nicht auf `true` gesetzt ist.

**HINWEIS aus Part-A T2:** Status ist `still-valid-with-new-mechanism` (NICHT `workaround-deprecated`).
`write_enabled` bleibt primary gate (`engine.ts:118-119`, `write-ops.ts:25`), v1.0 ergänzt `write_root` +
`allowed_write_paths` als **granulare Controls daneben** — kein Replacement. Empfehlung: beide Layer
(legacy `write_enabled` + neue granulare Controls) als Cookbook-Eintrag dokumentieren.

**v1.0-Status:** still-valid-with-new-mechanism (evidence: `engine.ts:118-119`
`writeEnabled = fsConfig?.write_enabled ?? false` + `write-ops.ts:25` `ensureWriteEnabled` prüft dieses
Gate; v1.0 ergänzt `write_root` (default: cwd) und `allowed_write_paths` als granulare Controls neben
`write_enabled` — `write_enabled=false` blockt weiterhin alle Writes).

**Empfohlener Workaround in v1.0:** `~/.markdownai/security.json` muss `{ "filesystem": { "write_enabled": true } }`
enthalten. Dies ist ein User-seitiger One-Time-Setup. Für neue Environments (CI/CD, neuer Entwickler) explizit
dokumentieren. Optional: `write_root` + `allowed_write_paths` für path-scoped Write-Control einsetzen
(hardening für CI-Environments).

**Migration-Pfad für Library-Files:** optional — bestehende `security.json`-Konfigurationen behalten
`write_enabled: true`; zusätzlich können `write_root` + `allowed_write_paths` als optionales Hardening
konfiguriert werden.

---

## §10 — `npm config prefix` + PATH-Export `~/.npm-global` (User-Env-Side-Effect)

**Symptom (v0.1.1):** Nach `npm link` ist das `mai`-Binary nur in neuen Shell-Sessions auf `PATH`.

**v1.0-Status:** still-valid (User-Env-Side-Effect, kein Engine-Code-Pfad).

**Empfohlener Workaround in v1.0:** `~/.zshrc`-Export `export PATH=/home/tholo/.npm-global/bin:$PATH` (one-time),
oder absoluter Pfad in Subagent-Calls.

**Migration-Pfad für Library-Files:** no — doc-only Hinweis in `mdai/core/hard-rules.md` empfohlen für
neue Devs/CI.

---

## §11 — Design: Prosa-Pointer `mcp__markdownai__call_macro` vs. `@call` (Modus A vs. B)

**Symptom (v0.1.1):** Unklare Modus-Wahl für Macro-Aufrufe aus anderen Files.

**v1.0-Status:** still-valid (Konvention, kein Engine-Constraint).

**Empfohlener Workaround in v1.0:** Modus A (Prosa-Pointer via `call_macro`) für Library-Packs außerhalb von
`body.mdai.md`. Modus B (`@import` + `@call`) für Bootstrap-Macros in `body.mdai.md`. Pro Phase nicht mischen.

**Migration-Pfad für Library-Files:** no.

---

## §12 — Bootstrap-Macros resolved via per-call wrapper synthesis (NICHT global scope)

**Symptom (v0.1.1):** `@call mdai_bootstrap()` und `@call detect_mai_hook_version()` in `body.mdai.md`
funktionieren ohne expliziten `@import ${MDAI_LIBRARY_ROOT}/core/startup-check.md` im selben Dokument — die
Engine löst Bootstrap-Macros via globalem Macro-Scope auf, der beim Session-Start aus `source_root`-Config
befüllt wird.

**KORREKTUR aus Part-A T2:** Original-Framing „global scope" ist **falsch**. Mechanismus ist
`markdownai/packages/mcp/src/tools/call_macro.ts:71` per-invocation wrapper-synthesis:
`@markdownai\n@import ./<filePath>\n@call <macro>(args)`. Deshalb braucht der Konsument kein explizites
`@import startup-check.md` — der Wrapper wird pro `call_macro`-Aufruf injiziert. By-Design, nicht
implicit-global. Es gibt keinen globalen Macro-Registry oder `source_root`-getriebenen Scope (0 Matches für
`globalScope`/`global_scope`/`globalMacro` in `engine.ts`/`macros.ts`). Titel und Body des v0.1.1-Docs
entsprechend re-gerahmt.

**v1.0-Status:** still-valid (evidence: `call_macro.ts:71` synthesizes
`"@markdownai\n@import ./${filePath}\n@call macro(args)"` dynamisch; kein globaler Macro-Registry oder
`source_root`-getriebener Scope in `engine.ts` oder `macros.ts`; Macros lösen sich auf, weil `call_macro`
die Target-File immer explizit importiert).

**Empfohlener Workaround in v1.0:** Kein Workaround nötig — Verhalten ist by design von `call_macro` (Modus A):
per-invocation Wrapper-Synthesis. Kein expliziter `@import` in aufrufenden Files erforderlich. Mechanismus in
`mdai/core/hard-rules.md` dokumentieren, damit zukünftige Maintainer verstehen, warum `@import` in
Library-Pack-Call-Sites fehlt.

**Migration-Pfad für Library-Files:** no — keine Änderung an Library-Files nötig; nur Doc-Update in
`hard-rules.md`.

---

## Was in v0.1.3 fließt

**Cleanup-Status (T6 — 2026-05-26):**
- `~/.markdownai/hooks/preToolUse.mjs.pre-v1.0` — deleted.
- `~/.claude/settings.json.pre-v1.0` — deleted.
- `markdownai/` tag `pre-v1.0-bump` — deleted.
- `~/.markdownai/security.json.pre-audit` — deleted.

Pointer auf `docs/mdai/audits/2026-05-26-mdai-v1.0-adoption-audit.md` Sektion 3 (Action-Items).

Aus User-Decision T0.3 (`v0.1.3_action_items: [§1..§12 all-12]`): alle 12 Findings werden im v0.1.3-Plan
adressiert — maximale Coverage.

- **§1** — doc-only: Workaround-Text in v0.1.1-Doc korrigieren (`@endforeach` → `@end`); Closer-Disziplin in
  `mdai/core/hard-rules.md` dokumentieren.
- **§2** — macro-author guidance: `@set`-Listenliterale als `@foreach`-Source verbieten; `@list`-Direktive
  oder Literal-Comma-Lists empfehlen; bestehende Macros migrieren.
- **§3** — macro-author guidance: `{{var}}`-Interpolation als `file.containsLine`-Pfad-Arg verbieten; auf
  literal Strings oder Agent-seitige Delegation umstellen; bestehende Macros migrieren.
- **§4** — upstream Engine-Issue tracken: absolut-Pfade in `file.*`-Predicates sind hartes Constraint;
  `ctx_shell`-Delegation dokumentieren; kein Code-Change im Repo möglich.
- **§5** — macro-author guidance: `@date` in `@set`-String-Literalen verbieten; direkte Verwendung am
  Point-of-Use dokumentieren; bestehende Macros migrieren.
- **§6** — doc-only: `@render-template from=… to=… force @end`-Form in `hard-rules.md` kanonisch festhalten;
  `output=`-Syntax als invalid markieren.
- **§7** — optional doc-only: Warnung in `hard-rules.md`: MCP-Server erbt `MDAI_LIBRARY_ROOT` vom
  Spawn-Prozess; Subagents müssen `cwd` immer mitgeben.
- **§8** — doc-only: No-Frontmatter-Konvention für Include-Files in `hard-rules.md` bestätigen; beide
  Strategien (a/b) dokumentieren.
- **§9** — optional hardening: `write_root` + `allowed_write_paths` als Cookbook-Eintrag neben
  `write_enabled: true` dokumentieren.
- **§10** — doc-only: PATH-Export-Hinweis in `hard-rules.md` für neue Devs/CI.
- **§11** — doc-only: Modus-A-vs-B-Regel in `hard-rules.md` kanonisch festhalten.
- **§12** — doc-only: per-call Wrapper-Synthesis-Mechanismus in `hard-rules.md` erklären; „kein `@import`
  in Library-Pack-Call-Sites" als Expected Behaviour dokumentieren.
