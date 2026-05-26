# mdai-Skill v0.1.1 Refactor (Wave 3–5 + Lazy-Load + call_macro) — Implementation Plan (Part 2 / 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Subagent-Modell-Politik (festgehalten per User-Anweisung):**
> - Default-Modell für ALLE Subagent-Dispatches: **`sonnet`** (Anthropic Sonnet).
> - Ausnahme: Phase 4 (Wave-3–5 Audit) DARF bei Bedarf auf **`opus`** hochgezogen werden
>   (höhere Reasoning-Kapazität für systematischen Cross-File-Audit).
>   Default bleibt sonnet — Opus nur, wenn Sonnet einen konkreten Reasoning-Schritt nicht löst.
> - Beim Dispatch über `Agent`/`TaskCreate`: `model: sonnet` (bzw. `opus` für P4 nach Bedarf) immer explizit setzen.
> - Tool-Allowlist pro Subagent: Serena-Tools + lean-ctx (`ctx_*`) + JetBrains-MCP + `mcp__markdownai__*`. Kein `Bash` außer wo explizit erlaubt (z.B. `git`, `npm`, `node`).

**Goal:** mdai-brainstorm-Skill auf v0.1.1 refactoren: (a) Wave-3–5-Direktiven-Audit aller mdai-Files mit Adoption der empfohlenen Patches, (b) Lazy-Load-Refactor (L1/L2/L3) mit `call_macro` als MCP-first-Pattern für L2, (c) zwei neue Library-Packs (`lean_context_audit` + `library_spec_audit`) als `call_macro`-Endpoints.

**Architecture:** Drei sequentielle Phasen: P4 Audit + Adoption pro File, P5 Lazy-Load-Refactor (L1 `@include`-only, L2 `call_macro` MCP-first, L3 `@include`-only), P6 Library-Pack-Distribution mit zwei neuen Audit-Packs in `mdai/core/`. Phasen mit hartem Gate. Spec §6/§7/§8.

**Tech Stack:** markdownai v1.0 (vorausgesetzt aus Part 1), MCP-Tools `mcp__markdownai__*` (`call_macro`/`resolve_phase`/`list_phases`/`read_file`), Serena (symbolic edits), JetBrains (`reformat_file`), `ctx_*` (lean-ctx).

**Hard Dependency:** Part 1 (`docs/mdai/plans/2026-05-25-mdai-v1.0-engine-adoption.md`) MUSS komplett abgeschlossen sein. Konkret:
- `mai --version` liefert `1.0.0`
- `mcp__markdownai__list_phases(file="mdai/skills/mdai-brainstorm/body.mdai.md", cwd="<repo>")` liefert 5 Phasen ohne ENOENT
- Alle `@include mdai/...` / `@import mdai/...` in `mdai/` migriert auf `${MDAI_LIBRARY_ROOT}/...`
- `~/.markdownai/hooks/preToolUse.mjs` enthält `isMarkdownAIDocument`-Marker
- `MDAI_LIBRARY_ROOT` env-var ist gesetzt

**Spec:** `docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md` — Abschnitte §6, §7, §8, §9, §10.

**Predecessor-Plan:** `docs/mdai/plans/2026-05-25-mdai-v1.0-engine-adoption.md`

---

## Pre-Flight-Check (Part 1 Pre-Conditions)

### Task 0: Hard-Dependency-Gate verifizieren

- [ ] **Step 1: Part-1-End-State prüfen**

```bash
ctx_shell "mai --version"
ctx_search "@include mdai/[^$]" mdai/
ctx_search "@import mdai/[^$]" mdai/
ctx_search "isMarkdownAIDocument" ~/.markdownai/hooks/preToolUse.mjs
ctx_shell "env | grep MDAI_LIBRARY_ROOT"
```

Erwartet:
- `mai --version` = `1.0.0`
- `@include mdai/[^$]` = 0 matches
- `@import mdai/[^$]` = 0 matches
- `isMarkdownAIDocument` ≥ 1 match
- `MDAI_LIBRARY_ROOT=/home/tholo/Scripts/lean-ctx/mdai`

Bei Fail: STOP, zurück zu Part 1 und das fehlende Gate fixen.

- [ ] **Step 2: list_phases-Smoke**

```
mcp__markdownai__list_phases(
  file="mdai/skills/mdai-brainstorm/body.mdai.md",
  cwd="<repo>"
)
```

Erwartet: `phases: [...]` mit ≥ 4 Phasen-Namen, keine ENOENT.

> **Achtung — blockiert durch Carry-Over-Backlog B1 (siehe Sektion unten):** `body.mdai.md:290` löst aktuell `ParseError: @call requires a macro name` aus. Step 2 wird FAIL liefern bis B1 behoben ist. Workaround für reines MCP-Smoke (ohne body.mdai.md): `mcp__markdownai__list_phases(file="markdownai/MDs/tests/test-phase-amnesia.md")` → 3 Phasen (red/blue/green) — empirisch verifiziert in Part 1 Phase-1 End-Gate.

---

## Backlog aus Part 1 (Carry-Over zu Part 2)

Persistierte Findings + Decisions aus dem Part-1-Run (siehe `ctx_session action="finding"` Einträge mit Prefix `[mdai-v1.0-part1-*]`, `[mdai-p1c]`, `[mdai-p1b-*]`, `[mdai-p2-task9]`, `[mdai-p3-sweep]`).

### B1: `body.mdai.md:290` — v1.0-Engine `@call`-Parse-Fehler

**Status:** OFFEN — blockiert Part-2-Pre-Flight Step 2 und alle MCP-Calls gegen `body.mdai.md`.

**Symptom:**
```
ParseError: [.../mdai/skills/mdai-brainstorm/body.mdai.md:290] @call requires a macro name
```

**Reproduktion (drei unabhängige Aufruf-Pfade scheitern identisch):**
```
mcp__markdownai__list_phases(file="mdai/skills/mdai-brainstorm/body.mdai.md")
mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="pre-context")
env MDAI_LIBRARY_ROOT=... node markdownai/packages/core/dist/cli.js render mdai/skills/mdai-brainstorm/body.mdai.md
```

**Vermutete Ursache:** Zeile 290 lautet (nach `${MDAI_LIBRARY_ROOT}`-Migration):
```
@call render_spec(slug={{ slug }}, target={{ render_target | default("none") }})
```
Die v1.0-Engine ist strikter beim `@call`-Parsing: der `default("none")`-Pipe-Filter innerhalb des `{{ ... }}`-Argument-Ausdrucks (oder die kombinierte Named-Args + Pipe-Filter-Syntax) wird nicht erkannt. v0.0.24 hat diese Stelle akzeptiert (Test 1 in Part 1 lieferte 5 Phasen zurück).

**Fix-Optionen (Audit + Entscheidung in Phase 5 / `body.mdai.md`-Rewrite):**
1. Syntax umschreiben: `default`-Filter aus Named-Arg entfernen, default extern via `@set` oder `@if` vorberechnen → `@set render_target_resolved = render_target | default("none")`, dann `@call render_spec(target={{ render_target_resolved }})`.
2. Engine-Lenience: Upstream-PR zur v1.0-Engine, der die Kombination Named-Arg + Pipe-Filter weiter zulässt.
3. Komplett auf positional args zurückwechseln (verliert Klarheit).

**Adoptions-Anweisung:** In Phase 4 (Wave-3–5-Audit) explizit als Audit-Punkt aufnehmen + in Phase 5 (Lazy-Load-Refactor) als Teil des `body.mdai.md`-Rewrites auflösen. Bevorzugte Variante: Option 1 (`@set`-Pre-compute) — keine Engine-Änderung nötig, lokale Cause-Fix.

### B2: respondTool-Wrapper — upstream-PR-Vorbereitung

**Status:** OFFEN — Decision `[mdai-p1c]` ist final (Branch B), Wrapper bleibt permanent aktiv.

**Empirik (Part 1 Task 2/3):**
- WITH wrapper + 0.0.24: `list_phases` → JSON response (PASS, 5 Phasen)
- WITHOUT wrapper + v1.0: `get_env(HOME)` → silent dropped Success-Response (FAIL)
- Schlussfolgerung: MCP-Client droppt raw-object Success-Responses ohne `content[]`+`structuredContent`-Envelope.

