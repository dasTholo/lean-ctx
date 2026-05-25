# MDAI Real-Plan-Benchmark

Datum: 2026-05-21 · Methode: empirisch v3 (mit MCP-Live-Test) · Autoren: Sonnet-Subagent (v2), Claude (MCP-Ergänzung)

---

## Korrektur gegenüber v1 (2026-05-20)

Der vorherige Benchmark (v1) enthielt zwei fehlerhafte Diagnosen, die als Bugs
bezeichnet wurden. Es handelte sich um Syntax-Fehler des erzeugenden Subagenten,
nicht um MDAI-Defekte:

1. **`@import` angeblich „no-op im Output"** — War kein Bug. `@import` ist _by design_
   unsichtbar: es lädt `@define`-Macros, ohne selbst Content auszugeben. Der v1-Subagent
   hatte `@import` statt `@include` verwendet, um Inhalt inline zu bringen — falsche
   Direktive. `@include` (nicht `@import`) rendert Dateiinhalt sichtbar in den Output.
   Verifiziert an: `markdownai/MDs/tests/test-include-import.md`.

2. **`@phase title=` angeblich „verschwindet"** — War kein Bug. `@phase` nimmt keinen
   `title=`-Parameter. Korrekte Syntax: `@phase <id>`, danach normaler Markdown-Heading.
   Beispiel: `@phase setup` gefolgt von `## Setup Phase`.
   Verifiziert an: `markdownai/MDs/tests/test-phases.md`.

---

## Methodik

- `wc -c -l` auf alle Quelldateien und rendered Output
- `node markdownai/packages/core/dist/cli.js render <datei>` — Ausgabe via Pipe gemessen
- Token-Schätzung: Bytes ÷ 4 (englischer/gemischter Prosatext)
- Visuelle Verifikation: `grep` auf rendered Output für alle Direktiven
- Alle Pfade projekt-relativ zu `/home/tholo/Scripts/lean-ctx/`

---

## Test-Infrastruktur

**Verzeichnis:** `tmp/mdai-bench/`

```
tmp/mdai-bench/
├── macros/
│   ├── hard-rules.md                   (11 Z / 535 B)
│   ├── tool-quick-ref.md               (14 Z / 1 136 B)
│   └── step-reformat-commit.md         ( 7 Z / 186 B)
└── 2026-05-17-phase-4-S3a-debug-nodes-wire-and-foundation.mdai.md
                                        (162 Z / 7 056 B)
```

**Original-Plan:** `tmp/plans/2026-05-17-phase-4-S3a-debug-nodes-wire-and-foundation.md`
(943 Z / 35 336 B)
---

## Verwendete MDAI-Direktiven

| Direktive                                | Zweck                                    | Korrekte Syntax                                  |
|------------------------------------------|------------------------------------------|--------------------------------------------------|
| `@include macros/hard-rules.md`          | Hard-Rules-Block inline rendern          | Zieldatei braucht `@markdownai v1.0`-Header      |
| `@include macros/tool-quick-ref.md`      | Tool-Tabelle inline rendern              | Zieldatei braucht `@markdownai v1.0`-Header      |
| `@import macros/step-reformat-commit.md` | `@define stepReformatCommit(file)` laden | Kein sichtbarer Output — by design               |
| `@call stepReformatCommit(<file>)`       | Macro an 7 Commit-Stellen expandieren    | `{{ file }}` als Parameter                       |
| `@constraint id="..." severity="..."`    | Hard-Rule als CONSTRAINT-Block           | Erscheint als `> **CONSTRAINT [id] — SEVERITY**` |
| `@phase <id>` … `@end`                   | Phasen-Marker mit Heading dahinter       | Kein `title=`-Attribut                           |

**Wichtiger Unterschied `@include` vs. `@import`:**

- `@include <datei>` → rendert den Dateiinhalt **sichtbar** an dieser Stelle.
  Die Zieldatei muss `@markdownai v1.0` als erste Zeile haben.
- `@import <datei>` → lädt nur `@define`-Macros, **kein** sichtbarer Output.
  Ist kein Bug — ist die gewollte Semantik für Macro-Bibliotheken.

---

## Visuelle Verifikation (rendered Output)

Alle Prüfungen positiv:

| Prüfpunkt                                   | Ergebnis                                                                    |
|---------------------------------------------|-----------------------------------------------------------------------------|
| Hard-Rules sichtbar?                        | ✅ Zeile 9: `## Hard Rules (aus CLAUDE.md, immer-an)`                        |
| Tool-Quick-Ref sichtbar?                    | ✅ Zeile 21: `## Anhang — Tool-Quick-Reference`                              |
| `@call stepReformatCommit(...)` expandiert? | ✅ 7× `mcp__jetbrains__reformat_file <datei>` + `git add` + `git commit`     |
| Phasen-Headings vorhanden?                  | ✅ `## P0:`, `## A1:`, `## A2:`, `## A3:`, `## A4:`, `## A5:`, `## A-Final:` |
| `@constraint` als Blockquote?               | ✅ `> **CONSTRAINT [hard-rules] — HIGH**`                                    |
| Keine rohen `@`-Direktiven im Output?       | ✅ `grep "@phase\|@include\|@import\|@call"` → leer                          |

---

## Token-Vergleich: S3a

| Variante                        |  Bytes | Zeilen |   ~Tokens | vs. Original |
|---------------------------------|-------:|-------:|----------:|:------------:|
| **Original S3a**                | 35 336 |    943 | **8 834** |   Baseline   |
| MDAI Source (main .mdai.md)     |  7 056 |    162 |     1 764 |    −80 %     |
| MDAI Source + 3 Macros (gesamt) |  8 913 |    194 | **2 228** |  **−75 %**   |
| **MDAI rendered** (vollständig) |  9 062 |    173 | **2 266** |  **−74 %**   |

Token-Schätzung: Bytes ÷ 4.

---

## Bewertung: Ist die Ersparnis echt?

**Ja, der Hauptteil der 74 % Reduktion kommt aus echter inhaltlicher Verdichtung.**

Der Rendered Output (9 062 B / ~2 266 Tokens) ist vollständig verwendbar:
alle Phasen, alle Checkboxen, alle Commit-Schritte, Hard-Rules und Tool-Tabelle
sind sichtbar. Das Original (35 336 B / ~8 834 Tokens) enthält denselben Plan-Inhalt
plus umfangreiche Erläuterungsprose, Querverweise, Spec-Zitat-Blöcke, Beispiel-Code
und detaillierte Begründungen pro Schritt.

**Aufschlüsselung der Ersparnis:**

| Quelle der Reduktion                                          | Schätzung |
|---------------------------------------------------------------|-----------|
| Weglassen von Erklärungsprosa und Querverweisen               | ~40–50 %  |
| `@define`/`@call` für 7 Commit-Schritte (reformat+add+commit) | ~8–10 %   |
| `@include` für Hard-Rules + Tool-Tabelle (statt Inline-Copy)  | ~5 %      |
| `@constraint` statt ausführlichem Hard-Rules-Block            | ~2 %      |

**Wo MDAI echten Gewinn bringt (über manuelle Verdichtung hinaus):**

- `@define`/`@call`: 7 Commit-Schritte werden zu 7 `@call`-Zeilen in der Source.
  Im rendered Output expandiert jeder `@call` zum vollen 3-Zeilen-Block —
  der Leser sieht alles, die Source ist kompakt. Kein manuelles Duplizieren nötig.
- `@include`: Hard-Rules und Tool-Tabelle werden aus geteilten Macro-Dateien gezogen.
  Änderung an einer Stelle → alle Pläne, die `@include` nutzen, sind aktuell.
- `@phase ... @end`: Phasengrenzen sind maschinenlesbar — Voraussetzung für
  zukünftiges selektives Phase-Rendering (`mai render --phase A3`).

**Was die ~74 % nicht belegt:**

Die Prose-Verdichtung (Weglassen von Erklärungen) ist manuell und unabhängig von MDAI.
Dasselbe `.md` hätte manuell auf ~10 000 B gebracht werden können ohne MDAI.
Der MDAI-spezifische Beitrag liegt in den ~10–15 % durch `@define`/`@call` + `@include`.

---

## Phase-Isolation via MCP (Live-Verifikation, 2026-05-21)

Der eigentliche Token-Gewinn entsteht beim **Subagent-Dispatch**: ein Subagent
bekommt nur die zu bearbeitende Phase, nicht den gesamten Plan. Die MDAI-MCP-Tools
liefern Phase-isolierten Inhalt via `read_file(path, phase=X, format=ai)`.

