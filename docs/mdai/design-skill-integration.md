# MDAI ↔ lean-ctx ↔ superpowers — Skill-Integration

Datum: 2026-05-21 · Status: **v3 freigegeben** · Bezug: `mdai-benchmark.md` (v3) · markdownai v0.0.22 (+lokaler
`respondTool`-Patch)

---

## 1. Zielsetzung

Drei Wrapper-Skills bauen, die MarkdownAI-Direktiven (`@phase`, `@include`,
`@define`/`@call`, `@constraint`) im bestehenden superpowers/lean-ctx-Workflow
nutzbar machen — **ohne** `superpowers` selbst zu verändern.

**Erfolgskriterien:**

1. Pläne können in MDAI-Syntax geschrieben werden und werden vom Plan-Hook akzeptiert.
2. Subagent-Dispatch kann eine einzelne Phase via `mcp__markdownai__read_file(phase=…)`
   anfordern und arbeitet damit (empirisch: 704 Tokens statt 8 834).
3. Bestehende superpowers-Skills (`writing-plans`, `executing-plans`,
   `subagent-driven-development`) bleiben unverändert lauffähig für Pläne, die
   kein MDAI nutzen.
4. Macro-Bibliothek-Wartbarkeit: Eine Änderung an einer geteilten Macro-Datei
   (`hard-rules.md`, `tool-quick-ref.md`, `step-reformat-commit.md`) propagiert
   beim nächsten Render automatisch in alle Pläne, die sie via `@include` /
   `@import` referenzieren — keine manuellen Duplikate-Updates.

---

## 2. Empirische Grundlagen (Bezug: `mdai-benchmark.md`)

| Messpunkt               |  Tokens | vs. Original | MDAI-Anteil     |
|-------------------------|--------:|-------------:|-----------------|
| Source schreiben        |   2 228 |        −75 % | ~10–15 %        |
| Voll-Render             |   2 266 |        −74 % | ~10–15 %        |
| **MCP-Phase-Isolation** | **704** |    **−92 %** | **strukturell** |

Workflow-Gewinn bei 7 Subagents über S3a: **61 838 → 5 250 Tokens** (Faktor 12).

Verifizierte Voraussetzungen:

- `mcp__markdownai__list_phases`, `get_constraints`, `read_file(phase=, format=ai)`
  funktionieren live in Claude Code (nach Server-Patch `respondTool()` in
  `markdownai/packages/mcp/src/server.ts`).
- Relative Pfade nötig (`cwd` = Projekt-Root, by default).
- MCP-Protocol-Patch lokal angewendet — Upstream-PR ist offener Punkt.

---

## 3. Architektur-Überblick

```
                        ┌─────────────────────────────────┐
                        │   superpowers (UNVERÄNDERT)     │
                        │   • writing-plans               │
                        │   • executing-plans             │
                        │   • subagent-driven-development │
                        └────────────────┬────────────────┘
                                         │  (User triggert wahlweise)
                ┌────────────────────────┴───────────────────────┐
                │                                                │
       ┌────────▼────────┐                              ┌────────▼────────┐
       │  Klassischer    │                              │   MDAI-Pfad     │
       │  .md-Plan       │                              │  .mdai.md-Plan  │
       └────────┬────────┘                              └────────┬────────┘
                │                                                │
                │                                                ▼
                │                                  ┌───────────────────────────┐
                │                                  │   Wrapper-Skills (NEU)    │
                │                                  │   • mdai-plans            │
                │                                  │   • mdai-execution        │
                │                                  │   • mdai-memory           │
                │                                  └────────┬──────────────────┘
                │                                           │
                │                                           ▼
                │                                  ┌───────────────────────────┐
                │                                  │ mcp__markdownai__*        │
                │                                  │ • list_phases             │
                │                                  │ • read_file(phase=, ai)   │
                │                                  │ • get_constraints         │
                │                                  └───────────────────────────┘
                │
                └─────► Hooks ENTFERNT (Skill-only-Ansatz, §7):
                        ⌧ plan-discipline.py
                        ⌧ skill-plan-injector.py
                        Discipline-Logik lebt in den Skill-Bodies selbst.
```

**Kernprinzip:** Die drei neuen Skills sind *zusätzliche Trigger* — sie ersetzen
keinen superpowers-Skill. Wer einen klassischen `.md`-Plan schreibt, bleibt im
alten Flow. Wer `.mdai.md` schreibt, landet automatisch im neuen Flow.

