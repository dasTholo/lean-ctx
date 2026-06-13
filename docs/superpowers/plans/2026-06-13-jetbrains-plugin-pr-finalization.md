# JetBrains-Plugin-PR finalisieren — Implementierungsplan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Die JetBrains-Plugin-Referenzdocs vollständig + englisch auf Branch-Stand bringen und einen sauberen, reviewbaren PR-Branch (`pr/jetbrains-plugin`, ein Squash-Commit ab `main`) ohne Tooling-/Prozessartefakte erzeugen.

**Architecture:** Zwei sequenzielle Phasen. **Teil A** arbeitet auf `feat-jetbrains-plugin`: Funktions-Inventar → vollständige EN-Übersetzung + neue Sektionen → README/Querverweise → committen. **Teil B** baut `pr/jetbrains-plugin` ab `main`, übernimmt den (inzwischen aktualisierten) `feat-jetbrains-plugin`-Tree, entfernt die Exclude-Pfade, verifiziert per Allowlist. **Teil C** ist Build-/Link-Smoke.

**Tech Stack:** Git, Markdown, lean-ctx (`ctx_read`/`ctx_search`/`ctx_shell`/`ctx_edit`), `mcp__jetbrains__reformat_file`, `cargo nextest`, Gradle.

**Quelle (Spec):** `docs/superpowers/specs/2026-06-13-jetbrains-plugin-pr-finalization-design.md`

**Werkzeug-Disziplin (Projektregeln):**
- Lesen → `ctx_read`; Suchen → `ctx_search`; Shell → `ctx_shell` (bare command + `cwd=`, **nie** `cd … &&`, **kein** `2>&1`).
- Markdown editieren → `ctx_edit` (native `Edit` ist für Nicht-Rust blockiert).
- Vor jedem `git add` einer geänderten Datei → `mcp__jetbrains__reformat_file`.
- Deferred-Tool → zuerst `ToolSearch(query="select:<tool>")`.

---

## Teil A — Doc-Aktualisierung + Übersetzung (auf `feat-jetbrains-plugin`)

### Task A1: Funktions-Inventar & Gap-Analyse