**Setup:** `.mcp.json` um `markdownai`-Eintrag erweitert (`type=stdio`, `command=node`,
`args=[dist/server.js]`). Claude Code startet den Server mit `cwd=Projekt-Root`.

**Live-Messung** (Phase A3 via `mcp__markdownai__read_file`):

| Variante | Bytes |   ~Tokens | vs. Original | vs. Voll-Render |
| **Original S3a (vollständig)**                                | 35 336 | **8 834** | Baseline | — |
| MDAI rendered (vollständig)                                   | 9 062 | **2 266** | −74 % | Baseline |
| **MDAI MCP read_file, phase=A3-rpc-wire-symbols, format=ai**  | 2 818 |   **704** |  **−92 %**   |    **−69 %**    |

Der `format=ai`-Output enthält automatisch: Constraints-Tabelle, YAML-Frontmatter,
Hard-Rules, Tool-Quick-Ref **plus** den eigentlichen Phase-Inhalt. Damit hat der
Subagent alles, was er für die Phase braucht — Hard Rules sind nicht weglassbar.

**Verifizierte MCP-Calls:**

| Tool                               | Parameter                                  | Resultat                            |
|------------------------------------|--------------------------------------------|-------------------------------------|
| `mcp__markdownai__list_phases`     | `file=<rel-path>`                          | 7 Phasen als `{name, transitions}`  |
| `mcp__markdownai__get_constraints` | `file=<rel-path>`                          | `[{id, severity, body}]`            |
| `mcp__markdownai__read_file`       | `path=<rel-path>, phase=A3-..., format=ai` | ~704 Tokens isolierter Phase-Inhalt |

**Server-Constraints** (empirisch ermittelt):

1. **Absolute Pfade werden geblockt** ("Path traversal blocked"). Der Server akzeptiert
   ausschließlich Pfade relativ zu `cwd`. Bei Launch via `.mcp.json` ist `cwd` = Projekt-Root.
2. **MCP-Protocol-Compliance-Patch nötig** (lokal angewendet, upstream offen):
   Der Server returnt für `tools/call` rohe Objekte (`{phases: [...]}`) statt der
   spezifizierten Form `{content: [{type: "text", text: ...}], structuredContent: {...}}`.
   Ohne Patch zeigt Claude Codes MCP-Client „completed with no output".
   Patch: neue `respondTool()`-Funktion in `markdownai/packages/mcp/src/server.ts`,
   ersetzt 9× `respond(id, …)` in `dispatchTool`. Rebuild via `npx tsc`. Verifiziert live.

---

## Token-Ersparnis: Wo, Wie viel, Warum

Die drei Mess-Szenarien sind nicht gleichwertig — jedes spart in unterschiedlicher
Phase des Workflows, und nicht jede Ersparnis ist „echtes" MDAI.

### Die drei Messpunkte im Vergleich

| Szenario                               | Tokens | vs. Original | MDAI-Anteil     | Was passiert hier                                                           |
|----------------------------------------|-------:|-------------:|-----------------|-----------------------------------------------------------------------------|
| **(1) Plan-Source schreiben**          |  2 228 |        −75 % | ~10–15 %        | Du tippst `.mdai.md` + Macros statt der ursprünglichen Prosa                |
| **(2) Plan vollständig rendern**       |  2 266 |        −74 % | ~10–15 %        | Mensch oder Orchestrator-Claude liest den ganzen Plan auf einmal            |
| **(3) Subagent-Dispatch (eine Phase)** |    704 |    **−92 %** | **strukturell** | Subagent bekommt nur Phase A3 + Hard Rules via `mcp__markdownai__read_file` |

**Die drei Zahlen messen verschiedene Dinge — sie sind nicht „dasselbe nochmal":**

- **(1) und (2)** sind eng verwandt. Die ~74 % Reduktion entsteht zur Hälfte daraus,
  dass beim Umschreiben in MDAI auch Erklärungs-Prosa, Querverweise und Spec-Zitate
  gekürzt wurden. Diese Verdichtung ist **manuell** und ginge auch ohne MDAI: ein
  schlanker geschriebener `.md` käme auf ~10 000 B.
- **(3)** ist der **strukturelle** Gewinn. Ohne MDAI-Phasen-Markup gibt es keinen Weg,
  einem Subagenten „nur Phase A3" zu übergeben — er bekommt entweder den vollen Plan
  (8 834 Tokens) oder du baust manuelles Splitting (fehleranfällig, redundant).

