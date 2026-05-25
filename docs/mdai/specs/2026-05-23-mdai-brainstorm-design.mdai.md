---
id: mdai-brainstorm
status: design
created: 2026-05-23
updated: 2026-05-23
spec_version: 2
supersedes: docs/mdai/design-skill-integration.md §4 (mdai-plans)
requires:
  mdai-library: ">=0.1.0"
  mcp__lean-ctx: ">=3.6.16"
  mcp__markdownai: ">=0.0.24"
---

@markdownai v1.0 consumer="ai"

# mdai-brainstorm — Markdownai-aware Brainstorming-Skill (Design)

Status: spec_version=2, an mdai-library v0.1.0 angepasst. Bezug: ersetzt den `mdai-plans`-Pfad aus
`docs/mdai/design-skill-integration.md` §4. Die restlichen Skills (`mdai-execution`, `mdai-memory`) aus jenem Design
bleiben unverändert. Voraussetzung: `mdai/MACROS.md` v0.1.0 (Library mit Skill-A-Pack: `write_spec`, `render_spec`,
`plan_frontmatter`, `plan_phase`, `plan_step`, `write_mdai_plan`, `spec_reviewer_prompt`).

## 1. Zielsetzung

Eine einzelne Skill `mdai-brainstorm` bündelt drei Aufgaben:

1. **Brainstorming-Dialog** (Anforderungen klären, Approach-Vergleich, Design-Sektionen) — hand-ported aus
   `superpowers:brainstorming` in `body.mdai.md`.
2. **Spec-Write** — schreibt versionierte Design-Dokumente als `*.mdai.md` mit `consumer="ai"` (Default).
   Render in `*.md` ist Opt-in über `@call render_spec(slug, target)` aus der Library.
3. **Plan-Write** — schreibt `.mdai.md`-Pläne mit `@phase`-Markern via `@call write_mdai_plan(slug, phases)`,
   kompatibel zu `mdai-execution`.

Trigger: `/mdai-brainstorm` oder Description-Match.

**Erfolgskriterien:**

1. `body.mdai.md` ruft `@call mdai_bootstrap()` als erste Zeile in `pre-context` auf. Bootstrap löst service-check
   + lang/tooling-detection auf, lädt bedingte Packs (`lang/rust.md` falls Rust-Repo, `tooling/jetbrains.md` /
   `tooling/serena.md` falls erkannt). Pre-context resolveiert lean-ctx-Kontext live (inkl. `ctx_graph`-Map),
   ohne Tool-Roundtrips im Dialog.
2. Upstream-`superpowers:brainstorming` wird **hand-ported** (Fork minimal): Dialog-Disziplin (Checklist, Process,
   Key Principles, Red-Flags, Rationalization-Table) wird **einmal** in `body.mdai.md` kopiert. Drift per
   `mdai-drift-check` (Backlog).
3. Skill A definiert **keine eigenen Macros**. Alle Macros werden aus `mdai/MACROS.md`-Library v0.1.0 importiert:
   `@import mdai/skills/mdai-brainstorm/{write-spec,write-mdai-plan,spec-reviewer}.md`. Spec-Frontmatter pinnt
   `requires.mdai-library: ">=0.1.0"`.
4. Spec wird als `*.mdai.md` mit `consumer="ai"` committet (Default). Render ist Opt-in mit drei Targets:
   `none` (Default), `chat` (inline via `mcp__markdownai__read_file`), `file` (zusätzliches `*.rendered.md` unter
   `docs/mdai/specs/rendered/` via `npx mai render`-CLI).
5. Plan wird als `.mdai.md` mit `@phase`-Markern geschrieben, kompatibel zu `mdai-execution`.
6. Schritte mit Datei-Operationen nutzen Library-Wrapper aus `mdai/core/ctx-tools.md`
   (`@call ctx_read` / `ctx_search` / `ctx_tree` / `ctx_shell` / `ctx_edit`), Projekt-Dependencies aus
   `mcp__lean-ctx__ctx_graph` (Mermaid-Diagramm).

## 2. Architektur-Überblick

```
│  /mdai-brainstorm                                                     │
│                                                                       │
│  SKILL.md  (~15 Z, pointer)                                           │
│    └─ Anweisung: "Lade body.mdai.md immer phase-für-phase via         │
│       mcp__markdownai__read_file(phase=…, format=ai). Niemals full."  │
│                                                                       │
│  body.mdai.md (~120 Z, live doc)                                      │
│    ├─ @markdownai v1.0                                                │
│    │                                                                  │
│    ├─ @import mdai/skills/mdai-brainstorm/write-spec.md               │
│    │   exports: write_spec, render_spec                               │
│    ├─ @import mdai/skills/mdai-brainstorm/write-mdai-plan.md          │
│    │   exports: plan_frontmatter, plan_phase, plan_step,              │
│    │            write_mdai_plan                                       │
│    ├─ @import mdai/skills/mdai-brainstorm/spec-reviewer.md            │
│    │   exports: spec_reviewer_prompt                                  │
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
│    │                                                                  │
│    ├─ @phase write-outputs                                            │
│    │    @call write_spec(slug=<topic>, body=<design-content>)         │
│    │    @call render_spec(slug=<topic>, target=<none|chat|file>)      │
│    │    @call write_mdai_plan(slug=<topic>, phases=<phase-list>)      │
│    │    @call list_phases(plan=<plan-path>)                           │
│    │                                                                  │
│    └─ @phase handoff                                                  │
│         @call list_phases(plan=<plan-path>)                           │
│         @call remember_plan(plan_id=<slug>, body=<state-json>)        │
│         Anweisung: "Next step: /mdai-execution <plan-path>"           │
```

**Verhältnis zu Bestandsskills:**