---

## 4. Skill A: `mdai-plans`

**Trigger-Description (für Skill-Front-Matter):**

> Use when writing a multi-step implementation plan that will be executed by
> multiple parallel subagents. Produces a `.mdai.md` file with `@phase` markers
> for per-phase token-isolated dispatch.

**Eingaben:**

- Eine Spec oder Anforderung (gleich wie `superpowers:writing-plans`)
- Hinweis, ob der Plan parallel-dispatched wird (sonst `writing-plans` empfehlen)
- Optional: `--from-issue <provider>:<id>` (siehe §7a.3) — holt Issue-Details
  via `ctx_provider` als Plan-Skeleton

**Output:**

- Ein `.mdai.md` File mit:
    - YAML-Frontmatter (`id`, `status`, `mdd_version`)
    - `@markdownai v1.0`-Header
    - `@include macros/hard-rules.md` und `@include macros/tool-quick-ref.md`
      (sofern vorhanden)
    - Pro Task ein `@phase <id>` … `@end`-Block mit `## <Title>`-Heading direkt
      darunter
    - Wiederholte Muster als `@define`/`@call`

**Größe:** ~80 Zeilen Markdown.

**Workflow-Schritt:**

1. Brainstorming (delegiere an `superpowers:brainstorming`)
2. Spec-Klärung (delegiere an `superpowers:writing-plans` für die Strukturierungs-
   Disziplin, aber Output-Format auf `.mdai.md` umlenken)
3. Macro-Datei-Check: existieren `tmp/mdai-bench/macros/hard-rules.md` etc.
   bereits? Wenn ja, `@include`. Wenn nein, einmalig anlegen.
4. `mai render <plan>.mdai.md` → Sanity-Check, dass alles expandiert
5. `mcp__markdownai__list_phases <plan>` → Phasen-IDs in Plan-Kommentar
   dokumentieren

**Was es NICHT macht:** Es schreibt den Plan-Inhalt nicht selbst. Die Logik
kommt aus `superpowers:writing-plans`, dieser Skill ist nur die Brille zur
MDAI-Syntax.

---

## 5. Skill B: `mdai-execution`

**Trigger-Description:**

> Use when executing a `.mdai.md` plan with parallel subagent dispatch.
> Each subagent receives only its phase via `mcp__markdownai__read_file`,
> not the full plan — saves ~92 % input tokens per subagent.

**Eingaben:**

- Pfad zu einer `.mdai.md`-Datei
- Optional: Liste der zu bearbeitenden Phasen (default: alle aus `list_phases`)

**Workflow-Schritt:**

1. `mcp__markdownai__list_phases <plan>` → alle Phasen-IDs holen
2. `mcp__markdownai__get_constraints <plan>` → Constraints in Subagent-Briefing übernehmen
3. `lean-ctx gotchas list --tag mdai` → bekannte Pitfalls anzeigen (§7a.5)
4. Für jede Phase einen Subagent dispatchen mit:
    - Vorab: `lean-ctx control pin "mdai-active-phase:<id>" --scope session` (§7a.4)
    - Prompt enthält **nur** den Aufruf
      `mcp__markdownai__read_file(<plan>, phase=<id>, format=ai)`
    - Plus 2–3 Zeilen Meta-Briefing (Constraints sind im read_file-Output schon drin)
    - Nach Abschluss: `lean-ctx control unpin "mdai-active-phase:<id>"`
5. Sammeln der Subagent-Reports, Validierung (Tests, Builds)
6. Auf Subagent-Sequenz achten: bei `@on complete -> @phase X` linear arbeiten;
   sonst parallel via `superpowers:dispatching-parallel-agents`
7. Nach Plan-Abschluss: `lean-ctx gain --tasks --since "<dauer>" --json` →
   reale Token-Ersparnis loggen (§7a.6)

**Größe:** ~120 Zeilen — enthält die Dispatch-Mechanik, Subagent-Prompt-Templates,
Validierungs-Checkliste.

**Verhältnis zu `superpowers:subagent-driven-development`:** parallel, nicht
ersetzend. `subagent-driven-development` arbeitet weiter für klassische `.md`-Pläne.

### 5.1 MDAI-Direktiven an lean-ctx routen

In den Plänen sollen Discovery-/Read-Direktiven konsequent auf lean-ctx-Tools
zeigen — die liefern komprimierte Outputs und werden via `[autonomy].auto_dedup`
automatisch memoisiert.