**Action-Item:** Upstream-Issue/PR im `markdownai`-Repo mit der `respondTool`-Wrapper-Patch + obigen Empirik-Daten. Bis dahin lebt der Patch lokal als Commit `3bdd5a7` auf `feat-mdai`-Branch im `markdownai`-Subdir (10 call-sites in `markdownai/packages/mcp/src/server.ts`).

**Snapshot-Tag:** `markdownai@pre-v1.0-bump` zeigt auf `0590beb` (Pre-merge-State mit ursprünglichem Patch).

### B3: PATH-Export für `~/.npm-global/bin` in `~/.zshrc` — User-Env-Side-Effect

**Status:** DOKUMENTIERT — kein action item, nur Awareness.

**Befund:** Während Part-1 Task 5 (`npm link`) hat der Implementer-Subagent `npm config set prefix /home/tholo/.npm-global` gesetzt und `export PATH="$HOME/.npm-global/bin:$PATH"` an `~/.zshrc` angehängt, weil das System-Prefix `/usr` ohne `sudo` nicht beschreibbar war.

**Konsequenz:** Aktive Shell-Sessions vor diesem Append haben `mai` nicht auf PATH. Verifikation `which mai` schlägt in solchen Sessions fehl, obwohl der Binary unter `/home/tholo/.npm-global/bin/mai` korrekt existiert (Symlink → `markdownai/packages/core/dist/cli.js`). Workaround: `source ~/.zshrc` oder neues Terminal.

**Für Part-2-Tasks:** Wenn ein Task `mai` direkt aufruft, entweder `env PATH=/home/tholo/.npm-global/bin:$PATH mai ...` oder den absoluten Pfad `/home/tholo/.npm-global/bin/mai` verwenden.

### B4: `~/.markdownai/hooks/preToolUse.mjs.pre-v1.0` Backup-Datei

**Status:** DEFER-CLEANUP — entfernen sobald v1.0-Hook über mehrere Session-Restarts stabil ist.

**Befund:** Vor `mai init --client claude-code` in Part-1 Task 8 wurde der alte v0.x-Hook (`preToolUse.mjs`, 1017 B) defensiv nach `preToolUse.mjs.pre-v1.0` kopiert. Plan-Spec sah keinen expliziten Backup vor, aber analog zum `~/.claude/settings.json.pre-v1.0` (Task 0) wurde es eingebaut.

**Cleanup-Trigger:** Nach mindestens 1 Woche / mehreren Claude-Code-Restarts ohne Hook-Probleme. Plan-2 Task 20 (Plan-Abschluss) entsprechend erweitern: Step zu "preToolUse.mjs.pre-v1.0 entfernen?" mit Default `behalten als Safety-Net` (analog zum `pre-v1.0-bump`-Tag).

### B5: T9 Step 2 (`sessionStart.mjs` silent-test) — DEFERRED

**Status:** DEFER bis manuelle Verifikation in neuer Claude-Code-Session.

**Befund:** Part-1 Task 9 Step 1 (PreToolUse-Blockierung) PASS empirisch (Read auf `.mdai.md` wird mit voller REDIRECT_MESSAGE geblockt). Step 2 (`sessionStart`-Hook silent: kein stderr-Noise, kein `additionalContext`-Inject ohne `CLAUDE-MarkdownAI.md`) konnte nicht in derselben Session getestet werden — der Hook feuert nur beim SessionStart-Event.

**Action (vor Part-2-Pre-Flight ODER beim ersten Part-2-Session-Start):**
- Kein stderr im Session-Start-Log
- Kein `additionalContext`-Inject in `CLAUDE.md`-Kette (weil `CLAUDE-MarkdownAI.md` nicht existiert)
- Hook exits 0 silent

Bei FAIL: Hook-Implementierung in `~/.markdownai/hooks/sessionStart.mjs` debuggen, evtl. via `mai init --update`.

### B6: MCP-Server-Restart-Disziplin nach `markdownai`-Builds

**Status:** PROZESS-NOTIZ — kein action item, nur Disziplin.

**Befund:** Der `markdownai`-MCP-Server (definiert in `.mcp.json` als `node markdownai/packages/mcp/dist/server.js`) wird bei Session-Start gespawnt und behält den `dist/server.js`-Code im Speicher. `npm run build` aktualisiert nur die `dist/`-Files — der laufende Server merkt davon nichts.

**Konsequenz für Part 2:** Jede Phase, die `markdownai/`-Code modifiziert + rebuilt, muss vor MCP-Verifikation entweder `/mcp reconnect markdownai` oder einen vollen Claude-Code-Session-Restart einplanen. Sonst sind alle nachfolgenden `mcp__markdownai__*`-Calls gegen veralteten Code.

**Pattern:** Subagents NIEMALS in MCP-Calls aufteilen, die einen Server-Restart erfordern (Controller-Only).

---

## File-Map (Touch-Inventar Part 2)

### Neu erstellte Dateien

| Pfad | Zweck |
| ---- | ----- |
| `mdai/core/lean-context-audit.md` | Library-Pack — `@define lean_context_audit(spec_path)` (6-Anchor-Sweep via `@foreach`) |
| `mdai/core/library-spec-audit.md` | Library-Pack — `@define library_spec_audit(spec_path)` (7 Checks via `@foreach`+`@switch`) |
| `mdai/skills/mdai-brainstorm/spec-directive-conventions.md` | L1-Include — 9-Use-Cases-Tabelle inkl. „v1.0 native equivalent"-Spalte |
| `mdai/skills/mdai-brainstorm/spec-self-review.md` | L2-Library-Pack — `@define spec_self_review(spec_path)` (5+1 Checks) |
| `mdai/skills/mdai-brainstorm/process-principles.md` | L3-Include — Process-Details + Key-Principles |
| `mdai/skills/mdai-brainstorm/templates/review-template.md` | `@render-template`-Source für `write_review_report` |
| `docs/mdai/audits/2026-05-25-wave-3-5-adoption-audit.md` | P4-Audit-Output |
| `docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.1-smoke.md` | Post-P6-Smoke-Artefakt |

### Modifizierte Dateien

| Pfad | Änderung |
| ---- | -------- |
| `mdai/core/startup-check.md` | `+ @define detect_mai_hook_version()`, `- @define detect_mdai_root()` (gestrichen), `detect_tooling` ggf. `@foreach`-refactor |
| `mdai/skills/mdai-brainstorm/body.mdai.md` | Phase-Refactor (Lazy-Load), `@call detect_mai_hook_version` statt `detect_mdai_root`, Process-Checklist phase-transitions, `@include` von L1/L3-Files, call_macro-Pointer im handoff |
| `mdai/skills/mdai-brainstorm/spec-reviewer.md` | Lean-Re-Shape (~45 Z, `mode: import-only`, `@define spec_reviewer_prompt(spec_path)`), call_macro-Pointer für audit + write_review_report |
| `mdai/skills/mdai-brainstorm/write-spec.md` | `+ @define write_review_report(...)` mit `@mkdir`+`@render-template`; lib_version 0.1.0 → 0.1.1 |
| Weitere `mdai/`-Files mit Shell-Workaround-Patterns | P4-Audit-Empfehlungen umgesetzt (laut Audit-Doc) |
| `docs/mdai/specs/2026-05-25-mdai-brainstorm-lazyload-and-namespace-resolver-design.mdai.md` | Frontmatter-Update: `status: superseded`, `superseded_by: ...`, `superseded_reason: ...` |

---

## Phase 4: Wave 3–5 Direktiven-Audit + Adoption

**Dispatch-Note:** Subagent für den Audit-Sweep darf `model: opus` nutzen (User-Anweisung: Sonnet default, Opus bei P4).

### Task 1: Audit-Sweep pro File

**Files:**
- Read: alle `mdai/core/*.md` (8 Files)
- Read: alle `mdai/skills/mdai-brainstorm/*.md` (6 Files)
- Create: `docs/mdai/audits/2026-05-25-wave-3-5-adoption-audit.md`

- [ ] **Step 1: Pro File 4-Suchen ausführen**