| Skill                        | Behandlung                                                                                                                                  |
| `superpowers:brainstorming`  | **Hand-ported** in dialog-phase (Disziplin-Slices in body.mdai.md kopiert; kein Skill-Invoke, kein `@include` zur Renderzeit)               |
| `superpowers:writing-plans`  | **Re-templated** als Library-Macros (`mdai/skills/mdai-brainstorm/write-mdai-plan.md`)                                                       |
| `superpowers:writing-skills` | Wird einmal invoked, um `mdai-brainstorm` selbst zu schreiben (Bootstrap)                                                                   |
| `mdai-plans` (alt)           | **Wird abgelöst** und im Plan deinstalliert (entfernt aus `~/.claude/skills/`)                                                              |
| `mdai-execution`             | Unverändert, bleibt als nächster Step nach `mdai-brainstorm`                                                                                |
| `mdai-memory`                | **Foundation gestartet** in `mdai/core/ctx-knowledge.md` (Library v0.1.0): `remember_plan`, `recall_plan`, `add_gotcha`, `list_gotchas`. Skill-Wrapper folgt — invocable von `mdai-execution`. |
| `mdai-drift-check` (Backlog) | Wird nach Implementation gebraucht, um Drift zwischen Upstream-`brainstorming/SKILL.md` und hand-ported dialog-phase zu detektieren         |

## 3. Datei-Layout

**Library (in git, bereits vorhanden — live-resolved via `@tree`, Pfad relativ zur Spec-Datei):**

```
@tree ../../../mdai/ depth=2
```

Exports pro Pack (Macro-Namen, snake_case):

- `core/startup-check.md`: `mdai_bootstrap`, `service_check`, `detect_project_lang`, `detect_tooling`, `load_lang_pack`, `load_tooling_packs`
- `core/hard-rules.md`: text only (`mode: include`)
- `core/tool-quick-ref.md`: text only (`mode: include`)
- `core/ctx-tools.md`: `ctx_read`, `ctx_search`, `ctx_tree`, `ctx_shell`, `ctx_edit`
- `core/mcp-markdownai.md`: `read_phase`, `list_phases`, `get_constraints`
- `core/ctx-knowledge.md`: `remember_plan`, `recall_plan`, `add_gotcha`, `list_gotchas` — **Foundation für `mdai-memory`**
- `lang/rust.md`: `cargo_nextest`, `cargo_clippy`, `cargo_fmt`, `rustfmt_file`, `format_file` (conditional `MDAI_PROJECT_LANG=rust`)
- `tooling/jetbrains.md`: `reformat_file`, `step_reformat_commit`, `get_file_errors` (conditional `MDAI_HAS_JETBRAINS=true`)
- `tooling/serena.md`: `find_symbol`, `replace_symbol_body`, `insert_*_symbol`, `symbols_overview` (conditional `MDAI_HAS_SERENA=true`)
- `skills/mdai-brainstorm/write-spec.md`: `write_spec`, `render_spec`
- `skills/mdai-brainstorm/write-mdai-plan.md`: `plan_frontmatter`, `plan_phase`, `plan_step`, `write_mdai_plan`
- `skills/mdai-brainstorm/spec-reviewer.md`: `spec_reviewer_prompt`

Vollständige Inventartabelle: `mdai/MACROS.md` (Library v0.1.0).

**Skill-A Source (neu, dieses Spec implementiert — koexistiert mit Library-Pack-Files):**

`mdai/skills/mdai-brainstorm/` existiert bereits (Library v0.1.0: `write-spec.md`, `write-mdai-plan.md`,
`spec-reviewer.md`). Skill A fügt zwei Dateien hinzu, der Ordner bleibt der gleiche:

```
mdai/skills/mdai-brainstorm/
  SKILL.md               ← Skill A pointer (neu, A1)
  body.mdai.md           ← Skill A live workflow (neu, A2)
  write-spec.md          ← Library pack (vorhanden)
  write-mdai-plan.md     ← Library pack (vorhanden)
  spec-reviewer.md       ← Library pack (vorhanden)
```

Die Namespace-Konvention `mdai/skills/<skill-name>/` macht Platz für zukünftige Migration von `mdai-execution` und
`mdai-memory` in dieselbe Struktur (Backlog).

**Install-Ziel während Entwicklung: project-local Symlink, KEIN globaler Install.**

```
.claude/skills/mdai-brainstorm  →  ../../mdai/skills/mdai-brainstorm  (Symlink)
```

Claude Code lädt Skills aus `.claude/skills/` automatisch projekt-scoped. Der Symlink macht den Source-Ordner unter
`mdai/skills/` zum Discovery-Target — Edits am Source sind sofort live, kein zweiter Build-Step. **Kein**
`~/.claude/skills/`-Install solange Skill noch in Entwicklung ist (vermeidet Cross-Projekt-Trigger / Cross-Projekt-Schaden bei Iteration). Globaler Install + `install.sh`-Script ist Backlog-Item (siehe §14.8) und erfolgt erst nach mind. einer stabilen `/mdai-brainstorm`-Session-Reihe im Repo.

## 4. SKILL.md (pointer)

Inhaltlich knapp, ~15 Zeilen. Verantwortung: Frontmatter mit Description (Trigger-Match), plus harte Anweisung an
Claude, **nur** phase-für-phase aus `body.mdai.md` zu lesen.

```markdown
---
name: mdai-brainstorm
description: Use when starting creative work that will produce both a versioned
  design spec under docs/mdai/specs/ and a multi-phase .mdai.md plan under
  docs/mdai/plans/ for parallel subagent dispatch.
---

# mdai-brainstorm — pointer

DO NOT read this file's body for the workflow. The live workflow is in
`body.mdai.md` and MUST be loaded phase-by-phase via:

mcp__markdownai__read_file(path="<...>/body.mdai.md", phase="<phase-id>", format="ai")

Start with phase `pre-context`. Then `dialog`, `write-outputs`, `handoff`
in order. Never call `read_file` without a `phase=` argument.

Phases: pre-context | dialog | write-outputs | handoff
```