### Mechanismus: Wie kommt die Ersparnis zustande?

Die ~10–15 % MDAI-spezifische Ersparnis in Source/Render entstehen aus drei Mechaniken:

**(a) `@define` / `@call` — Macro-Expansion für wiederholte Blöcke**

Beispiel: Der „reformat → git add → git commit"-Schritt taucht im Plan 7× auf (einer
pro Task). Ohne MDAI ist das 7× ein 3-Zeilen-Block (≈21 Zeilen, ~600 Bytes).
Mit MDAI:

```
@define stepReformatCommit(file)
- `mcp__jetbrains__reformat_file {{ file }}`
- `git add {{ file }}`
- `git commit -m "..."`
@end

@call stepReformatCommit(crates/rpc/src/lib.rs)   ← 1 Zeile, ~50 Bytes
```

Source-Ersparnis: 7 × (600 − 50) = **3 850 Bytes**. Der Renderer expandiert beim
Lesen wieder zum vollen Block, also bleibt der Output-Inhalt vollständig.

**(b) `@include` — gemeinsame Macro-Dateien für Hard-Rules / Tool-Tabelle**

Hard-Rules und Tool-Quick-Reference sind in allen Refactoring-Plänen identisch.
Ohne MDAI: Copy-Paste in jeden Plan (1 671 Bytes pro Plan × N Pläne).
Mit MDAI: 1× `macros/hard-rules.md` + `macros/tool-quick-ref.md`, jeder Plan
schreibt `@include macros/hard-rules.md`.

Ersparnis steigt linear mit der Anzahl der Pläne. Für einen einzelnen Plan
ist sie null (Inhalt taucht ja im Render trotzdem auf), aber für N Pläne sparst
du (N − 1) × 1 671 Bytes **Schreibarbeit + Sync-Aufwand bei Änderungen**.

**(c) `@constraint` — strukturierter Hard-Rule-Block statt freier Prosa**

```
@constraint id="hard-rules" severity="high"
Cargo: nextest run (nie cargo test). Vor git add: reformat_file.
@end
```

Wird zu `> **CONSTRAINT [hard-rules] — HIGH** ...` (~120 Bytes) statt eines
mehrzeiligen prosa-Blocks (~500 Bytes). Plus: `mcp__markdownai__get_constraints`
liefert sie strukturiert als JSON (`[{id, severity, body}]`), nutzbar für
Subagent-Briefings oder Hook-Validierung.

**(d) MCP-`read_file(phase=…)` — die strukturelle Phase-Isolation**

Das ist der eigentliche Hebel und der einzige Mechanismus, der ohne MDAI gar nicht
existiert. Der Server parst die `@phase … @end`-Marker, gibt **nur den umschlossenen
Inhalt** zurück, und prepended automatisch Constraints + Frontmatter + Hard-Rules
(letzteres ist Sinn: der Subagent muss die Regeln kennen).

Phase A3 isoliert: 2 818 Bytes / **704 Tokens**, davon ~480 Tokens für Hard-Rules
und Constraints (Overhead, by design), ~224 Tokens für den eigentlichen
A3-Phase-Inhalt.

### Realistischer Workflow-Gewinn

**Szenario:** Wir arbeiten S3a (7 Phasen, 8 Task-Blöcke) mit 7 dispatchen Subagents ab,
einer pro Phase.

| Variante                                        | Tokens pro Subagent | Gesamt (7 Subagents) |
|-------------------------------------------------|--------------------:|---------------------:|
| Ohne MDAI: jeder Subagent kriegt vollen Plan    |               8 834 |           **61 838** |
| Mit MDAI: jeder Subagent kriegt nur seine Phase |            ~700–800 |           **~5 250** |

Differenz: **~56 600 Tokens** = Faktor 12 weniger Eingabe für die Subagent-Phase.
Bei Sonnet-Preisen (3 USD / 1 M Input-Tokens) sind das ~17 Cent pro Plan-Durchlauf.
Bei Opus-Preisen ~85 Cent. Das skaliert linear mit der Anzahl ähnlicher Pläne.

### Wo MDAI NICHT hilft