Audit-Methodik aus Spec §6.3 — für jede Datei aus dem Inventar:

```
ctx_search "@query|@call ctx_shell|bash|if.*then" <file>
ctx_search "frontmatter|---|@define|@constraint" <file>
ctx_search "for each|iterate|loop|repeat" <file>
ctx_read(path="<file>", mode="signatures")  # oder "map"
```

- [ ] **Step 2: Audit-Doc schreiben**

`docs/mdai/audits/2026-05-25-wave-3-5-adoption-audit.md` mit Adoption-Map-Format aus Spec §6.4 pro File:

```markdown
### <path>
Status: keep-as-is | adopt-minor | adopt-major | refactor

#### Adoptions
- @<directive>: <line/anchor>
  Before: <one-line current pattern>
  After:  <one-line new pattern>
  Benefit: <one-sentence why>

#### Skipped
- @<directive>: <reason>
```

Hotspot-Files (laut Spec §6.5) müssen mind. drin sein:
- `mdai/skills/mdai-brainstorm/write-spec.md`
- `mdai/skills/mdai-brainstorm/spec-reviewer.md`
- `mdai/skills/mdai-brainstorm/body.mdai.md`
- `mdai/core/startup-check.md`
- `mdai/core/lean-context-audit.md` (geplant in P6 — Adoption-Empfehlung antizipiert)
- `mdai/core/library-spec-audit.md` (geplant in P6)

- [ ] **Step 3: Decision-Gate — User reviewed Audit**

User-Review-Gate (exact wording):

> "Wave-3–5-Audit-Dokument geschrieben unter `docs/mdai/audits/2026-05-25-wave-3-5-adoption-audit.md`.
> Bitte review, welche Adoptions du übernehmen willst (alle / Teilmenge / skip) — danach
> setze ich die Implementierung um."

WARTEN auf User-Antwort. Anschließend Implementierung der adopted-items.

### Task 2: Adopted-Items implementieren — `mdai/core/` zuerst

**Files:**
- Modify: `mdai/core/startup-check.md`
- Modify: weitere `mdai/core/*.md` laut Audit

- [ ] **Step 1: `mdai/core/startup-check.md` adopten**

Nur falls Audit das markiert. Typische Adoptions:
- `detect_tooling`: `@foreach` über tool-list statt repeated `@if`s.
- `detect_mdai_root` löschen — wird in Task 11 durch `detect_mai_hook_version` ersetzt (P4 vorausgreifend OK).

Symbolic Edit via Serena:

```
mcp__serena__jet_brains_find_symbol(name_path="detect_tooling", relative_path="mdai/core/startup-check.md", include_body=true)
```

Dann `replace_symbol_body` ODER (falls Symbolic-Tools im Markdown nicht greifen) `ctx_edit` für gezielten Block.

- [ ] **Step 2: Pro adopted-Core-File**

Edit ausführen, dann:

```
mcp__markdownai__read_file(path="<file>", cwd="<repo>")
```

Erwartet: 0 warnings, output anchors stimmen.

- [ ] **Step 3: Commit core/-Adoptions**

```bash
ctx_shell "git status"
```

Pro modifizierter Datei `mcp__jetbrains__reformat_file` aufrufen, dann:

```bash
ctx_shell "git add mdai/core/"
ctx_shell "git commit -m 'refactor(mdai/core): adopt Wave 3-5 directives (P4 audit findings)'"
```

### Task 3: Adopted-Items implementieren — `mdai/skills/mdai-brainstorm/`

**Files:**
- Modify: `mdai/skills/mdai-brainstorm/write-spec.md`
- Modify: `mdai/skills/mdai-brainstorm/spec-reviewer.md`
- Modify: `mdai/skills/mdai-brainstorm/body.mdai.md`

- [ ] **Step 1: `write-spec.md` adopten**

Hotspot laut Spec §6.5: `@query ctx_shell "mkdir -p ... && cat > ..."`-Pattern in `write_spec` und (neu in P5) `write_review_report` durch `@mkdir` + `@render-template` ersetzen.

`write_review_report` wird vollständig in Task 9 geschrieben. P4 fokussiert auf das bestehende `write_spec`-Pattern:

Before:
```
@query mcp lean-ctx ctx_shell cmd="
mkdir -p docs/mdai/specs &&
SPEC_PATH=docs/mdai/specs/...
cat > \"$SPEC_PATH\" <<'SPEC_EOF'
...
```

After (Strategie laut Audit):
- `@mkdir docs/mdai/specs` (atomar, jail-respecting)
- `@render-template ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/templates/spec-template.md output="..."` oder
- `@append-if-missing` build, falls `output=`-Argument an `@render-template` nicht verfügbar (Risk §10.1).

Konkrete Endform → Audit-Empfehlung übernehmen. Bei Unsicherheit ob `output=` supported ist: Fallback `@append-if-missing` benutzen (Spec §7.12 P5-Risiko).

- [ ] **Step 2: `spec-reviewer.md` adopten**

Hotspot §6.5: §5 conditional library-spec-checks → `@if file.containsLine`. §3 Calibration ggf. `@switch status`.

Hinweis: spec-reviewer.md wird in P5/Task 8 vollständig lean re-shaped — P4 ändert hier nur das, was die Audit-Empfehlung **isoliert** ausweist. Lean-Re-Shape verschoben.

- [ ] **Step 3: `body.mdai.md` adopten**

Hotspot §6.5:
- pre-context: `@call detect_mai_hook_version()` (kommt aus Task 11 in P5 — `body.mdai.md` Edit dort durchgeführt).
- Process-Checklist phase-transitions ggf. `@switch current_phase` — Audit-Empfehlung beachten.

P4-Anteil hier: nur Audit-spezifische Adoptions, die NICHT mit P5-Lazy-Load-Refactor kollidieren. Bei Konflikt: P5-Edit hat Vorrang, Audit-Item flaggt für post-P5-Folge-Edit.

- [ ] **Step 4: Per-File-Smoke nach jedem Edit**

```
mcp__markdownai__read_file(path="<file>", cwd="<repo>")
```

Erwartet: 0 warnings.

- [ ] **Step 5: Commit skill-Adoptions**

`mcp__jetbrains__reformat_file` pro modifizierter Datei, dann:

```bash
ctx_shell "git add mdai/skills/mdai-brainstorm/"
ctx_shell "git commit -m 'refactor(mdai/skills): adopt Wave 3-5 directives (P4 audit findings)'"
```

### Task 4: Phase-4 End-Gate

- [ ] **Step 1: Komplette P4-Verifikation aus Spec §6.7**

```
ctx_search "@query.*mkdir|@query.*ctx_shell.*mkdir" mdai/
ctx_search "@if file\.|@switch|@foreach|@set " mdai/
ctx_search "adoption-audit" docs/mdai/audits/
```

Erwartet: V1 = 0 matches, V2 ≥ 10 matches, V4 = 1 match.

- [ ] **Step 2: End-to-end resolve_phase auf jede Phase**

In Claude-Code-Session, pro Phase:

```
mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="pre-context", cwd="<repo>")
mcp__markdownai__resolve_phase(..., phase="dialog-rules", ...)
mcp__markdownai__resolve_phase(..., phase="dialog-process", ...)
mcp__markdownai__resolve_phase(..., phase="write-outputs", ...)
mcp__markdownai__resolve_phase(..., phase="handoff", ...)
```

Erwartet: V3 = 0 ENOENT-warnings, 0 unresolved-directive-warnings pro Phase.

(Hinweis: write-outputs/handoff bekommen ihre Lazy-Load-Form erst in P5. P4-End-Gate akzeptiert hier den jetzigen Zustand — Final-Smoke nach P5.)

---

## Phase 5: Lazy-Load-Refactor (L1 / L2 / L3) — MCP-first

### Task 5: L1 — `spec-directive-conventions.md` erstellen

**Files:**
- Create: `mdai/skills/mdai-brainstorm/spec-directive-conventions.md`

- [ ] **Step 1: Datei mit L1-Inhalt schreiben**