## 5. body.mdai.md (live workflow)

Skill A definiert **keine eigenen Macros**. Alle Macros werden aus der mdai-Library v0.1.0 importiert. Bootstrap
(`@call mdai_bootstrap()`) muss als erste Zeile in `pre-context` stehen — Library-Konvention aus `mdai/MACROS.md`.

### 5.1 Imports (Datei-Header, vor allen Phasen)

```markdown
@markdownai v1.0

@import mdai/skills/mdai-brainstorm/write-spec.md
@import mdai/skills/mdai-brainstorm/write-mdai-plan.md
@import mdai/skills/mdai-brainstorm/spec-reviewer.md
```

Diese drei `@import`-Anweisungen laden die `@define`-Blöcke der Library-Files (mode=import-only — kein Text-Output).
Verfügbare Macros nach Import: `write_spec`, `render_spec`, `plan_frontmatter`, `plan_phase`, `plan_step`,
`write_mdai_plan`, `spec_reviewer_prompt`. Naming-Konvention: snake_case (Library-Standard, vgl. `mdai/MACROS.md`).

Weitere Library-Wrapper werden über `@call mdai_bootstrap()` (siehe §5.2) transitiv geladen.

### 5.2 Phase: pre-context

Bootstrap-Call als erste Zeile. Anschließend pre-resolved Projekt-Kontext. Alle `@call`/`@query` zielen explizit auf
lean-ctx-Wrapper aus der Library, damit Output-Kompression greift.

**Tool-Selection-Policy (gilt für die gesamte body.mdai.md):**

- File lesen → `@call ctx_read(path, mode)` (nicht `ctx_shell(cmd="cat ...")`)
- Verzeichnis listen → `@call ctx_tree(path, depth)` (nicht `ctx_shell(cmd="ls ...")` oder `find`)
- Pattern-Suche → `@call ctx_search(pattern, path)` (nicht `ctx_shell(cmd="grep ...")` oder `rg`)
- File-Edit ohne Read → `@call ctx_edit(path, old, new)`
- Plan-Phase lesen → `@call read_phase(plan, phase_id)` (nicht raw `mcp__markdownai__read_file`)
- Plan-Phasen listen → `@call list_phases(plan)`
- Gotcha-Recall → `@call list_gotchas(query)`
- Plan-State persistieren → `@call remember_plan(plan_id, body)` / `@call recall_plan(plan_id)`
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
- Plan target: docs/mdai/plans/ (NOT docs/superpowers/plans/)
- Hard rules: see @include above
@end
```

`mdai_bootstrap()` ist per-render in v0.1.0 (kein Cache). Bei jedem `@call read_phase(plan="…/body.mdai.md",
phase_id="pre-context")` (Library-Wrapper aus `mdai/core/mcp-markdownai.md`) läuft die Detection neu. Akzeptiertes
Overhead (~3-5 Tool-Calls pro Bootstrap), Cache als Library-Backlog (siehe §14.7).

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
6. Write design doc to docs/mdai/specs/  ← OVERRIDDEN
7. Spec Self-Review (4 checks — see §"Spec Self-Review" below)
7.5 OPTIONAL: dispatched reviewer-subagent via `spec_reviewer_prompt` (mdai-Augmentation)
8. User reviews written spec (exact wording — see §"User-Review-Gate")
9. Transition: write .mdai.md plan to docs/mdai/plans/  ← OVERRIDDEN
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
| [Filled in A2 from reasoned counters; ~8–12 rows, alle 9 Discipline-Punkte (§10.4) abgedeckt] | |

Tabelle deckt alle 9 mdai-Discipline-Punkte (siehe §10.4) ab. Jede Zeile mit
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

  {{ @call ctx_read(path="~/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/visual-companion.md", mode="full") }}

  Start the companion server (persistent mockups under `.superpowers/brainstorm/`):

  @call ctx_shell(cmd="~/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/start-server.sh --project-dir \"$PWD\"")

  **Hinweis (Anti-Pattern-Check 5 / Versions-Pin):** Pfad `~/.claude/plugins/.../superpowers/5.1.0/...` ist auf Upstream-Version v5.1.0 gepinnt (`mdai-drift-check`-Backlog §14.1). Bei Upstream-Bump muss die Versionsnummer aktualisiert werden. Glob `*` vermieden, da `ctx_read` keine Globs auflöst und Anti-Pattern-Check 5 absolute home-paths in Library-Code verbietet — siehe §13 Risiko.

  Capture `screen_dir` and `state_dir` from the server-info JSON for subsequent screen pushes.
  Ensure `.superpowers/` is listed in `.gitignore` (see §15 P0).
@endif

## Spec Self-Review (step 7, MANDATORY, Claude himself)

After the spec source (`.mdai.md`) is written, look at it with fresh eyes and check:

1. **Placeholder scan:** any "TBD", "TODO", incomplete sections, vague requirements? Fix inline.
2. **Internal consistency:** sections contradict each other? Architecture matches feature descriptions?
3. **Scope check:** focused enough for a single plan? Or needs decomposition into sub-projects?
4. **Ambiguity check:** any requirement interpretable two different ways? Pick one, make it explicit.

Fix issues inline. No re-review loop — fix and move on. Hand-ported verbatim from upstream §"Spec Self-Review".

## Spec reviewer dispatch (step 7.5, OPTIONAL, mdai-Augmentation)

Optional augmentation **beyond upstream**. Dispatch a reviewer subagent with
`@call spec_reviewer_prompt(spec_path=<path>)` as prompt body (Library macro in
`mdai/skills/mdai-brainstorm/spec-reviewer.md`). Runs 8 mdai-spezifische
anti-pattern checks: MCP signatures, existing-store check, mai-CLI `@query`
block, frontmatter convention for `mode: include`, repo-relative paths,
language convention, parameter names, smoke-render. Returns `Status` +
`Issues` + `Recommendations`. Apply issues inline; surface recommendations.

Trigger: spec touches MCP signatures, Library packs, or render flow. Skip for
pure-prose specs (Self-Review §7 reicht).

## User-Review-Gate (step 8, exact wording, MANDATORY)

After Self-Review (and optional reviewer dispatch), ask the user with this wording:

> "Spec geschrieben und committed nach `<path>`. Bitte review und gib Feedback,
> ob du Änderungen willst, bevor wir den Implementations-Plan schreiben."

Wait for explicit response. If user requests changes → patch inline → re-run
Self-Review §7. Only proceed to write-outputs phase once user explicitly approves.

Collect for the next phase:

- `slug` — kebab-case topic name (e.g. "user-onboarding-flow")
- `design_content` — full design body as Markdown
- `phase_list` — phase IDs to emit in the .mdai.md plan
@end
```

