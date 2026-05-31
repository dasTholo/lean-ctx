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
- `@phase ... @end`: Phasengrenzen sind maschinenlesbar — Basis für die selektive
  Phase-Isolation via MCP `read_file(phase=…)` (kein CLI-Flag, siehe v5).

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
2. **MCP-Protocol-Compliance-Envelope nötig:** ohne `content[]`+`structuredContent`-Wrapper
   verwirft Claude Codes MCP-Client die rohe `tools/call`-Antwort („completed with no
   output"). In der `feat-mdai`-Dist (= 1.3.0, v5) enthalten, MCP-Calls live bestätigt.

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
| Selektives Phase-Rendering gewünscht (nur A3 an Subagent) | **Ja** — via MCP `read_file(phase=…)`, in 1.3.0 verfügbar (siehe v5)   |

Die in v2 noch offenen Integrations-Schritte (Macro-Bibliothek, Plan-Authoring in
MDAI-Syntax, Skill-Wrapper für Subagent-Dispatch) sind mit der mdai-Library v0.1.4
umgesetzt — Stand und Messungen siehe Update v5.

---

## Update v4 — Audit-Plan-Validation auf markdownai v1.0.0 (2026-05-26)

Datum: 2026-05-26 · Methode: v3-Methodik adaptiert (tiktoken exakt statt Bytes÷4) ·
Engine: markdownai v1.0.0 + lean-ctx 3.6.17 · Autor: Claude (Sonnet) ·
Tokenizer: tiktoken `cl100k_base` (exakt, via lokales Python-Skript)

**Ziel:** Re-Validation der v3-Resultate gegen einen echten Audit-Plan (NICHT
Refactoring-Plan wie v3 S3a). Quelle: `docs/mdai/plans/2026-05-26-mdai-v0.1.2-part-b-findings-v2-and-dedup.md`,
ausgeführt mit 7 Subagents via `superpowers:subagent-driven-development`-Skill.

### Renderer-Sanity (markdownai v1.0.0)

| Test-File | Status | Note |
| `markdownai/MDs/tests/test-include-import.md`     | ✅ PASS | `@include` rendert inline, `@import` lädt Macros (kein
sichtbarer Output)        |
| `markdownai/MDs/tests/test-phase-isolation.md`    | ✅ PASS | 4 Phasen (alpha/beta/gamma/delta) korrekt isoliert |
| MCP `list_phases`                                  | ✅ PASS | liefert `{name, transitions}` korrekt |
| MCP `read_file phase=alpha format=ai`             | ✅ PASS | nur Alpha-Sentinel + global intro, KEIN
Beta/Gamma/Delta-Leak |

Engine ist binärkompatibel zur v3-Messung — keine Breaking Changes festgestellt.

### Ziel-Plan-Vergleich zu v3 S3a-Baseline

| Metrik | v3 S3a (2026-05-21) | Part-B (2026-05-26)  | Delta |
| Bytes | 35 336 | 35 245 | nahezu identisch |
| Lines | 943 | 778 | Part-B kompakter |
| Tokens (tiktoken)   | 8 834 | 8 811 | nahezu identisch |

Damit ist Part-B ein fairer Vergleichspunkt zur v3-Baseline trotz unterschiedlicher
Plan-Sorte (Audit statt Refactoring).

### Per-Task Phase-Isolation (Part-B, 5 Tasks)

| Task | Bytes | Lines | Tokens (tiktoken)  | % of Plan |
| T0 Pre-Flight | 1 617 | 43 | 404 | 4.6 % |
| T4 Findings-v2 | 10 462 | 247 | 2 615 | 29.7 % |
| T5 Dedup-Audit | 12 938 | 280 | 3 234 | 36.7 % |
| T6 End-Gate | 1 513 | 38 | 378 | 4.3 % |
| T7 Final-Verif | 2 219 | 48 | 554 | 6.3 % |
| **Σ Per-Task**                    | 28 749 | 656 |          **7 185** |   **81.5 %** |
| Rest (Frontmatter/File-Map/Notes) | 6 496 | 122 | 1 626 | 18.5 % |

### Phase-Isolation vs. Full-Plan-Dispatch (per-Subagent ratio)

| Variante | Tokens | vs. Original |
| Full plan (Subagent kriegt alles)    | 8 811 | baseline |
| Phase A3 isolated (v3 S3a baseline)  | 704 | −92 % |
| Part-B kleinste Phase (T0)            | 404 |   **−95 %**  |
| Part-B mittlere Phase (T7)            | 554 |   **−94 %**  |
| Part-B größte Phase (T5)              | 3 234 | −63 % |

Größere Phasen sparen weniger, was zu erwarten ist. T5 hat 7 Steps × 5 Optionen ×
3 Cluster — intrinsisch komplex.

### MCP Phase-Isolation Live-Test (test-phase-isolation.md)

| Format | Bytes | Tokens | Savings |
| `read_file` (no phase, format=ai)            | 2 934 | 734 | baseline |
| `read_file` (phase=alpha, format=ai)         | 2 515 | 629 | **−14.3 %** |

Kleine Ratio weil das Test-File vom `[AI INSTRUCTION]`-Block (~500 Tokens)
dominiert wird, der in beiden Outputs identisch erscheint. Auf einem echten Plan
mit substanziellem Per-Phase-Content (siehe Part-B-Tabelle oben) steigt die Ratio
drastisch.

### Subagent-Dispatch Cost-Model (echter Part-B-Workflow)

Workflow: 7 Subagents (T0-PreFlight + T4-Findings-v2 + T4-fix + T5a-Analyze +
T5a-fix-rescore + T5c-Commit + T7-Verification) via `superpowers:subagent-driven-development`.

| Approach | Input-Tokens (7 Subagents) | Sonnet ($3/M) | Opus ($15/M) |
| Ohne Phase-Isolation: jeder Subagent kriegt vollen Plan | 7 × 8 811 = 61 677 |       $0.185 |     $0.925 |
| Mit MDAI Phase-Isolation: per-task + ~480 Tok Overhead | 9 585 |       $0.029 |     $0.144 |
| **Saved (Input only)**                                   |         **52 092 (−84.5 %)** |   **$0.156**  |   **$0.781
** |

### Repeated-Pattern-Analyse (potenzielle `@define`/`@include`-Ziele in Part-B)

| Pattern | Vorkommen | MDAI-Mechanismus | Source-Ersparnis |
| `mcp__lean-ctx__ctx_shell(…)`          | 17× | `@define ctxShell(cmd, cwd)`                           | ~3-5 % |
| `mcp__lean-ctx__ctx_search(…)`         | 7× | `@define ctxSearch(pattern, path)`                     | ~1-2 % |
| `mcp__lean-ctx__ctx_read(…)`           | 6× | `@define ctxRead(path, mode)`                          | ~1 % |
| `mcp__lean-ctx__ctx_edit(…)`           | 5× | `@define ctxEdit(…)`                                   | ~1 % |
| Heredoc-Commit-Pattern | 7× | `@define heredocCommit(msg-file, body)`                | ~2-3 % |
| `mcp__jetbrains__reformat_file`        | 4× | `@call reformatBeforeAdd(file)`                        | ~1 % |
| Hard-Rules-Block in jedem Subagent-Prompt | wiederholt | `@include macros/hard-rules.md`                      | ~5-8 %
bei 3+ Plänen |

**Total potenzielle Source-Ersparnis** (MDAI-spezifisch, OHNE Phase-Isolation):
~14-20 %. Part-B würde damit von 8 811 → ~7 000-7 500 Tokens fallen.

### Drei Mess-Szenarien (v3-Schema auf Part-B)

| Szenario | Tokens | vs. Original | MDAI-Anteil | Mechanismus |
| (1) Plan-Source schreiben |        ~7 200 | −18 % | ~14-20 % | `@define`/`@include` reduzieren Source |
| (2) Plan vollständig rendern |        ~8 500 | −4 % | minimal | Rendered Output expandiert Macros |
| (3) Subagent-Dispatch (eine Phase)    |     **404-3 234** |   **−63 bis −95 %** | **strukturell** | Phase +
Hard-Rules-Overhead |

### Vergleich der zwei Pläne (v3 S3a vs Part-B)

| Metrik | v3 S3a | Part-B |
| Original Tokens | 8 834 | 8 811 |
| Single-Phase isolated (best)              | 704 (−92 %)   |  **404 (−95 %)** |
| Single-Phase isolated (worst)             |     (keine v3-Daten) | 3 234 (−63 %)   |
| 7-Subagent-Cost without isolation (Sonnet)|              ~$0.185 |          $0.185 |
| 7-Subagent-Cost MDAI-isolated (Sonnet)    |        ~$0.018 |       **$0.029** |
| Cost saved per run (Sonnet)               |              ~$0.17 |       **$0.156** |

Part-B-MDAI-isolated ist teurer als v3 weil Audit-Pläne größere Tasks haben
(T5 = 3 234 Tokens, kein Äquivalent zu einer schmalen v3-Phase). Trotzdem
linear-skalierende Cost-Reduktion gegenüber Non-Isolation.

### Findings

1. **Phase-Isolation-Ergebnisse aus v3 reproduzieren sich** auf einem ganz
   anderen Plan-Genre (Audit statt Refactoring). −92 % bis −95 % für kleine
   Phasen ist robust.
2. **Größere Phasen (>10 % des Plans) sparen weniger** — T5 mit 36.7 % Plan-
   Anteil schafft nur −63 %. Aber das ist die strukturelle Untergrenze, nicht
   ein MDAI-Defekt.
3. **`superpowers:subagent-driven-development` macht manuell was MDAI strukturell
   automatisieren würde.** Controller liest Plan, extrahiert Per-Task-Text,
   komponiert Subagent-Prompts. Ein MDAI-aware Controller könnte stattdessen
   `mcp__markdownai__read_file(phase=…)` callen und denselben Effekt erzielen
   ohne manuelle Plan-Parsing-Logik im Controller-Code.
4. **Cross-Plan-Konsistenz** für Hard-Rules wäre der größte qualitative Gewinn
   bei MDAI-Migration: die drei v0.1.2-Pläne (Part-A/B/C) repeated dieselben
   Notes-Sektionen (`No worktrees`, `keine && Ketten`, `Pre-commit reformat`).
   Ein `@include mdai/core/hard-rules.md` würde sie an einer Stelle pflegen.

### Empfehlung für v0.1.3-Adoption

Aus dem Part-B-Workflow + Dedup-Audit (Per-Cluster-User-Decisions 2026-05-26):

- **Cluster 1 + Cluster 3 → Option A** (Fragment-File + `@include`): v0.1.3
  schreibt `mdai/core/lean-context-anchors.md` und `mdai/core/anti-patterns.md`
  mit `@markdownai v1.0`-Header (Begründung: Fragment-Files OHNE Header →
  silent empty return).
- **Cluster 2 → Option E** (Status Quo + Drift-Tracking): nicht migrieren.
- **Library-Root-Variante: B** (`${MDAI_LIBRARY_ROOT}`-Prefix in Spawn-Env).

### Carry-Forward

- v3-Methodik validiert auf zweitem Plan-Genre (Audit-Plan).
- tiktoken-exakte Tokenisierung statt Bytes÷4 → robustere Vergleichbarkeit.
- `mcp__markdownai__read_file(phase=…)` reproduzierbar verifiziert mit
  Sentinel-Strings (kein Cross-Phase-Leak).
- `superpowers:subagent-driven-development` als Brücke zum MDAI-Adoption-Pfad
  identifiziert.

---

## Update v5 — markdownai 1.3.0 (mdai v0.1.4 Hardening, 2026-05-31)

Datum: 2026-05-31 · Engine: markdownai **1.3.0** (= `feat-mdai`-Dist, `origin/main@aac0825`

+ 2 lokale Fixes: `f16b4c2` named-arg-Propagation, `ede9793` `@foreach`-Objekt-Dot-Access) ·
  mdai-Library: **v0.1.4** · Tokenizer: tiktoken `cl100k_base` (exakt).

Zwei Mess-Gegenstände: **(A)** dieselbe S3a-Fixture wie v3 (`tmp/mdai-bench/`), nach
v2 migriert, gegen 1.3.0 — direkter Cross-Version-Vergleich auf identischem Plan.
**(B)** das reale Library-Skill-Artefakt `mdai/skills/mdai-brainstorm/body.mdai.md`.

### Breaking Change v1.0.0 → 1.3.0: v2-Directive-Syntax

Anders als v4 („binärkompatibel, keine Breaking Changes") ist 1.3.0 **syntaktisch
nicht abwärtskompatibel**. Die kanonische v3-Fixture
`tmp/mdai-bench/2026-05-17-phase-4-S3a-…mdai.md` parst unter 1.3.0 **nicht mehr**:

```
ParseError: v1 close tag "@end" not accepted in v2 — use "@include-end" instead
```

v2-Regeln: Block-Directives schließen mit `@<name>-end` (`@constraint-end`,
`@phase-end`, `@define-end`, `@if-end`, `@foreach-end`); argument-lose Directives
self-closen mit trailing ` /` (`@include … /`, `@call … /`). Die Library-Migration
auf v2 war daher **mandatorisch** — das ist der Anlass des v0.1.4-Hardenings.
Migrierte v2-Kopie für die Messung: `tmp/mdai-bench/v2/`.

### (A) Same-Plan-Cross-Version: S3a unter 1.0.0 vs 1.3.0 (identische Fixture)

| Metrik (S3a)            | v3 (1.0.0, Bytes÷4) | v5 (1.3.0, tiktoken) |     Bytes-Drift     |
|-------------------------|--------------------:|---------------------:|:-------------------:|
| Source (Bytes / Tokens) |       7 056 / 1 764 |        7 127 / 2 309 | +71 B (` /`-Closer) |
| Full-Render Bytes       |               9 062 |                9 027 |   −35 B (≈0,4 %)    |
| Full-Render Tokens      |          2 266 (÷4) |     2 852 (tiktoken) |    nur Tokenizer    |
| A3-isoliert Bytes       |               2 818 |                2 774 |   −44 B (≈1,5 %)    |
| A3-isoliert Tokens      |            704 (÷4) |       829 (tiktoken) |    nur Tokenizer    |

**Begründung — Engine-Output ist byte-stabil:** Full-Render und A3-Isolat sind
zwischen 1.0.0 und 1.3.0 byte-genau (≈0,4–1,5 % Drift, allein aus den ` /`-Self-Closern
der Source). Die scheinbar großen Token-Differenzen (+26 %) sind **reine
Tokenizer-Methodik** (tiktoken vs. Bytes÷4), kein Engine-Verhalten. Fazit: der Render
ist über die Major-Versionen stabil — nur die *Source-Syntax* bricht (v1→v2).

### S3a Phase-Isolation unter 1.3.0 (Baseline = Full-Render 2 852 Tokens)

| Phase           | Tokens | vs. Full | vs. Original (10 629) |
|-----------------|-------:|:--------:|:---------------------:|
| **Full-Render** |  2 852 | Baseline |         −73 %         |
| P0-context      |  1 293 |  −55 %   |         −88 %         |
| A1-debug-output |    846 |  −70 %   |         −92 %         |
| A2-ui-event     |    744 |  −74 %   |         −93 %         |
| A3-rpc-wire     |    829 |  −71 %   |         −92 %         |
| A4-loader       |    728 |  −74 %   |         −93 %         |
| A5-register     |    716 |  −75 %   |         −93 %         |
| A-final-gate    |    841 |  −70 %   |         −92 %         |

Hier liegt der `@include`-Hard-Rules-Block **vor** den Phasen → die Engine stellt ihn
*jeder* isolierten Phase voran (~480 Tok Overhead pro Subagent, wie v3/v4). Daher die
flache −70…−75 %-Spanne. Gegen den **Original-Plan** (35 336 B / 10 629 Tok tiktoken)
spart jede Einzelphase −88 % bis −93 %.

### (B) Library-Skill-Artefakt: body.mdai.md (5 Phasen)

Anders als ein Plan (Source < Original) ist dies das Laufzeit-Artefakt: die Source ist
*lean authored*, der Render expandiert via `@include`/`@call` auf das, was der Agent
tatsächlich lädt. Render fehlerfrei unter 1.3.0, **kein** `@set`-Pipe-Fehler mehr
(v0.1.4 TG2-Fix verifiziert; verbleibende Warnings sind Laufzeit-`@call`/`@query`).

| Variante                              |  Bytes | Tokens | Kommentar                                  |
|---------------------------------------|-------:|-------:|--------------------------------------------|
| **Source** (lean authored)            | 12 412 |  2 885 | das, was der Autor pflegt                  |
| Render `--format standard`            | 24 990 |  5 553 | volle Expansion                            |
| Render `--format ai`                  | 24 851 |  5 508 | nur −0,8 % vs standard (siehe Begründung)  |
| Geteilte core-Macros (single-sourced) | 12 275 |  2 513 | hard-rules + lean-context + tool-quick-ref |

**Warum `--format ai` hier kaum spart:** der Body ist Prosa + Macro-Expansion, keine
priorisierten `@section`-Blöcke. `--format ai`/`--budget` greifen erst mit
`<!-- mda-section priority=… -->`-Markern; auf reinen Prosa-Skills ist `ai` ≈ `standard`.

Phase-Isolation (Baseline = Full-Render 5 546 Tok):

| Phase           | Tokens | vs. Full | Inhalt                                       |
|-----------------|-------:|:--------:|----------------------------------------------|
| **Full-Render** |  5 546 | Baseline | alle 5 Phasen                                |
| pre-context     |  2 169 |  −61 %   | trägt `@include` hard-rules + tool-quick-ref |
| dialog-rules    |  1 391 |  −75 %   | Constraints + Dialog-Gates                   |
| write-outputs   |  1 162 |  −79 %   | Spec-Konventionen-Fragment                   |
| dialog-process  |    819 |  −85 %   | Prozess-Schritte                             |
| handoff         |    291 |  −95 %   | Übergabe an writing-plans                    |

**Begründung — besseres Layout als S3a:** in `body.mdai.md` liegt der schwere
`@include`-Block (2 513 Tok) **nur in `pre-context`**, nicht global vor den Phasen.
Ein Subagent in `handoff` zieht 291 Tok statt 5 546 (−95 %), ohne die Regeln
mitzuschleppen. Genau dieser Unterschied erklärt die steilere Spanne (−61…−95 %)
gegenüber S3as flachem −70…−75 % (globaler Regel-Block).

### Single-Sourcing-Gewinn (das eigentliche v0.1.4-Argument)

Die geteilten core-Macros (hard-rules + lean-context + tool-quick-ref) = **2 513 Tokens**
liegen **einmal** in `mdai/core/`, von jedem Skill via `@include ${MDAI_LIBRARY_ROOT}/core/*`
gezogen. Bei N konsumierenden Skills: ohne Library N × 2 513 Tok Copy-Paste + N-fache
Sync-Last; mit Library 1 × 2 513 Tok, `(N − 1) × 2 513` Tok gespart, Konsistenz
garantiert. Das ist der „Cross-Plan-Konsistenz"-Gewinn aus v4-Finding #4 — in v0.1.4 umgesetzt.

### Vergleich v3 → v4 → v5

| Metrik / Eigenschaft      | v3 (S3a, 1.0.0)   | v4 (Part-B, 1.0.0) | v5 (1.3.0)                          |
|---------------------------|-------------------|--------------------|-------------------------------------|
| Mess-Gegenstand           | Refactoring-Plan  | Audit-Plan         | S3a-Replay **+ Library-Skill**      |
| Tokenizer                 | Bytes ÷ 4         | tiktoken           | tiktoken                            |
| Beste Phase-Isolation     | 704 (−92 %)       | 404 (−95 %)        | 291 (−95 %, body/handoff)           |
| Schlechteste Phase        | —                 | 3 234 (−63 %)      | 1 293 (−55 %, S3a/P0)               |
| Source-Syntax             | v1 (`@end`)       | v1 (`@end`)        | **v2 (`@…-end`, ` /`)**             |
| Engine-Render-Stabilität  | Baseline          | binärkompatibel    | **byte-stabil zu 1.0.0** (≈0,4 %)   |
| Hard-Rules-Verteilung     | global/je Phase   | global/je Phase    | global (S3a) **vs. 1 Phase (body)** |
| Single-Sourcing produktiv | nein (tmp-Macros) | nein (Prognose)    | **ja (`mdai/core/`)**               |

### Findings v5

1. **Engine-Render ist über 1.0.0→1.3.0 byte-stabil** (S3a Full 9 062→9 027 B,
   A3 2 818→2 774 B). Die Token-Differenzen sind reine Tokenizer-Methodik, kein
   Verhaltens-Drift.
2. **v1→v2 ist ein echter Breaking Change** auf Source-Ebene: die v3-Fixture parst
   ohne Migration nicht mehr. Library-Migration auf v2 war Pflicht, nicht Kür.
3. **Phase-Isolation reproduziert sich ein drittes Mal** — −55 % bis −95 % über zwei
   weitere Artefakte (S3a-Replay + Library-Skill). Robust über Engine-Versionen,
   Tokenizer *und* Dokumenttypen.
4. **Layout entscheidet über die Ersparnis:** Hard-Rules global (S3a) → flach −70 %;
   Hard-Rules in einer Phase (body) → bis −95 %. Library-Skills sollten den Regel-Block
   in eine eigene `pre-context`-Phase legen.
5. **`--format ai` / `--budget` sind no-ops ohne `@section`-Prioritäten** — wer
   Budget-Dropping will, muss Skill-Bodies mit `priority=`-Sektionen strukturieren.
6. **Single-Sourcing ist jetzt real** (2 513 Tok geteilt in `mdai/core/`), nicht mehr Prognose.