| Situation                                                   | Warum kein Gewinn                                                             |
|-------------------------------------------------------------|-------------------------------------------------------------------------------|
| One-Shot-Plan mit einem einzigen linearen Subagent          | Phase-Isolation bringt nichts, wenn nur ein Agent eh alles liest              |
| Plan ohne Wiederholungs-Muster                              | `@call` spart nur, wenn es 3+ identische Blöcke gibt                          |
| Einzelner Plan ohne weitere ähnliche Pläne im Repo          | `@include`-Macros lohnen sich erst ab 2–3 Plänen, die dieselben Macros teilen |
| Plan mit viel Erklärungs-Prosa, die alle Subagents brauchen | Diese Prosa landet im rendered Output sowieso, MDAI ändert nichts daran       |
| Erste Plan-Lesung durch Orchestrator-Claude                 | Der lädt den vollen `format=ai`-Output (2 266 Tokens) — wie ohne MDAI auch    |

**Bottom Line:** Der echte, nicht-marginale Token-Gewinn liegt bei
**parallelem Subagent-Dispatch über `mcp__markdownai__read_file(phase=…)`**.
Die ~74 %-Zahl im Voll-Render ist überwiegend manuelle Verdichtung und damit
„hätte man auch ohne MDAI haben können". Wer MDAI **nur** für Source-Verkleinerung
einsetzt und nicht für Dispatch-Routing, hebt das eigentliche Token-Potential nicht.

---

## Vergleich mit v1-Ergebnis

| Metrik                 | v1 (2026-05-20)         | v2 (2026-05-21)       | Differenz       |
|------------------------|-------------------------|-----------------------|-----------------|
| Original Bytes         | 35 336                  | 35 336                | identisch       |
| MDAI Source + Macros   | 19 024                  | 8 913                 | −10 111 B       |
| MDAI rendered          | 16 399                  | 9 062                 | −7 337 B        |
| Rendered vs. Original  | −54 %                   | **−74 %**             | +20 % Ersparnis |
| `@include` Hard-Rules  | fehlte (falsch @import) | ✅ sichtbar            | korrigiert      |
| `@call` Expansion      | nicht vorhanden         | ✅ 7× expandiert       | neu             |
| `@phase title=`-Fehler | als Bug gemeldet        | kein Feature, korrekt | korrigiert      |
| `@import` als „Bug"    | als Bug gemeldet        | by design, korrekt    | korrigiert      |

---

## Empfehlung

**Lohnt sich MDAI bei echten Rust-Refactoring-Plänen?**

| Szenario                                                  | Empfehlung                                                             |
|-----------------------------------------------------------|------------------------------------------------------------------------|
| Plan mit ≥3 Tasks, jeder mit gleichem Commit-Muster       | **Ja** — `@define`/`@call` spart Duplikate, expandiert korrekt         |
| Mehrere Pläne teilen dieselben Hard-Rules / Tool-Tabellen | **Ja** — `@include` aus gemeinsamen Macro-Dateien, eine Stelle pflegen |
| Plan mit viel Erklärungsprosa ohne Struktur-Redundanz     | **Bedingt** — manuelle Verdichtung nötig, MDAI allein reicht nicht     |
| Selektives Phase-Rendering gewünscht (nur A3 an Subagent) | **Ja, sobald** `mai render --phase <id>` implementiert ist             |

**Konkrete nächste Schritte (falls Integration gewünscht):**

1. Phase-Isolation läuft bereits via MCP (`mcp__markdownai__read_file` mit `phase=`).
   Kein zusätzlicher `mai render --phase`-CLI-Flag nötig — Live-Messung: 704 Tokens
   für A3 statt 8 834 für den vollständigen Original-Plan.
2. Macro-Bibliothek für Standardmuster aufbauen: Hard-Rules, Tool-Ref, Commit-Schritte
   (Vorbild: `tmp/mdai-bench/macros/`).
3. Pläne künftig direkt in MDAI-Syntax schreiben statt nachträglich zu konvertieren.
4. **MCP-Protocol-Compliance-Patch upstream einreichen** (`respondTool()` in
   `markdownai/packages/mcp/src/server.ts`) — sonst geht der Patch bei jedem
   `npm install` / `git pull` im markdownai-Repo verloren.
5. Skill-Wrapper bauen, die `mcp__markdownai__*` für Subagent-Dispatch nutzen
   (`mdai-plans`, `mdai-execution`, `mdai-memory`) — siehe Brainstorming.