**Drift-Tracking:** Header der hand-ported Sektion enthält Verweis auf Upstream-Quelle (
`# Hand-ported from superpowers/5.1.0/.../brainstorming/SKILL.md, lines 16-20, 22-32, 70-104, 140-145`).
`mdai-drift-check` (Backlog) liest diese Annotation, hasht die Source-Zeilen, vergleicht mit gespeichertem Hash, meldet
Diffs.

### 5.4 Phase: write-outputs

```markdown
@phase write-outputs

@call write_spec(slug={{ slug }}, body={{ design_content }})
@call render_spec(slug={{ slug }}, target={{ render_target | default("none") }})
@call write_mdai_plan(slug={{ slug }}, phases={{ phase_list | map(p => plan_phase(p.id, p.title, p.files, p.steps)) | join }})

Default output (two files staged in working tree):

- docs/mdai/specs/<date>-<slug>-design.mdai.md (spec source, consumer="ai")
- docs/mdai/plans/<date>-<slug>.mdai.md (plan, consumer=ai)

Opt-in render targets (passed via `render_target` collected in dialog step 6):

- target="none" (default) → no render
- target="chat" → render inline via mcp__markdownai__read_file (no file)
- target="file" → adds docs/mdai/specs/rendered/<date>-<slug>.rendered.md via npx mai render CLI

Verification (lean-ctx-first, kein ctx_shell wo Wrapper existiert):
@call list_phases(plan="docs/mdai/plans/<date>-<slug>.mdai.md")  # validates plan parses
@call ctx_tree(path="docs/mdai/", depth=2)                       # verifies new files appear in tree

Note: commit is left to the user (per user CLAUDE.md rules — never auto-commit).
@end
```

### 5.5 Phase: handoff

```markdown
@phase handoff

Plan ready for execution. Next step:

/mdai-execution docs/mdai/plans/<date>-<slug>.mdai.md

Phase inventory:
{{ @call list_phases(plan="docs/mdai/plans/<date>-<slug>.mdai.md") }}

Persist plan state (for cross-session resume):
@call remember_plan(plan_id="{{ slug }}", body='{"phases": [...], "current_phase": "P0", "status": "planned"}')
@end
```

## 6. Lean-ctx-Routing im generierten Plan

Generierter `.mdai.md`-Plan enthält keine direkten Shell-Calls. Alle Discovery/Read-Direktiven gehen über Library-Wrapper
(`mdai/core/ctx-tools.md`, transitiv via `mdai_bootstrap()` geladen):

| Plan-Intent        | Macro (preferred)                          | Begründung                  |
| Datei lesen        | `@call ctx_read(path, mode)`               | Cached, ~13 Tok bei Re-Read |
| Verzeichnis listen | `@call ctx_tree(path, depth)`              | Kompakter als `ls -R`       |
| Pattern-Suche      | `@call ctx_search(pattern, path)`          | Token-effizient             |
| Shell-Op           | `@call ctx_shell(cmd)`                     | 95+ Kompressions-Pattern    |
| Edit ohne Read     | `@call ctx_edit(path, old, new)`           | Wenn Read nicht verfügbar   |

Die Mapping-Referenz für Plan-Autoren ist `mdai/core/tool-quick-ref.md` (im pre-context `@include`-ed). Plan-Bodies
referenzieren die snake_case-Macros direkt — kein zusätzlicher `@include`-Block erforderlich.

## 7. Spec ↔ Plan Output-Formate

| Artefakt | Source-Datei | Render-Datei                                                 | Format-Direktive                  | Konsument                                                                |
| Spec     | `*.mdai.md`  | **— (Default: kein Render-File)**                            | `@markdownai v1.0 consumer="ai"`  | mdai-execution-Subagents; User via `.mdai.md`-Source oder Opt-in Render |
| Plan     | `*.mdai.md`  | — (kein eigenständiges Artefakt)                             | `@markdownai v1.0` (consumer=ai)  | mdai-execution-Subagents                                                |

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
Risiko: mai-CLI blockiert `@query`). Plan wird **nie** für Human gerendert (exklusiv Subagent-Input, phase-isoliert via
`read_file(phase=…, format=ai)`).

## 8. Slash-Trigger und Description-Match

**Slash-Command:** `/mdai-brainstorm` — Skill-Name = Command-Name (Konvention von Claude Code).

**Description (Frontmatter):**