| MDAI-Direktive im Plan       | Soll-Befehl im `@query`-Body                 | Begründung                                 |
|------------------------------|----------------------------------------------|--------------------------------------------|
| `@tree . depth=N`            | Built-in MDAI-Directive (kein Wrapper nötig) | nativ vorhanden, siehe `test-rendering.md` |
| `@query <directory-listing>` | `lean-ctx ctx_tree <path> --depth=N`         | Kompakter als `ls -R` / `find`             |
| `@query <code-search>`       | `lean-ctx ctx_search <pattern> <path>`       | Token-effizient statt `grep -r`/`rg`       |
| `@query <file-read>`         | `lean-ctx ctx_read <path> mode=lines:N-M`    | Cached, ~13 Tokens bei Re-Read             |
| `@query <shell-op>`          | `lean-ctx ctx_shell "<cmd>"`                 | 95+ Kompressions-Pattern für git/npm/cargo |

**Auto-Memoisierung:** lean-ctx Section `[autonomy]` aktivieren in
`.lean-ctx.toml`:

```toml
[autonomy]
auto_dedup = true            # 3+ identische Calls → ab dem 3. aus dem Cache
auto_consolidate = true      # Chunks mit Ranking mergen
auto_preload = true          # Aktive-Inferenz-Prefetch nach Zugriffsmuster
```

Damit ist im Skill **keine** manuelle Memoisierung nötig. Ein Subagent, der in
seiner Phase 5× `@query lean-ctx ctx_tree crates/` macht, zahlt die Tokens
nur einmal.

---

## 6. Skill C: `mdai-memory`

**Trigger-Description:**

> Use at start of a multi-phase MDAI plan execution to register the plan in
> session memory, and at end of each phase to record progress for cross-session
> continuity.

**Eingaben:**

