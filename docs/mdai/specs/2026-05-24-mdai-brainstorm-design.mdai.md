---
id: mdai-brainstorm
status: design
created: 2026-05-23
updated: 2026-05-24
spec_version: 3
supersedes:
  - docs/mdai/design-skill-integration.md §4 (mdai-plans)
  - docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md (v2, scope: spec+plan)
requires:
  mdai-library: ">=0.1.0"
  mcp__lean-ctx: ">=3.6.16"
  mcp__markdownai: ">=0.0.24"
---

@markdownai v1.0 consumer="ai"

# mdai-brainstorm — Markdownai-aware Brainstorming-Skill (Design v3)

Status: spec_version=3, Scope-Cut gegenüber v2. **Skill schreibt ausschliesslich Spec/Design** unter `docs/mdai/specs/`,
**keinen Plan**. Plan-Schreibung läuft nach Skill-Ende manuell via `/superpowers:writing-plans` (bzw. künftig
`/mdai-writing-plans`, siehe §14 Backlog #1). Damit deckungsgleich mit upstream `superpowers:brainstorming`-Workflow
(Terminal-State: invoke `writing-plans`). Bezug zu v2: ersetzt komplett; v2-Inhalte bleiben in git-history.

Voraussetzung: mdai-Library v0.1.0 (`mdai/core/*` + Skill-A-Pack `mdai/skills/mdai-brainstorm/{write-spec,spec-reviewer}.md`).
`plan_*`-Macros bleiben für den künftigen `mdai-writing-plans`-Skill als skill-owned Asset reserviert.

## 1. Zielsetzung

**Hauptziel:** Specs erstellen, die **aktiv markdownai-Funktionen nutzen** (`@call`, `@include`, `@import`, `@list`,
`@render`, `@tree`, `@constraint`, `@phase`). Die mdai-macro-library v0.1.0 ist **hard requirement**, nicht optional —
sie liefert die Bausteine, mit denen Specs live-aktualisierte Inhalte (Project-Tree, Dependency-Graph, Inventories,
Constraints) produzieren statt sie zur Schreibzeit hart einzukodieren.

Eine einzelne Skill `mdai-brainstorm` bündelt zwei Aufgaben:

1. **Brainstorming-Dialog** (Anforderungen klären, Approach-Vergleich, Design-Sektionen) — hand-ported aus
   `superpowers:brainstorming` in `body.mdai.md`.
2. **Spec-Write** — schreibt versionierte Design-Dokumente als `*.mdai.md` mit `consumer="ai"` (Default), die
   markdownai-Direktiven im Body nutzen (siehe Discipline-Punkt §10.4 #9). Render in `*.md` ist Opt-in über
   `@call render_spec(slug, target)` aus dem Skill-A-Pack.

**Nicht mehr Teil von Skill A** (gegenüber v2): Plan-Write. Nach erfolgreicher Spec-Session ist der nächste Schritt
explizit Skill-extern — Aufruf von `/superpowers:writing-plans <spec-path>` (bzw. künftig `/mdai-writing-plans`).

Trigger: `/mdai-brainstorm` oder Description-Match.

**Erfolgskriterien:**

1. `body.mdai.md` ruft `@call mdai_bootstrap()` als erste Zeile in `pre-context` auf. Bootstrap löst service-check
    + lang/tooling-detection auf, lädt bedingte Packs (`lang/rust.md` falls Rust-Repo, `tooling/jetbrains.md` /
      `tooling/serena.md` falls erkannt). Pre-context resolveiert lean-ctx-Kontext live (inkl. `ctx_graph`-Map),
      ohne Tool-Roundtrips im Dialog.
2. Upstream-`superpowers:brainstorming` wird **hand-ported** (Fork minimal): Dialog-Disziplin (Checklist, Process,
   Key Principles, Red-Flags, Rationalization-Table) wird **einmal** in `body.mdai.md` kopiert. Drift per
   `mdai-drift-check` (Backlog).
3. Skill A definiert **keine eigenen Macros**. Alle Macros werden aus der mdai-Library v0.1.0 (`mdai/core/*`) + Skill-A-Pack
   importiert: `@import mdai/skills/mdai-brainstorm/{write-spec,spec-reviewer}.md`. Spec-Frontmatter pinnt
   `requires.mdai-library: ">=0.1.0"`.
4. Spec wird als `*.mdai.md` mit `consumer="ai"` committet (Default). Render ist Opt-in mit drei Targets:
   `none` (Default), `chat` (inline via `mcp__markdownai__read_file`), `file` (zusätzliches `*.rendered.md` unter
   `docs/mdai/specs/rendered/` via `npx mai render`-CLI).
5. Nach Spec-Approval (User-Review-Gate) zeigt `handoff`-Phase explizit auf den nächsten Skill-Aufruf
   (`/superpowers:writing-plans <spec-path>` oder künftig `/mdai-writing-plans`). Skill A produziert **keinen**
   Plan-File.
6. Schritte mit Datei-Operationen nutzen Library-Wrapper aus `mdai/core/ctx-tools.md`
   (`@call ctx_read` / `ctx_search` / `ctx_tree` / `ctx_shell` / `ctx_edit`), Projekt-Dependencies aus
   `mcp__lean-ctx__ctx_graph` (Mermaid-Diagramm).
7. **Produzierte Specs nutzen markdownai-Direktiven aktiv für Live-Inhalte**: z.B. `@tree mdai/ depth=2` für
   Verzeichnis-Sektionen, `@call ctx_overview(task=...)` für Projekt-Kontext, `@constraint`-Blöcke für maschinen-lesbare
   Regeln, `@list ... | @render type="table"` für strukturierte Daten aus externen Files. Reine Plain-Markdown-Specs
   (ohne markdownai-Direktiven im Body) verfehlen das Hauptziel und werden in Self-Review §7 + Reviewer-Dispatch §7.5
   geflaggt.

## 2. Architektur-Überblick

```
│  /mdai-brainstorm                                                     │
│                                                                       │
│  SKILL.md  (~15 Z, pointer)                                           │
│    └─ Anweisung: "Lade body.mdai.md immer phase-für-phase via         │
│       mcp__markdownai__read_file(phase=…, format=ai). Niemals full."  │
│                                                                       │
│  body.mdai.md (~100 Z, live doc)                                      │
│    ├─ @markdownai v1.0                                                │
│    │   (no header imports — lazy-load per phase, see §5.1)            │
│    │                                                                  │
│    ├─ @phase pre-context                                              │
│    │    @call mdai_bootstrap()                                        │
│    │    @include mdai/core/hard-rules.md                              │
│    │    @include mdai/core/tool-quick-ref.md                          │
│    │    @call ctx_shell(cmd="git log --oneline -10")                  │
│    │    @query mcp lean-ctx ctx_overview task="$user_task"            │
│    │    @query mcp lean-ctx ctx_graph action=diagram kind=deps depth=2│
│    │    @query mcp lean-ctx ctx_graph action=context                  │
│    │    @call ctx_tree(path=".", depth=2)                             │
│    │    @call list_gotchas(query="")                                  │
│    │                                                                  │
│    ├─ @phase dialog                                                   │
│    │    Hand-ported Disziplin aus upstream brainstorming              │
│    │    (Checklist, Process, Principles, Red-Flags,                   │
│    │    Rationalization-Table). Kein Skill-Invoke.                    │
│    │    Step 7.5: @import spec-reviewer.md + @call (lazy)             │
│    │    Step 9 transition: invoke writing-plans skill (extern).       │
│    │                                                                  │
│    ├─ @phase write-outputs                                            │
│    │    @import mdai/skills/mdai-brainstorm/write-spec.md  (lazy)     │
│    │    @call write_spec(slug=<topic>, body=<design-content>)         │
│    │    @call render_spec(slug=<topic>, target=<none|chat|file>)      │
│    │                                                                  │
│    └─ @phase handoff                                                  │
│         Anweisung: "Next step (manual): /superpowers:writing-plans    │
│         <spec-path>  — oder /mdai-writing-plans wenn existent."       │
```

**Verhältnis zu Bestandsskills:**

| Skill | Behandlung |
| `superpowers:brainstorming`  | **Hand-ported** in dialog-phase (Disziplin-Slices in body.mdai.md kopiert; kein
Skill-Invoke, kein `@include` zur Renderzeit)                                              |
| `superpowers:writing-plans`  | **Bleibt zuständig** für Plan-Schreibung post-Spec, bis `mdai-writing-plans` (§14
Backlog #1) existiert. Skill A invokiert nicht selbst, sondern dokumentiert in handoff-Phase. |
| `superpowers:writing-skills` | Wird einmal invoked, um `mdai-brainstorm` selbst zu schreiben (
Bootstrap)                                                                                                  |
| `mdai-plans` (alt)           | **Wird abgelöst** und im Plan deinstalliert (entfernt aus `~/.claude/skills/` falls
global, sonst aus `.claude/skills/`)                                                   |
| `mdai-writing-plans` (neu)   | **§14 Backlog #1** — separater Skill, eigene Spec via `/mdai-brainstorm`. Konsumiert
die existierenden Library-Packs `write-mdai-plan.md` + `spec-reviewer.md`. |
| `mdai-execution`             | Unverändert, bleibt als nächster Step nach `mdai-writing-plans` (Spec → Plan →
Execution)                                                                                  |
| `mdai-memory`                | **Foundation gestartet** in `mdai/core/ctx-knowledge.md` (Library v0.1.0):
`remember_plan`, `recall_plan`, `add_gotcha`, `list_gotchas`. Skill-Wrapper folgt — invocable von `mdai-execution`. |
| `mdai-drift-check` (Backlog) | Wird nach Implementation gebraucht, um Drift zwischen Upstream-`brainstorming/SKILL.md`
und hand-ported dialog-phase zu detektieren |

## 3. Datei-Layout

**Library (in git, bereits vorhanden — live-resolved via `@tree`, Pfad relativ zur Spec-Datei):**

```
@tree ../../../mdai/ depth=2
```

Exports pro Pack (Macro-Namen, snake_case):

- `core/startup-check.md`: `mdai_bootstrap`, `service_check`, `detect_project_lang`, `detect_tooling`, `load_lang_pack`,
  `load_tooling_packs`
- `core/hard-rules.md`: text only (`mode: include`)
- `core/tool-quick-ref.md`: text only (`mode: include`)
- `core/ctx-tools.md`: `ctx_read`, `ctx_search`, `ctx_tree`, `ctx_shell`, `ctx_edit`
- `core/mcp-markdownai.md`: `read_phase`, `list_phases`, `get_constraints`
- `core/ctx-knowledge.md`: `remember_plan`, `recall_plan`, `add_gotcha`, `list_gotchas` — **Foundation für `mdai-memory`
  **
- `core/file-utils.md`: `file_check` — generischer Filesystem-Status-Helper (Pattern aus README Z 282-293). Verwendet in
  `write-spec.md`-Smoke-Tests (§8.5.2) und für `@call file_check`-Reports in generierten Specs/Plänen.
- `lang/rust.md`: `cargo_nextest`, `cargo_clippy`, `cargo_fmt`, `rustfmt_file`, `format_file` (conditional
  `MDAI_PROJECT_LANG=rust`)
- `tooling/jetbrains.md`: `reformat_file`, `step_reformat_commit`, `get_file_errors` (conditional
  `MDAI_HAS_JETBRAINS=true`)
- `tooling/serena.md`: `find_symbol`, `replace_symbol_body`, `insert_*_symbol`, `symbols_overview` (conditional
  `MDAI_HAS_SERENA=true`)
  **Skill-A-Assets (NICHT Library, sondern skill-owned — siehe §3-Boundary-Note unten):**

- `skills/mdai-brainstorm/write-spec.md`: `write_spec`, `render_spec`
- `skills/mdai-brainstorm/spec-reviewer.md`: `spec_reviewer_prompt` — **wird von Skill A v3 gepatched** (siehe unten +
  Appendix A)

**Scope-Cut (2026-05-24):** `skills/mdai-brainstorm/write-mdai-plan.md` (war ursprünglich im Library-Inventar v0.1.0
gelistet, ist aber tatsächlich ein Skill-Asset) wurde **entfernt** — orphaned nach v3-Scope-Cut. Pack wird erst wieder
gebraucht, sobald `mdai-writing-plans`-Skill (Backlog #1) seine eigene Spec hat, dann mit skill-spezifischem Pack unter
`mdai/skills/mdai-writing-plans/`. Library scope (per `mdai/changelog.md` 2026-05-24 clarification): `core/` +
`lang/` + `tooling/` only — Skill-owned Assets sind co-located unter `mdai/skills/<skill-name>/` für Filesystem-
Proximity, werden aber im jeweiligen Skill-Spec dokumentiert (Single Source of Truth pro Skill).

Verfügbare Library-Packs werden via `@call ctx_tree(path="mdai/", depth=2)` bzw.
`@call ctx_search(pattern="^@define", path="mdai/core/")` zur Skill-Runtime ermittelt — kein zentraler Index-File
(`mdai/MACROS.md` wurde am 2026-05-24 gelöscht, siehe `mdai/changelog.md`). Jedes Pack-File trägt seine eigene
`lib_version: 0.1.0`-Frontmatter; Versionshistorie steht im Changelog.

**Skill-A Source (neu, dieses Spec implementiert — koexistiert mit Library-Pack-Files):**

`mdai/skills/mdai-brainstorm/` existiert bereits (Library v0.1.0: `write-spec.md`, `write-mdai-plan.md`,
`spec-reviewer.md`). Skill A v3 fügt zwei Dateien hinzu und patcht eine bestehende:

```
mdai/skills/mdai-brainstorm/
  SKILL.md               ← Skill A v3 pointer (neu, A1)
  body.mdai.md           ← Skill A v3 live workflow (neu, A2)
  write-spec.md          ← Library pack (vorhanden, unverändert)
  spec-reviewer.md       ← Library pack (vorhanden) — **wird gepatched (A2.5)**
  (write-mdai-plan.md    ← entfernt am 2026-05-24, war orphaned nach v3-Scope-Cut)
```

**Skill-A-Asset-Patches (Library-Boundary-Note):** `spec-reviewer.md` liegt zwar physisch unter `mdai/`, ist aber per
strikter Library-Definition (`core/` + `lang/` + `tooling/`) ein **Skill-A-Asset**, nicht Library-Pack. Patch bleibt
unter `lib_version: 0.1.0` (pre-stable, kein Library-Bump). Patch-Inhalt wurde am 2026-05-24 direkt in die Live-Datei
übernommen (synchron mit diesem Spec); Appendix A ist auf eine kurze Referenz reduziert.

Die Namespace-Konvention `mdai/skills/<skill-name>/` macht Platz für zukünftige Migration von `mdai-execution`,
`mdai-memory` und neu `mdai-writing-plans` in dieselbe Struktur (Backlog).

**Install-Ziel während Entwicklung: project-local Symlink, KEIN globaler Install.**

```
.claude/skills/mdai-brainstorm  →  ../../mdai/skills/mdai-brainstorm  (Symlink)
```

Claude Code lädt Skills aus `.claude/skills/` automatisch projekt-scoped. Der Symlink macht den Source-Ordner unter
`mdai/skills/` zum Discovery-Target — Edits am Source sind sofort live, kein zweiter Build-Step. **Kein**
`~/.claude/skills/`-Install solange Skill noch in Entwicklung ist (vermeidet Cross-Projekt-Trigger /
Cross-Projekt-Schaden bei Iteration). Globaler Install + `install.sh`-Script ist Backlog-Item (siehe §14.9) und erfolgt
erst nach mind. einer stabilen `/mdai-brainstorm`-Session-Reihe im Repo.

## 4. SKILL.md (pointer)

Inhaltlich knapp, ~15 Zeilen. Verantwortung: Frontmatter mit Description (Trigger-Match), plus harte Anweisung an
Claude, **nur** phase-für-phase aus `body.mdai.md` zu lesen.

```markdown
---
name: mdai-brainstorm
description: Use when starting creative work that will produce a versioned design
  spec under docs/mdai/specs/. After spec approval, the next step is to invoke
  the writing-plans skill (superpowers:writing-plans, or mdai-writing-plans
  when available) — this skill does not write plans.
---

# mdai-brainstorm — pointer

DO NOT read this file's body for the workflow. The live workflow is in
`body.mdai.md` and MUST be loaded phase-by-phase via:

mcp__markdownai__read_file(path="<...>/body.mdai.md", phase="<phase-id>", format="ai")

Start with phase `pre-context`. Then `dialog`, `write-outputs`, `handoff`
in order. Never call `read_file` without a `phase=` argument.

Phases: pre-context | dialog | write-outputs | handoff
```

**Description-Diff zu v2:** Plan-Erwähnung raus, expliziter Hinweis auf separates writing-plans als next-step (
verhindert
Description-Drift, der Claude zum Plan-Write verleiten würde).

## 5. body.mdai.md (live workflow)

Skill A definiert **keine eigenen Macros**. Alle Macros werden aus der mdai-Library v0.1.0 + Skill-A-Pack importiert.
Bootstrap (`@call mdai_bootstrap()`) muss als erste Zeile in `pre-context` stehen — Library-Konvention aus
`mdai/core/startup-check.md` (Bootstrap-Macro selbst).

### 5.1 Datei-Header (minimal, keine globalen Imports)

```markdown
@markdownai v1.0
```

**Lazy-Load-Prinzip:** Skill-A-Pack-`@import`s werden **nicht** am Header gemacht (kein eager-load aller Packs), sondern
**in der Phase, die das Macro tatsächlich braucht** (just-in-time). Konkret:

- `write-outputs` phase importiert `write-spec.md` direkt vor dem `@call write_spec(...)` (siehe §5.4)
- `dialog` phase Step 7.5 importiert `spec-reviewer.md` direkt vor dem `@call spec_reviewer_prompt(...)` (siehe §5.3)
- `pre-context` und `handoff` brauchen keine Skill-A-Packs

Begründung: Workflow ist linear (Brainstorm → Write-Spec → Review-Spec). Pre-context und Dialog laufen ohne `write_spec`
oder `spec_reviewer_prompt`. Phase-isoliertes `mcp__markdownai__read_file(phase=...)` lädt nur die `@define`-Bodies, die
in der jeweiligen Phase tatsächlich `@import`-ed sind — kein Overhead für ungenutzte Macros.

Library-Wrapper aus `mdai/core/*.md` (`ctx_read`, `ctx_search`, etc.) werden über `@call mdai_bootstrap()` (siehe §5.2)
transitiv in der `pre-context` phase geladen — diese sind cross-phase wiederverwendet, daher dort sinnvoll.

**Diff zu v2:** v2 hatte 3 Header-Imports (`write-spec`, `write-mdai-plan`, `spec-reviewer`). v3 hat **null**
Header-Imports — Packs werden lazy in ihrer phase importiert. `write-mdai-plan.md` ist ohnehin entfernt (§3 Scope-Cut).

**Hinweis zur Library-Discovery:** Markdownai lädt nur Files, die via expliziter `@import`/`@include`/`@call`-Chain
referenziert sind — kein "Auto-Load aller Library-Files". Skill A referenziert zur Runtime nur die **drei
obligatorischen Library-Packs** (siehe §5.2 Tabelle "Bootstrap-Transitive-Load") plus die zwei lazy Skill-A-Packs in
den jeweiligen Phasen. Reviewer einer generierten Spec, die wissen will welche Library-Packs verfügbar wären,
ermittelt das via `@call ctx_tree(path="mdai/", depth=2)` + `@call ctx_search(pattern="^@define", path="mdai/core/")`
— kein zentraler Index-File seit 2026-05-24 (MACROS.md gelöscht, siehe `mdai/changelog.md`).

### 5.2 Phase: pre-context

Bootstrap-Call als erste Zeile. Anschliessend pre-resolved Projekt-Kontext. Alle `@call`/`@query` zielen explizit auf
lean-ctx-Wrapper aus der Library, damit Output-Kompression greift.

**Tool-Selection-Policy (gilt für die gesamte body.mdai.md):**

- File lesen → `@call ctx_read(path, mode)` (nicht `ctx_shell(cmd="cat ...")`)
- Verzeichnis listen → `@call ctx_tree(path, depth)` (nicht `ctx_shell(cmd="ls ...")` oder `find`)
- Pattern-Suche → `@call ctx_search(pattern, path)` (nicht `ctx_shell(cmd="grep ...")` oder `rg`)
- File-Edit ohne Read → `@call ctx_edit(path, old, new)`
- Plan-Phase lesen → `@call read_phase(plan, phase_id)` (nicht raw `mcp__markdownai__read_file`)
- Plan-Phasen listen → `@call list_phases(plan)`
- Gotcha-Recall → `@call list_gotchas(query)`
- `@call ctx_shell(cmd=...)` **nur als Last-Resort** — für git-Ops (`git branch`, `git log`, `git status`),
  Shell-Skripte (`start-server.sh`), oder Tools ohne lean-ctx-Wrapper. Wenn ein Wrapper existiert, IMMER den
  Wrapper nutzen — spec-reviewer Anti-Pattern-Check 1 (`mdai/skills/mdai-brainstorm/spec-reviewer.md`).

```markdown
@phase pre-context

@call mdai_bootstrap()

@include mdai/core/hard-rules.md
@include mdai/core/tool-quick-ref.md

## Pre-resolved project context

**Branch:** {{ @call ctx_shell(cmd="git branch --show-current") }}
**Recent commits:**
{{ @call ctx_shell(cmd="git log --oneline -10") }}

**Project map (task-scoped):**
{{ @query mcp lean-ctx ctx_overview task="$user_task" }}

**Dependency graph (Mermaid, depth=2):**
{{ @query mcp lean-ctx ctx_graph action="diagram" kind="deps" depth=2 }}

**Task-relevant subgraph:**
{{ @query mcp lean-ctx ctx_graph action="context" }}

**Tree (depth=2):**
{{ @call ctx_tree(path=".", depth=2) }}

**Known gotchas:**
{{ @call list_gotchas(query="") }}

Constraints for the dialog phase:

- Spec target: docs/mdai/specs/ (NOT docs/superpowers/specs/)
- NO plan target — plan-write is a separate skill invocation (handoff phase)
- Hard rules: see @include above
  @end
```

`mdai_bootstrap()` ist per-render in v0.1.0 (kein Cache). Bei jedem `@call read_phase(plan="…/body.mdai.md",
phase_id="pre-context")` (Library-Wrapper aus `mdai/core/mcp-markdownai.md`) läuft die Detection neu. Akzeptiertes
Overhead (~3-5 Tool-Calls pro Bootstrap), Cache als Library-Backlog (siehe §14).

**Bootstrap-Transitive-Load (was Skill NICHT explizit importieren muss):**

`@call mdai_bootstrap()` setzt voraus, dass `core/startup-check.md` verfügbar ist, und lädt **transitiv** weitere
Library-Packs in den Render-Scope. Skill A muss diese nicht selbst importieren.

| Pack                                      | Load-Mechanik                       | Bedingung                                                |
|-------------------------------------------|-------------------------------------|----------------------------------------------------------|
| `core/startup-check.md`                   | implizit verfügbar (Bootstrap-Pack) | always — pre-condition für `@call mdai_bootstrap()`      |
| `core/ctx-tools.md`                       | transitiv via bootstrap             | always                                                   |
| `core/mcp-markdownai.md`                  | transitiv via bootstrap             | always                                                   |
| `core/ctx-knowledge.md`                   | transitiv via bootstrap             | always                                                   |
| `core/file-utils.md`                      | transitiv via bootstrap             | always                                                   |
| `lang/rust.md`                            | transitiv via bootstrap             | `MDAI_PROJECT_LANG=rust` (detection)                     |
| `tooling/jetbrains.md`                    | transitiv via bootstrap             | `MDAI_HAS_JETBRAINS=true` (detection)                    |
| `tooling/serena.md`                       | transitiv via bootstrap             | `MDAI_HAS_SERENA=true` (detection)                       |
| `core/hard-rules.md`                      | **explizit** via `@include`         | always — pre-context phase                               |
| `core/tool-quick-ref.md`                  | **explizit** via `@include`         | always — pre-context phase                               |
| `skills/mdai-brainstorm/write-spec.md`    | **explizit lazy** via `@import`     | write-outputs phase (vor `@call write_spec`)             |
| `skills/mdai-brainstorm/spec-reviewer.md` | **explizit lazy** via `@import`     | dialog phase Step 7.5 (vor `@call spec_reviewer_prompt`) |

**Konsequenz:** Skill-A-`body.mdai.md` braucht zur Skill-Runtime genau **3 explizite Library-Verweise** in
pre-context (`@call mdai_bootstrap()` + `@include hard-rules.md` + `@include tool-quick-ref.md`) plus **2 lazy
Skill-A-Pack-Imports** in den jeweiligen Phasen. Alle anderen Packs sind transitiv via bootstrap erreichbar — kein
Header-Import nötig.

Hinweis: Der **allererste** Pre-Context-Load erfolgt aus SKILL.md heraus per raw `mcp__markdownai__read_file` (Library
ist noch nicht geladen). Sobald Bootstrap durchgelaufen ist, nutzen alle weiteren Phase-Loads in dieser Session
`@call read_phase(plan, phase_id)`.

### 5.3 Phase: dialog (hand-ported aus superpowers:brainstorming)

Disziplin-Slices wurden einmal aus `~/.claude/plugins/.../superpowers/5.1.0/skills/brainstorming/SKILL.md` extrahiert
und in unser body.mdai.md kopiert. Keine Skill-Invokation. Keine `@include` zur Renderzeit. Bei Upstream-Bumps prüft
`mdai-drift-check` (Backlog) per Hash-Diff, ob Re-Port nötig ist.

```markdown
@phase dialog

@constraint id="hard-gate" severity="high"
Do NOT invoke any implementation skill, write any code, scaffold any project,
or take any implementation action until the user has approved a design.
Applies to EVERY project regardless of perceived simplicity.

This skill writes a SPEC ONLY. Do NOT write a plan in this skill — the plan
is produced by a separate skill invocation after this one ends (see handoff
phase).
@end

<!-- §10.1 Red-Flags-Liste — filled in A2 from reasoned counters (§10.4 + upstream observations) -->

## Red Flags — STOP and re-enter discipline

If any of these thoughts arise, STOP, re-read the HARD-GATE constraint, and
return to the checklist:

- [Filled in A2 from reasoned counters (§10.4 + upstream observations); ~5–8 sentence-form
  entries, mind. einer pro Pressure-Kategorie cold/time/authority]

## Anti-Pattern: "This Is Too Simple To Need A Design"

[hand-ported from upstream, lines 16-20]

## Process Checklist

**Each item MUST become a `TaskCreate` entry and be completed in order** (Upstream §Checklist mandate).

[hand-ported from upstream §Checklist, adapted to mdai targets:

1. Explore project context  (already done in pre-context phase)
2. Offer visual companion (if visual) — own message (see Visual-Companion section)
3. Ask clarifying questions — one at a time
4. Propose 2-3 approaches with trade-offs
5. Present design sections, get approval after each
6. Write design doc to docs/mdai/specs/ ← OVERRIDDEN
7. Spec Self-Review (4 checks — see §"Spec Self-Review" below)
   7.5 OPTIONAL: dispatched reviewer-subagent via `spec_reviewer_prompt` (mdai-Augmentation)
8. User reviews written spec (exact wording — see §"User-Review-Gate")
9. Transition: invoke writing-plans skill (currently superpowers:writing-plans;
   future: mdai-writing-plans once that skill exists per §14 Backlog #1)
   — THIS SKILL DOES NOT WRITE THE PLAN.
   ]

## The Process — Details

[hand-ported from upstream, lines 70-104]

- Understanding the idea: scope check, decomposition for large projects, one question at a time
- Exploring approaches: 2-3 alternatives with trade-offs, lead with recommendation
- Presenting the design: scaled to complexity, approval-per-section
- Design for isolation and clarity: small units, clear interfaces
- Working in existing codebases: follow existing patterns, no unrelated refactoring

## Key Principles

[hand-ported from upstream, lines 140-145]

- One question at a time
- Multiple choice preferred
- YAGNI ruthlessly
- Explore alternatives before settling
- Incremental validation
- Be flexible — go back when something doesn't make sense

<!-- §10.2 Rationalization-Table — filled in A2 from reasoned counters (§10.4 + upstream observations) -->

## Rationalization-Table

| Excuse | Reality |
|---|---|
| [Filled in A2 from reasoned counters; ~8 rows, alle 8 Discipline-Punkte (§10.4) abgedeckt] | |

Tabelle deckt alle 8 mdai-Discipline-Punkte (siehe §10.4) ab. Jede Zeile mit
`[reasoned-counter]` markiert; bei späterer Drift-Beobachtung kann eine Zeile
durch verbatim-Quote ersetzt werden (siehe §10 Intro).

## Visual companion offer (step 2, conditional)

When upcoming questions will involve visual content (mockups, layouts, diagrams),
offer the companion **as its own message** — never combined with clarifying
questions or context summaries. German offer-template:

> "Manches davon ist leichter im Browser zu zeigen als zu beschreiben. Ich kann
> Mockups, Diagramme, Vergleiche und andere Visuals dazu bauen. Das Feature ist
> noch neu und token-intensiv. Willst du es ausprobieren? (Öffnet eine lokale URL.)"

Wait for response. If user declines → text-only. If user accepts:

@if visual_companion_active
Per-question decision: use the browser **only** when content IS visual
(mockups, wireframes, layout comparisons, architecture diagrams, side-by-side
visual designs). Conceptual/text questions stay in the terminal — the test is
"would the user understand this better by seeing than reading?".

Read the upstream guide for HTML-fragment patterns, CSS classes, event-stream format:

{{ @call ctx_read(path="~
/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/visual-companion.md", mode="
full") }}

Start the companion server (persistent mockups under `.superpowers/brainstorm/`):

@call ctx_shell(cmd="~
/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/start-server.sh
--project-dir \"$PWD\"")

**Hinweis (Anti-Pattern-Check 5 / Versions-Pin):** Pfad `~/.claude/plugins/.../superpowers/5.1.0/...` ist auf
Upstream-Version v5.1.0 gepinnt (`mdai-drift-check`-Backlog §14.1). Bei Upstream-Bump muss die Versionsnummer
aktualisiert werden. Glob `*` vermieden, da `ctx_read` keine Globs auflöst und Anti-Pattern-Check 5 absolute home-paths
in Library-Code verbietet — siehe §13 Risiko.

Capture `screen_dir` and `state_dir` from the server-info JSON for subsequent screen pushes.
Ensure `.superpowers/` is listed in `.gitignore` (see §15 P0).
@endif

## Spec Self-Review (step 7, MANDATORY, Claude himself)

After the spec source (`.mdai.md`) is written, look at it with fresh eyes and check:

1. **Placeholder scan:** any "TBD", "TODO", incomplete sections, vague requirements? Fix inline.
2. **Internal consistency:** sections contradict each other? Architecture matches feature descriptions?
3. **Scope check:** focused enough for a single plan? Or needs decomposition into sub-projects?
4. **Ambiguity check:** any requirement interpretable two different ways? Pick one, make it explicit.
5. **mdai-Direktiven-Nutzung (Discipline §10.4 #9):** Enthält der Spec-Body markdownai-Direktiven für Live-Inhalte
   (`@call`, `@include`, `@list | @render`, `@tree`, `@constraint`) wo semantisch sinnvoll? Falls Spec rein
   Plain-Markdown ist: ist das gerechtfertigt mit `markdownai_directives_omitted` in der Frontmatter? Wenn nein:
   Spec-Body um passende Direktiven ergänzen (z.B. `@tree mdai/` statt statischer Verzeichnis-Liste, `@call
   ctx_overview` statt manuell zusammenkopierter Projekt-Beschreibung).

Fix issues inline. No re-review loop — fix and move on. Checks 1-4 hand-ported verbatim from upstream §"Spec
Self-Review". Check 5 ist mdai-Augmentation (Hauptziel §1 enforcement).

## Spec reviewer dispatch (step 7.5, OPTIONAL, mdai-Augmentation)

Optional augmentation **beyond upstream**. **Lazy-load** the reviewer macro just before dispatch:

```markdown
@import mdai/skills/mdai-brainstorm/spec-reviewer.md
```

Then dispatch a reviewer subagent with `@call spec_reviewer_prompt(spec_path=<path>)` as prompt body (Skill-A-Asset
in `mdai/skills/mdai-brainstorm/spec-reviewer.md`, gepatched per §15 A2.5/A2.6 — Live-Datei synchron mit diesem
Spec, Implementation-Referenz in Appendix A).

Der gepatchte Reviewer kombiniert:

- **§0 Lean-Context-Discipline** via `@include mdai/core/lean-context.md` (Defaults/Exceptions-Tabelle für
  `ctx_read`/`ctx_shell`/`@include`/`find_symbol`/`fresh=true` — bindend für den Reviewer-Prompt selbst).
- mdai-spezifische Anti-Pattern-Checks (11 numbered: MCP signatures, existing-store, mai-CLI `@query`,
  frontmatter, repo-relative paths, language, parameter names, smoke-render, **mdai-Direktiven-Nutzung im
  Spec-Body**, **Lean-Context-Defaults**, **Structured-Data-via-`@read`/`@list`**).
- Upstream-Inhalte aus `superpowers:brainstorming/spec-document-reviewer-prompt.md`: 5-Spalten-Quick-Scan-Tabelle
  (Completeness/Consistency/Clarity/Scope/YAGNI), Calibration-Paragraph (Anti-Pedantry-Bremse),
  Recommendations-Sektion (advisory, do not block approval).

Returns `Status` (Approved | Needs-Revision | Needs-Clarification) + `Strengths` + `Gaps` + `Concrete patches` +
`Recommendations`. Apply issues inline; surface recommendations.

Trigger: spec touches MCP signatures, Library packs, or render flow. Skip for pure-prose specs (Self-Review §7 reicht).

## User-Review-Gate (step 8, exact wording, MANDATORY)

After Self-Review (and optional reviewer dispatch), ask the user with this wording:

> "Spec geschrieben und committed nach `<path>`. Bitte review und gib Feedback,
> ob du Änderungen willst, bevor du als nächsten Schritt
> `/superpowers:writing-plans <path>` aufrufst (oder `/mdai-writing-plans`
> sobald dieser Skill existiert)."

Wait for explicit response. If user requests changes → patch inline → re-run
Self-Review §7. Only proceed to write-outputs phase once user explicitly approves.

**Wichtig:** Skill endet nach handoff-Phase. Es gibt keinen automatischen Übergang zum Plan-Write — der User ruft
den nächsten Skill manuell auf.

Collect for the next phase:

- `slug` — kebab-case topic name (e.g. "user-onboarding-flow")
- `design_content` — full design body as Markdown
- (kein `phase_list` mehr — Plan-Phasen sind nicht Skill-A's Sorge)
  @end

```

**Drift-Tracking:** Header der hand-ported Sektion enthält Verweis auf Upstream-Quelle (
`# Hand-ported from superpowers/5.1.0/.../brainstorming/SKILL.md, lines 16-20, 22-32, 70-104, 107-136, 140-145`).
v3 fügt **lines 107-136** zur Liste hinzu (After-the-Design-Sektion mit Documentation, Self-Review, User-Review-Gate,
Implementation-Transition). `mdai-drift-check` (Backlog) liest diese Annotation, hasht die Source-Zeilen, vergleicht
mit gespeichertem Hash, meldet Diffs.

### 5.4 Phase: write-outputs

```markdown
@phase write-outputs

@import mdai/skills/mdai-brainstorm/write-spec.md

@call write_spec(slug={{ slug }}, body={{ design_content }})
@call render_spec(slug={{ slug }}, target={{ render_target | default("none") }})

Default output (one file staged in working tree):

- docs/mdai/specs/<date>-<slug>-design.mdai.md (spec source, consumer="ai")

Opt-in render targets (passed via `render_target` collected in dialog step 6):

- target="none" (default) → no render
- target="chat" → render inline via mcp__markdownai__read_file (no file)
- target="file" → adds docs/mdai/specs/rendered/<date>-<slug>.rendered.md via npx mai render CLI

Verification (lean-ctx-first):
@call ctx_tree(path="docs/mdai/specs/", depth=1)   # verifies new file appears in listing

Note: commit is left to the user (per user CLAUDE.md rules — never auto-commit).
Note: NO plan file is written here. Plan-write is a separate skill invocation
(see handoff phase).
@end
```

**Diff zu v2:** `@call write_mdai_plan(...)` entfernt. `@call list_phases(plan=...)` entfernt (kein Plan zum
inspizieren). Output-Liste auf ein File reduziert.

### 5.5 Phase: handoff

```markdown
@phase handoff

Spec ready for plan-write. Next step (manual, separate skill invocation):

/superpowers:writing-plans docs/mdai/specs/<date>-<slug>-design.mdai.md

This skill does NOT write the plan. Plan-write is the responsibility of a
separate writing-plans skill:

- **Now:** /superpowers:writing-plans <spec-path> (upstream)
- **Future:** /mdai-writing-plans <spec-path> — once §14 Backlog #1 is shipped,
  use that instead (produces .mdai.md plan with @phase markers, compatible
  with mdai-execution).

Verify spec file is in place:
@call ctx_read(path="docs/mdai/specs/<date>-<slug>-design.mdai.md", mode="map")
@end
```

**Diff zu v2:** `@call list_phases(plan=...)` und `@call remember_plan(...)` entfernt — kein Plan-State zum
persistieren. Pointer auf `/mdai-execution` ersetzt durch Pointer auf `writing-plans` (Spec → Plan → Execution).

### 5.6 Spec-Body mdai-Direktiven-Konventionen (Wie Claude Specs baut)

Diese Konvention setzt Discipline-Punkt §10.4 #9 (Specs nutzen markdownai-Funktionen aktiv) operationell um. Sie ist
Pflichtlektüre für Claude beim dialog-phase Step 6 (Write design doc). Im body.mdai.md dialog-phase Block als
Reference-Sektion einzubauen.

**Best-Practice-Patterns für Spec-Body (Live-Inhalte statt hart kodierter Snapshots):**

| Use-Case                     | mdai-Direktive (Best Practice)                                   | Anti-Pattern (vermeiden)                                |
|------------------------------|------------------------------------------------------------------|---------------------------------------------------------|
| Datum in File-Pfaden         | `{{ @date format='YYYY-MM-DD' }}`                                | hartkodiertes `2026-05-24` im Spec-Body                 |
| Verzeichnis-Listing          | `@tree mdai/ depth=2`                                            | manuell zusammen-getippte Tree-Ausgabe                  |
| File-System-Status (Report)  | `@call file_check(path="...")` (aus `core/file-utils.md`)        | `ls -la` Output kopiert + commited                      |
| Branching auf File-Existenz  | inline `@if file.exists "..."` + `@else` + `@endif` am Call-Site | `@call file_check` (das ist nur Status)                 |
| Conditional content          | `@if {{ var }} == "..."` + `@elseif` + `@endif`                  | Markdown-Kommentare wie "TODO: pick one"                |
| Strukturierte Daten          | `@list <file.yaml> \| @render type="table" columns="..."`        | Plain-Markdown-Tabelle bei >50 Zeilen oder externer SoT |
| Counts / Statistics          | `{{ @count ./src "*.ts" }}` (inline)                             | hartkodierte Zahlen, die altern                         |
| Cross-File-Content           | `@include ./CHANGELOG.md` oder `@include <file> lines=N-M`       | Copy-Paste zwischen Specs                               |
| Machine-Readable Constraints | `@constraint id="..." severity="high"` + body + `@end`           | Prosaische "Wichtig:"-Hinweise                          |
| Project-Context (live)       | `@call ctx_overview(task="...")` oder `@call ctx_tree(...)`      | manuell kopierte Projekt-Beschreibung                   |
| Block-Consumer-Tagging       | `@consumer=ai`/`@consumer=human` (Block-Level) + `@note visible consumer="human"` (Reviewer-Hinweise) + `@prompt ... @end` (AI-Anweisungen) | nur Header-Level `consumer="ai"` ohne Block-Tagging     |
| Erweiterte Condition-Operatoren | `@if "{{ slug }}".startsWith("mdai-")` / `endsWith` / `includes` / `file.isFile` / `file.isDir` | nur `==`/`!=` für komplexere Bedingungen                |

**Anti-Pattern: `file_check` ist nicht Branching.**

`@call file_check(path="x.md")` rendert nur Status (`- x.md exists` oder `- x.md MISSING`) — kein Control-Flow.
Für Branching IMMER inline am Call-Site:

```markdown
@if file.exists "x.md"

- ... do this when exists ...
  @else
- ... do that when missing ...
  @endif
```

**Beispiel-Block für eine generierte Spec (illustrativ):**

```markdown
## Pre-flight checks

@call file_check(path=".lean-ctx.toml")
@call file_check(path="mdai/core/lean-context.md")
@call file_check(path="docs/mdai/specs/")

## Project tree (live, depth=2)

@tree mdai/ depth=2

## Architecture constraint

@constraint id="library-scope" severity="high"
Library = mdai/core/ + mdai/lang/ + mdai/tooling/ only.
Skills/<name>/ are skill-owned, not Library packs.
@end
```

→ Beim Render zeigt die Spec live-aktualisierte Inhalte. Reviewer sehen `docs/mdai/specs/<date>-<slug>-design.mdai.md`
nicht als statischen Snapshot, sondern als ausführbares Dokument, das sich beim mai-render dem aktuellen Projekt-State
anpasst.

**Ausnahme** (per §10.4 #9): Specs für rein algorithmische Themen ohne File-/Tool-/Daten-Bezug dürfen plain Markdown
sein — dann `markdownai_directives_omitted: <reason>` in Frontmatter.

### 5.7 Lean-Context-Discipline (Cross-File-Reads + Shell + Cache-Bypass)

Lean-Context-Discipline ist in `mdai/core/lean-context.md` kodifiziert (Single Source of Truth, `mode: include`,
ohne YAML-Frontmatter, ohne `@define`-Blöcke). Spec-reviewer `@include`-t das File als §0, sodass die
Discipline-Tabelle (Default / Exception pro Tool) direkt im gerenderten Reviewer-Prompt erscheint. Wrapper für die
bounded-read Modi liegen in `mdai/core/ctx-tools.md` (`@call ctx_read_lines(path, start, end)` → `mode="lines:N-M"`,
`@call ctx_read_map(path)` → `mode="map"`, `@call ctx_read_signatures(path)` → `mode="signatures"`). Kurz-Referenz
mit Mapping in `mdai/core/tool-quick-ref.md`.

**Kernregeln (gelten für generierte Specs UND für den Reviewer-Prompt selbst):**

- `ctx_read mode="full"` nur auf die Spec-Source (Reviewer Step 1); jedes andere `mode="full"` braucht
  `@note visible consumer="human"`-Justification.
- Cross-File-Scan → `ctx_read_map` / `ctx_read_signatures`. Nach `ctx_search` / `find_symbol` → `ctx_read_lines`.
- `ctx_shell raw=true` nur mit Justification.
- `@include <file>` ohne `lines=N-M` nur bei Files ≤50 Z (sonst Token-Bloat).
- `ctx_read fresh=true` NUR direkt nach einem Write/Edit auf denselben Pfad (Cache invalidiert automatisch via
  mtime, sonst Token-Verschwendung).
- Serena `find_symbol(...body=true)` nur mit Justification (Default `body=false` reicht für Symbol-Header-Scan).

**Reviewer-Enforcement:** §5 Anti-Pattern-Check **#10** (lean-context-defaults) prüft alle obigen Regeln in der
Spec-Source via `@call ctx_search`-Suchen. Patch-Vorschlag bei Fail: search-then-`ctx_read_lines` statt `mode="full"`.
Check **#11** (structured-data-via-`@read`/`@list`) flaggt Tabellen, die externe SoT-Daten hartkodiert duplizieren.

## 6. Spec-Output-Format

| Artefakt | Source-Datei | Render-Datei | Format-Direktive | Konsument |
| Spec | `*.mdai.md`  | **— (Default: kein Render-File)** | `@markdownai v1.0 consumer="ai"`  |
mdai-writing-plans-Subagent (nach diesem Skill); User via `.mdai.md`-Source oder Opt-in Render |

**Default-Verhalten Spec:** Nur die `*.mdai.md`-Source wird committet. Render in `*.md` ist Opt-in über das
`render_spec(slug, target)`-Macro (aus `mdai/skills/mdai-brainstorm/write-spec.md`):

- `target="none"` → kein Render (Default)
- `target="chat"` → Render via `mcp__markdownai__read_file` (MCP, consumer="human"), Output inline in Claudes Antwort
  (kein File)
- `target="file"` → Render via `cd markdownai && npx mai render` (CLI), Output nach
  `docs/mdai/specs/rendered/<date>-<slug>.rendered.md` (zusätzliches versioniertes Artefakt, User-Acknowledgement
  Pflicht)

Hinter dem Vorhang: `mcp__markdownai__read_file(consumer="human", …)` überschreibt den `consumer="ai"`-Header zur
Render-Zeit — Source-File bleibt unverändert. `npx mai render` rendert statisch ohne `@query`-Execution (siehe §13
Risiko: mai-CLI blockiert `@query`).

**Diff zu v2:** Tabelle hatte zwei Zeilen (Spec + Plan). Plan-Zeile entfernt. Diese Skill produziert keine Pläne.

## 7. Slash-Trigger und Description-Match

**Slash-Command:** `/mdai-brainstorm` — Skill-Name = Command-Name (Konvention von Claude Code).

**Description (Frontmatter):**

> Use when starting creative work that will produce a versioned design spec under docs/mdai/specs/. After spec
> approval, the next step is to invoke the writing-plans skill (superpowers:writing-plans, or mdai-writing-plans
> when available) — this skill does not write plans.

**Trigger-Disziplin:** Primärer Trigger ist der explizite Slash-Command `/mdai-brainstorm`. Description-Match
ist sekundär — der Pfad-Trigger (`docs/mdai/specs/`) grenzt den Skill von `superpowers:brainstorming` ab (das nach
`docs/superpowers/specs/` schreibt). Description-Wording macht **explizit klar**, dass kein Plan geschrieben wird —
verhindert Description-induzierte Drift, die Claude zum Plan-Write verleiten würde.

## 8. Smoke-Tests

**Bewusst kein RED-Baseline und kein GREEN-Re-Run:** Skill A wird ausschliesslich per explizitem
`/mdai-brainstorm`-Slash-Command getriggert (siehe §7 Trigger-Disziplin). Damit ist Trigger-Discovery-Drift kein
relevantes Risiko, und der `writing-skills` "Iron Law: RED-GREEN-REFACTOR" wird durch reasoned-counter-Seeding der
Red-Flags + Rationalization-Table (§10) ersetzt. Falls später Pressure-Induced-Drift *innerhalb* der Skill-Session
beobachtet wird (z.B. Session-Reviewer flaggt eine Discipline-Lücke), kann eine RED-Baseline retroaktiv hinzugefügt
werden.

### 8.1 Trigger-Test (GREEN — Pointer-Compliance)

**Drei Test-Runs**, jeweils frische Session, `/mdai-brainstorm` aufrufen.

**Pass-Kriterien (alle drei Runs müssen pass):**

1. Erster Tool-Call nach SKILL.md-Load ist `mcp__markdownai__read_file(phase=pre-context, format=ai)`.
2. Kein `mcp__markdownai__read_file` ohne `phase=`-Argument.
3. Kein `lean-ctx ctx_read` auf `body.mdai.md` mit `mode=full`.

**Mess-Befehl** (nach Test-Run die neueste Session-Transcript-JSONL inspizieren):

```bash
SESSION=$(ls -t ~/.claude/projects/-home-tholo-Scripts-lean-ctx/*.jsonl | head -1)
jq -r 'select(.type == "tool_use") |
       select(.input.path | tostring | contains("body.mdai.md")) |
       "\(.name) mode=\(.input.mode // "?") phase=\(.input.phase // "?")"' "$SESSION"
```

Pass: nur Zeilen `mcp__markdownai__read_file mode=? phase=<id>`.
Fail: irgendeine Zeile mit `mode=full` oder leerem `phase=`-Argument auf `body.mdai.md`.

**Re-Architektur-Trigger:**

- **3/3 Pass** → grün, weiter zu §8.2.
- **2/3 Pass** → **manuelle Diagnose** (mögliche Ursachen: MCP-Disconnect,
  Cache-Effekt, Setup-Rauschen). Nicht auto-Fallback. Erst nach Root-Cause-
  Analyse entscheiden zwischen Re-Test (bei klarer Glitch-Ursache),
  Skill-Iteration (bei Pattern-Verbesserungsbedarf) oder File-System-Split
  (5-File-Layout: `phases/<id>.mdai.md`, falls Root-Cause auf strukturelle
  Skill-Loader-Schwäche zeigt — Migration als separate Spec).
- **0/3 oder 1/3 Pass** → File-System-Split aktivieren, A2 re-do als separate Spec.

### 8.2 Discipline-Fidelity-Test (GREEN)

dialog-phase führt zu interaktivem Dialog mit Klärungsfragen (one-at-a-time), 2-3
Approach-Vorschlägen, Design-Sektion-by-Section. Verifiziere: kein
`Skill(superpowers:brainstorming)`-Invoke im Tool-Log, Claude folgt der hand-ported
Checklist ohne Upstream-Skill-Load. Red-Flags + Rationalization-Table aus §10
zur Skill-Laufzeit gefüllt und respektiert.

**Zusätzlich für v3:** Verifiziere, dass am Ende der Session **kein** Plan-File
geschrieben wird (kein `write` / `ctx_shell mkdir` auf `docs/mdai/plans/`). Die
handoff-Phase zeigt nur auf `/superpowers:writing-plans` als Next-Step, ohne
eigenständigen Write.

### 8.3 Output-Test (GREEN)

write-outputs erzeugt im Default-Pfad genau **ein** File:

- `docs/mdai/specs/<date>-<slug>-design.mdai.md` (Spec-Source, `consumer="ai"`)

Verifiziere via `git status` + `@call ctx_tree(path="docs/mdai/specs/", depth=1)`. Bei Opt-in
`render_spec(target="file")` kommt zusätzlich `docs/mdai/specs/rendered/<date>-<slug>.rendered.md` dazu — separat
verifizieren. Kein File unter `docs/superpowers/specs/`. **Kein File unter `docs/mdai/plans/`** (Plan-Write ist nicht
Verantwortung dieses Skills).

**Diff zu v2:** "genau zwei Files" → "genau ein File". Plan-File-Reference komplett raus.

### 8.4 Phase-Budget-Test

Pro Phase in `body.mdai.md` Worte-Budget gegen §11-Tabelle prüfen:

```bash
for phase in pre-context dialog write-outputs handoff; do
  count=$(mcp__markdownai__read_file path=mdai/skills/mdai-brainstorm/body.mdai.md \
            phase=$phase format=ai | wc -w)
  echo "$phase: $count words"
done
```

Pass-Kriterien: alle Phasen unter Budget aus §11. Bei Überschreitung Kürzung
oder Sub-Phase-Split, **nicht** Budget-Aufweichung.

### 8.5 Library-Import-Smoke-Test

Spec-Reviewer Anti-Pattern-Check 8 (`mdai/skills/mdai-brainstorm/spec-reviewer.md`): Static-render-test pre-Install.

```bash
cd markdownai && npx mai render ../mdai/skills/mdai-brainstorm/body.mdai.md
```

Pass-Kriterien: `exit 0`; alle `@call`s aus Skill-A-Pack (`write_spec`, `render_spec`, `spec_reviewer_prompt`) und
Core-Wrappers (`ctx_read`, `ctx_shell`, `list_phases`, `remember_plan`, `list_gotchas`) sind aufgelöst; keine
`unknown directive` Errors; `mode: include` Text (hard-rules, tool-quick-ref) erscheint im Output; `mode: import-only`
Source-Text erscheint NICHT. `@query`-Direktiven liefern leere Strings (mai-CLI blockiert Live-Execution — siehe §13).

**Diff zu v2:** Fixture importiert nur 2 Skill-A-Packs (`write-spec.md`, `spec-reviewer.md`) statt 3.
`write_mdai_plan`, `plan_frontmatter`, `plan_phase`, `plan_step` sind **nicht** in der Pass-Kriterien-Liste — diese
Macros existieren in der Library, werden aber von Skill A nicht importiert. Verifiziert nur Plumbing, nicht Live-MCP.

### 8.5.1 `@date`-Auflösungs-Test (Sub-Test zu §8.5)

`write-spec.md` nutzt `{{ @date format='YYYY-MM-DD' }}` als Inline-Interpolation in `@query ... command="..."`-
Strings (siehe §10.4 #9 Best-Practice-Demonstration). **Inline-Auflösung ist in `markdownai/README.md` Z 491-494
explizit dokumentiert**:

> "These utilities work in inline expressions too:
> `Generated: {{ @date format=\"YYYY-MM-DD\" }} | TypeScript files: {{ @count ./src \"*.ts\" }}`"

Smoke-Test §8.5.1 verifiziert, dass das auch **innerhalb von `@query ... command="..."`-Strings** funktioniert
(README-Beispiel ist plain markdown text, nicht in einem Direktiv-Argument — leichte Extrapolation, daher Smoke-Test).

**Quote-Wahl-Begründung:** `format='YYYY-MM-DD'` mit Single Quotes (statt README-`"..."`) vermeidet Quote-Konflikt mit
umschließendem `command="..."`. Markdownai akzeptiert typischerweise beide; Smoke-Test verifiziert konkret die
Single-Quote-Variante. Falls Fail: switch zu escaped double quotes `format=\\"YYYY-MM-DD\\"` (ungetestet, alternative
Quoting-Stufe).

**Test-Fixture** (transient, nicht committen):

```bash
cat > /tmp/mdai-date-resolve-test.mdai.md <<'EOF'
@markdownai v1.0

@import mdai/skills/mdai-brainstorm/write-spec.md

@call write_spec(slug="smoke-test", body="dummy")
@call render_spec(slug="smoke-test", target="none")
EOF
```

```bash
cd markdownai && npx mai render ../tmp/mdai-date-resolve-test.mdai.md
```

**Pass-Kriterien:**

1. `exit 0`.
2. Output enthält **kein** literal `{{ @date format='YYYY-MM-DD' }}` (= alle aufgelöst).
3. Output enthält ein Datum im Format `YYYY-MM-DD` (z.B. `2026-05-24`) — verifiziert via Grep:
   ```bash
   cd markdownai && npx mai render ../tmp/mdai-date-resolve-test.mdai.md | \
     grep -E 'docs/mdai/specs/[0-9]{4}-[0-9]{2}-[0-9]{2}-smoke-test-design\.mdai\.md'
   ```
4. `@query`-Direktiven werden literal gerendert (mai-CLI blockiert Execution, siehe §13), **aber** mit aufgelöstem
   `@date` und `{{ slug }}` im command-Body — kein `${DATE}` oder `$(date)` shell-substitution mehr (das war v2-Pattern,
   ersetzt durch markdownai-native `@date`).

**Fail-Modus + Fallback:**

Inline-Auflösung ist per README Z 491-494 dokumentiert für plain-markdown-Inline-Expressions. Falls die Extrapolation
auf `@query ... command="..."`-Argument-Strings **nicht** funktioniert (Output enthält literal `{{ @date... }}` statt
Datum), greift Fallback-Stufe:

1. **Stufe 1 (geprüft im Smoke):** `{{ @date format='YYYY-MM-DD' }}` inline in `command="..."` — aktuelles Pattern.
2. **Stufe 2 (bei Stufe-1-Fail):** Quote-Variante `{{ @date format=\"YYYY-MM-DD\" }}` mit escaped double quotes —
   näher am README-Beispiel, aber escape-aufwendig.
3. **Stufe 3 (bei Stufe-2-Fail):** Separate `@date`-Direktive mit Label vor `@query`:

```markdown
@define write_spec(slug, body)
@date format="YYYY-MM-DD" label=today
@query mcp lean-ctx ctx_shell command="
mkdir -p docs/mdai/specs &&
SPEC_PATH=docs/mdai/specs/{{ today }}-{{ slug }}-design.mdai.md &&
..."
@end
```

4. **Stufe 4 (bei Stufe-3-Fail):** Shell-side `$(date -u +%Y-%m-%d)` als pragmatischer Rollback — verletzt §10.4 #9
   mild (mischt shell-substitution mit markdownai-Direktiven), aber funktional verifiziert. Letzte Stufe.

`@date label=...` ist in der README nicht direkt dokumentiert (README zeigt `label=` nur für `@query`). Stufen 3 + 4
sind Notfallpfade. Entscheidung welcher Fallback gewählt wird, hängt vom Smoke-Test-Output ab — bei §8.5.1 Fail wird
in A2.5 (oder A4 Smoke-Test-Iteration) entschieden, nicht jetzt spekulativ.

**Hinweis zum Render-Determinismus:** `@date` löst zur Render-Zeit auf. Mehrfach-Verwendung im selben Render-Pass
(z.B. `render_spec target="file"` hat 2 `@date`-Aufrufe für source + rendered Pfad) ist atomar — beide bekommen das
gleiche Datum. Über mehrere Render-Pässe hinweg ändert sich das Datum bei Tag-Wechsel — nicht relevant für unsere
Smoke-Tests (single-pass), aber dokumentiert für künftige Reviewer.

### 8.5.2 `@if file.exists`-Conditional-Test (Sub-Test zu §8.5)

`write-spec.md` nutzt `@if file.exists "..."` als Overwrite-Protection (in `write_spec`) und Existence-Check (in
`render_spec` chat/file-targets). README Z 1095 listet `file.exists` als Built-in Condition-Operator; README Z 282-293
zeigt das Pattern in einem `@define file-check`-Beispiel mit `@if file.exists "{{ path }}"` + `@else` + `@endif`.

**Test-Fixture A (Overwrite-Protection):**

```bash
# Pre-condition: leeres docs/mdai/specs/, dann write_spec zweimal
rm -f docs/mdai/specs/$(date -u +%Y-%m-%d)-smoke-overwrite-design.mdai.md
cat > /tmp/mdai-overwrite-test.mdai.md <<'EOF'
@markdownai v1.0

@import mdai/skills/mdai-brainstorm/write-spec.md

@call write_spec(slug="smoke-overwrite", body="first body")
@call write_spec(slug="smoke-overwrite", body="second body — should ABORT")
EOF
cd markdownai && npx mai render ../tmp/mdai-overwrite-test.mdai.md
```

**Pass-Kriterien A:**

1. Erster `@call` rendert den `@query mcp lean-ctx ctx_shell` Body (mai-CLI blockiert die Execution, aber rendered
   output zeigt cat-heredoc literal).
2. Zweiter `@call` rendert den `@else`-Body NICHT — sondern den `@if`-Body: "ABORT: Spec file already exists..." (weil
   Mockup im File-System nach erstem Call existieren würde — siehe Hinweis unten zu mai-CLI-Verhalten).
3. **Hinweis zum mai-CLI:** Da mai-CLI `@query`-Direktiven nicht ausführt, wird die Spec im Test eigentlich NICHT
   geschrieben — `file.exists` ist also `false` bei beiden Calls. **Echte Verifikation der Overwrite-Protection nur in
   Live-Claude-Code-Session** (wo `@query` aufgelöst wird und der erste Call die Datei tatsächlich schreibt).
4. Statischer Render-Test verifiziert **Plumbing**: `@if file.exists`-Direktive wird gerendert (kein `unknown directive`
   -Error), `@else`-Branch parsed korrekt.

**Test-Fixture B (Existence-Check in render_spec):**

```bash
rm -f docs/mdai/specs/$(date -u +%Y-%m-%d)-smoke-render-design.mdai.md
cat > /tmp/mdai-render-missing-test.mdai.md <<'EOF'
@markdownai v1.0

@import mdai/skills/mdai-brainstorm/write-spec.md

@call render_spec(slug="smoke-render", target="chat")
EOF
cd markdownai && npx mai render ../tmp/mdai-render-missing-test.mdai.md
```

**Pass-Kriterien B:**

1. Output enthält "ERROR: Cannot render — spec file does not exist at docs/mdai/specs/`<date>`
   -smoke-render-design.mdai.md"
2. KEIN `@query mcp markdownai read_file` Direktiv im Output (`@if`-False-Branch wurde gewählt).
3. Plumbing-Verifikation: `@if file.exists` + `@else` parsen sauber, kein `unknown directive`-Error.

**Fail-Modus + Fallback:**

Falls `file.exists` nicht als Inline-Operator im `@if`-Statement funktioniert (gegen README-Doku — sollte aber gehen):
Fallback wäre shell-side `[ -f "$SPEC_PATH" ] && echo "exists" || cat > ...` — mischt shell-Logic mit markdownai-
Direktiven, verletzt §10.4 #9 mild. Nur bei verifiziertem Fail in Smoke-Test §8.5.2 anwenden.

**Best-Practice-Demonstration zu §10.4 #9:** `write-spec.md` nutzt drei markdownai-native Features in Kombination —
`@date` (Inline-Interpolation), `@if file.exists` (Filesystem-Operator), `@if/@elseif/@else/@endif` (Conditional-Logic)
— statt shell-Hacks. Patterns + Anti-Patterns für Spec-Body-Direktiven sind zentral in **§5.6** dokumentiert (inkl.
Separation-of-Concerns zu `core/file-utils.md` `file_check`).

### 8.6 Lean-Context-Discipline-Test (Sub-Test zu §8.5)

Verifiziert, dass die neuen Wrapper-Macros (`ctx_read_lines`, `ctx_read_map`, `ctx_read_signatures`) in
`core/ctx-tools.md` definiert sind, dass `mdai/core/lean-context.md` existiert und vom spec-reviewer live
included wird, und dass keine `mode="full"` außerhalb erlaubter Whitelist-Positionen im Library- und Skill-A-Pack
auftaucht.

```bash
# Wrappers existieren in ctx-tools.md
@call ctx_search(pattern="@define ctx_read_(lines|map|signatures)", path="mdai/core/ctx-tools.md")
# Expected: 3 matches.

# Spec-reviewer included lean-context.md
@call ctx_search(pattern="^@include mdai/core/lean-context\\.md", path="mdai/skills/mdai-brainstorm/spec-reviewer.md")
# Expected: 1 match.

# tool-quick-ref erwähnt die 3 neuen Wrapper
@call ctx_search(pattern="ctx_read_(lines|map|signatures)", path="mdai/core/tool-quick-ref.md")
# Expected: 3 matches.

# mode="full" Whitelist-Check: erlaubt nur in spec-reviewer §1 (Reviewer-Target-Read)
@call ctx_search(pattern='mode="full"', path="mdai/")
# Expected: 1 match (spec-reviewer.md §1 — Reviewer-Target-Read). Jeder weitere match muss
# eine @note visible consumer="human" Justification haben (Check #10 enforced).
```

**Pass-Kriterien:** alle 4 Suchen liefern die expected counts. Bei Fail: spec-reviewer Check #10 würde das
Asset als needs-revision flaggen — der Smoke-Test fängt es früher.

## 9. (entfällt in v3)

v2 §9 "Smoke-Tests" wurde in v3 zu §8 verschoben (weil v2 §6 "Lean-ctx-Routing im generierten Plan" und v2 §7
"Spec ↔ Plan Output-Formate" zu einer kombinierten §6 "Spec-Output-Format" verschmolzen sind). §-Nummerierung ab
§10 (Bulletproofing) bleibt bewusst stabil gegenüber v2, damit Querverweise aus älteren Dokumenten (Library-Plan,
Bootstrap-Findings) ohne Reverify weiter lesbar sind. Diese §9 ist Intentional-Skip, kein Placeholder.

## 10. Bulletproofing — Red-Flags + Rationalization-Table

Discipline-enforcing Skills brauchen explizite Anti-Rationalisierungs-Strukturen
(writing-skills/SKILL.md §"Bulletproofing"). Zwei Artefakte werden in
`body.mdai.md` dialog-phase verankert. Daten kommen aus **reasoned counters**:
8 Discipline-Punkte (§10.4) + Upstream-Pattern-Observations
(`superpowers:brainstorming/SKILL.md` §Anti-Pattern + §Red Flags-Pendant in
`using-superpowers/SKILL.md`). Kein RED-Baseline (siehe §8 Intro) — explizit-invoke
macht Trigger-Discovery-Drift irrelevant; Pressure-Patterns leiten wir aus dem
8-Discipline-Cross-Check ab.

### 10.1 Red-Flags-Liste (Position: direkt nach HARD-GATE-@constraint)

**Format** in `body.mdai.md` dialog-phase:

```markdown
## Red Flags — STOP and re-enter discipline

If any of these thoughts arise, STOP, re-read the HARD-GATE constraint, and
return to the checklist:

- [Sentence-shaped self-check, one per line]
```

**Source:** reasoned-counter Self-Checks pro Discipline-Punkt (§10.4), formuliert als
„If you catch yourself thinking X, STOP". Inspiriert von Upstream-`using-superpowers/SKILL.md`
§"Red Flags" Tabelle. Erwartete Anzahl: 5–8 Einträge — mindestens einer pro Pressure-Kategorie
(cold-start, time-pressure, authority-pressure).

**Mindestens eine Zeile bezieht sich auf v3-Scope-Drift:**
> „Ich schreibe jetzt schnell den Plan dazu, der User wird sich freuen" → STOP. Skill schreibt
> KEINEN Plan. Plan-Write ist Aufgabe von `/superpowers:writing-plans` post-Spec.

### 10.2 Rationalization-Table (Position: nach Process-Details, vor Visual-Companion-Dispatch)

**Format** in `body.mdai.md` dialog-phase:

```markdown
| Excuse | Reality |
|---|---|
| [reasoned-counter pro Discipline-Punkt §10.4] | [Konter-Argument, 1 Satz] |
```

**Source:** reasoned-counter Zeilen pro Discipline-Punkt (§10.4) + Upstream-Pattern-Observations.
Die 8 mdai-Discipline-Punkte sind Kategorie-Anchor — Tabelle deckt alle 8 ab. Jede Zeile markiert
als `[reasoned-counter]` (kein verbatim-Baseline). Wenn später Drift beobachtet wird (Session-Review),
kann eine Zeile durch verbatim-Quote aus dem Drift-Vorfall ersetzt werden. Erwartete Anzahl: 8 Zeilen
(eine pro Discipline-Punkt), Spannweite 7–10 zulässig.

### 10.3 A2-Workflow (Sub-Schritte für §15 A2)

A2 schreibt `body.mdai.md` inkl. dialog-phase mit gefüllten Red-Flags +
Rationalization-Table aus reasoned counters.

**A2-Sub-Schritte:**

1. Header minimal: nur `@markdownai v1.0` (KEINE globalen Imports — Lazy-Load per Phase per §5.1).
2. Pro Discipline-Punkt (§10.4) einen 1-Satz-Self-Check formulieren → Red-Flags-Liste füllen
   (5–8 Einträge, mind. einer pro Pressure-Kategorie cold/time/authority, **plus mind. einer zur Skill-Scope-Drift
   gegen Plan-Write**).
3. Pro Discipline-Punkt eine Excuse/Reality-Zeile formulieren → Rationalization-Table füllen
   (9 Zeilen, alle 9 Discipline-Punkte abgedeckt, jede Zeile `[reasoned-counter]` markiert).
4. Upstream-`using-superpowers/SKILL.md` §"Red Flags" und `superpowers:brainstorming/SKILL.md`
   §Anti-Pattern als Quervergleich — fehlt eine gängige Rationalisierung in unserer Tabelle?
5. Lazy-Imports inline in phases einbauen: `@import write-spec.md` in `write-outputs` (vor `@call write_spec`),
   `@import spec-reviewer.md` in `dialog` Step 7.5 (vor `@call spec_reviewer_prompt`).
6. **§5.6 Konvention "Spec-Body mdai-Direktiven" als Reference-Sektion in dialog phase einbauen** — Tabelle
   (Use-Case/Best-Practice/Anti-Pattern), `file_check` Anti-Pattern-Note, und Beispiel-Block für eine generierte
   Spec. Dies ist Pflichtlektüre für Claude beim Step 6 (Write design doc) und macht §10.4 #9 operationell.
7. `wc -w` auf jede Phase (siehe §11) — falls Budget gerissen, Sub-Phase-Split (§11.3). Bei Sub-Phase-Split kann
   die §5.6-Konventions-Sektion in eine eigene `dialog-conventions`-Subphase ausgelagert werden.

### 10.4 Die 9 mdai-Discipline-Punkte (Cross-Check-Anchor)

Die `mdai-brainstorm`-Skill setzt diese 9 Disziplin-Punkte durch. Die
Rationalization-Table muss alle 9 abdecken:

1. HARD-GATE: kein Code/Plan-Write vor User-Design-Approval. **Und in v3: kein Plan-Write überhaupt — Skill schreibt
   nur Spec.**
2. Spec-Pfad: `docs/mdai/specs/` — **nicht** `docs/superpowers/specs/`.
3. File-Endung Spec: `.mdai.md` — **nicht** `.md`.
4. One-question-at-a-time im Brainstorm-Dialog.
5. Approach-Vergleich (2–3 Alternativen) **vor** Design-Präsentation.
6. Per-Section-Approval beim Design-Walkthrough.
7. Spec-Self-Review **vor** User-Review-Gate.
8. `body.mdai.md` nur phase-by-phase via MCP lesen, **nie** full.
9. **Specs nutzen markdownai-Funktionen aktiv + Lean-Context-Defaults** — (a) der produzierte Spec-Body enthält
   `@call`, `@include`, `@import`, `@list`, `@render`, `@tree`, `@constraint` etc. wo sinnvoll (Live-Inhalte statt
   hart kodierte Snapshots). Reine Plain-Markdown-Specs (nur Prosa + Tabellen + Headers, ohne markdownai-Direktiven
   im Body) verfehlen das Hauptziel (§1) und gelten als Discipline-Verletzung. (b) Cross-File-Reads verwenden die
   Lean-Context-Defaults aus `mdai/core/lean-context.md` (§5.7): `@call ctx_read_map(path)` /
   `ctx_read_signatures(path)` zum Scan, `@call ctx_read_lines(path, start, end)` nach `ctx_search` / `find_symbol`.
   `mode="full"` ist Ausnahme mit `@note visible consumer="human"`-Justification (einzig erlaubte default-Stelle:
   spec-reviewer §1 Spec-Source-Read). `ctx_shell raw=true` und `ctx_read fresh=true` ebenfalls nur mit Justification
   (`fresh=true` nur direkt nach Write/Edit auf denselben Pfad — Cache invalidiert sonst automatisch via mtime).
   Reviewer-Enforcement: §5 Anti-Pattern-Checks #10 (lean-context-defaults) + #11 (structured-data-via-@read/@list).
   Ausnahmen zu (a): Specs für Themen, wo Live-Inhalte semantisch keinen Sinn ergeben (z.B. rein algorithmische
   Design-Diskussion ohne File-/Tool-/Daten-Bezug) — diese Ausnahme muss explizit in der Spec-Frontmatter mit
   `markdownai_directives_omitted: <reason>` dokumentiert werden.

**Diff zu v2:** v2 hatte 9 Punkte. Plan-Pfad-Punkt (#3 in v2) entfernt — Plan-Pfad ist nicht Verantwortung von Skill A.
Punkte 5–9 (v2) wurden zu Punkten 4–8 (v3). Punkt 1 erweitert um v3-Scope-Klausel. **Neuer Punkt 9** (Specs nutzen
markdownai-Funktionen aktiv) reflektiert das Hauptziel aus §1.

## 11. Token-Budget pro Phase

Phase-Isolation funktioniert nur, wenn jede einzelne Phase klein ist. Hartes Cap pro Phase, Verifikation via `wc -w`
auf phase-isolierten Output. Macros sind extern (Library + Skill-A-Pack) — Phase-Bodies enthalten nur
`@call`/`@query`/`@include`-Direktiven plus Anweisungstext, nicht die Macro-Definitionen.

### 11.1 Budget-Tabelle

| Phase                  | Budget (Worte)  | Begründung                                                                                                                                                                                                                                                                                                    |
|------------------------|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| pre-context            | ≤250            | `@call mdai_bootstrap()` + Includes + `@query`-Liste + Constraints. Macros extern.                                                                                                                                                                                                                            |
| dialog                 | ≤580            | Hand-ported Checklist (~80) + Process (~120) + Principles (~50) + Red-Flags (~60) + Rationalization-Table (~110, 8 statt 9 Zeilen) + Visual-Companion (~40) + Spec-Reviewer-Dispatch (~40) + Anti-Pattern-Intro (~50) + HARD-GATE mit v3-Scope (~50) = ~580. Knapp über writing-skills Soft-Norm — toleriert. |
| write-outputs          | ≤50             | Nur 2 `@call`-Aufrufe (write_spec, render_spec) + Output-Liste + Verification-Hinweis.                                                                                                                                                                                                                        |
| handoff                | ≤80             | Next-Step-Anweisung + Backlog-Hinweis + ein `@call ctx_read` zur Verifikation.                                                                                                                                                                                                                                |
| **Total body.mdai.md** | **≤1010 Worte** | Sum + ~50 für `@import`-Header.                                                                                                                                                                                                                                                                               |

**Diff zu v2:** `write-outputs` ≤100 → ≤50 (kein `write_mdai_plan`, kein `list_phases(plan=)`). `handoff` ≤100 → ≤80
(kein `remember_plan`). `dialog` ≤600 → ≤580 (Rationalization-Table eine Zeile weniger). Total: 1050 → 1010.

### 11.2 Verifikations-Befehl

```bash
for phase in pre-context dialog write-outputs handoff; do
  count=$(mcp__markdownai__read_file \
            path=mdai/skills/mdai-brainstorm/body.mdai.md \
            phase=$phase format=ai | wc -w)
  echo "$phase: $count words"
done
```

Identisch zu §8.4 Phase-Budget-Test.

### 11.3 Eskalation bei Budget-Überschreitung

**Reihenfolge:**

1. **Sub-Phase-Split:** Falls `dialog` >580 Worte, splitten in `dialog-rules`
   (HARD-GATE + Red-Flags + Rationalization-Table) und `dialog-process`
   (Checklist + Process + Principles + Companion + Reviewer-Dispatch).
   Jede Sub-Phase <400 Worte. Kostet einen extra `read_file`-Roundtrip pro Brainstorm-Session — toleriert.
2. **Inline-Kürzung:** Process-Details und Principles auf Bullet-Form ohne
   erläuternden Text. Risiko: hand-port verliert Disziplin-Bedeutung. Nur
   wenn Split nicht möglich.
3. **Budget-Aufweichung:** Letzte Option, braucht explizite Begründung in
   `body.mdai.md`-Kommentar + neuer Eintrag in §13 Risiken.

## 12. Non-Goals

1. **`superpowers/`-Skills modifizieren** — bleiben read-only. Kein Patch in Upstream-Body.
2. **Plan-Write irgendeiner Form** — Skill schreibt **keine** Pläne, weder `.md` noch `.mdai.md`. Plan-Generierung
   ist explizit Aufgabe eines separaten Skills (`superpowers:writing-plans` jetzt, `mdai-writing-plans` zukünftig).
3. **Drift-Detection im aktuellen Scope** — separater Skill `mdai-drift-check` als Backlog-Item, kein Bestandteil
   dieser Spec.
4. **Plan-State-Persistenz** — `mdai-memory` (aus altem Design) bleibt zuständig, der künftige `mdai-writing-plans`-
   Skill ruft es auf. `mdai-brainstorm` ruft es **nicht** auf (kein Plan, kein State).
5. **`mdai-execution` und `mdai-memory` Migration** auf `mdai/skills/`-Layout — separate Spec.
6. **Library-Macros modifizieren** — `mdai/core/*`, `mdai/lang/*`, `mdai/tooling/*` sind Library-Code v0.1.0.
   Änderungen erfolgen via separater Library-Spec + Library-Version-Bump. **Ausnahme erlaubt:**
   `mdai/skills/mdai-brainstorm/spec-reviewer.md` ist per strikter Library-Definition Skill-A-Asset (siehe §3) und
   wird in §15 A2.5/A2.6 im Rahmen dieser Skill-Spec gepatched (Live-Datei synchron — Appendix A ist die
   Implementation-Referenz).

**Diff zu v2:** Punkt 2 (Plan-Write) komplett neu geschärft — v2 sagte "Automatische `.md` → `.mdai.md`-Migration alter
Pläne", v3 sagt "kein Plan-Write irgendeiner Form". Punkt 6 erweitert um Library-Boundary-Klarstellung für
spec-reviewer.md.

## 13. Risiken

| Risiko | Schweregrad | Mitigation |
| Claude folgt der hand-ported Disziplin nicht so streng wie ein echter Skill-Invoke (kein Skill-Engine-Enforcement) | *
*Mittel**  | dialog-phase formuliert Anweisungen als `@constraint id="..." severity="high"` plus HARD-GATE mit
explizitem v3-Scope-Hinweis. Discipline-Fidelity-Test §8.2 prüft Verhalten inkl. "no plan written" check.|
| Upstream-`brainstorming/SKILL.md` ändert sich nach Bump → hand-ported Slices (jetzt inkl. lines 107-136) veralten | *
*Mittel**  | `mdai-drift-check`-Backlog: Hash-Vergleich + Diff-Report. Bis dahin: bei jedem `superpowers`-Versions-Bump
manuelle Review der Source-Zeilen (16-20, 22-32, 70-104, 107-136, 140-145). |
| mdai-Library bumped → Macro-Signaturen ändern → Skill A bricht | **Mittel**  | `requires.mdai-library: ">=0.1.0"`
pinnt Minimum-Version im Spec-Frontmatter. Library `changelog.md` dokumentiert Breaking Changes. Re-Smoke-Test (§8.5)
bei jedem Library-Bump im jeweiligen Bump-Plan. |
| Claude verwechselt v3-Scope mit v2-Verhalten und schreibt trotz HARD-GATE einen Plan | **Mittel**  | Vier Bremsen: (a)
SKILL.md Description sagt explizit "this skill does not write plans"; (b) HARD-GATE-`@constraint` enthält
v3-Scope-Klausel; (c) Red-Flags-Liste enthält Plan-Drift-Self-Check; (d) Discipline-Fidelity-Test §8.2 verifiziert "no
plan file written". |
| `mai` CLI blockiert `@query`-Direktiven zur Render-Zeit (engine-include.ts security policy)                        |
Niedrig | Bekannt seit Library v0.1.0 bootstrap-findings. Smoke-Render-Test §8.5 verifiziert nur Plumbing (
Imports/Includes/Defines), nicht Live-MCP-Behavior. Live-Verifikation nur aus Claude-Code-Session möglich.|
| `mcp__markdownai__read_file(phase=…)` Server disconnected mitten in Session | Niedrig | Reconnect via `/mcp`,
headless-Fallback dokumentiert. |
| `body.mdai.md` wird trotz pointer-Anweisung full gelesen | Mittel | SKILL.md formuliert die Anweisung als
Hard-Constraint („MUST"), nicht „SHOULD". Smoke-Test §8.1 prüft. Bei Fail: 5-File-Layout-Migration als separate Spec. |
| `mcp__lean-ctx__ctx_graph` ist nicht gebaut (kein Index) → pre-context liefert leeren Graph | Niedrig |
Pre-context-Phase führt `@if`-Check: bei leerem Graph fällt zurück auf `ctx_tree` + `ctx_overview`.
`ctx_graph action="build"` einmalig in P0. |
| `mdai_bootstrap()` läuft per-render (kein Cache in Library v0.1.0) → Overhead bei jedem pre-context-Load | Niedrig |
Akzeptiert für v1. Cache-Backlog (`ctx_session`-basiert) in Library v0.2 geplant — siehe §14. |
| User-Global-Skills haben niedrigere Priorität als Plugin-Skills bei Description-Match | Niedrig | Smoke-Test §8 prüft.
Falls Konflikt: explizites `/mdai-brainstorm`-Trigger nutzen. |
| spec-reviewer.md Patch (A2.5/A2.6) konflikt mit künftigem Library-v0.1.1-Bump | Niedrig | Patch ist seit 2026-05-24
direkt in der Live-Datei `mdai/skills/mdai-brainstorm/spec-reviewer.md`; bei künftigem Library-Bump muss
`mdai/changelog.md` den Skill-A-eigenen Patch übernehmen oder bewusst überschreiben. Boundary-Diskussion siehe §3
+ §12.6. |

| `write-outputs`-Phase-Budget überschritten (Plan-Phase A2.7) | Niedrig | Budget ≤50 W per §11.1 war strukturell zu eng — tatsächlicher Inhalt (~86 W: lazy `@import write-spec.md` + `@call write_spec` + `@call render_spec` + opt-in render-targets-Liste + verification call + 2 commit/plan-write Notes). Mitigation: Budget aufgeweicht auf ≤100 W (§11.3 Stufe 3). Re-budget bei v0.2. |

**Diff zu v2:** Plan-bezogenes Risiko ("Claude schreibt trotz HARD-GATE einen Plan") neu hinzugefügt (v3-spezifisch).
Hand-ported-Slices-Risiko erweitert um lines 107-136. spec-reviewer-Patch-Konflikt neu.
**Diff zu A2:** write-outputs-Budget-Risiko ergänzt (Plan-Phase A2.7, Stufe-3-Aufweichung).

## 14. Backlog (separate Specs)

Explizit-deferred Parking-List. Jeder Eintrag bekommt bei Bedarf eine eigene Spec via
`/mdai-brainstorm`. Backlog ≠ Open Items — diese hier sind bewusst aufgeschoben.

1. **`mdai-writing-plans`-Skill** — schreibt `.mdai.md`-Pläne mit `@phase`-Markern für `mdai-execution`. Orientiert
   sich an `superpowers:writing-plans`. Schreibt eigenen plan-write-Pack unter `mdai/skills/mdai-writing-plans/`
   (die ursprüngliche `mdai/skills/mdai-brainstorm/write-mdai-plan.md` wurde am 2026-05-24 als orphaned entfernt —
   §3 Scope-Cut-Note; neuer Pack wird mit Skill-Anforderungen co-designed, nicht resurrected). `spec-reviewer.md`
   kann weiter konsumiert werden. Eigene Spec via `/mdai-brainstorm`. **Trigger für separate Spec:** nach erster
   stabiler `/mdai-brainstorm`-Session, vor erstem produktiven Bedarf an `.mdai.md`-Plänen mit `@phase`-Markern.
   **Priorität: hoch.**
2. **`mdai-drift-check`** — Skill zum Upstream-Hash-Vergleich + Diff-Report. Hash-Store in
   `docs/mdai/upstream-hashes.json`. Manueller Trigger oder periodisch via `loop`/`schedule`.
   Trigger für separate Spec: spätestens beim ersten `superpowers`-Versions-Bump nach Release.
3. **`mdai-execution`-Migration** auf `mdai/skills/mdai-execution/`-Layout. Inhaltlich unverändert, nur Pfad-Umzug.
   Trigger: vor nächster substantieller mdai-execution-Änderung.
4. **`mdai-memory`-Migration** dito.
5. **Upstream-PR an markdownai** für `respondTool()`-Fix (separat, blockt nichts). Trigger: jederzeit.
6. **Plugin-Packaging** — Bündelung aller mdai-Skills als ein Claude-Code-Plugin (eigene `package.json`,
   `hooks.json` Stub). Trigger: nach mind. einem Monat stabiler Nutzung der Skills.
7. **Spec-Human-Render-Wrapper-Template** — separate `.mdai.md`-Datei
   unter `mdai/skills/mdai-brainstorm/templates/` mit Layout-Macros (Cover-Page mit Branch/Date, automatische
   TOC, Constraint-Glossar). Bei aktiviertem Render zielt `render_spec` auf den Wrapper. Aktuell nicht nötig —
   `render_spec` rendert direkt das Source-Spec. Trigger für separate Spec: sobald ein Reviewer
   Cover/TOC/Glossar reproduzierbar verlangt.
8. **`mdai_bootstrap`-Cache** (Library v0.2-Backlog) — Session-scoped Cache via
   `ctx_session action="finding"/"status"` reduziert per-render-Overhead. Marker-Format aus Library
   v0.1.0 changelog: `[mdai-bootstrap-cache] tooling=detected lang=<LANG> jetbrains=<bool> serena=<bool>`.
   Trigger: messbarer Overhead bei Skill-A-Live-Nutzung.
9. **Globaler Install + `install.sh`** — `mdai/skills/mdai-brainstorm/scripts/install.sh` schreiben (kopiert `SKILL.md`
    + `body.mdai.md` nach `~/.claude/skills/mdai-brainstorm/`). Macht Skill projekt-übergreifend verfügbar. **Während
      Entwicklung bewusst deferred** — vermeidet Cross-Projekt-Trigger und Schaden bei Skill-Iteration. Trigger: nach
      mind. einer stabilen `/mdai-brainstorm`-Session-Reihe im lean-ctx-Repo (z.B. 3 Specs ohne Drift) und vor
      Plugin-Packaging (§14.6).

**Diff zu v2:** Neuer Eintrag #1 (`mdai-writing-plans`-Skill) mit **hoher Priorität** — alle anderen rutschen eine
Position nach hinten.

## 15. Implementierungsschritte (high-level, der echte Plan kommt via writing-plans-Skill)

| Phase | Aufgabe |
| P0 | `mcp__lean-ctx__ctx_graph action="build"` einmalig laufen lassen, damit pre-context-Phase einen Index hat. *
*Plus:** `.superpowers/` in `.gitignore` aufnehmen (persistente Visual-Companion-Mockups via `--project-dir "$PWD"`). *
*Hinweis:** `docs/mdai/macros/`-Mirror entfällt — durch Library v0.1.0 (`mdai/core/*`, `mdai/skills/mdai-brainstorm/*`)
abgedeckt. Kein RED-Baseline (siehe §8 Intro). |
| A1 | `mdai/skills/mdai-brainstorm/SKILL.md` schreiben (~15 Z, pointer). Description aus §4 (Trigger-only, expliziter "
does not write plans"-Hinweis). |
| A2 | `mdai/skills/mdai-brainstorm/body.mdai.md` schreiben (~100 Z, alle 4 Phasen). **Sub-Schritte (§10.3):** (a)
Header minimal (`@markdownai v1.0` only, **keine globalen Imports** — Lazy-Load per Phase per §5.1); (b) Red-Flags-Liste
aus reasoned counters füllen (5–8 sentence-form, **mind. einer zur v3-Scope-Drift gegen Plan-Write**, §10.1); (c)
Rationalization-Table aus reasoned counters füllen (9 Zeilen, alle 9 Discipline-Punkte §10.4 abgedeckt, §10.2); (d)
Upstream-Cross-Check (`using-superpowers/SKILL.md` §"Red Flags" + `superpowers:brainstorming` §Anti-Pattern); (e)
Lazy-Imports inline in phases: `@import write-spec.md` in write-outputs, `@import spec-reviewer.md` in dialog Step 7.5;
**(f) §5.6 Konventions-Block in dialog phase als Reference-Sektion einbauen** (
Use-Case/Best-Practice/Anti-Pattern-Tabelle + `file_check`-Trennung + Beispiel-Block); (g) `wc -w` pro Phase gegen
§11.1-Budget — bei Überschreitung Sub-Phase-Split (§11.3). |
| A2.5 | `mdai/skills/mdai-brainstorm/spec-reviewer.md` **patchen** (Calibration-Paragraph + Quick-Scan-Tabelle +
Recommendations-Sektion gemergt aus upstream `spec-document-reviewer-prompt.md`, **plus Anti-Pattern-Check #9** für
mdai-Direktiven-Nutzung per §10.4 #9 + §5.6, **plus §0 Lean-Context-Discipline** via
`@include mdai/core/lean-context.md`, **plus Checks #10 lean-context-defaults + #11 structured-data-via-@read/@list**
per §5.7). `lib_version: 0.1.0` bleibt (Skill-A-Asset, kein Library-Bump). Synchron mit diesem Spec am 2026-05-24
in die Live-Datei übernommen. |
| A2.6 | **Library-Additive (kein Bump):** (a) `mdai/core/lean-context.md` neu anlegen (mode: include, kein
Frontmatter, kein @define; nur Discipline-Rules-Tabelle + "Why bounded by default"-Absatz + Naming-Convention).
(b) `mdai/core/ctx-tools.md` Frontmatter-Exports um `ctx_read_lines`, `ctx_read_map`, `ctx_read_signatures`
erweitern + die drei `@define`-Blöcke hinzufügen. (c) `mdai/core/tool-quick-ref.md` um 3 Zeilen ergänzen (Mapping
Task → neuer Wrapper). (d) `mdai/changelog.md` zwei additive Einträge (Lean-Context-Discipline + MACROS.md-removal).
(e) `mdai/MACROS.md` **löschen** — Inventory wird durch `ctx_search` auf `mdai/core/*` ersetzt; nur die Naming-
Convention wandert nach `lean-context.md`. Library bleibt v0.1.0 (additive Update, pre-stable). |
| A3 | **Project-local Test-Setup:**
`mkdir -p .claude/skills && ln -sf ../../mdai/skills/mdai-brainstorm .claude/skills/mdai-brainstorm`. Verifiziere:
`ls -la .claude/skills/mdai-brainstorm/` zeigt Symlink. **Hinweis:** kein `install.sh`, kein globaler Install während
Entwicklung — siehe §3 und §14.9. |
| A4 | **Smoke-Tests gegen project-local Symlink (§8.1–§8.5).** Inkl. Library-Import-Smoke-Test (§8.5). Skill ist nur in
diesem Repo aktiv, beeinflusst andere Projekte nicht. Iteriere am Source unter `mdai/skills/mdai-brainstorm/`, bis alle
Tests grün sind. **Bei §8.1 Fail (0/3 oder 1/3 Pass):** A2 re-do als 5-File-Layout-Migration (separate Spec). |
| A5 | `mdai-plans` deinstallieren (falls global vorhanden): `rm -rf ~/.claude/skills/mdai-plans/`, verifiziere via
`/mdai-plans` triggert nicht mehr. Falls nur lokal: `rm -rf .claude/skills/mdai-plans`. |
| A5.5 | **Workflow-Übergang dokumentieren** (klein, optional aber empfohlen): Übergang ist primär in `body.mdai.md`
handoff-Phase (§5.5) hartkodiert. Zusätzlich kurzer Hinweis in `mdai/skills/mdai-brainstorm/README.md` (neue Datei, ~10
Z) — **nicht** in SKILL.md (bleibt schlank per §4). Inhalt: nach `/mdai-brainstorm` Spec-Approval **manuell**
`/superpowers:writing-plans <spec-path>` aufrufen, bis `mdai-writing-plans` (§14 Backlog #1) existiert. |

**Diff zu v2:** v2 §15 hatte A6 (Self-Bootstrap: Plan-Datei per `/mdai-brainstorm` produziert) — **entfällt komplett**.
Skill kann seinen eigenen Plan nicht mehr produzieren (schreibt keine Pläne). A5.5 neu eingeführt für Workflow-Übergang-
Dokumentation. A2.5 neu eingeführt für spec-reviewer.md-Patch (Skill-A-Asset, nicht Library).

## 16. Annahmen, die in Smoke-Tests zu verifizieren sind

1. `mcp__markdownai__read_file(consumer="human", format="standard")` rendert eine Spec für Human-Konsum korrekt
   (verifiziert mit mai-CLI v0.0.24, MCP wrapped denselben Renderer). Smoke-Test §8 prüft den MCP-Output direkt.
2. `mcp__markdownai__read_file(phase=…, format=ai)` returniert eine self-contained Phase inkl. relevanter
   `@define`-Macros (geerbt von der Datei-Header via `@import`). Falls nicht: Macros müssen in jeder Phase neu
   deklariert
   werden — wäre Library-Bug, blockiert Skill A.
3. Claude Codes Skill-Loader respektiert die pointer-Anweisung „lies body.mdai.md nicht full". Verifikation §8.1
   Pointer-Compliance-Test (3 Runs, `jq`-Grep auf Session-Transcript). Bei 0/3 oder 1/3 Pass → 5-File-Layout-Migration
   als separate Spec.
4. Claude folgt der hand-ported Disziplin in `dialog`-phase ohne `Skill(superpowers:brainstorming)`-Invoke (
   Discipline-Fidelity-Test §8.2). Plus: **Claude schreibt am Ende der Session keinen Plan-File** — verifiziert in
   §8.2 v3-Augmentation. Falls Claude die Disziplin lockerer interpretiert als ein echter Skill-Invoke:
   `@constraint id="hard-gate" severity="high"`-Block mit v3-Scope-Klausel schärfen, oder als Fallback Hybrid (
   Skill-Invoke + hand-ported Pre-Briefing) zurück erwägen.
5. `mcp__lean-ctx__ctx_graph` ist im Projekt bereits gebaut (oder wird in P0 einmal indiziert). Falls Index leer:
   pre-context-Phase fällt auf `ctx_tree` + `ctx_overview` zurück.
6. `@import mdai/skills/mdai-brainstorm/write-spec.md` (in write-outputs-Phase, lazy) und
   `@import mdai/skills/mdai-brainstorm/spec-reviewer.md` (in dialog-phase Step 7.5, lazy) laden die Skill-A-Pack-
   `@define`s korrekt — Static-render-test §8.5 verifiziert. Falls Imports nicht aufgelöst werden (z.B.
   Pfad-relativ-Problem): Library-Bug, blockiert Skill A. **Nur 2 Packs existieren** (gegenüber v2: 3) —
   `write-mdai-plan.md` wurde am 2026-05-24 entfernt (§3 Scope-Cut-Note), kommt erst mit `mdai-writing-plans`
   (§14 Backlog #1) zurück. **Wichtig:** v3 lädt Packs phase-lokal, nicht am Datei-Header (siehe §5.1
   Lazy-Load-Prinzip).

Diese Annahmen sind die einzigen blockierenden Unbekannten. Alle anderen Risiken sind mitigierbar oder akzeptiert.

**Diff zu v2:** Annahme 4 erweitert um "schreibt keinen Plan-File"-Check. Annahme 6 reduziert von 3 auf 2 Packs.

---

## Appendix A — `spec-reviewer.md` Patch (synchronized 2026-05-24)

Der vollständige gemergte Body, der hier ursprünglich inlined war, lebt jetzt **direkt** in
`mdai/skills/mdai-brainstorm/spec-reviewer.md` (synchron in derselben Patch-Session am 2026-05-24 übernommen).
Implementation-Referenz für A2.5/A2.6:

- **§0 Lean-Context-Discipline** (`@include mdai/core/lean-context.md` — rendert die Discipline-Tabelle inline im
  Reviewer-Prompt)
- **§1 Read the spec** (`mode="full"` auf `{{ spec_path }}` — einziger erlaubter `mode="full"`-Aufruf im Review-Job)
- **§2 What to Check** (Quick-Scan, 5-Spalten-Tabelle aus upstream — Anti-Pedantry-Schwellenwert)
- **§3 Systematic Deep-Checks** (mdai-spezifisch — Objective, Assumptions, Risks, Non-Goals, Cross-Spec,
  RED/GREEN-Setup)
- **§4 Calibration** (verbatim from upstream — Anti-Pedantry-Bremse)
- **§5 mdai-spezifische Anti-Pattern-Checks #1–#11:**
  - #1–#8 unverändert (MCP-Signatures, Existing-Store, mai-CLI, Frontmatter-Convention, Repo-Paths, Language,
    Parameter-Names, Smoke-Render)
  - #9 markdownai-directives-active (per §10.4 #9 + §5.6 + Sub-Check `file_check` is not branching)
  - **#10 lean-context-defaults** (per §5.7 — alle Tools aus `core/lean-context.md` Discipline-Tabelle)
  - **#11 structured-data-via-`@read`/`@list`** (Tables mit externer SoT oder >50 Z)
- **§6 Report Format** (Status / Strengths / Gaps / Concrete patches / Recommendations — gemergte mdai + upstream)
- **§7 Output** (`docs/mdai/reviews/<basename>-review.md`)
- **§8 Tools** (Lean-Context-Defaults FIRST — `ctx_read_map` / `ctx_read_signatures` / `ctx_read_lines` — dann
  library wrappers, dann native MCP fallback; **no `mode="full"` außer §1**)

**Verifikation der Synchronisation:** Smoke-Test §8.6 prüft via `ctx_search`, dass `@include mdai/core/lean-context.md`
im spec-reviewer Live-File präsent ist und die drei Wrapper-Macros in `core/ctx-tools.md` definiert sind. Drift
zwischen dieser Appendix-Sektion und der Live-Datei wird durch §8.6 detektiert.

**Merge-Diff gegenüber Library v0.1.0 Initialzustand:**

- **Neu §0** Lean-Context-Discipline via `@include` (2026-05-24).
- **Neu §2** Quick-Scan-Tabelle (5 Spalten) aus upstream — Quick-Scan vor Deep-Checks.
- **Neu §4** Calibration-Paragraph verbatim aus upstream — Anti-Pedantry-Bremse.
- **Neu in §6** Recommendations-Sektion (advisory, do not block approval) aus upstream.
- **Geändert in §6** Gaps von "≥3 or 'none'" zu "≥0; calibrate per §4" — entfernt Forced-Pedantry, behält
  Anti-Pattern-Check-Erzwingung.
- **Neue Checks #9, #10, #11** in §5 (markdownai-directives-active, lean-context-defaults,
  structured-data-via-`@read`/`@list`).
- **Unverändert** §5 #1–#8 mdai-spezifische Anti-Pattern-Checks.
- **Unverändert** §7 Output-Pfad und §8 Tool-Präferenz.
- **Header & Macro-Signatur** unverändert (`spec_reviewer_prompt(spec_path)`).