Frontmatter `mode: include`, ~360 W. Inhalt:
- 9 Use-Cases × 3 Spalten (Use-Case / Best-Practice / Anti-Pattern) — aus aktuellem `body.mdai.md` dialog-process-Block extrahiert
- Neue 4. Spalte: „v1.0 native equivalent" (z.B. `shell-mkdir` → `@mkdir`, `ctx_shell echo > file` → `@append-if-missing`)
- `file_check`-Anti-Pattern-Block übernehmen
- Plain-Markdown-Exception übernehmen

Beispiel-Skelett:

```markdown
---
mode: include
lib_version: 0.1.1
---
@markdownai v1.0

# Spec Body Directive Conventions (L1 — write-outputs phase)

Operationalizes Discipline §10.4 #9. Mandatory at the "Write design doc" step.

| Use-Case | Best Practice | Anti-Pattern | v1.0 native equivalent |
| -------- | ------------- | ------------ | ---------------------- |
| Date in file paths | `{{ @date format='YYYY-MM-DD' }}` | hard-coded `2026-05-24` | — |
| Directory listing | `@tree mdai/ depth=2` | manually typed-out tree | — |
| File-system status (report) | `@call file_check(path="...")` | `ls -la` output copy | — |
| Branching on file existence | inline `@if file.exists "..."` + `@else` | `@call file_check` for branching | — |
| Structured data | `@list <file.yaml> \| @render type="table"` | plain Markdown table at >50 rows | — |
| Counts / Statistics | `{{ @count ./src "*.ts" }}` | hard-coded numbers | — |
| Cross-File-Content | `@include ./CHANGELOG.md` or lines=N-M | copy-paste between specs | — |
| Machine-Readable Constraints | `@constraint id="..." severity="high"` + body + `@end` | prosaic "Important:" hints | — |
| Project-Context (live) | `@call ctx_overview(task="...")` | manually copied project description | — |
| Filesystem writes (NEW v1.0) | `@mkdir <path>` / `@copy src dst` | `@query ctx_shell "mkdir -p ..."` | `@mkdir` / `@copy` / `@append-if-missing` |
| YAML-frontmatter mutate (NEW v1.0) | `@update-frontmatter file=... key=... value=...` | shell `sed -i` über `---`-Block | `@update-frontmatter` |
| Sub-Render mit args (NEW v1.0) | `@render-template <path> args={...}` | manual string-concat in @query | `@render-template` |
| Conditional anchor-check (NEW v1.0) | `@if file.containsLine "<file>" "<anchor>"` | grep-output check | `file.containsLine` |
| Iteration über list/anchors (NEW v1.0) | `@foreach item in items` + body + `@endforeach` | repeated `@if`-Blöcke | `@foreach` + `@set` |
| Multi-branch (NEW v1.0) | `@switch <var>` + `@case "..."` + `@default` | repeated `@if`/`@elseif`-chain | `@switch`/`@case`/`@default` |

**Anti-pattern: `file_check` is not branching.** [verbatim aus aktuellem body.mdai.md]

For branching ALWAYS inline at the call site:

@if file.exists "x.md"
- do this when exists
@else
- do that when missing
@endif

**Exception** (per §10.4 #9): specs for purely algorithmic topics without
file/tool/data dependencies may stay plain Markdown — then set
`markdownai_directives_omitted: <reason>` in the frontmatter.

<!-- Drift-Tracking: hand-ported from body.mdai.md dialog-process phase,
     consolidated with v1.0 Wave-3–5 native equivalents column. -->
```

- [ ] **Step 2: Smoke-Render**

```
mcp__markdownai__read_file(path="mdai/skills/mdai-brainstorm/spec-directive-conventions.md", cwd="<repo>")
```

Erwartet: 0 warnings, Tabelle rendered.

### Task 6: L3 — `process-principles.md` erstellen

**Files:**
- Create: `mdai/skills/mdai-brainstorm/process-principles.md`

- [ ] **Step 1: Datei mit L3-Inhalt schreiben**

Frontmatter `mode: include`, ~250 W. Inhalt aus aktuellem `body.mdai.md` dialog-process phase extrahieren:
- Block „The Process — Details" (hand-ported from superpowers:brainstorming/SKILL.md)
- Block „Key Principles"

```markdown
---
mode: include
lib_version: 0.1.1
---
@markdownai v1.0

# The Process — Details (L3 — dialog-process phase)

[hand-ported from superpowers:brainstorming/SKILL.md, lines 70-104]

- **Understanding the idea:** scope check, decomposition for large projects,
  one question at a time, no batched "tell me everything" prompts.
- **Exploring approaches:** 2–3 alternatives with explicit trade-offs, lead
  with recommendation but make alternatives real (not strawmen).
- **Presenting the design:** scaled to complexity, approval-per-section,
  user reads each section before next is drafted.
- **Design for isolation and clarity:** small units, clear interfaces, no
  premature abstractions.
- **Working in existing codebases:** follow existing patterns, no unrelated
  refactoring, no scope-creep into adjacent files.

## Key Principles

[hand-ported from superpowers:brainstorming/SKILL.md, lines 140-145]

- One question at a time.
- Multiple choice preferred over open-ended where it fits.
- YAGNI ruthlessly.
- Explore alternatives before settling.
- Incremental validation.
- Be flexible — go back when something doesn't make sense.
```

- [ ] **Step 2: Smoke-Render**

```
mcp__markdownai__read_file(path="mdai/skills/mdai-brainstorm/process-principles.md", cwd="<repo>")
```

### Task 7: L2 — `spec-self-review.md` erstellen (Library-Pack, TDD)

**Files:**
- Create: `mdai/skills/mdai-brainstorm/spec-self-review.md`

- [ ] **Step 1: Schreibe failing smoke-test (TDD)**

Smoke-Test = call_macro auf das noch nicht existierende Library-Pack:

```
mcp__markdownai__call_macro(
  file="mdai/skills/mdai-brainstorm/spec-self-review.md",
  macro="spec_self_review",
  args={"spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md"},
  cwd="<repo>"
)
```

Erwartet: `found: false` (Datei existiert nicht) ODER ENOENT.

- [ ] **Step 2: Datei schreiben**

Frontmatter `mode: import-only`, `exports: [spec_self_review]`. ~290 W. Inhalt: 5+1 Checks aus aktuellem `body.mdai.md` „Spec Self-Review"-Block.

```markdown
---
mode: import-only
exports: [spec_self_review]
lib_version: 0.1.1
---
@markdownai v1.0

@define spec_self_review(spec_path)
  # Spec Self-Review — {{ spec_path }}

  After the spec source is written, look at it with fresh eyes:

  ## Check #1 — Placeholder scan
  Any "TBD", "TODO", incomplete sections, vague requirements? Fix inline.

  ## Check #2 — Internal consistency
  Sections contradict each other? Architecture matches feature descriptions?

  ## Check #3 — Scope check
  Focused enough for a single plan? Or needs decomposition into sub-projects?

  ## Check #4 — Ambiguity
  Any requirement interpretable two different ways? Pick one, make it explicit.

  ## Check #5 — mdai directive usage (Discipline §10.4 #9)
  Does the spec body include markdownai directives for live content where
  semantically appropriate? If pure plain Markdown: justified with
  `markdownai_directives_omitted: <reason>` in the frontmatter?

  ## Check #6 — Lean-Context Anchors
  @set anchors=["mode=\"full\"", "raw=true", "fresh=true",
                 "Grep|rg ", "cat |head |tail ", "bash |sh "]
  @foreach anchor in anchors
    @if file.containsLine "{{ spec_path }}" "{{ anchor }}"
      - FLAGGED: `{{ anchor }}` found. Check adjacent `@note visible consumer="human"` block.
    @else
      - clean: `{{ anchor }}` not present.
    @endif
  @endforeach

  ## Reviewer Dispatch (optional)
  Trigger: spec touches MCP signatures, Library packs, or render flow.
  Invoke via:
    mcp__markdownai__call_macro(
      file="mdai/skills/mdai-brainstorm/spec-reviewer.md",
      macro="spec_reviewer_prompt",
      args={"spec_path": "{{ spec_path }}"},
      cwd="<repo>"
    )

  Fix issues inline. No re-review loop — fix and move on.
@end
```