> Use when starting creative work that will produce both a versioned design spec under docs/mdai/specs/ and a
> multi-phase .mdai.md plan under docs/mdai/plans/ for parallel subagent dispatch.

**Trigger-Disziplin:** Primärer Trigger ist der explizite Slash-Command `/mdai-brainstorm`. Description-Match
ist sekundär — die Pfad-Trigger (`docs/mdai/specs/`, `docs/mdai/plans/`, `.mdai.md`) grenzen den Skill von
`superpowers:brainstorming` ab (das nach `docs/superpowers/specs/` schreibt). Keine Implementation-Details
in der Description (Begründung: writing-skills CSO §1 — Description-Workflow-Summaries erzeugen einen
Shortcut, dem Claude folgt, statt body.mdai.md zu lesen).

## 9. Smoke-Tests

**Bewusst kein RED-Baseline und kein GREEN-Re-Run:** Skill A wird ausschließlich per explizitem
`/mdai-brainstorm`-Slash-Command getriggert (siehe §8 Trigger-Disziplin). Damit ist Trigger-Discovery-Drift kein
relevantes Risiko, und der `writing-skills` "Iron Law: RED-GREEN-REFACTOR" wird durch reasoned-counter-Seeding der
Red-Flags + Rationalization-Table (§10) ersetzt. Falls später Pressure-Induced-Drift *innerhalb* der Skill-Session
beobachtet wird (z.B. Session-Reviewer flaggt eine Discipline-Lücke), kann eine RED-Baseline retroaktiv hinzugefügt
werden.

### 9.1 Trigger-Test (GREEN — Pointer-Compliance)

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

- **3/3 Pass** → grün, weiter zu §9.2.
- **2/3 Pass** → **manuelle Diagnose** (mögliche Ursachen: MCP-Disconnect,
  Cache-Effekt, Setup-Rauschen). Nicht auto-Fallback. Erst nach Root-Cause-
  Analyse entscheiden zwischen Re-Test (bei klarer Glitch-Ursache),
  Skill-Iteration (bei Pattern-Verbesserungsbedarf) oder File-System-Split
  (5-File-Layout: `phases/<id>.mdai.md`, falls Root-Cause auf strukturelle
  Skill-Loader-Schwäche zeigt — Migration als separate Spec).
- **0/3 oder 1/3 Pass** → File-System-Split aktivieren, A2 re-do als separate Spec.

### 9.2 Discipline-Fidelity-Test (GREEN)

dialog-phase führt zu interaktivem Dialog mit Klärungsfragen (one-at-a-time), 2-3
Approach-Vorschlägen, Design-Sektion-by-Section. Verifiziere: kein
`Skill(superpowers:brainstorming)`-Invoke im Tool-Log, Claude folgt der hand-ported
Checklist ohne Upstream-Skill-Load. Red-Flags + Rationalization-Table aus §10
zur Skill-Laufzeit gefüllt und respektiert.

### 9.3 Output-Test (GREEN)

write-outputs erzeugt im Default-Pfad genau **zwei** Files:

- `docs/mdai/specs/<date>-<slug>-design.mdai.md` (Spec-Source, `consumer="ai"`)
- `docs/mdai/plans/<date>-<slug>.mdai.md` (Plan, consumer=ai)

Verifiziere via `git status` + `@call list_phases(plan=...)` (Plan listet erwartete
Phase-IDs). Bei Opt-in `render_spec(target="file")` kommt zusätzlich
`docs/mdai/specs/rendered/<date>-<slug>.rendered.md` dazu — separat verifizieren. Kein File unter
`docs/superpowers/specs/` (Default-Pfad existiert in unserem Flow nicht mehr).

### 9.4 Phase-Budget-Test

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

### 9.5 Library-Import-Smoke-Test

Spec-Reviewer Anti-Pattern-Check 8 (`mdai/skills/mdai-brainstorm/spec-reviewer.md`): Static-render-test pre-Install.

```bash
cd markdownai && npx mai render ../mdai/skills/mdai-brainstorm/body.mdai.md
```

Pass-Kriterien: `exit 0`; alle `@call`s aus Skill-A-Pack (`write_spec`, `render_spec`, `write_mdai_plan`,
`spec_reviewer_prompt`, `plan_frontmatter`, `plan_phase`, `plan_step`) und Core-Wrappers (`ctx_read`, `ctx_shell`,
`list_phases`, `remember_plan`, `list_gotchas`) sind aufgelöst; keine `unknown directive` Errors; `mode: include` Text
(hard-rules, tool-quick-ref) erscheint im Output; `mode: import-only` Source-Text erscheint NICHT. `@query`-Direktiven
liefern leere Strings (mai-CLI blockiert Live-Execution — siehe §13). Verifiziert nur Plumbing, nicht Live-MCP.

## 10. Bulletproofing — Red-Flags + Rationalization-Table

Discipline-enforcing Skills brauchen explizite Anti-Rationalisierungs-Strukturen
(writing-skills/SKILL.md §"Bulletproofing"). Zwei Artefakte werden in
`body.mdai.md` dialog-phase verankert. Daten kommen aus **reasoned counters**:
9 Discipline-Punkte (§10.4) + Upstream-Pattern-Observations
(`superpowers:brainstorming/SKILL.md` §Anti-Pattern + §Red Flags-Pendant in
`using-superpowers/SKILL.md`). Kein RED-Baseline (siehe §9 Intro) — explizit-invoke
macht Trigger-Discovery-Drift irrelevant; Pressure-Patterns leiten wir aus dem
9-Discipline-Cross-Check ab.

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

### 10.2 Rationalization-Table (Position: nach Process-Details, vor Visual-Companion-Dispatch)

**Format** in `body.mdai.md` dialog-phase:

```markdown
| Excuse | Reality |
|---|---|
| [reasoned-counter pro Discipline-Punkt §10.4] | [Konter-Argument, 1 Satz] |
```

**Source:** reasoned-counter Zeilen pro Discipline-Punkt (§10.4) + Upstream-Pattern-Observations.
Die 9 mdai-Discipline-Punkte sind Kategorie-Anchor — Tabelle deckt alle 9 ab. Jede Zeile markiert
als `[reasoned-counter]` (kein verbatim-Baseline). Wenn später Drift beobachtet wird (Session-Review),
kann eine Zeile durch verbatim-Quote aus dem Drift-Vorfall ersetzt werden. Erwartete Anzahl: 8–12 Zeilen.

### 10.3 A2-Workflow (Sub-Schritte für §15 A2)

A2 schreibt `body.mdai.md` inkl. dialog-phase mit gefüllten Red-Flags +
Rationalization-Table aus reasoned counters.

**A2-Sub-Schritte:**

1. Library-Imports am File-Head schreiben (`@import mdai/skills/mdai-brainstorm/{write-spec,write-mdai-plan,spec-reviewer}.md`).
2. Pro Discipline-Punkt (§10.4) einen 1-Satz-Self-Check formulieren → Red-Flags-Liste füllen
   (5–8 Einträge, mind. einer pro Pressure-Kategorie cold/time/authority).
3. Pro Discipline-Punkt eine Excuse/Reality-Zeile formulieren → Rationalization-Table füllen
   (8–12 Zeilen, alle 9 Discipline-Punkte abgedeckt, jede Zeile `[reasoned-counter]` markiert).
4. Upstream-`using-superpowers/SKILL.md` §"Red Flags" und `superpowers:brainstorming/SKILL.md`
   §Anti-Pattern als Quervergleich — fehlt eine gängige Rationalisierung in unserer Tabelle?
5. `wc -w` auf jede Phase (siehe §11) — falls Budget gerissen, Sub-Phase-Split.

### 10.4 Die 9 mdai-Discipline-Punkte (Cross-Check-Anchor)

Die `mdai-brainstorm`-Skill setzt diese 9 Disziplin-Punkte durch. Die
Rationalization-Table muss alle 9 abdecken:

1. HARD-GATE: kein Code/Plan-Write vor User-Design-Approval.
2. Spec-Pfad: `docs/mdai/specs/` — **nicht** `docs/superpowers/specs/`.
3. Plan-Pfad: `docs/mdai/plans/` — **nicht** `docs/superpowers/plans/`.
4. File-Endung: beide `.mdai.md` — **nicht** `.md`.
5. One-question-at-a-time im Brainstorm-Dialog.
6. Approach-Vergleich (2–3 Alternativen) **vor** Design-Präsentation.
7. Per-Section-Approval beim Design-Walkthrough.
8. Spec-Self-Review **vor** User-Review-Gate.
9. `body.mdai.md` nur phase-by-phase via MCP lesen, **nie** full.

## 11. Token-Budget pro Phase

Phase-Isolation funktioniert nur, wenn jede einzelne Phase klein ist. Hartes Cap pro Phase, Verifikation via `wc -w`
auf phase-isolierten Output. Macros sind extern (Library) — Phase-Bodies enthalten nur `@call`/`@query`/`@include`-
Direktiven plus Anweisungstext, nicht die Macro-Definitionen.

### 11.1 Budget-Tabelle

| Phase | Budget (Worte) | Begründung |
|---|---|---|
| pre-context | ≤250 | `@call mdai_bootstrap()` + Includes + `@query`-Liste + Constraints. Macros extern. |
| dialog | ≤600 | Hand-ported Checklist (~80) + Process (~120) + Principles (~50) + Red-Flags (~60) + Rationalization-Table (~120) + Visual-Companion (~40) + Spec-Reviewer-Dispatch (~40) + Anti-Pattern-Intro (~50) + HARD-GATE (~40) = ~600. Knapp über writing-skills Soft-Norm — toleriert. |
| write-outputs | ≤100 | Nur `@call`-Aufrufe (write_spec, render_spec, write_mdai_plan, list_phases) + Output-Liste. |
| handoff | ≤100 | Phase-Inventory + `@call remember_plan` + Next-Step-Anweisung. |
| **Total body.mdai.md** | **≤1050 Worte** | Sum + ~50 für `@import`-Header. |

### 11.2 Verifikations-Befehl

```bash
for phase in pre-context dialog write-outputs handoff; do
  count=$(mcp__markdownai__read_file \
            path=mdai/skills/mdai-brainstorm/body.mdai.md \
            phase=$phase format=ai | wc -w)
  echo "$phase: $count words"
done
```

Identisch zu §9.4 Phase-Budget-Test.

### 11.3 Eskalation bei Budget-Überschreitung

**Reihenfolge:**

1. **Sub-Phase-Split:** Falls `dialog` >600 Worte, splitten in `dialog-rules`
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
2. **Automatische `.md` → `.mdai.md`-Migration** alter Pläne — erstmal manuelles Re-Writing.
3. **Drift-Detection im aktuellen Scope** — separater Skill `mdai-drift-check` als Backlog-Item, kein Bestandteil dieser
   Spec.
4. **Plan-State-Persistenz** — `mdai-memory` (aus altem Design) bleibt zuständig, `mdai-brainstorm` ruft es nur am Ende
   einmal via `@call remember_plan` auf.
5. **`mdai-execution` und `mdai-memory` Migration** auf `mdai/skills/`-Layout — separate Spec.
6. **Library-Macros modifizieren** — `mdai/skills/mdai-brainstorm/*.md` ist Library-Code. Änderungen erfolgen via
   separater Library-Spec (`docs/mdai/specs/2026-05-24-mdai-macro-library-design.mdai.md`) + Library-Version-Bump.