- Pfad zum laufenden `.mdai.md`-Plan
- Aktueller Phase-Stand (oder „start" / „phase-N-done" / „all-done")

**Speichermechanismus — Multi-Layer**, jede Schicht hat eine klare Verantwortung:

| Layer                | Speichert                                                                         | Lebensdauer                | Tool                                                         |
|----------------------|-----------------------------------------------------------------------------------|----------------------------|--------------------------------------------------------------|
| **Plan-State**       | Plan-Metadata: `phases[]`, `current_phase`, `completed[]`, `started_at`, `status` | persistent (cross-session) | `mcp__lean-ctx__ctx_knowledge.remember/recall` (siehe §7a.2) |
| **Session-Snapshot** | Aktive Phase-ID + Plan-Pfad (≤2 KB Hard-Limit!)                                   | bis Session-Ende           | `mcp__lean-ctx__ctx_session`                                 |
| **Subagent-Diary**   | Per-Subagent-Erkenntnisse: Bugs, Workarounds, Tool-Calls die geholfen haben       | persistent                 | `mcp__lean-ctx__ctx_agent action=diary`                      |
| **Phase-Overlay**    | Inhalt der aktuell aktiven Phase, pinned                                          | Session-scoped             | `lean-ctx control pin` (siehe §7a.4)                         |
| **Pitfall-Memory**   | Bekannte MDAI-Stolpersteine, syntax-fehler, etc.                                  | persistent                 | `lean-ctx gotchas` (siehe §7a.5)                             |

**Begründung Multi-Layer statt monolithisch:**

- Unterschiedliche Lifetimes (Session vs. persistent) — ein Tool kann das nicht
  sauber abbilden.
- Unterschiedliche Sichtbarkeit: Subagent-Diary ist privat pro Subagent, Plan-State
  global lesbar.
- 2 KB-Limit von `ctx_session` zwingt zur Trennung: nur Zustand-Pointer rein, nicht
  Plan-Inhalt.

**Workflow-Schritt:**

1. Bei „start":
    - `ctx_knowledge.remember(topic="mdai-plan:<id>", body={phases, started_at, current_phase="P0"})`
    - `ctx_session.snapshot({active_plan: "<path>", phase: "P0"})`
    - `gotchas list --tag mdai` → Pitfalls dem Subagent zeigen
2. Bei „phase-N-start":
    - `lean-ctx control pin "mdai-active-phase:<id>"` mit aktuellem Phase-Inhalt
    - Subagent öffnet privates Diary: `ctx_agent diary --append`
3. Bei „phase-N-done":
    - `ctx_knowledge.remember` mit `current_phase=<next>` + `completed[]` erweitert
    - `ctx_session.snapshot` aktualisieren (nur ID + Pfad — bleibt unter 2 KB)
    - `control unpin "mdai-active-phase:<id>"`
    - Wenn Subagent neue Pitfalls entdeckt: `gotchas add --tag mdai ...`
4. Bei „all-done":
    - `ctx_knowledge.remember` mit `status="done"`
    - `ctx_session.snapshot` leeren
    - `gain --tasks --since "<dauer>" --json` für Audit-Eintrag (siehe §7a.6)
5. In **neuer Session** (Resume):
    - `ctx_knowledge.recall(topic="mdai-plan:<id>")` → kompletter Plan-Stand
    - Bei Bedarf Pin neu setzen für aktive Phase

**Größe:** ~60 Zeilen — jede Layer ist 2–3 helper-Funktionen mit klarem Vertrag.
Komplexer als der schlanke Wrapper, aber löst die Lifetime-Frage sauber.

---

## 7. Hook-Entfernung — Discipline wandert in die Skills

**Entscheidung:** `plan-discipline.py` und `skill-plan-injector.py` werden
**entfernt** (nicht erweitert). Die Disziplin-Logik (Drift-Patterns,
DRIFT_BLOCK-Injektion, Scope-Erzwingung) zieht in die Skills selbst.

**Begründung:**

- Hooks greifen global auf alle Write-/TaskCreate-Calls — kollidieren mit Skills,
  die ihren Scope selbst kennen.
- Hook-Logik in Python lebt parallel zur Skill-Logik in Markdown → doppelte Pflege.
- Skill-Wrapper sind der explizite Ein-/Ausstiegspunkt — natürlicher Ort für
  Discipline-Checks, da der User sie bewusst aufruft.

### 7.1 Was die Hooks heute tun (Ist-Zustand)

| Hook                             | Trigger                                                                        | Aufgabe                                                                             |
|----------------------------------|--------------------------------------------------------------------------------|-------------------------------------------------------------------------------------|
| `plan-discipline.py` (~161 Z)    | PreToolUse `Write` / `TaskCreate` mit Scope `docs/superpowers/(plans\|specs)/` | Block bei Drift-Patterns (Slop-Prosa, „TODO", „ich denke", übermäßige Listen-Tiefe) |
| `skill-plan-injector.py` (~87 Z) | PostToolUse `Skill` für `{writing-plans, requesting-code-review}`              | Injiziert `DRIFT_BLOCK`-Reminder ins Tool-Output                                    |

### 7.2 Wohin die Logik wandert

| Hook-Verantwortung                          | Neuer Ort im Skill                                                                                                                                |
|---------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| Drift-Pattern-Erkennung                     | In `mdai-plans` als Checkliste „nicht-schreiben"-Beispiele direkt im Skill-Body. Selbstkontrolle via Skill-Anweisung, nicht via PreToolUse-Block. |
| Scope-Erzwingung (welcher Pfad ist erlaubt) | Skill schreibt explizit nach `docs/mdai/plans/<id>.mdai.md` — kein freies Wählen, kein Scope-Check nötig.                                         |
| DRIFT_BLOCK-Reminder                        | Wird Teil der `mdai-plans`-Skill-Body (statt PostToolUse-Injektion).                                                                              |
| Validierung Plan-Struktur                   | `mai render <plan>` + `mcp__markdownai__list_phases` zur Skill-Laufzeit (siehe §7a.7)                                                             |

### 7.3 Implikation für klassische `superpowers:writing-plans`-Flow

**Entscheidung getroffen (User, 2026-05-21): Option (a) — Hooks komplett raus,
ohne Ersatz.**

Der klassische `superpowers:writing-plans`-Flow (`.md` unter
`docs/superpowers/plans/`) läuft damit **ohne Discipline-Hook**. Drift-Prävention
hängt allein an der Skill-eigenen Checkliste. Das ist konsequent (Skill-only,
keine parallele Python-Hook-Schicht), birgt aber das Risiko einer Regression
bei klassischen Plänen — siehe §9 (Risiken).

**Konkrete Schritte:**

1. `~/.claude/hooks/plan-discipline.py` löschen
2. `~/.claude/hooks/skill-plan-injector.py` löschen
3. `~/.claude/settings.json` (oder projekt-lokale `settings.local.json`) — Hook-Einträge entfernen
4. Verifizieren: `superpowers:writing-plans` läuft weiter; nur ohne PreToolUse-Block

### 7.4 Kein neuer Hook nötig

`mai render` plus `mcp__markdownai__list_phases` liefern bereits Validierung
zur Skill-Laufzeit. Ein eigenständiger MDAI-Hook würde nur duplizieren.

---

## 7a. Integration mit lean-ctx-Features

Aus der lean-ctx-Doku ziehen wir mehrere Funktionen direkt in die Skills, statt
eigene Mechanik zu bauen.

### 7a.1 Macros als `lean-ctx pack`

**Problem:** Wo lagern `hard-rules.md`, `tool-quick-ref.md`,
`step-reformat-commit.md`? Aktuell unter `tmp/mdai-bench/macros/` — projekt-lokal,
nicht teilbar.

**Lösung:** lean-ctx Context-Pack erstellen:

```bash
lean-ctx pack create mdai-macros tmp/mdai-bench/macros/
lean-ctx pack install mdai-macros            # in jedem Folge-Projekt
lean-ctx pack auto-load mdai-macros          # bei Session-Start automatisch laden
```

Die Macros sind dann projekt-übergreifend verfügbar. `@include macros/hard-rules.md`
funktioniert in jedem Repo identisch. **Ersparnis:** zentrale Pflege statt N
Kopien, Updates propagieren via `pack update`.

### 7a.2 `mdai-memory` ≡ `ctx_knowledge`

**Vorheriger Entwurf (§6):** Eigenes 3-Layer-System (ctx_session, ctx_knowledge,
ctx_agent diary) händisch koordiniert.

**Stattdessen:** Direkter Wrapper um `mcp__lean-ctx__ctx_knowledge`:

```python
# Plan-Start
ctx_knowledge.remember(
  topic=f"mdai-plan:{plan_id}",
  body={"phases": [...], "started_at": "...", "current_phase": "P0"}
)

# Phase-Wechsel
ctx_knowledge.remember(
  topic=f"mdai-plan:{plan_id}",
  body={..., "current_phase": "A3", "completed": ["P0","A1","A2"]}
)

# Recall in neuer Session
state = ctx_knowledge.recall(topic=f"mdai-plan:{plan_id}")
```

`ctx_knowledge` deckt Persistent-Storage, Search und Lifecycle bereits ab.
Skill C: ~60 → ~30 Zeilen.

### 7a.3 Plan-Input via `ctx_provider`

**Optional**, nicht required. `mdai-plans` akzeptiert einen Issue-URL-Hint:

```
/mdai-plans --from-issue gh:dasTholo/vjc-core#42
/mdai-plans --from-issue jira:VJC-1234
```

Intern: `ctx_provider query github issue --id 42 --repo dasTholo/vjc-core` →
Title, Body, Labels, Acceptance-Criteria werden als initiale Phasen-Skeleton
eingesetzt. Spart manuelles Copy-Paste.

### 7a.4 Overlays für aktive Phase

`mdai-execution` setzt beim Phase-Start einen Overlay-Pin:

```bash
lean-ctx control pin "mdai-active-phase:A3" \
  --scope session \
  --content "$(mcp__markdownai__read_file <plan> --phase A3 --format ai)"
```

Bei Disconnect/Reconnect ist die aktive Phase aus dem Overlay rekonstruierbar.
`scope=session` heißt: hält die Session, verschwindet beim Exit. Reversibel via
`lean-ctx control unpin`.

### 7a.5 Gotchas-Tracking für MDAI-Pitfalls

Bekannte MDAI-Stolpersteine werden in `lean-ctx gotchas` registriert:

```bash
lean-ctx gotchas add \
  --tag mdai \
  --title "@import vs @include verwechselt" \
  --body "@import lädt nur @define-Macros (kein sichtbarer Output). \
         Für Inline-Content @include nutzen."
```

`mdai-plans` und `mdai-execution` rufen am Skill-Start `lean-ctx gotchas list --tag mdai`,
zeigen den Subagenten die bekannten Fallen.

### 7a.6 Gain-Measurement post-hoc

Am Ende eines `mdai-execution`-Laufs:

```bash
lean-ctx gain --tasks --since "1h" --json > .lean-ctx/last-run.json
```

Loggt die real gemessene Token-Ersparnis pro Plan-Durchlauf. Schreibt die
Differenz in `mdai-benchmark.md` oder einen eigenen Audit-Trail. Damit haben
wir empirische Daten über die Zeit, nicht nur einmalige Benchmark-Werte.

### 7a.7 Cache-Invalidation als Template-Muster

Wenn eine Macro-Datei geändert wird, sind alle Pläne, die sie via `@include`
referenzieren, im MDAI-Cache veraltet. Template-Snippet für jeden Skill:

```bash
# Nach Edit an macros/hard-rules.md
mcp__lean-ctx__ctx_shell "lean-ctx cache invalidate --glob '*.mdai.md'"
mcp__markdownai__invalidate_cache    # MDAI-internen Cache leeren
```

In `mdai-plans` und `mdai-execution` als post-edit-Schritt verankert — der User
muss nicht daran denken.

---

## 8. Non-Goals (was wir NICHT machen)

- `superpowers/`-Skills modifizieren. Sie bleiben Read-only.
- MDAI-Rust-Implementierung. Wir bleiben beim TypeScript-MCP-Server, bis die
  Stabilität in Produktion gemessen ist.
- `mai render --phase`-CLI-Flag. Phase-Isolation läuft via MCP — kein CLI-Bedarf.
- `lean-ctx custom_aliases` für Macro-Execution. Verifiziert: `custom_aliases`
  sind Shell-Kompressions-Patterns, keine Macros.
- Automatische `.md` → `.mdai.md`-Konversion. Erstmal manuelles Re-Writing,
  bis wir Workflow-Erfahrung haben.

---

## 9. Risiken & Open Items

| Risiko                                                                       | Schweregrad | Mitigation                                                                                                                                                                                   |
|------------------------------------------------------------------------------|-------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| MCP-Server-Patch geht bei `npm install` im markdownai-Repo verloren          | Mittel      | Upstream-PR + lokaler Patch-File als Backup                                                                                                                                                  |
| MCP-Server disconnected mitten in Session (siehe lean-ctx vor wenigen Tagen) | Niedrig     | Reconnect via `/mcp`, headless-Fallback dokumentiert                                                                                                                                         |
| Subagent versteht den `format=ai`-Output nicht → ignoriert Hard-Rules        | Mittel      | Im Subagent-Prompt explizit auf Constraints-Tabelle hinweisen                                                                                                                                |
| Macro-Dateien werden unauffindbar nach reboot                                | Niedrig     | `lean-ctx pack` (§7a.1) macht sie portabel und projekt-unabhängig                                                                                                                            |
| `@phase`-Markup wird in normalen `.md`-Plänen nicht erkannt                  | Niedrig     | Nur `.mdai.md`-Endung triggert den MDAI-Pfad                                                                                                                                                 |
| `ctx_knowledge` skaliert nicht bei vielen Plänen parallel                    | Mittel      | Topics namensraumen mit Projekt-Prefix; siehe `lean-ctx knowledge status` für Health-Check                                                                                                   |
| **Klassische `.md`-Pläne driften nach Hook-Removal** (§7.3 Option a)         | **Mittel**  | Skill-Body von `superpowers:writing-plans` enthält bereits eine Drift-Checkliste — die wirkt weiter, nur ohne harten PreToolUse-Block. Periodisches Review der Plan-Qualität als Mitigation. |

**Open Items:**

1. Upstream-PR an markdownai mit `respondTool()`-Fix
2. MCP-Server-Stabilität (Restart-Loop) in separater Session prüfen (siehe Prompt)
3. `lean-ctx pack create/install/auto-load`-Workflow für `mdai-macros` empirisch testen
   (bisher nur aus Doku übernommen, noch nicht selbst ausprobiert)
4. `ctx_provider`-Integration für GitHub/Jira: Authentifizierung pro User aufsetzen
   (Bearer/API-Key in `~/.config/lean-ctx/providers/`)
5. Schema für `ctx_knowledge`-Topics festlegen — wie heißen die Keys, was ist Pflicht,
   was optional? Vorschlag: `mdai-plan:<plan-id>` mit `{phases[], current, completed[],
   started_at, status}`. Validierung via `lean-ctx knowledge schema`?

---

## 10. Implementierungsschritte (high-level)

1. **lean-ctx-Setup**
    - `lean-ctx pack create mdai-macros tmp/mdai-bench/macros/` + `pack auto-load`
    - `ctx_knowledge`-Topic-Schema dokumentieren (Vorschlag: §9 Open-Item 5)
    - Gotchas-Seed: 3–5 initiale MDAI-Pitfalls eintragen (`@import` vs `@include` etc.)
2. **Skills schreiben**
    - `mdai-plans` (~80 Z) — mit optionalem `--from-issue`-Pfad (§7a.3)
    - `mdai-execution` (~120 Z) — inkl. Overlay-Pin und `gain`-Logging (§7a.4, §7a.6)
    - `mdai-memory` (~30 Z, schlanker Wrapper um `ctx_knowledge`)
3. **Hook-Removal** (§7.3): `plan-discipline.py` + `skill-plan-injector.py` löschen,
   Hook-Einträge aus `~/.claude/settings.json` entfernen, verifizieren dass
   `superpowers:writing-plans` weiter läuft
4. **Smoke-Tests**
    - `mdai-plans` mit `--from-issue` gegen ein echtes GitHub-Issue
    - `mdai-execution` mit S3a-Plan (Re-Use `tmp/mdai-bench/...mdai.md`)
    - `mdai-memory` cross-session Recall verifizieren
5. **Upstream-PR** an markdownai mit `respondTool()`-Patch (separat, blockiert nicht)

---

## 11. Implementierungsplan (Pointer)

Der konkrete Schritt-für-Schritt-Plan für die Umsetzung dieses Designs wird als
eigene Datei **dogfooded** geschrieben — d.h. selbst im MDAI-Format mit
`@phase`-Markern, damit wir ihn beim ersten Durchlauf bereits via
`mcp__markdownai__read_file(phase=…)` per-Phase an Subagents dispatchen können.

**Pfad:** `docs/mdai/plans/2026-05-21-mdai-skill-integration.mdai.md`

**Phasen-Skeleton (Vorschlag, gemäß §10):**

| Phase-ID                  | Inhalt                                                                                              |
|---------------------------|-----------------------------------------------------------------------------------------------------|
| `P0-leanctx-setup`        | `lean-ctx pack create mdai-macros`, `ctx_provider`-Auth, `ctx_knowledge`-Topic-Schema, gotchas-Seed |
| `A1-mdai-plans-skill`     | Skill `mdai-plans` schreiben (~80 Z) inkl. `--from-issue`-Modus                                     |
| `A2-mdai-execution-skill` | Skill `mdai-execution` schreiben (~120 Z) inkl. §5.1-Routing, Overlay-Pin, Gain-Logging             |
| `A3-mdai-memory-skill`    | Skill `mdai-memory` schreiben (~60 Z, Multi-Layer)                                                  |
| `A4-hook-removal`         | `plan-discipline.py` + `skill-plan-injector.py` löschen, Settings bereinigen (§7.3)                 |
| `A5-smoke-tests`          | Drei Verifikations-Läufe gemäß §10.4                                                                |
| `A6-upstream-pr`          | Fork-Branch + PR-Beschreibung für `respondTool()`-Patch (separat, blockiert nicht)                  |
| `A-final-quality-gate`    | Cross-Check: alle Erfolgskriterien aus §1 erfüllt? Benchmark-Werte stabil?                          |

**Erstellung:** Solange `mdai-plans` selbst noch nicht existiert, wird der Plan
einmalig **manuell** angelegt — in MDAI-Syntax, aber händisch geschrieben.
Ab `A1` ist `mdai-plans` verfügbar und kann sich theoretisch selbst überarbeiten
(echtes Bootstrap-Szenario).

**Constraints im Plan (Vorschlag):**

```
@constraint id="design-source-of-truth" severity="high"
Vor jeder Phase: docs/mdai/design-skill-integration.md neu konsultieren — kein
Drift zur Spec.
@end

@constraint id="hard-rules" severity="high"
Cargo: nextest run (nie cargo test). Vor git add: reformat_file. Keine
&&-Chains. Keine Worktrees.
@end
```

**Macros, die der Plan verwenden wird** (alle aus `tmp/mdai-bench/macros/`,
später via `lean-ctx pack` zentralisiert):

- `@include macros/hard-rules.md`
- `@include macros/tool-quick-ref.md`
- `@import macros/step-reformat-commit.md` → `@call stepReformatCommit(<file>)`

**Eigener Audit-Eintrag nach Plan-Abschluss:**

- `lean-ctx gain --tasks --since "<dauer>" --json` → Differenz Original- vs.
  MDAI-Dispatch loggen, in `mdai-benchmark.md` als „Erstmessung Implementation"-Eintrag.