- [ ] **Step 3: Smoke-Test wiederholen — sollte jetzt PASSEN**

```
mcp__markdownai__call_macro(
  file="mdai/skills/mdai-brainstorm/spec-self-review.md",
  macro="spec_self_review",
  args={"spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md"},
  cwd="<repo>"
)
```

Erwartet: `found: true`, output enthält 5+1 Checks + Reviewer-Dispatch-Pointer, 0 warnings.

### Task 8: `spec-reviewer.md` Lean-Re-Shape

**Files:**
- Modify: `mdai/skills/mdai-brainstorm/spec-reviewer.md`

- [ ] **Step 1: Aktuelle Datei lesen**

```
mcp__serena__jet_brains_get_symbols_overview(relative_path="mdai/skills/mdai-brainstorm/spec-reviewer.md")
```

(Falls Markdown nicht symbolic indizierbar: `ctx_read(path="...", mode="full")`.)

- [ ] **Step 2: Lean Re-Shape (~45 Z statt 168 Z)**

Ersetze gesamten Body durch:

```markdown
---
mode: import-only
exports: [spec_reviewer_prompt]
lib_version: 0.1.1
---
@markdownai v1.0
@define spec_reviewer_prompt(spec_path)
  You are a spec doc reviewer. Verify {{ spec_path }} is complete and ready.

  ## 1. Read the spec
  mcp__markdownai__read_file(path="{{ spec_path }}", cwd="<repo>")

  ## 2. What to Check
  @prompt role="reference"
  | Category | What to Look For |
  | -------- | ---------------- |
  | Completeness | TODOs, placeholders, "TBD", incomplete sections |
  | Consistency | Internal contradictions, conflicting requirements |
  | Clarity | Requirements ambiguous enough to cause wrong builds |
  | Scope | Focused enough for a single plan |
  | YAGNI | Unrequested features, over-engineering |

  ## 3. Calibration
  @prompt role="calibration"
  Only flag issues that would cause real problems during impl planning.
  Approve unless there are serious gaps.

  ## 4. mdai-Augmentations (universal)
  a. Language convention (CLAUDE.md): spec body German, code/snippets English.
  b. mdai directives in body (Discipline §10.4 #9): ≥3 distinct directive
     types in body, OR frontmatter has `markdownai_directives_omitted: <reason>`.
  c. Lean-context audit: invoke via
     mcp__markdownai__call_macro(file="mdai/core/lean-context-audit.md",
                                   macro="lean_context_audit",
                                   args={"spec_path": "{{ spec_path }}"},
                                   cwd="<repo>")

  ## 5. Heavy library-spec checks (conditional)
  @if file.containsLine "{{ spec_path }}" "target_library:"
    Invoke via mcp__markdownai__call_macro(
      file="mdai/core/library-spec-audit.md",
      macro="library_spec_audit",
      args={"spec_path": "{{ spec_path }}"},
      cwd="<repo>")
  @endif

  ## 6. Output
  Invoke via mcp__markdownai__call_macro(
    file="mdai/skills/mdai-brainstorm/write-spec.md",
    macro="write_review_report",
    args={"spec_path": "{{ spec_path }}", "status": "...", ...},
    cwd="<repo>")
@end
```

Implementation via `Write` (komplette Datei-Überschreibung) ODER falls Read-Step done: `Edit` mit dem gesamten alten Body als `old_string`.

- [ ] **Step 3: Smoke-Test**

```
mcp__markdownai__call_macro(
  file="mdai/skills/mdai-brainstorm/spec-reviewer.md",
  macro="spec_reviewer_prompt",
  args={"spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md"},
  cwd="<repo>"
)
```

Erwartet: `found: true`, output enthält §1–§6, conditional §5 fires (weil Spec `target_library:` enthält).

### Task 9: `write-spec.md` — `write_review_report` + template

**Files:**
- Modify: `mdai/skills/mdai-brainstorm/write-spec.md`
- Create: `mdai/skills/mdai-brainstorm/templates/review-template.md`

- [ ] **Step 1: Template-File anlegen**

`mdai/skills/mdai-brainstorm/templates/review-template.md`:

```markdown
# Spec Review — {{ spec_path | basename }}

**Date:** {{ @date format='YYYY-MM-DD' }}
**Status:** {{ status }}
**Spec:** `{{ spec_path }}`

## Strengths
{{ strengths }}

## Issues
{{ issues }}

## Recommendations
{{ recommendations }}
```

- [ ] **Step 2: `write-spec.md` — `write_review_report` einfügen**

Aktuelle write-spec.md lesen via `ctx_read(path="...", mode="full")` (kurze Datei).

Hinter `@define render_spec(...)` insert (via `mcp__serena__jet_brains_insert_after_symbol` ODER `Edit`):

```markdown
@define write_review_report(spec_path, status, strengths, issues, recommendations)
  @set report_path="docs/mdai/reviews/{{ spec_path | basename | replace('.mdai.md', '') }}-review.md"
  @mkdir docs/mdai/reviews
  @render-template ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/templates/review-template.md \
                   args={ "spec_path": spec_path, "status": status, "strengths": strengths,
                          "issues": issues, "recommendations": recommendations } \
                   output="{{ report_path }}"
@end
```

- [ ] **Step 3: `lib_version` bump 0.1.0 → 0.1.1**

In `write-spec.md` Frontmatter:

Before:
```
lib_version: 0.1.0
```

After:
```
lib_version: 0.1.1
```

- [ ] **Step 4: Smoke-Test `write_review_report`**

```
mcp__markdownai__call_macro(
  file="mdai/skills/mdai-brainstorm/write-spec.md",
  macro="write_review_report",
  args={
    "spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md",
    "status": "approved",
    "strengths": "test",
    "issues": "none",
    "recommendations": "ship it"
  },
  cwd="<repo>"
)
```

Erwartet: `found: true`, `docs/mdai/reviews/2026-05-25-mdai-v1.0-adoption-design-review.md` existiert mit gerendertem Inhalt.

Falls `@render-template output=` nicht supported (Spec §7.12 Risk):
- Fallback: `@append-if-missing`-build statt `output=`. Macro umstellen.
- Smoke-Test wiederholen.

### Task 10: `body.mdai.md` Phase-Refactor

**Files:**
- Modify: `mdai/skills/mdai-brainstorm/body.mdai.md`

- [ ] **Step 1: Aktuelle body.mdai.md vollständig lesen**

```
ctx_read(path="mdai/skills/mdai-brainstorm/body.mdai.md", mode="full")
```

(Bereits durch P3-Edit teilweise modifiziert — finale Form aus Spec §7.2.)

- [ ] **Step 2: pre-context phase — `detect_mai_hook_version` einbauen**

In `pre-context`-Block:
- Statt `@call detect_mdai_root()` (das gestrichen ist) → `@call detect_mai_hook_version()`.
- `detect_mai_hook_version` ist in `mdai/core/startup-check.md` definiert (Task 11).

Edit-Diff:

Before (falls noch detect_mdai_root vorhanden):
```
@call mdai_bootstrap()
@include ${MDAI_LIBRARY_ROOT}/core/hard-rules.md
```

After:
```
@call mdai_bootstrap()
@call detect_mai_hook_version()
@include ${MDAI_LIBRARY_ROOT}/core/hard-rules.md
```

- [ ] **Step 3: dialog-process phase — Shrink (Spec §7.8)**

Aus dialog-process-Block entfernen / ersetzen:
- Block „The Process — Details" → ersetzen durch `@include ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/process-principles.md` (L3 file).
- Block „Key Principles" → fällt mit oberen Include zusammen (in L3 file enthalten).
- Block „Spec Self-Review (step 7, MANDATORY, Claude himself)" → entfernen (wandert nach L2 file).
- Block „Spec reviewer dispatch (step 7.5, OPTIONAL, mdai-Augmentation)" → entfernen (wandert nach handoff phase als call_macro pointer).
- Block „Spec body mdai directive conventions (mandatory reading for Step 6)" → entfernen (wandert nach write-outputs phase als L1 include).