**Files:**
- Lesen: `packages/jetbrains-lean-ctx/src/main/resources/META-INF/plugin.xml`
- Lesen: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/*.kt` (NavHandlers, StructureHandlers, InspectionHandlers, EditHandlers, RefactorHandlers)
- Lesen: `docs/reference/19-jetbrains-plugin-de.md` (Bestand §0–8), `docs/reference/appendix-jetbrains-plugin-de.md`
- Create: `docs/superpowers/plans/_inventory-jetbrains.md` (temporäre Gap-Liste; wird in Teil B als Prozessartefakt ausgeschlossen)

- [ ] **Step 1: Registrierte Funktionen aus `plugin.xml` extrahieren**

`ctx_read` auf `plugin.xml`. Notiere ALLE Extensions + Actions:
- `statusBarWidgetFactory` → LeanCtxStatusBarFactory (Token-Savings-Widget)
- `postStartupActivity` → LeanCtxStartupActivity (HTTP-Server-Boot)
- `toolWindow id=LeanCtxGain` → Gain-ToolWindow
- `registryKey leanctx.editor.signal.enabled` → editor-focus reporter (#500, opt-out, path only)
- `supportsKotlinPluginMode supportsK2="true"` → K2-Mode
- Actions-Gruppe `LeanCtx.Menu` (ToolsMenu): Setup, Doctor, Gain Report, Dashboard

- [ ] **Step 2: HTTP-Endpunkte aus den Handlern extrahieren**

`ctx_search pattern="POST|GET|/[a-zA-Z]+Preview|/[a-zA-Z]+Apply" path="packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint"` — Liste aller Routen je Handler. Erwartete Funktionsfamilien (v1→v2d, **ohne** Codenamen zu notieren): Navigation, Struktur, Inspektionen, Symbol-Body-Edits, Rename, Reformat, Move, Safe-Delete, Inline.

- [ ] **Step 3: Gap-Liste schreiben**

In `_inventory-jetbrains.md` eine Tabelle `Funktion | in Doku §X? | Quelle`. Markiere alles ohne Doku-Abdeckung. Erwartete Gaps: Gain-ToolWindow, editor-focus reporter, Status-Bar-Widget, Tools-Menü-Actions, K2-Mode, ANSI-Strip-Util.

- [ ] **Step 4: Verifikation**

Jede `plugin.xml`-Extension + jede Action + jeder Endpunkt muss in der Gap-Liste genau eine Zeile haben. Prüfe gegen Step 1+2: keine Funktion fehlt.

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/plans/_inventory-jetbrains.md
git commit -m "docs(jetbrains): function inventory + doc gap analysis"
```

---

### Task A2: Bestand übersetzen + Datei umbenennen + Codenamen bereinigen

**Files:**
- Rename/Create: `docs/reference/19-jetbrains-plugin.md` (neu, EN)
- Lesen: `docs/reference/19-jetbrains-plugin-de.md` (Quelle)

- [ ] **Step 1: Neue Datei mit EN-Übersetzung von §0–8 anlegen**

`ctx_read` der DE-Quelle in Abschnitten (`mode="lines:N-M"`). Erzeuge `19-jetbrains-plugin.md` via `ctx_edit(path, new_string=..., create=true)`. Übersetze §0–8 vollständig ins Englische. **Verlustfrei** — kein Endpunkt, keine Tabelle, kein Guard entfällt. Fachbegriffe/Code-Identifier/Endpoint-Namen unverändert.

- [ ] **Step 2: Codenamen-Bereinigung beim Übersetzen**

- `v2a`/`v2b`/`v2c`/`v2d`/`v1` als Versions-Marker NICHT übernehmen — funktional umschreiben.
- „Phase 1/Phase 2" beim Two-Phase-Rename (entspricht §333–355) als Protokoll-Beschreibung **behalten**.
- Zeile ~467 (temp-Dateiname `.<name>.lean-ctx.v2a.tmp.<pid>`): Der Rust-Code nutzt das Muster `.{name}.tmp.{pid}` (vgl. `rust/src/cloud_client.rs:64`) — die `v2a`-Variante ist überholt. Generisch schreiben: `.<name>.lean-ctx.tmp.<pid>` (atomarer Write + `rename`), ohne Versions-Suffix.

- [ ] **Step 3: Verifikation — keine Codenamen**

```
ctx_search pattern="\\bv2[a-d]\\b|\\bv1\\b" path="docs/reference/19-jetbrains-plugin.md"
```
Erwartet: 0 Treffer (außer ggf. legitime Nicht-Versions-Vorkommen — prüfen, sonst umschreiben).

- [ ] **Step 4: Verifikation — kein deutscher Resttext**

Stichprobe per `ctx_search pattern="\\b(und|oder|nicht|wird|werden|über|für|Datei|Verwendung)\\b" path="docs/reference/19-jetbrains-plugin.md"` — Treffer = unübersetzte Stellen, korrigieren. (In Code-Blöcken/Strings sind solche Wörter zulässig — manuell bewerten.)

- [ ] **Step 5: Commit**

```bash
git add docs/reference/19-jetbrains-plugin.md
git commit -m "docs(jetbrains): translate reference journey 19 to English (sections 0-8)"
```

---

### Task A3: Neue Sektion „Gain Tool Window"

**Files:**
- Modify: `docs/reference/19-jetbrains-plugin.md`
- Lesen: `docs/lean-md/specs/2026-06-09-leanctx-jetbrains-gain-toolwindow-design.md`
- Lesen: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/toolwindow/` (GainPanel, GainPollController, GainService, LeanCtxGainToolWindowFactory)
- Lesen: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/GainData.kt`

- [ ] **Step 1: Quellen lesen**

`ctx_read` der Gain-Spec + `mode="signatures"` auf die 4 toolwindow-Dateien + GainData.kt. Erfasse: DTOs `GainSummaryDTO`, `ModelDTO`, `ScoreDTO`, `TaskRow`, `FileRow`; `GainCodec.parse(json)`.

- [ ] **Step 2: Sektion schreiben**

Neue Sektion in §2/§3-Bereich (vor „Verhaltensgarantien"). Inhalt:
- **Zweck:** Bottom-ToolWindow `LeanCtxGain`, rendert den `gain`-Report (Hero-Score, Subscores, Task-/File-Tabellen, Footer) mit States (Loading/Empty/Error/Data).
- **Datenfluss:** `GainService` ruft `lean-ctx gain --json` als Subprozess (10 s Timeout) → `GainCodec.parse` (Gson, snake_case via `@SerializedName`) → typisiertes Load-Result → `GainPanel`.
- **Schema-Vertrag:** `gain --json`-Keys gegen DTO gelockt (Drift-Test `e82ddbec`). Tabelle der Top-Level-Keys (summary/model/score/tasks/files) auflisten.
- **Poll-Controller:** `GainPollController` ist visibility-gated — lädt sofort beim Sichtbarwerden, pollt nur bei sichtbarem ToolWindow.
- **Trigger:** Statusbar-Click + Tools-Menü „Gain Report" öffnen das ToolWindow (Konstante `GAIN_TOOL_WINDOW_ID`).
- **Hygiene:** ANSI-Escapes werden vor Anzeige gestript (`util/AnsiText`, Fix `b933e510`).

- [ ] **Step 3: Verifikation**

`ctx_search pattern="LeanCtxGain|gain --json|GainPollController" path="docs/reference/19-jetbrains-plugin.md"` → Treffer vorhanden. DTO-Keys in der Doku stimmen mit `GainData.kt` überein (manueller Abgleich).

- [ ] **Step 4: Commit**

```bash
git add docs/reference/19-jetbrains-plugin.md
git commit -m "docs(jetbrains): add Gain tool window section"
```

---

### Task A4: Neue Sektion „Editor-Focus Reporter"

**Files:**
- Modify: `docs/reference/19-jetbrains-plugin.md`
- Lesen: `docs/lean-md/specs/2026-06-13-leanctx-jetbrains-editor-focus-design.md`
- Lesen: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/EditorFocusReporter.kt`

- [ ] **Step 1: Quellen lesen**

`ctx_read` der editor-focus-Spec + EditorFocusReporter.kt (`mode="signatures"`).

- [ ] **Step 2: Sektion schreiben**

Inhalt:
- **Zweck (#500 producer parity):** meldet den Pfad der fokussierten Editor-Datei an lean-ctx fürs Context-Ranking — **nur Pfad, niemals Inhalt**.
- **Opt-out:** Registry-Key `leanctx.editor.signal.enabled` (Default `true`), deaktivierbar.
- **Mechanik:** Producer-Seite, wie/wohin der Pfad gemeldet wird (aus Spec).
- **Privacy-Hinweis:** explizit „path only".

- [ ] **Step 3: Verifikation**

`ctx_search pattern="leanctx.editor.signal.enabled|editor-focus|#500" path="docs/reference/19-jetbrains-plugin.md"` → Treffer vorhanden.

- [ ] **Step 4: Commit**

```bash
git add docs/reference/19-jetbrains-plugin.md
git commit -m "docs(jetbrains): add editor-focus reporter section"
```

---

### Task A5: Neue Sektion „IDE-UI-Integration" (Statusbar, Actions, K2)

**Files:**
- Modify: `docs/reference/19-jetbrains-plugin.md`
- Lesen: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStatusBarFactory.kt`, `StatsReader.kt`, `actions/LeanCtxActions.kt`, `util/AnsiText.kt`

- [ ] **Step 1: Quellen lesen**

`ctx_read mode="signatures"` auf die 4 Dateien.

- [ ] **Step 2: Sektion schreiben**

Inhalt:
- **Status-Bar-Widget** (`LeanCtxStatusBarFactory` + `StatsReader`): zeigt Echtzeit-Token-Savings; Click öffnet das Gain-ToolWindow; Reihenfolge `after encodingWidget`.
- **Tools-Menü „lean-ctx"** (`actions/`): Setup, Doctor, Gain Report, Dashboard — was jede Action ausführt; Doctor/Setup-Ausgabe via Messages-Popup mit ANSI-Strip (`util/AnsiText`).
- **K2-Mode:** Plugin unterstützt K2 (`supportsKotlinPluginMode supportsK2="true"`); kurzer Kompatibilitätshinweis.

- [ ] **Step 3: Verifikation**

`ctx_search pattern="status bar|Tools menu|Setup|Doctor|Dashboard|K2" path="docs/reference/19-jetbrains-plugin.md"` → alle Aspekte vorhanden.

- [ ] **Step 4: Gap-Liste-Abgleich**

`_inventory-jetbrains.md` öffnen: JEDE als „fehlt" markierte Funktion ist jetzt in §-Doku abgedeckt. Hake ab; offene Punkte = sofort ergänzen.

- [ ] **Step 5: Commit**

```bash
git add docs/reference/19-jetbrains-plugin.md
git commit -m "docs(jetbrains): add IDE UI integration section (status bar, actions, K2)"
```

---

### Task A6: Appendix übersetzen + umbenennen + ergänzen

**Files:**
- Rename/Create: `docs/reference/appendix-jetbrains-plugin.md` (neu, EN)
- Lesen: `docs/reference/appendix-jetbrains-plugin-de.md` (Quelle)

- [ ] **Step 1: EN-Übersetzung anlegen**

`ctx_read` der DE-Quelle (74 Z.). `appendix-jetbrains-plugin.md` via `ctx_edit(create=true)`. Vollständig übersetzen (Koordinaten/Aufruf, Funktionen, Guards, Fehler-Codes, Siehe-auch).

- [ ] **Step 2: Tabellen ergänzen**

Funktions-/Guards-/Fehler-Tabellen um Gain-ToolWindow, editor-focus reporter, Statusbar/Actions erweitern (kompakter Lookup, konsistent mit den neuen §-Sektionen). Querverweis auf `19-jetbrains-plugin.md` setzen.

- [ ] **Step 3: Verifikation**

`ctx_search pattern="\\bv2[a-d]\\b|\\bv1\\b" path="docs/reference/appendix-jetbrains-plugin.md"` → 0 Treffer. Gain/editor-focus in Tabellen vorhanden.

- [ ] **Step 4: Commit**

```bash
git add docs/reference/appendix-jetbrains-plugin.md
git commit -m "docs(jetbrains): translate appendix to English + add new functions"
```

---

### Task A7: README + Querverweise + DE-Dateien entfernen

**Files:**
- Modify: `docs/reference/README.md` (Zeilen ~35, ~46)
- Modify: `docs/reference/19-jetbrains-plugin.md`, `docs/reference/appendix-jetbrains-plugin.md` (interne Links)
- Delete: `docs/reference/19-jetbrains-plugin-de.md`, `docs/reference/appendix-jetbrains-plugin-de.md`

- [ ] **Step 1: Interne Querverweise in den EN-Dateien**

In `19-jetbrains-plugin.md`: Links auf `appendix-jetbrains-plugin.md` (statt `-de`). In `appendix-jetbrains-plugin.md`: Links auf `19-jetbrains-plugin.md`. Per `ctx_edit`.

- [ ] **Step 2: README aktualisieren**

`ctx_read mode="lines:30-50" docs/reference/README.md`. Zeilen ~35 + ~46: Pfade auf `19-jetbrains-plugin.md` / `appendix-jetbrains-plugin.md`, „(DE)"-Marker entfernen, Beschreibungstext beibehalten.

- [ ] **Step 3: Repo-weiter Querverweis-Scan**

```
ctx_search pattern="jetbrains-plugin-de" path="."
```
Jeden weiteren Treffer (außerhalb `docs/lean-md`, `docs/mdai`, `docs/superpowers` — die fliegen sowieso raus) auf die neuen Pfade umstellen.

- [ ] **Step 4: DE-Dateien löschen**

```bash
git rm docs/reference/19-jetbrains-plugin-de.md docs/reference/appendix-jetbrains-plugin-de.md
```

- [ ] **Step 5: Verifikation — keine toten Links**

```
ctx_search pattern="jetbrains-plugin-de" path="docs/reference"
```
Erwartet: 0 Treffer.

- [ ] **Step 6: reformat + Commit**

Vor `git add` auf jede geänderte `.md`:
```
mcp__jetbrains__reformat_file (README.md, 19-jetbrains-plugin.md, appendix-jetbrains-plugin.md)
```
```bash
git add docs/reference/README.md docs/reference/19-jetbrains-plugin.md docs/reference/appendix-jetbrains-plugin.md
git commit -m "docs(jetbrains): retarget README + cross-refs to English files, drop DE versions"
```

---

## Teil B — Sauberer PR-Branch (`pr/jetbrains-plugin`, Squash ab `main`)

> Voraussetzung: Teil A ist vollständig committet auf `feat-jetbrains-plugin`.

### Task B1: Branch ab `main` + Branch-Tree übernehmen

**Files:** keine Inhaltsänderung, nur Git.

- [ ] **Step 1: Sauberen Arbeitszustand prüfen**

```
ctx_shell command="git status --porcelain" 
```
Erwartet: leer (oder nur untracked, die nicht stören). Falls dirty → erst committen/stashen.

- [ ] **Step 2: Neuen Branch ab main**

```
ctx_shell command="git checkout -b pr/jetbrains-plugin main"
```

- [ ] **Step 3: Gesamten feat-Tree in Index + Working-Dir holen**

```
ctx_shell command="git checkout feat-jetbrains-plugin -- ."
```
Erwartet: alle Branch-Dateien sind jetzt im Index gestaged.

- [ ] **Step 4: Verifikation**

```
ctx_shell command="git diff --cached --stat main"
```
Sollte den vollen Branch-Diff zeigen (inkl. der Tooling-Ordner — die entfernen wir in B2).

---

### Task B2: Exclude-Pfade entfernen

**Files:** Entfernen aus Index/WD.

- [ ] **Step 1: Exclude-Ordner entfernen**

```
ctx_shell command="git rm -r --cached --quiet -- .claude .idea .serena docs/lean-md docs/mdai docs/superpowers mdai rust/.claude"
```
Danach die Verzeichnisse auch aus dem Working-Dir entfernen (untracked-Reste vermeiden) — nur falls vorhanden, sonst überspringen.

- [ ] **Step 2: Exclude-Root-Dateien entfernen**

```
ctx_shell command="git rm --cached --quiet -- CLAUDE.md .lean-ctx.toml .mcp.json madai-todo.md mdai-benchmark.md spec-reviewer-v1.md"
```

- [ ] **Step 3: `rust/.config/` BEHALTEN — Kontrolle**

```
ctx_shell command="git diff --cached --name-only main -- rust/.config"
```
Erwartet: `rust/.config/nextest.toml` + `rust/.config/nextest-tmpdir.sh` sind weiterhin gestaged (NICHT entfernen).

- [ ] **Step 4: Verifikation Excludes weg**

```
ctx_shell command="git diff --cached --name-only main -- .claude .idea .serena docs/lean-md docs/mdai docs/superpowers mdai rust/.claude CLAUDE.md .lean-ctx.toml .mcp.json madai-todo.md mdai-benchmark.md spec-reviewer-v1.md"
```
Erwartet: leer (0 Zeilen).

---

### Task B3: Geänderte main-Dateien zurücksetzen + `.gitignore` kürzen

**Files:**
- Reset: `AGENTS.md`, `.cursorrules` (auf main-Stand)
- Modify: `.gitignore`

- [ ] **Step 1: AGENTS.md + .cursorrules auf main**

```
ctx_shell command="git checkout main -- AGENTS.md .cursorrules"
```

- [ ] **Step 2: .gitignore — nur Hygiene-Zeilen behalten**

`ctx_read .gitignore`. Auf main-Stand zurück (`git checkout main -- .gitignore`), dann per `ctx_edit` nur die Sicherheitsnetz-Zeilen für die Exclude-Ordner ergänzen:
```
/tmp/
tmp/
markdownai/
.idea/
.serena/
.claude/
```
(Branch-spezifische sonstige Zeilen NICHT übernehmen.)

- [ ] **Step 3: Verifikation**

```
ctx_shell command="git diff --cached --name-only main -- AGENTS.md .cursorrules"
```
Erwartet: leer (identisch mit main). `.gitignore` zeigt nur die Hygiene-Ergänzung.

---

### Task B4: Squash-Commit

- [ ] **Step 1: .gitignore stagen**

```
ctx_shell command="git add .gitignore"
```

- [ ] **Step 2: Commit**

```
ctx_shell command="git commit -m 'feat(jetbrains): JetBrains IDE plugin — PSI navigation, refactoring engine, gain tool window, editor-focus reporter'"
```

- [ ] **Step 3: Verifikation Single-Commit**

```
ctx_shell command="git rev-list --count main..pr/jetbrains-plugin"
```
Erwartet: `1`.

---

### Task B5: Allowlist-Verifikation (verbindlich)

- [ ] **Step 1: Vollständige Diff-Dateiliste ziehen**

```
ctx_shell command="git diff --name-only main pr/jetbrains-plugin" raw=true
```

- [ ] **Step 2: Jede Datei einer erlaubten Kategorie zuordnen**

Erlaubte Kategorien:
- `packages/jetbrains-lean-ctx/**` (Kotlin-Plugin + Tests + gradle)
- `rust/src/**` Plugin-Feature-Backend (jetbrains_backend, `ctx_refactor`, `rust/src/core/gain/**`, editor-focus)
- `rust/tests/**`
- `rust/.config/**`
- `docs/reference/19-jetbrains-plugin.md`, `docs/reference/appendix-jetbrains-plugin.md`, `docs/reference/README.md`, ggf. `docs/reference/generated/**`
- durch den main→branch-Merge legitim geänderte Kerndateien (manuell prüfen)

- [ ] **Step 3: Fremdkörper entfernen**

Jede Datei, die in keine Kategorie passt → `git rm --cached <pfad>` bzw. `git checkout main -- <pfad>`, danach `git commit --amend --no-edit`. Wiederhole Step 1–3, bis der Diff sauber ist.

- [ ] **Step 4: Verifikation Exclude-Ordner endgültig leer**

```
ctx_shell command="git diff --stat main pr/jetbrains-plugin -- .claude .idea .serena docs/lean-md docs/mdai docs/superpowers mdai rust/.claude"
```
Erwartet: leer.

---

## Teil C — Verifikation & Smoke (auf `pr/jetbrains-plugin`)

### Task C1: Diff-Sauberkeit

- [ ] **Step 1: Gesamt-Diff sichten**

```
ctx_shell command="git diff --stat main pr/jetbrains-plugin"
```
Bestätige: 0 Dateien in den 8 Exclude-Ordnern, keine Root-Tooling-Dateien (`CLAUDE.md` etc.), `rust/.config/` vorhanden.

### Task C2: Build + Tests

- [ ] **Step 1: Rust-Tests**

```
ctx_shell command="cargo nextest run" cwd="rust"
```
Erwartet: Summary grün. (Bei großem grünem Lauf: `cargo nextest run --status-level fail`.)

- [ ] **Step 2: Plugin-Build**

```
ctx_shell command="./gradlew build" cwd="packages/jetbrains-lean-ctx"
```
Erwartet: `BUILD SUCCESSFUL`.

### Task C3: Docs-Link-Check

- [ ] **Step 1: Keine `*-de`-Referenzen mehr**

```
ctx_shell command="git grep -n 'jetbrains-plugin-de' -- docs/reference"
```
Erwartet: leer (Exit 1 / keine Ausgabe).

- [ ] **Step 2: Codenamen-Scan final**

```
ctx_shell command="git grep -nE '\\bv2[a-d]\\b' -- docs/reference/19-jetbrains-plugin.md docs/reference/appendix-jetbrains-plugin.md"
```
Erwartet: keine unerwünschten Treffer.

---

## Definition of Done

- `docs/reference/19-jetbrains-plugin.md` + `appendix-jetbrains-plugin.md` existieren, englisch, decken **alle** Funktionen v1→v2d + Gain-ToolWindow + editor-focus + Statusbar/Actions/K2 ab, **ohne** interne Versions-Codenamen.
- DE-Dateien gelöscht, README + Querverweise konsistent.
- `pr/jetbrains-plugin` = **ein** Commit ab `main`, ohne die 8 Exclude-Ordner + 6 Root-Tooling-Dateien, mit `rust/.config/`.
- `cargo nextest` grün, `gradlew build` SUCCESSFUL.
