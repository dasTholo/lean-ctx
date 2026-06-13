# Design-Spec — JetBrains-Plugin-PR finalisieren

**Datum:** 2026-06-13
**Branch (Quelle):** `feat-jetbrains-plugin` (347 Commits vor `main`)
**Ziel-Branch (neu):** `pr/jetbrains-plugin` (sauberer Squash ab `main`)

## Ziel

Den JetBrains-Plugin-Branch in einen reviewbaren, sauberen PR überführen. Zwei
Arbeitspakete:

1. **Docs aktualisieren + ins Englische übersetzen** — die beiden einzigen
   `-de`-Referenzdateien auf den vollen Branch-Stand bringen und an die
   englische Namens-/Inhaltskonvention der übrigen Journeys angleichen.
2. **Sauberen PR-Branch erstellen** — ein einzelner Squash-Commit ab `main`,
   der ausschließlich die echte Plugin-Arbeit + die übersetzten Referenzdocs
   enthält. Alle Agent-/IDE-/Prozess-/mdai-Tooling-Artefakte bleiben draußen.

## Nicht-Ziele

- Inhaltliche Änderungen an der Plugin-Logik (rust/kotlin) — der Code bleibt
  funktional unverändert.
- Aufräumen/Umschreiben der 347-Commit-Historie über einen Squash hinaus.
- Migration der mdai-/superpowers-/lean-md-Prozessartefakte irgendwohin — sie
  verbleiben auf `feat-jetbrains-plugin`, nur eben nicht im PR-Branch.

---

## Teil A — Doc-Aktualisierung + Übersetzung

Arbeitet auf `feat-jetbrains-plugin`, committet dort.

### A.1 `19-jetbrains-plugin-de.md` → `19-jetbrains-plugin.md`

- **Übersetzung:** Bestehende Sektionen 0–8 (559 Zeilen) vollständig ins
  Englische. Fachbegriffe/Code-Identifier unverändert.