Übrig in dialog-process bleibt:
- Process Checklist (Z 117-133 in aktueller body.mdai.md, mit phase-transition hints aus §7.10)
- Visual-Companion-Block
- User-Review-Gate Verweis (Wording wandert nach handoff)

Ziel: dialog-process source ≤ 600 W (§1.4 Success #4, §7.8).

Phase-Checklist-Schritte 6-9 ersetzen mit Spec §7.10 Wording:

```markdown
## Process Checklist

1. Explore project ctx (already done in pre-context phase).
2. Offer visual companion — own msg (see Visual-Companion section).
3. Ask clarifying questions — one at a time.
4. Propose 2–3 approaches with trade-offs.
5. Present design sections, get approval after each.
6. Switch to write-outputs phase:
   mcp__markdownai__resolve_phase(file=..., phase="write-outputs", cwd=...)
   - Apply spec-directive-conventions while finalizing design_content.
   - Invoke write_spec via call_macro.
7. Switch to handoff phase:
   mcp__markdownai__resolve_phase(file=..., phase="handoff", cwd=...)
   7a. Invoke spec_self_review via call_macro.
   7b. Apply review findings inline.
   7c. opt: dispatch spec_reviewer_prompt via call_macro.
8. User-Review-Gate (in same handoff phase, exact wording).
9. Transition: invoke writing-plans skill.

@include ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/process-principles.md
```

- [ ] **Step 4: write-outputs phase — L1 include + call_macro pointer**

In write-outputs-Block, das aktuelle `@import` + `@call write_spec` ersetzen durch:

```markdown
@phase write-outputs

@include ${MDAI_LIBRARY_ROOT}/skills/mdai-brainstorm/spec-directive-conventions.md

Apply the conventions above when finalizing design_content. Then invoke write_spec
via call_macro:

  mcp__markdownai__call_macro(
    file="mdai/skills/mdai-brainstorm/write-spec.md",
    macro="write_spec",
    args={ "slug": "{{ slug }}", "body": "{{ design_content }}" },
    cwd="<repo>"
  )

Optional inline-render (only when explicitly requested):

  mcp__markdownai__call_macro(
    file="mdai/skills/mdai-brainstorm/write-spec.md",
    macro="render_spec",
    args={ "slug": "{{ slug }}", "target": "{{ render_target | default('none') }}" },
    cwd="<repo>"
  )

Default output (one file staged in working tree):
- `docs/mdai/specs/<date>-<slug>-design.mdai.md` (spec source, consumer="ai")

Verification:
@call ctx_tree(path="docs/mdai/specs/", depth=1)

Note: commit is left to the user (per CLAUDE.md — never auto-commit).
Note: NO plan file is written here. Plan-write is a separate skill invocation.
@end
```

- [ ] **Step 5: handoff phase — Pointer + User-Gate**

handoff-Block ersetzen durch Spec §7.4-Form:

```markdown
@phase handoff

Spec Self-Review (5+1 checks). Invoke library-pack:

  mcp__markdownai__call_macro(
    file="mdai/skills/mdai-brainstorm/spec-self-review.md",
    macro="spec_self_review",
    args={ "spec_path": "{{ spec_path }}" },
    cwd="<repo>"
  )

Apply review findings inline.

Optional: dispatch full reviewer subagent:

  mcp__markdownai__call_macro(
    file="mdai/skills/mdai-brainstorm/spec-reviewer.md",
    macro="spec_reviewer_prompt",
    args={ "spec_path": "{{ spec_path }}" },
    cwd="<repo>"
  )

## User-Review-Gate (exact wording, MANDATORY)

> "Spec written and committed to `<path>`. Please review and give feedback on
> whether you want changes, before invoking `/superpowers:writing-plans <path>`
> as the next step (or `/mdai-writing-plans` once that skill exists)."

Wait for explicit response. If user requests changes → patch inline → re-run
spec_self_review via call_macro. Only proceed once user explicitly approves.

Next: invoke writing-plans skill.
@end
```

- [ ] **Step 6: Per-Phase-Smoke**

Pro Phase aufrufen:

```
mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="<phase>", cwd="<repo>")
```

Phasen: `pre-context`, `dialog-rules`, `dialog-process`, `write-outputs`, `handoff`.

Erwartet je Phase: `warnings: []`, content rendered.

- [ ] **Step 7: Phase-Budget verifizieren**

```
mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="dialog-process", cwd="<repo>")
```

Erwartet: content ≤ 600 W (§1.4 Success #4).

```
mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="handoff", cwd="<repo>")
```

Erwartet: content ≤ 100 W.

### Task 11: `startup-check.md` — `detect_mai_hook_version` + cleanup

**Files:**
- Modify: `mdai/core/startup-check.md`

- [ ] **Step 1: Aktuelle startup-check.md lesen**

```
mcp__serena__jet_brains_get_symbols_overview(relative_path="mdai/core/startup-check.md")
```

- [ ] **Step 2: `detect_mdai_root` löschen**

Falls noch nicht in P4 erfolgt (Task 2 Step 1):

Find symbol und delete:

```
mcp__serena__jet_brains_find_symbol(name_path="detect_mdai_root", relative_path="mdai/core/startup-check.md", include_body=true)
```

Dann `replace_symbol_body` mit leerem Body ODER `Edit` mit dem alten `@define ... @end`-Block als `old_string` und leerem `new_string`.

- [ ] **Step 3: `detect_mai_hook_version` einfügen**

Insert nach `mdai_bootstrap` oder am Ende der Datei:

```markdown
@define detect_mai_hook_version()
  @if file.exists "~/.markdownai/hooks/preToolUse.mjs"
    @if file.containsLine "~/.markdownai/hooks/preToolUse.mjs" "isMarkdownAIDocument"
      [mdai-bootstrap] mai-hook: v1.0 (frontmatter-aware)
    @else
      [mdai-bootstrap] mai-hook: v0.x — RUN `node markdownai/packages/core/dist/cli.js init`
    @endif
  @else
    [mdai-bootstrap] mai-hook: not installed — RUN init
  @endif
@end
```

Edit via `mcp__serena__jet_brains_insert_after_symbol` (Anchor: letztes `@define` ODER `@end`-Block am EOF).

- [ ] **Step 4: Smoke-Test**

```
mcp__markdownai__call_macro(
  file="mdai/core/startup-check.md",
  macro="detect_mai_hook_version",
  args={},
  cwd="<repo>"
)
```

Erwartet: `found: true`, output zeigt eine der 3 mai-hook-Statusen je nach realem Hook-State (sollte nach Part 1 P2 = „v1.0 frontmatter-aware").

- [ ] **Step 5: Verifizieren dass detect_mdai_root weg ist**

```
ctx_search "detect_mdai_root" mdai/
```

Erwartet: 0 matches.

### Task 12: Phase-5 End-Gate

- [ ] **Step 1: Komplette P5-Verifikation aus Spec §7.11**

```
mcp__markdownai__list_phases(file="mdai/skills/mdai-brainstorm/body.mdai.md", cwd="<repo>")
mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="handoff", cwd="<repo>")
mcp__markdownai__call_macro(file="mdai/skills/mdai-brainstorm/spec-self-review.md", macro="spec_self_review", args={"spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md"}, cwd="<repo>")
mcp__markdownai__resolve_phase(file="mdai/skills/mdai-brainstorm/body.mdai.md", phase="write-outputs", cwd="<repo>")
ctx_search "@include \./spec-self-review\.md" mdai/skills/mdai-brainstorm/
ctx_search "@define spec_self_review" mdai/skills/mdai-brainstorm/spec-self-review.md
ctx_search "detect_mdai_root" mdai/
```

Erwartet:
- V1: 5 phases.
- V2: content ~70 W, Pointer auf call_macro spec_self_review enthalten, keine 5+1 Check-Texte inline.
- V3: output enthält 5+1 Checks + Reviewer-Dispatch-Pointer, found:true, warnings:[].
- V4: content ~280 W, L1-Tabelle inline + Pointer für write_spec.
- V5: 0 matches (L2 ist MCP-first).
- V6: 1 match (L2 Library-Pack vorhanden).
- V7: 0 matches (gestrichen).

- [ ] **Step 2: Commit P5**

`mcp__jetbrains__reformat_file` pro modifizierter Datei:
- `mdai/skills/mdai-brainstorm/body.mdai.md`
- `mdai/skills/mdai-brainstorm/write-spec.md`
- `mdai/skills/mdai-brainstorm/spec-reviewer.md`
- `mdai/skills/mdai-brainstorm/spec-directive-conventions.md`
- `mdai/skills/mdai-brainstorm/spec-self-review.md`
- `mdai/skills/mdai-brainstorm/process-principles.md`
- `mdai/skills/mdai-brainstorm/templates/review-template.md`
- `mdai/core/startup-check.md`

```bash
ctx_shell "git add mdai/"
ctx_shell "git commit -m 'refactor(mdai-brainstorm): L1/L2/L3 lazy-load + spec-reviewer lean re-shape (v0.1.1)'"
```

---

## Phase 6: call_macro Library-Distribution

### Task 13: `mdai/core/lean-context-audit.md` — neu (TDD)

**Files:**
- Create: `mdai/core/lean-context-audit.md`

- [ ] **Step 1: Failing smoke-test schreiben (TDD)**

```
mcp__markdownai__call_macro(
  file="mdai/core/lean-context-audit.md",
  macro="lean_context_audit",
  args={"spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md"},
  cwd="<repo>"
)
```

Erwartet: `found: false` (Datei existiert nicht) ODER ENOENT.

- [ ] **Step 2: Datei schreiben**

Komplett aus Spec §8.5:

```markdown
---
mode: import-only
exports: [lean_context_audit]
lib_version: 0.1.1
---
@markdownai v1.0
@define lean_context_audit(spec_path)
  # Lean-Context Audit für {{ spec_path }}

  @set anchors=["mode=\"full\"", "raw=true", "fresh=true",
                 "Grep|rg ", "cat |head |tail ", "bash |sh "]

  ## 6-Anchor-Sweep
  @foreach anchor in anchors
    @if file.containsLine "{{ spec_path }}" "{{ anchor }}"
      - FLAGGED: `{{ anchor }}` found. Check adjacent `@note visible consumer="human"` block.
    @else
      - clean: `{{ anchor }}` not present.
    @endif
  @endforeach
@end
```

- [ ] **Step 3: Smoke-Test wiederholen — sollte PASSEN**

```
mcp__markdownai__call_macro(
  file="mdai/core/lean-context-audit.md",
  macro="lean_context_audit",
  args={"spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md"},
  cwd="<repo>"
)
```

Erwartet: `found: true`, output enthält 6 audit-Zeilen („FLAGGED" / „clean"), warnings:[].

### Task 14: `mdai/core/library-spec-audit.md` — neu (TDD)

**Files:**
- Create: `mdai/core/library-spec-audit.md`

- [ ] **Step 1: Failing smoke-test schreiben**

```
mcp__markdownai__call_macro(
  file="mdai/core/library-spec-audit.md",
  macro="library_spec_audit",
  args={"spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md"},
  cwd="<repo>"
)
```

Erwartet: `found: false`.

- [ ] **Step 2: Datei schreiben (Struktur aus Spec §8.6, vollständig ausgeschrieben)**

```markdown
---
mode: import-only
exports: [library_spec_audit]
lib_version: 0.1.1
---
@markdownai v1.0
@define library_spec_audit(spec_path)
  # Library-Spec Audit für {{ spec_path }}

  @set checks=[
    {"id": 1, "anchor": "MCP-Signatur-Verifikation",  "type": "anchor_search",     "guidance": "Add a section enumerating MCP call signatures used by exported macros."},
    {"id": 2, "anchor": "mode: import-only",          "type": "anchor_search",     "guidance": "Each new pack must declare its mdai-pack mode in frontmatter."},
    {"id": 3, "anchor": "Render-Flow-Tests",          "type": "anchor_search",     "guidance": "Spec should list call_macro / resolve_phase smoke-tests per exported macro."},
    {"id": 4, "anchor": "@constraint id=",            "type": "anchor_search",     "guidance": "Use @constraint id=... severity=... for machine-readable rules."},
    {"id": 5, "field":  "lib_version",                "type": "frontmatter_check", "guidance": "Bump lib_version when pack contents change. Spec should mention target version."},
    {"id": 7, "anchor": "Discipline §10.4",           "type": "anchor_search",     "guidance": "Library specs should map back to brainstorm-Discipline mismatches if any."},
    {"id": 8, "anchor": "Drift-Tracking",             "type": "anchor_search",     "guidance": "Hand-ported blocks must carry Drift-Tracking comment with source provenance."}
  ]

  @foreach check in checks
    @switch check.type
      @case "anchor_search"
        @if file.containsLine "{{ spec_path }}" "{{ check.anchor }}"
          - Check #{{ check.id }} ({{ check.anchor }}): PRESENT
        @else
          - Check #{{ check.id }} ({{ check.anchor }}): MISSING — {{ check.guidance }}
        @endif
      @case "frontmatter_check"
        @if file.frontmatterField "{{ spec_path }}" "{{ check.field }}"
          - Check #{{ check.id }} (field {{ check.field }}): SET
        @else
          - Check #{{ check.id }} (field {{ check.field }}): NOT-SET — {{ check.guidance }}
        @endif
    @endswitch
  @endforeach
@end
```

- [ ] **Step 3: Smoke-Test wiederholen — sollte PASSEN**

```
mcp__markdownai__call_macro(
  file="mdai/core/library-spec-audit.md",
  macro="library_spec_audit",
  args={"spec_path": "docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md"},
  cwd="<repo>"
)
```

Erwartet: `found: true`, output enthält 7 check-Zeilen mit jeweils PRESENT/MISSING/SET/NOT-SET, warnings:[].

### Task 15: Modus-A-Migration verifizieren

**Files:**
- Read: `mdai/skills/mdai-brainstorm/body.mdai.md` (über MCP)

- [ ] **Step 1: Verifizieren dass kein `@import` mehr für Modus-A-Macros existiert**

```
ctx_search "@import.*mdai/skills/mdai-brainstorm/(write-spec|spec-self-review|spec-reviewer)" mdai/skills/mdai-brainstorm/body.mdai.md
```

Erwartet: 0 matches (alle Modus-A migriert auf call_macro-Pointer).

Falls Matches: Edit body.mdai.md, `@import` durch call_macro-Pointer ersetzen (Spec §8.3 Modus A).

- [ ] **Step 2: Modus-B-Aufrufe für Bootstrap-Macros bestätigen**

```
ctx_search "@import \\${MDAI_LIBRARY_ROOT}/core/startup-check" mdai/skills/mdai-brainstorm/body.mdai.md
ctx_search "@call mdai_bootstrap" mdai/skills/mdai-brainstorm/body.mdai.md
ctx_search "@call detect_mai_hook_version" mdai/skills/mdai-brainstorm/body.mdai.md
```

Erwartet: jeder Search ≥ 1 match (Modus-B-Bootstrap aktiv).

### Task 16: P6-End-Gate

- [ ] **Step 1: Komplette P6-Verifikation aus Spec §8.8**

```
ctx_search "@import.*mdai/skills/mdai-brainstorm/(write-spec|spec-self-review|spec-reviewer)" mdai/skills/mdai-brainstorm/body.mdai.md
```

Plus die 4 call_macro-Aufrufe aus Spec §8.8 V2/V3/V4/V5 — jeweils `found: true` und expected output.

- [ ] **Step 2: End-to-end-Smoke (manuell, User-Decision)**

User-driven, im Brainstorm-Run gegen synthetisches Test-Topic:
- alle Phasen `resolve_phase` rendern ohne ENOENT
- alle `call_macro`-Aufrufe `found: true`
- Final-Spec-File geschrieben in `docs/mdai/specs/`
- 5+1 Self-Review-Checks ausgeführt
- User-Review-Gate erreicht

Status persistieren via `ctx_session`.

- [ ] **Step 3: Commit P6**

`mcp__jetbrains__reformat_file` pro neuer Datei:
- `mdai/core/lean-context-audit.md`
- `mdai/core/library-spec-audit.md`

```bash
ctx_shell "git add mdai/core/lean-context-audit.md mdai/core/library-spec-audit.md"
ctx_shell "git commit -m 'feat(mdai/core): lean-context-audit + library-spec-audit library packs (P6 call_macro distribution)'"
```

---

## Post-Phase: Vorgänger-Spec & Green-Verification-Artefakt

### Task 17: Vorgänger-Spec als superseded markieren

**Files:**
- Modify: `docs/mdai/specs/2026-05-25-mdai-brainstorm-lazyload-and-namespace-resolver-design.mdai.md`

- [ ] **Step 1: Frontmatter updaten**

Edit (Frontmatter-Header):

Before (status-Zeile suchen):
```
status: ready-for-review
```

After:
```
status: superseded
superseded_by: docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md
superseded_reason: markdownai v1.0.0 release ersetzt Engine-Resolver-Patch durch source_root config. Lazy-Load-Anteil wandert in neue Spec mit aktualisierten Wave-3–5-Direktiven.
```

(Falls die Vorgänger-Spec schon andere status-Zeile hat: entsprechend anpassen.)

- [ ] **Step 2: Commit**

```bash
ctx_shell "git add docs/mdai/specs/2026-05-25-mdai-brainstorm-lazyload-and-namespace-resolver-design.mdai.md"
ctx_shell "git commit -m 'docs(mdai/specs): mark lazyload+namespace-resolver spec as superseded by v1.0-adoption'"
```

### Task 18: Green-Verification-Artefakt schreiben

**Files:**
- Create: `docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.1-smoke.md`

- [ ] **Step 1: Smoke-Doc-Skelett anlegen**

Format analog `docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md`. Sektionen:

```markdown
# mdai-brainstorm v0.1.1 — Green Verification (Smoke)

**Date:** 2026-05-25 (oder Implementations-Datum)
**Spec:** docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md
**Plans:** docs/mdai/plans/2026-05-25-mdai-v1.0-engine-adoption.md + docs/mdai/plans/2026-05-25-mdai-brainstorm-v0.1.1-refactor.md
**markdownai version:** v1.0.0 (post Part 1)
**skill version:** v0.1.1

## Smoke-Suite §8.1–§8.15

| Smoke | Status | Notes |
| ----- | ------ | ----- |
| §8.1 | deferred | user-driven |
| §8.2 | deferred | user-driven |
| §8.3 | deferred | user-driven |
| §8.4 | <PASS/FAIL> | dialog-process ≤600W |
| §8.5 | <PASS/FAIL> | ${MDAI_LIBRARY_ROOT} globs |
| §8.6 | <PASS/FAIL> | lean-context-discipline |
| §8.7 | n/a | gestrichen (namespace-resolver) |
| §8.8 | <PASS/FAIL> | phase-transition workflow |
| §8.9 | <PASS/FAIL> | lean-reviewer dispatch |
| §8.10 | <PASS/FAIL> | audit-macro composability |
| §8.11 | <PASS/FAIL> | write_review_report |
| §8.12 | <PASS/FAIL/Branch?> | respondTool-Empirik (Part 1 P1c) |
| §8.13 | <PASS/FAIL> | hook re-install (Part 1 P2) |
| §8.14 | <PASS/FAIL> | source_root-config (Part 1 P3) |
| §8.15 | <PASS/FAIL> | call_macro library-distribution (Part 2 P6) |

## Phase-Budget-Tabelle

| Phase | Vorher (W) | Nachher (W) | Budget | Δ |
| ----- | ---------- | ----------- | ------ | -- |
| pre-context | 165 | <measured> | — | <Δ> |
| dialog-rules | 703 | <measured> | — | <Δ> |
| dialog-process | 990 | <measured> | ≤600 | <Δ> |
| write-outputs | 92 | <measured> | — | <Δ> |
| handoff | 69 | <measured> | ≤100 | <Δ> |
| Σ src | 2019 | <measured> | — | <Δ> |

## Diagnose-Notes pro non-pass-Test

(eine Sektion pro nicht-PASS Smoke-Item — leer wenn alles PASS)

## Re-Verification-Trigger

- Patch in `mdai/skills/mdai-brainstorm/` (alle 9 Files)
- Patch in `mdai/core/` (alle 8+2 Files inkl. neue audit-Packs)
- markdownai-Engine-Bump > v1.0.0 mit directive-Verhaltens-Änderungen
- Hook-Script-Update (v1.0.x patch-bumps) — `detect_mai_hook_version` flagged Drift
- Upstream-Bump von `superpowers:brainstorming` (Versions-Pin in visual-companion-offer.md)
- §8.1/§8.2/§8.3 nachgeholt (User-Action)

## Outstanding-Liste

(Backlog-Items aus Spec §10.4 die nach Implementation noch offen sind)
```

- [ ] **Step 2: Measured-Werte einfüllen**

Im Verlauf der Phase-5/Phase-6 End-Gates die `<measured>`-Werte und PASS/FAIL-States eintragen.

- [ ] **Step 3: Commit**

```bash
ctx_shell "git add docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.1-smoke.md"
ctx_shell "git commit -m 'docs(mdai/green-verification): mdai-brainstorm v0.1.1 smoke verification'"
```

---

## Final-Verification (gesamt)

### Task 19: End-to-End-Smoke Brainstorm-Run

**Files:**
- Reference: `mdai/skills/mdai-brainstorm/body.mdai.md`

- [ ] **Step 1: Manueller Brainstorm-Run gegen synthetisches Topic**

Im Claude-Code:
- `/mdai-brainstorm` (oder Skill-Aufruf) mit kleinem Test-Topic („Smoke-Test").
- Komplette Phase-Sequenz durchlaufen lassen: pre-context → dialog-rules → dialog-process → write-outputs → handoff.
- Alle MCP-Calls protokollieren (idealerweise via `ctx_session action="finding"`).

- [ ] **Step 2: Pass-Kriterien aus Spec §9.2**

- alle Phasen `resolve_phase` rendern ohne ENOENT
- alle `call_macro`-Aufrufe `found: true`
- Final-Spec-File geschrieben in `docs/mdai/specs/`
- 5+1 Self-Review-Checks ausgeführt
- User-Review-Gate erreicht

- [ ] **Step 3: Ergebnis in green-verification eintragen**

Smoke-Doc aus Task 18 mit konkreten PASS/FAIL aus dem End-to-End-Run finalisieren.

### Task 20: Plan-Abschluss

- [ ] **Step 1: Pre-v1.0-Tag aufräumen?**

User-Entscheidung. Default: Tag behalten als Safety-Net.

```bash
ctx_shell "cd markdownai && git tag --list pre-v1.0-bump"
```

- [ ] **Step 2: settings.json-Backup aufräumen?**

User-Entscheidung. Default: Backup für mind. 1 Woche behalten.

- [ ] **Step 3: User-Handoff**

User informieren:
- Beide Plan-Parts (Part 1 + Part 2) committed.
- Smoke-Suite §8.1/§8.2/§8.3 noch deferred (user-driven).
- Backlog laut Spec §10.4 für Folge-Specs verfügbar.
- Plan-Status (Part 2): complete.

---

## Notes

- **Bash-Politik (CLAUDE.md):** Keine `&&`-Ketten. Jeden Befehl einzeln.
- **Pre-commit-Reformat (CLAUDE.md):** Vor jedem `git add` für modifizierte Files: `mcp__jetbrains__reformat_file(absolutePath="<file>")`.
- **Tests:** Cargo nicht relevant in diesem Plan (kein Rust-Code geändert). Nicht verwendet: `cargo nextest run`.
- **Worktree-Politik (CLAUDE.md):** Keine Worktrees — direkt auf `feat-mdai`.
- **Sprache (CLAUDE.md):** Plan-Body Deutsch (mit Umlauten), Code/Snippets Englisch — eingehalten.
- **Subagent-Modell:** Bei Dispatch über `Agent`/`TaskCreate`: `model: sonnet` (Standard) bzw. `model: opus` für P4 (Tasks 1–4) wenn nötig.
- **Hard-Dependency:** Part 1 (Engine-Adoption) MUSS abgeschlossen sein bevor dieser Plan startet — siehe Pre-Flight-Task 0.