## 13. Risiken

| Risiko                                                                                                             | Schweregrad | Mitigation                                                                                                                                                                                                |
| Claude folgt der hand-ported Disziplin nicht so streng wie ein echter Skill-Invoke (kein Skill-Engine-Enforcement) | **Mittel**  | dialog-phase formuliert Anweisungen als `@constraint id="..." severity="high"` plus HARD-GATE. Discipline-Fidelity-Test §9.2 prüft Verhalten.                                                             |
| Upstream-`brainstorming/SKILL.md` ändert sich nach Bump → hand-ported Slices veralten                              | **Mittel**  | `mdai-drift-check`-Backlog: Hash-Vergleich + Diff-Report. Bis dahin: bei jedem `superpowers`-Versions-Bump manuelle Review der Source-Zeilen (16-20, 22-32, 70-104, 140-145).                              |
| mdai-Library bumped → Macro-Signaturen ändern → Skill A bricht                                                     | **Mittel**  | `requires.mdai-library: ">=0.1.0"` pinnt Minimum-Version im Spec-Frontmatter. Library `changelog.md` dokumentiert Breaking Changes. Re-Smoke-Test (§9.5) bei jedem Library-Bump im jeweiligen Bump-Plan.   |
| `mai` CLI blockiert `@query`-Direktiven zur Render-Zeit (engine-include.ts security policy)                        | Niedrig     | Bekannt seit Library v0.1.0 bootstrap-findings. Smoke-Render-Test §9.5 verifiziert nur Plumbing (Imports/Includes/Defines), nicht Live-MCP-Behavior. Live-Verifikation nur aus Claude-Code-Session möglich.|
| `mcp__markdownai__read_file(phase=…)` Server disconnected mitten in Session                                        | Niedrig     | Reconnect via `/mcp`, headless-Fallback dokumentiert.                                                                                                                                                     |
| `body.mdai.md` wird trotz pointer-Anweisung full gelesen                                                           | Mittel      | SKILL.md formuliert die Anweisung als Hard-Constraint („MUST"), nicht „SHOULD". Smoke-Test §9.1 prüft. Bei Fail: 5-File-Layout-Migration als separate Spec.                                                |
| `mcp__lean-ctx__ctx_graph` ist nicht gebaut (kein Index) → pre-context liefert leeren Graph                        | Niedrig     | Pre-context-Phase führt `@if`-Check: bei leerem Graph fällt zurück auf `ctx_tree` + `ctx_overview`. `ctx_graph action="build"` einmalig in P0.                                                             |
| `mdai_bootstrap()` läuft per-render (kein Cache in Library v0.1.0) → Overhead bei jedem pre-context-Load           | Niedrig     | Akzeptiert für v1. Cache-Backlog (`ctx_session`-basiert) in Library v0.2 geplant — siehe §14.                                                                                                             |
| User-Global-Skills haben niedrigere Priorität als Plugin-Skills bei Description-Match                              | Niedrig     | Smoke-Test §9 prüft. Falls Konflikt: explizites `/mdai-brainstorm`-Trigger nutzen.                                                                                                                        |

## 14. Backlog (separate Specs)

Explizit-deferred Parking-List. Jeder Eintrag bekommt bei Bedarf eine eigene Spec via
`/mdai-brainstorm`. Backlog ≠ Open Items — diese hier sind bewusst aufgeschoben.

1. **`mdai-drift-check`** — Skill zum Upstream-Hash-Vergleich + Diff-Report. Hash-Store in
   `docs/mdai/upstream-hashes.json`. Manueller Trigger oder periodisch via `loop`/`schedule`.
   Trigger für separate Spec: spätestens beim ersten `superpowers`-Versions-Bump nach Release.
2. **`mdai-execution`-Migration** auf `mdai/skills/mdai-execution/`-Layout. Inhaltlich unverändert, nur Pfad-Umzug.
   Trigger: vor nächster substantieller mdai-execution-Änderung.
3. **`mdai-memory`-Migration** dito.
4. **Upstream-PR an markdownai** für `respondTool()`-Fix (separat, blockt nichts). Trigger: jederzeit.
5. **Plugin-Packaging** — Bündelung aller drei mdai-Skills als ein Claude-Code-Plugin (eigene `package.json`,
   `hooks.json` Stub). Trigger: nach mind. einem Monat stabiler Nutzung der drei Skills.
6. **Spec-Human-Render-Wrapper-Template** — separate `.mdai.md`-Datei
   unter `mdai/skills/mdai-brainstorm/templates/` mit Layout-Macros (Cover-Page mit Branch/Date, automatische
   TOC, Constraint-Glossar). Bei aktiviertem Render zielt `render_spec` auf den Wrapper. Aktuell nicht nötig —
   `render_spec` rendert direkt das Source-Spec. Trigger für separate Spec: sobald ein Reviewer
   Cover/TOC/Glossar reproduzierbar verlangt.
7. **`mdai_bootstrap`-Cache** (Library v0.2-Backlog) — Session-scoped Cache via
   `ctx_session action="finding"/"status"` reduziert per-render-Overhead. Marker-Format aus Library
   v0.1.0 changelog: `[mdai-bootstrap-cache] tooling=detected lang=<LANG> jetbrains=<bool> serena=<bool>`.
   Trigger: messbarer Overhead bei Skill-A-Live-Nutzung.
8. **Globaler Install + `install.sh`** — `mdai/skills/mdai-brainstorm/scripts/install.sh` schreiben (kopiert `SKILL.md`
   + `body.mdai.md` nach `~/.claude/skills/mdai-brainstorm/`). Macht Skill projekt-übergreifend verfügbar. **Während
   Entwicklung bewusst deferred** — vermeidet Cross-Projekt-Trigger und Schaden bei Skill-Iteration. Trigger: nach
   mind. einer stabilen `/mdai-brainstorm`-Session-Reihe im lean-ctx-Repo (z.B. 3 Pläne ohne Drift) und vor
   Plugin-Packaging (§14.5).

## 15. Implementierungsschritte (high-level, der echte Plan kommt via writing-plans-Skill)

| Phase | Aufgabe                                                                                                                                                                                                                                            |
| P0    | `mcp__lean-ctx__ctx_graph action="build"` einmalig laufen lassen, damit pre-context-Phase einen Index hat. **Plus:** `.superpowers/` in `.gitignore` aufnehmen (persistente Visual-Companion-Mockups via `--project-dir "$PWD"`). **Hinweis:** `docs/mdai/macros/`-Mirror entfällt — durch Library v0.1.0 (`mdai/core/*`, `mdai/skills/mdai-brainstorm/*`) abgedeckt. Kein RED-Baseline (siehe §9 Intro). |
| A1    | `mdai/skills/mdai-brainstorm/SKILL.md` schreiben (~15 Z, pointer). Description aus §4 (Trigger-only, keine Workflow-Summary). |
| A2    | `mdai/skills/mdai-brainstorm/body.mdai.md` schreiben (~120 Z, alle Phasen + Library-Imports). **Sub-Schritte (§10.3):** (a) Library-Imports am Head; (b) Red-Flags-Liste aus reasoned counters füllen (5–8 sentence-form, §10.1); (c) Rationalization-Table aus reasoned counters füllen (8–12 Zeilen, alle 9 Discipline-Punkte §10.4 abgedeckt, §10.2); (d) Upstream-Cross-Check (`using-superpowers/SKILL.md` §"Red Flags" + `superpowers:brainstorming` §Anti-Pattern); (e) `wc -w` pro Phase gegen §11.1-Budget — bei Überschreitung Sub-Phase-Split (§11.3). |
| A3    | **Project-local Test-Setup:** `mkdir -p .claude/skills && ln -sf ../../mdai/skills/mdai-brainstorm .claude/skills/mdai-brainstorm`. Verifiziere: `ls -la .claude/skills/mdai-brainstorm/` zeigt Symlink. **Hinweis:** kein `install.sh`, kein globaler Install während Entwicklung — siehe §3 und §14.8. |
| A4    | **Smoke-Tests gegen project-local Symlink (§9.1–§9.5).** Inkl. Library-Import-Smoke-Test (§9.5). Skill ist nur in diesem Repo aktiv, beeinflusst andere Projekte nicht. Iteriere am Source unter `mdai/skills/mdai-brainstorm/`, bis alle Tests grün sind. **Bei §9.1 Fail (0/3 oder 1/3 Pass):** A2 re-do als 5-File-Layout-Migration (separate Spec). |
| A5    | `mdai-plans` deinstallieren (falls global vorhanden): `rm -rf ~/.claude/skills/mdai-plans/`, verifiziere via `/mdai-plans` triggert nicht mehr. Falls nur lokal: `rm -rf .claude/skills/mdai-plans`. |
| A6    | Self-Bootstrap: Plan-Datei selbst (`docs/mdai/plans/<date>-mdai-brainstorm-impl.mdai.md`) wird per `/mdai-brainstorm` produziert — `@call list_phases` + Library-Import-Smoke-Test (§9.5) als finale Validation. |

## 16. Annahmen, die in Smoke-Tests zu verifizieren sind

1. `mcp__markdownai__read_file(consumer="human", format="standard")` rendert eine Spec für Human-Konsum korrekt
   (verifiziert mit mai-CLI v0.0.24, MCP wrapped denselben Renderer). Smoke-Test §9 prüft den MCP-Output direkt.
2. `mcp__markdownai__read_file(phase=…, format=ai)` returniert eine self-contained Phase inkl. relevanter
   `@define`-Macros (geerbt von der Datei-Header via `@import`). Falls nicht: Macros müssen in jeder Phase neu deklariert
   werden — wäre Library-Bug, blockiert Skill A.
3. Claude Codes Skill-Loader respektiert die pointer-Anweisung „lies body.mdai.md nicht full". Verifikation §9.1
   Pointer-Compliance-Test (3 Runs, `jq`-Grep auf Session-Transcript). Bei 0/3 oder 1/3 Pass → 5-File-Layout-Migration
   als separate Spec.
4. Claude folgt der hand-ported Disziplin in `dialog`-phase ohne `Skill(superpowers:brainstorming)`-Invoke (
   Discipline-Fidelity-Test §9.2). Falls Claude die Disziplin lockerer interpretiert als ein echter Skill-Invoke:
   `@constraint id="hard-gate" severity="high"`-Block und HARD-GATE-Anweisung schärfen, oder als Fallback Hybrid (
   Skill-Invoke + hand-ported Pre-Briefing) zurück erwägen.
5. `mcp__lean-ctx__ctx_graph` ist im Projekt bereits gebaut (oder wird in P0 einmal indiziert). Falls Index leer:
   pre-context-Phase fällt auf `ctx_tree` + `ctx_overview` zurück.
6. `@import mdai/skills/mdai-brainstorm/*.md` lädt die Library-`@define`s korrekt — Static-render-test §9.5 verifiziert
   gegen Library v0.1.0. Falls Imports nicht aufgelöst werden (z.B. Pfad relativ-Problem): Library-Bug, blockiert
   Skill A.

Diese Annahmen sind die einzigen blockierenden Unbekannten. Alle anderen Risiken sind mitigierbar oder akzeptiert.