- **Inhaltliches Update (voll auf Branch-Stand):**
  - **+ Neue Sektion: Gain-ToolWindow.** Quelle:
    `docs/lean-md/specs/2026-06-09-leanctx-jetbrains-gain-toolwindow-design.md`
    + Kotlin-Code unter
    `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/toolwindow/`.
    Abdecken: Hero/Subscores/Tabellen/Footer-Rendering, States,
    Visibility-gated Poll-Controller (immediate-load-on-visible),
    Statusbar-Click + Menu-Action-Trigger, `GAIN_TOOL_WINDOW_ID`-Konstante,
    `gain --json`-Schemavertrag (DTO-Keys, snake_case-Mapping, gegen
    Drift-Test gelockt), 10s-Timeout, ANSI-Escape-Strip vor Messages-Popup.
  - **+ Neue Sektion: editor-focus reporter.** Quelle:
    `docs/lean-md/specs/2026-06-13-leanctx-jetbrains-editor-focus-design.md`
    (#500 producer parity).
  - **Verifikation Bestand:** Refactoring-Sektionen (Rename/Reformat/Move/
    Safe-Delete/Inline) gegen aktuellen Code/Endpoints prüfen und ggf.
    korrigieren (v2c resolveTarget-Indentation-Fix, v2d inline/reformat,
    headless-conflict direct-PSI safe_delete).
- **Querverweise:** interne Links auf `appendix-jetbrains-plugin.md`
  aktualisieren (Zeilen ~11, ~552).

### A.2 `appendix-jetbrains-plugin-de.md` → `appendix-jetbrains-plugin.md`

- Vollständige Übersetzung (74 Zeilen).
- Tabellen (Funktionen, Guards, Fehler-Codes) um Gain-ToolWindow +
  editor-focus ergänzen.
- Querverweis auf `19-jetbrains-plugin.md` aktualisieren (Zeilen ~5, ~73).

### A.3 README + Repo-weite Querverweise

- `docs/reference/README.md`: Tabellen-Zeilen ~35 und ~46 — Pfade auf die
  neuen Dateinamen, „(DE)"-Marker entfernen.
- Repo-weit nach weiteren Referenzen auf `19-jetbrains-plugin-de` /
  `appendix-jetbrains-plugin-de` suchen (`ctx_search`) und anpassen.

### A.4 Abschluss

- Alte DE-Dateien `git rm`, englische Dateien committen.
- Vor `git add`: `mcp__jetbrains__reformat_file` auf die geänderten Dateien
  (Projektregel).

---

## Teil B — Sauberer PR-Branch (Squash ab `main`)

### B.1 Methode

```
git checkout -b pr/jetbrains-plugin main
git checkout feat-jetbrains-plugin -- .      # gesamten Branch-Tree in den Index/WD
# → danach Exclude-Liste anwenden (B.2), main-Dateien zurücksetzen (B.3),
#   .gitignore kürzen (B.4)
git commit -m "feat(jetbrains): JetBrains IDE plugin (PSI nav, refactoring engine, gain toolwindow)"
```

Ergebnis: **ein** Commit, dessen Diff gegen `main` exakt
(`feat-jetbrains-plugin` minus Excludes) entspricht. Keine DE-Commit-Messages,
keine 347-Commit-Historie.

### B.2 Exclude — bekannte Tooling-Ordner (Blocklist, Startpunkt)

Neue Ordner auf `feat-jetbrains-plugin`, die NICHT auf `main` existieren und
NICHT in den PR gehören:

1. `.claude/`
2. `.idea/`
3. `.serena/`
4. `docs/lean-md/`
5. `docs/mdai/`
6. `docs/superpowers/`
7. `mdai/`
8. `rust/.claude/`

**Behalten (NICHT excluden):** `rust/.config/` — enthält `nextest.toml` +
`nextest-tmpdir.sh` (Test-Runner-Infrastruktur, von der Test-Suite benötigt).
Bleibt im PR.

Root-Tooling-Dateien (NEU, alle raus):
`CLAUDE.md`, `.lean-ctx.toml`, `.mcp.json`, `madai-todo.md`,
`mdai-benchmark.md`, `spec-reviewer-v1.md`.

Bereits via `.gitignore` ausgeschlossen (kommen ohnehin nie in den PR):
`tmp/`, `markdownai/`.

### B.3 Auf `main`-Stand zurücksetzen (existieren auf main, wurden geändert)

```
git checkout main -- AGENTS.md .cursorrules
```

Agent-Tooling-Edits gehören nicht in den Plugin-PR.

### B.4 `.gitignore`

Nur die Hygiene-Zeilen für die ausgeschlossenen Ordner behalten
(`tmp/`, `markdownai/`, `.idea/`, `.serena/`, `.claude/...`), als
Sicherheitsnetz gegen versehentliches Tracking. Übrige branch-spezifische
Zeilen verwerfen.

### B.5 Allowlist-Verifikation (verbindlich — ersetzt reine Blocklist)

Die Blocklist allein ist fehleranfällig (drei Ordner mehr als anfangs
genannt). Daher nach B.1–B.4:

```
git diff --name-only main pr/jetbrains-plugin
```

**Jede** Datei in diesem Diff einzeln bestätigen, dass sie zu einer der
erlaubten Kategorien gehört:

- `packages/jetbrains-lean-ctx/**` (Kotlin-Plugin + Tests + gradle)
- `rust/src/**` Backend-Code der Plugin-Features (jetbrains_backend,
  `ctx_refactor`, `rust/src/core/gain/**`, editor-focus reporter)
- `rust/tests/**` zugehörige Tests
- `rust/.config/**` (nextest.toml + nextest-tmpdir.sh — Test-Infrastruktur)
- `docs/reference/19-jetbrains-plugin.md`,
  `docs/reference/appendix-jetbrains-plugin.md`, `docs/reference/README.md`,
  ggf. `docs/reference/generated/**` (nur falls CI-drift-relevant)
- ggf. weitere durch den main→branch-Merge legitim geänderte Kerndateien

Taucht eine Datei auf, die in keine Kategorie passt → entfernen
(`git rm` / `git checkout main -- <pfad>`) und Commit amenden.

---

## Teil C — Verifikation & Smoke

1. **Diff-Sauberkeit:** `git diff --stat main pr/jetbrains-plugin` zeigt
   **0 Dateien** in den 8 Exclude-Ordnern und keine Root-Tooling-Dateien.
2. **Build/Tests (auf `pr/jetbrains-plugin`):**
   - `cargo nextest run` (bare command, cwd=`rust`) — grün.
   - `./gradlew build` im Plugin-Verzeichnis — BUILD SUCCESSFUL.
3. **Docs-Links:** keine toten Verweise auf `*-de.md` mehr
   (`ctx_search "jetbrains-plugin-de"` → 0 Treffer).

## Risiken / offene Punkte

- **`docs/reference/generated/**`:** Auto-generierte MCP-Tool-Referenz —
  CI-drift-getestet. Nur aufnehmen, wenn frisch generiert und konsistent.
- **Merge-Commits in der Historie:** Da Squash, irrelevant für den Diff; der
  finale Diff ist rein baum-basiert (`main`-Tree vs. konstruierter Tree).
