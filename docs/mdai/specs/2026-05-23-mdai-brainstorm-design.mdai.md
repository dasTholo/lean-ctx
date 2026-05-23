---
id: mdai-brainstorm
status: design
created: 2026-05-23
spec_version: 1
supersedes: docs/mdai/design-skill-integration.md §4 (mdai-plans)
---

> **Render-broken nach mdai-library v0.1.0 release (2026-05-24).** Diese Spec verweist noch auf `docs/mdai/macros/*.md`. Patches in library-spec §10 müssen vor Skill-A-A1 angewendet werden. Bis dahin schlagen Renders dieser Spec fehl. Tracking: `docs/mdai/plans/2026-05-24-mdai-macro-library-impl.mdai.md` Task 21.

@markdownai v1.0 consumer="ai"

# mdai-brainstorm — Markdownai-aware Brainstorming-Skill (Design)
wir brauce
Status: Brainstorming abgeschlossen, wartet auf Spec-Review. Bezug: ersetzt den `mdai-plans`-Pfad aus
`docs/mdai/design-skill-integration.md` §4. Die restlichen Skills (`mdai-execution`, `mdai-memory`) aus jenem Design
bleiben unverändert.

## 1. Zielsetzung

Eine einzelne Skill `mdai-brainstorm` bündelt drei Aufgaben, die im alten Design auf zwei Skills verteilt waren:

1. **Brainstorming-Dialog** (Anforderungen klären, Approach-Vergleich, Design-Sektionen) — gewrappt von
   `superpowers:brainstorming`.
2. **Spec-Write** — produziert ein versioniertes Design-Dokument als markdownai-Live-Doc mit **`consumer="ai"`**
   (Default). Nach dem Write bietet der Skill optional `mai render`-Output an: inline im Chat oder als zusätzliche
   `.md`-Datei. Render ist **Opt-in**, kein dualer Default-Commit.
3. **Plan-Write** — produziert direkt einen `.mdai.md`-Plan mit `@phase`-Markern für Subagent-Dispatch, ohne Umweg über
   einen separaten `mdai-plans`-Skill.

Trigger: `/mdai-brainstorm` oder Description-Match.

**Erfolgskriterien:**

1. Der Skill ist als markdownai-Live-Doc strukturiert (`body.mdai.md` mit `@phase` + `@define`-Macros). Pre-Phase
   liefert lean-ctx-Kontext pre-resolved (inkl. `ctx_graph`-Live-Map), ohne Tool-Roundtrips im Dialog.
2. Upstream-`superpowers:brainstorming` wird **hand-ported** (Fork minimal): die Dialog-Disziplin (Checklist,
   Process-Regeln, Key Principles) wird **einmal** in `body.mdai.md` kopiert — kein Skill-Invoke, kein `@include` zur
   Renderzeit. Drift wird per `mdai-drift-check` (Backlog) als manuelles Audit-Item gehandhabt.
3. Spec wird als **`*.mdai.md` mit `consumer="ai"`** committet (Default). Render in `.md` ist **Opt-in**: nach
   Spec-Write bietet der Skill drei Optionen — (a) kein Render, nur `.mdai.md`; (b) Render inline im Chat anzeigen
   (kein File); (c) Render als zusätzliche `.md` schreiben (committet, aber Pflicht-Acknowledgement durch User).
4. Plan wird als `.mdai.md` mit `@phase`-Markern geschrieben, kompatibel zu `mdai-execution` aus dem alten Design.
5. `superpowers:writing-plans` wird **re-templated** (Disziplin in `@define`-Macros überführt), nicht gewrappt — saubere
   Output-Kontrolle.
6. Schritte mit Datei-Operationen nutzen explizit `mcp__lean-ctx__*`-Tools (über `@query lean-ctx ...` im Skill-Body).
   Projekt-Dependencies kommen aus `mcp__lean-ctx__ctx_graph` (Mermaid-Diagramm), nicht aus einer hand-gepflegten
   `connections.md`.

## 2. Empirische Grundlage — Token-Math

Vier Strategien wurden gegeneinander geschätzt. Werte sind Größenordnungen, nicht Messwerte:

| Strategie                             | Beim Skill-Aufruf                             | Bei Dialog-Eintritt                     | Bei Plan-Write                | Summe aktiv               | Drift-Risiko                                     |
|---------------------------------------|-----------------------------------------------|-----------------------------------------|-------------------------------|---------------------------|--------------------------------------------------|
| Skill-Chain / Sandwich (Skill-Invoke) | pointer + voller body (~800)                  | + Upstream-brainstorming (~1500)        | + writing-plans (~800)        | ~3100                     | mittel (Skill-Engine erzwingt Disziplin)         |
| `@include + augment` (eager merge)    | pointer + body+upstream merged (~1900), eager | —                                       | —                             | ~1900, keine Lazy-Loading | hoch (Line-Shifts)                               |
| Selective `@include lines=N-M` (lazy) | pointer + pre-phase (~350)                    | + selective slice (~400 zur Renderzeit) | + post-phase + Macros (~1000) | ~700–1100                 | **hoch** (stumme Line-Shift-Brüche)              |
| **Hand-port (Fork minimal, gewählt)** | **pointer + pre-phase + dialog (~600)**       | — (Disziplin schon im body)             | **+ Macros inline (~200)**    | **~800 gesamt**           | mittel (explizit per mdai-drift-check auditiert) |

Hand-port gewinnt, weil:

1. **Kein Upstream-Skill-Load.** Disziplin wurde einmal in `body.mdai.md` kopiert — Claude liest sie als Teil unseres
   eigenen Skill-Bodies.
2. **Keine Render-Zeit-Drift.** `@include` vom Upstream entfällt; Upstream-Edits können uns nicht mehr stumm brechen.
3. **`@phase`-Isolation greift weiter** — pre-context, dialog, write-outputs, handoff bleiben disjunkt.
   `mcp__markdownai__read_file(phase=…)` lädt nur ~150–250 Tok pro Phase.
4. **Drift bleibt explizit:** Bei Upstream-Bumps gibt `mdai-drift-check` (Backlog) einen Hash-Diff-Report aus. Manuelles
   Re-Port ist günstiger und sichtbarer als stille Render-Zeit-Brüche.

**Caveat:** Phase-Isolation spart Tokens nur, wenn der Agent **explizit** `mcp__markdownai__read_file(phase=…)` aufruft
statt die ganze `body.mdai.md` zu lesen. Das wird in der pointer-`SKILL.md` als verbindliche Anweisung verankert.

## 3. Architektur-Überblick

```
┌───────────────────────────────────────────────────────────────────────┐
│  /mdai-brainstorm                                                     │
│                                                                       │
│  SKILL.md  (~15 Z, pointer)                                           │
│    └─ Anweisung: "Lade body.mdai.md immer phase-für-phase via         │
│       mcp__markdownai__read_file(phase=…, format=ai). Niemals full."  │
│                                                                       │
│  body.mdai.md (~150 Z, live doc)                                      │
│    ├─ @markdownai v1.0                                                │
│    ├─ @include docs/mdai/macros/hard-rules.md                         │
│    ├─ @include docs/mdai/macros/tool-quick-ref.md                     │
│    ├─ @import  docs/mdai/macros/step-reformat-commit.md               │
│    │                                                                  │
│    ├─ @define planFrontmatter(id, spec)        ← writing-plans Macros │
│    ├─ @define planPhase(id, title, files, steps)                      │
│    ├─ @define planStep(check, body)                                   │
│    ├─ @define writeSpec(slug, body)            ← Spec-Override Macros │
│    ├─ @define writeMdaiPlan(slug, phases)                             │
│    ├─ @define specReviewerPrompt(spec_path)   ← hand-port reviewer    │
│    │                                                                  │
│    ├─ @phase pre-context                                              │
│    │    @query lean-ctx ctx_overview --task "$task"                   │
│    │    @query lean-ctx ctx_tree . --depth=2                          │
│    │    @query mcp lean-ctx ctx_graph action=context                  │
│    │    @query mcp lean-ctx ctx_graph action=diagram kind=deps depth=2│
│    │    @query git log --oneline -10                                  │
│    │    @query lean-ctx ctx_knowledge --recall "mdai-gotcha"          │
│    │                                                                  │
│    ├─ @phase dialog                                                   │
│    │    Hand-ported Disziplin (aus upstream brainstorming, Z. 22-32   │
│    │    Checklist + Z. 70-104 Process + Z. 140-145 Principles).       │
│    │    Claude folgt der Disziplin DIREKT — kein Skill-Invoke.        │
│    │                                                                  │
│    ├─ @phase write-outputs                                            │
│    │    @call writeSpec(slug=<topic>, body=<design-content>)          │
│    │    @call writeMdaiPlan(slug=<topic>, phases=<phase-list>)        │
│    │                                                                  │
│    └─ @phase handoff                                                  │
│         @query mcp markdownai list_phases path=<plan-path>            │
│         Anweisung: "Next step: /mdai-execution <plan-path>"           │
└───────────────────────────────────────────────────────────────────────┘
```

**Verhältnis zu Bestandsskills:**

| Skill                        | Behandlung                                                                                                                                  |
|------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------|
| `superpowers:brainstorming`  | **Hand-ported** in dialog-phase (Disziplin-Slices einmal in body.mdai.md kopiert; kein Skill-Invoke, kein `@include` zur Renderzeit)        |
| `superpowers:writing-plans`  | **Re-templated** als `@define`-Macros in write-outputs-phase                                                                                |
| `superpowers:writing-skills` | Wird einmal invoked, um `mdai-brainstorm` selbst zu schreiben (Bootstrap)                                                                   |
| `mdai-plans` (alt)           | **Wird abgelöst** und im Plan deinstalliert (entfernt aus `~/.claude/skills/`)                                                              |
| `mdai-execution`             | Unverändert, bleibt als nächster Step nach `mdai-brainstorm`                                                                                |
| `mdai-memory`                | Unverändert, kann von `mdai-execution` weiterhin invoked werden                                                                             |
| `mdai-drift-check` (Backlog) | Wird nach Implementation gebraucht, um Drift zwischen Upstream-`brainstorming/SKILL.md` und unserer hand-ported dialog-phase zu detektieren |

## 4. Datei-Layout

**Source (in git, Projekt-Repo):**

```
skills/mdai/
  mdai-brainstorm/
    SKILL.md
    body.mdai.md
    scripts/install.sh
```

Die Namespace-Konvention `skills/mdai/<skill-name>/` macht Platz für zukünftige Migration von `mdai-execution` und
`mdai-memory` in dieselbe Struktur (Backlog).

**Test-Ziel (project-local, in diesem Repo via Symlink):**

```
.claude/skills/mdai-brainstorm  →  ../../skills/mdai/mdai-brainstorm  (Symlink)
```

Claude Code lädt Skills aus `.claude/skills/` automatisch projekt-scoped. Der Symlink macht den Source-Ordner unter
`skills/mdai/` zum Discovery-Target — Edits am Source sind sofort live, kein zweiter Build-Step.

**Install-Ziel (user-global, NICHT in git — DEFERRED bis Smoke-Tests grün):**

```
~/.claude/skills/mdai-brainstorm/
  SKILL.md
  body.mdai.md
```

Claude Codes Skill-Loader erwartet einen flachen Tree unter `~/.claude/skills/`. Der Namespace `skills/mdai/...`
existiert nur im Source. **Globaler Install erfolgt erst nach grünen Smoke-Tests** (siehe §16 A6 → A7).

**install.sh (bleibt im Repo, wird aber erst nach Smoke-Test gerufen):**

```bash
#!/usr/bin/env bash
set -euo pipefail
DST="${HOME}/.claude/skills/mdai-brainstorm"
SRC="$(dirname "$(readlink -f "$0")")/.."
mkdir -p "$DST"
cp "$SRC/SKILL.md" "$DST/SKILL.md"
cp "$SRC/body.mdai.md" "$DST/body.mdai.md"
echo "Installed mdai-brainstorm → $DST"
```

## 5. SKILL.md (pointer)

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

## 6. body.mdai.md (live workflow)

Detaillierter Inhalt pro Phase. Macros werden vor den Phasen definiert, damit sie phasen-übergreifend verfügbar sind.

### 6.1 Macros

```markdown
@define planFrontmatter(id, spec)
---
id: {{ id }}
status: planned
mdd_version: "1.0"
created: {{ @date format="YYYY-MM-DD" }}
spec: {{ spec }}
---

@markdownai v1.0
@end

@define planPhase(id, title, files, steps)
@phase {{ id }}

## {{ title }}. Ic

**Files:**
{{ files }}

{{ steps }}

@end
@end

@define planStep(check, body)

- [{{ check }}] {{ body }}
  @end

@define writeSpec(slug, body)
@query lean-ctx ctx_shell "mkdir -p docs/mdai/specs"
@query lean-ctx ctx_shell "cat > docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md << 'EOF'
{{ body }}
EOF"
@end

@define renderSpec(slug, target)

# target ∈ { "none" (default), "chat", "file" }

# Caller passes the user's selection after offering the three options.

# Uses mcp__markdownai__read_file (MCP) — no npx spawn, server is already running for phase-loading.

@if target == "chat"
Rendered for chat (human-readable, inline, no file):
{{ @query mcp markdownai read_file path="docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"
consumer="human" format="standard" }}
@elseif target == "file"
Rendered to file (additional .md alongside .mdai.md, user must explicitly acknowledge):
Step 1: render via MCP → capture output
{{ @query mcp markdownai read_file path="docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.mdai.md"
consumer="human" format="standard" }}
Step 2: write captured output to disk via ctx_shell heredoc
@query lean-ctx ctx_shell "cat > docs/mdai/specs/{{ @date format='YYYY-MM-DD' }}-{{ slug }}-design.md << 'EOF'
<captured-output>
EOF"
@else
No render. Only the .mdai.md source is present in the working tree.
@endif
@end

@define writeMdaiPlan(slug, phases)
@query lean-ctx ctx_shell "mkdir -p docs/mdai/plans"
@query lean-ctx ctx_shell "cat > docs/mdai/plans/{{ @date format='YYYY-MM-DD' }}-{{ slug }}.mdai.md << 'EOF'
{{ phases }}
EOF"
@query mcp markdownai list_phases path="docs/mdai/plans/{{ @date format='YYYY-MM-DD' }}-{{ slug }}.mdai.md"
@end

@define specReviewerPrompt(spec_path)
You are a spec doc reviewer. Verify this spec is complete and ready for planning.

**Spec to review:** {{ spec_path }}

## What to Check

| Category | What to Look For |
| --- | --- |
| Completeness | TODOs, placeholders, "TBD", incomplete sections |
| Consistency | Internal contradictions, conflicting requirements |
| Clarity | Requirements ambiguous enough to cause someone to build the wrong thing |
| Scope | Focused enough for a single plan — not covering multiple independent subsystems |
| YAGNI | Unrequested features, over-engineering |

## Calibration

Only flag issues that would cause real problems during impl planning. A missing section,
a contradiction, or a requirement so ambiguous it could be interpreted two different ways —
those are issues. Minor wording improvements and stylistic preferences are not.

Approve unless there are serious gaps that would lead to a flawed plan.

## Output Format

**Status:** Approved | Issues Found
**Issues (if any):** [section / specific issue / why it matters for planning]
**Recommendations (advisory):** [suggestions for improvement]
@end
```

**Hand-port note:** `specReviewerPrompt` is hand-ported from upstream `spec-document-reviewer-prompt.md` (49 Z, full content folded into the macro). Path override happens via the `spec_path` parameter — caller passes `docs/mdai/specs/<date>-<slug>-design.md`, no hardcoded `docs/superpowers/specs/` reference. Drift handled by `mdai-drift-check` (Backlog).

### 6.2 Phase: pre-context

Pre-resolved Projekt-Kontext beim Skill-Load. Keine Tool-Roundtrips im Dialog. Alle `@query` zielen explizit auf
lean-ctx, damit die Output-Kompression greift. `ctx_graph` liefert einen live-Dependency-Graphen statt einer
hand-gepflegten `connections.md`.

```markdown
@phase pre-context

## Pre-resolved project context

**Branch:** {{ @query lean-ctx ctx_shell "git branch --show-current" }}
**Recent commits:**
{{ @query lean-ctx ctx_shell "git log --oneline -10" }}

**Project map (task-scoped):**
{{ @query mcp lean-ctx ctx_overview --task "$user_task" }}

**Dependency graph (Mermaid, depth=2):**
{{ @query mcp lean-ctx ctx_graph action="diagram" kind="deps" depth=2 }}

**Task-relevant subgraph:**
{{ @query mcp lean-ctx ctx_graph action="context" }}

**Tree (depth=2):**
{{ @query mcp lean-ctx ctx_tree . --depth=2 }}

**Known gotchas:**
{{ @query mcp lean-ctx ctx_knowledge --recall "mdai-gotcha" }}

@include docs/mdai/macros/hard-rules.md
@include docs/mdai/macros/tool-quick-ref.md

Constraints for the dialog phase:

- Spec target: docs/mdai/specs/ (NOT docs/superpowers/specs/)
- Plan target: docs/mdai/plans/ (NOT docs/superpowers/plans/)
- Hard rules: see @include above
  @end
```

### 6.3 Phase: dialog (hand-ported aus superpowers:brainstorming)

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

<!-- §11.1 Red-Flags-Liste — filled in A2 from docs/mdai/red-baseline/INDEX.md (§10.0) -->

## Red Flags — STOP and re-enter discipline

If any of these thoughts arise, STOP, re-read the HARD-GATE constraint, and
return to the checklist:

- [Filled in A2 from RED-baseline §10.0; ~5–8 sentence-form entries, one per
  observed pressure-pattern from R1/R2/R3 reports]

## Anti-Pattern: "This Is Too Simple To Need A Design"

[hand-ported from upstream, lines 16-20]

## Process Checklist

[hand-ported from upstream, lines 22-32, adapted to mdai targets:

1. Explore project context  (already done in pre-context phase)
2. Offer visual companion (if visual)
3. Ask clarifying questions — one at a time
4. Propose 2-3 approaches with trade-offs
5. Present design sections, get approval after each
6. Write design doc to docs/mdai/specs/  ← OVERRIDDEN
7. Spec self-review (placeholders/consistency/scope/ambiguity)
8. User reviews written spec
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

<!-- §11.2 Rationalization-Table — filled in A2 from docs/mdai/red-baseline/INDEX.md (§10.0) -->

## Rationalization-Table

| Excuse | Reality |
|---|---|
| [Filled in A2 from RED-baseline §10.0; ~8–12 rows] | |

Tabelle deckt alle 9 mdai-Discipline-Punkte (siehe §11) ab. Lücken ohne
Baseline-Daten werden mit "no baseline data yet — reasoned counter" markiert
statt erfunden.

## Visual companion dispatch (step 2, conditional)

@if visual_companion_active
  Visual-companion mode is active. Read the upstream guide for HTML-fragment patterns,
  CSS classes, and event-stream format:

  {{ @query lean-ctx ctx_shell "cat ~/.claude/plugins/cache/claude-plugins-official/superpowers/*/skills/brainstorming/visual-companion.md" }}

  Start the companion server (persistent mockups under .superpowers/brainstorm/):

  @query lean-ctx ctx_shell "~/.claude/plugins/cache/claude-plugins-official/superpowers/*/skills/brainstorming/scripts/start-server.sh --project-dir \"$PWD\""

  Capture `screen_dir` and `state_dir` from the server-info JSON for subsequent screen pushes.
  Ensure `.superpowers/` is listed in `.gitignore` (see §16 P0).
@endif

## Spec reviewer dispatch (step 7)

After the spec source (`.mdai.md`) is written but before the user-review gate, dispatch a
reviewer subagent with `@call specReviewerPrompt(spec_path={{ slug }}-design.md)` as the
prompt body. The reviewer returns `Status` + `Issues` + `Recommendations`. Apply issues
inline; surface recommendations for the user to consider. Then proceed to step 8.

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

### 6.4 Phase: write-outputs

```markdown
@phase write-outputs

@call writeSpec(slug={{ slug }}, body={{ design_content }})
@call writeMdaiPlan(slug={{ slug }}, phases={{ phase_list | map(p => planPhase(p.id, p.title, p.files, p.steps)) |
join }})

Default output (two files staged in working tree):

- docs/mdai/specs/{{ @date format="YYYY-MM-DD" }}-{{ slug }}-design.mdai.md (spec source, consumer="ai")
- docs/mdai/plans/{{ @date format="YYYY-MM-DD" }}-{{ slug }}.mdai.md (plan, consumer=ai)

Optional opt-in (call `@call renderSpec(slug, target="chat"|"file")` after spec write):
- target="chat" → render inline, no file
- target="file" → adds docs/mdai/specs/{{ @date format="YYYY-MM-DD" }}-{{ slug }}-design.md

Verification:
@query lean-ctx ctx_shell "git status docs/mdai/"

Note: commit is left to the user (per user CLAUDE.md rules — never auto-commit).
@end
```

### 6.5 Phase: handoff

```markdown
@phase handoff

Plan ready for execution. Next step:

/mdai-execution docs/mdai/plans/{{ @date format="YYYY-MM-DD" }}-{{ slug }}.mdai.md

Phase inventory:
{{ @query mcp markdownai list_phases path="docs/mdai/plans/{{ @date format='YYYY-MM-DD' }}-{{ slug }}.mdai.md" }}

Persist plan state (for cross-session resume):
{{ @query mcp lean-ctx ctx_knowledge --remember topic="mdai-plan:{{ slug }}" body='{"phases": [...], "current_phase": "P0", "status": "planned"}' }}
@end
```

## 7. Wrap-Mechanik im Detail

### 7.1 Hand-port (Fork minimal) statt Skill-Invoke

Upstream-`superpowers:brainstorming/SKILL.md` wird **einmal** beim Bootstrap manuell gelesen und die relevanten Slices (
Z. 16-20, 22-32, 70-104, 140-145) in die `dialog`-Phase von `body.mdai.md` kopiert. Anschließend wird der Upstream-Skill
nicht mehr berührt — kein `Skill(superpowers:brainstorming)`-Invoke, kein `@include` zur Renderzeit.

**Output-Kontrolle:** Vollständig bei uns. Die Schritte 6 ("Write design doc") und 9 ("Transition") der
Upstream-Checklist werden beim Hand-Port direkt auf unsere Targets (`docs/mdai/specs/`, `docs/mdai/plans/`)
umgeschrieben. Es gibt keinen Default-Pfad, der versehentlich getroffen werden könnte.

**Drift-Mitigation:** Wird durch das Backlog-Skill `mdai-drift-check` adressiert:

1. Hand-ported Sektionen tragen Header-Annotation:
   `# Hand-ported from superpowers/X.Y.Z/.../SKILL.md, lines 16-20, 22-32, 70-104, 140-145`.
2. `mdai-drift-check` hasht die referenzierten Upstream-Zeilen, vergleicht mit gespeichertem Hash in
   `docs/mdai/upstream-hashes.json`.
3. Bei Diff: Mensch-lesbarer Diff-Report unter `docs/mdai/drift-reports/`, mit Vorschlag welche Stellen in
   `body.mdai.md` ggf. zu aktualisieren sind.
4. Trigger: manuell via `/mdai-drift-check`, oder periodisch via `loop` / `schedule`.

Solange `mdai-drift-check` nicht existiert, wird Drift einmalig bei jedem Upstream-Bump manuell verglichen (akzeptiertes
Risiko, siehe §14).

### 7.2 Re-template statt wrap für writing-plans

Begründung:

1. **Output-Format inkompatibel.** `superpowers:writing-plans` schreibt `.md` mit Checkboxes nach
   `docs/superpowers/plans/`. Wir wollen `.mdai.md` mit `@phase`-Markern nach `docs/mdai/plans/`. Wrap → signifikantes
   Post-Processing.
2. **Disziplin ist kompakt.** Upstream-Body ~50 Zeilen, im Kern: Task-Decomposition (3+ unabhängige Tasks),
   Phase-Granularität, Files/Steps-Schema. Übersetzt sauber in drei Macros: `planFrontmatter`, `planPhase`, `planStep`.
3. **Token-Kosten:** Wrap = +800 Token Upstream-Load. Macros = +200 Token inline. Faktor 4.
4. **Drift-Risiko:** Wird durch das Backlog-Skill `mdai-drift-check` adressiert (siehe §15).

### 7.3 Lean-ctx-Routing im Plan-Body

Generierter `.mdai.md`-Plan enthält keine direkten Shell-Calls. Alle Discovery/Read-Direktiven gehen über
lean-ctx-MCP-Tools:

| Plan-Intent        | Macro/Direktive                            | Begründung                  |
|--------------------|--------------------------------------------|-----------------------------|
| Datei lesen        | `mcp__lean-ctx__ctx_read(path, mode)`      | Cached, ~13 Tok bei Re-Read |
| Verzeichnis listen | `mcp__lean-ctx__ctx_tree(path, depth)`     | Kompakter als `ls -R`       |
| Pattern-Suche      | `mcp__lean-ctx__ctx_search(pattern, path)` | Token-effizient             |
| Shell-Op           | `mcp__lean-ctx__ctx_shell(command)`        | 95+ Kompressions-Pattern    |
| Edit ohne Read     | `mcp__lean-ctx__ctx_edit(path, old, new)`  | Wenn Read nicht verfügbar   |

Diese Mapping-Tabelle wird in `docs/mdai/macros/tool-quick-ref.md` zentralisiert und via `@include` in jeden generierten
Plan eingezogen.

## 8. Spec ↔ Plan Output-Formate

| Artefakt | Source-Datei | Render-Datei                | Format-Direktive                                                    | Konsument                          |
|----------|--------------|-----------------------------|---------------------------------------------------------------------|------------------------------------|
| Spec     | `*.mdai.md`  | **— (Default: kein Render-File)** | `@markdownai v1.0 consumer="ai"`  | mdai-execution-Subagents; User via `.mdai.md`-Source oder Opt-in Render |
| Plan     | `*.mdai.md`  | — (kein eigenständiges Artefakt)  | `@markdownai v1.0` (consumer=ai)  | mdai-execution-Subagents                                                |

**Default-Verhalten Spec:** Nur die `*.mdai.md`-Source wird committet. Render in `*.md` ist **Opt-in** über das
`renderSpec(slug, target)`-Macro (siehe §6.1):

- `target="none"` → kein Render (Default)
- `target="chat"` → Render via `mcp__markdownai__read_file(consumer="human", format="standard")`, Output inline in
  Claudes Antwort (kein File)
- `target="file"` → Render via MCP, Output via `ctx_shell`-heredoc in `*.md` geschrieben (zusätzliches versioniertes
  Artefakt, User-Acknowledgement Pflicht)

Hinter dem Vorhang: `mcp__markdownai__read_file(consumer="human", …)` überschreibt den `consumer="ai"`-Header zur
Render-Zeit — Source-File bleibt unverändert. Plan wird **nie** für Human gerendert (exklusiv Subagent-Input,
phase-isoliert via `read_file(phase=…, format=ai)`).

## 9. Slash-Trigger und Description-Match

**Slash-Command:** `/mdai-brainstorm` — Skill-Name = Command-Name (Konvention von Claude Code).

**Description (Frontmatter):**

> Use when starting creative work that will produce both a versioned design spec under docs/mdai/specs/ and a
> multi-phase .mdai.md plan under docs/mdai/plans/ for parallel subagent dispatch.

**Trigger-Disziplin:** Primärer Trigger ist der explizite Slash-Command `/mdai-brainstorm`. Description-Match
ist sekundär — die Pfad-Trigger (`docs/mdai/specs/`, `docs/mdai/plans/`, `.mdai.md`) grenzen den Skill von
`superpowers:brainstorming` ab (das nach `docs/superpowers/specs/` schreibt). Keine Implementation-Details
in der Description (Begründung: writing-skills CSO §1 — Description-Workflow-Summaries erzeugen einen
Shortcut, dem Claude folgt, statt body.mdai.md zu lesen).

## 10. Smoke-Tests

### 10.0 RED-Phase — Baseline ohne mdai-brainstorm

**Pflicht-Schritt vor A1.** Iron Law aus `writing-skills/SKILL.md` § "RED-GREEN-REFACTOR":
ohne dokumentiertes Baseline-Verhalten kein Skill-Write.

**Zeitpunkt:** Direkt vor A1 (siehe §16). Skill darf noch nicht existieren — weder
unter `skills/mdai/` noch via `.claude/skills/`-Symlink. Falls A4 schon gemacht
wurde, Symlink temporär entfernen.

**Dispatch:** Drei parallele Subagents via `Agent`-Tool in einer Nachricht,
alle `model="sonnet"`, `subagent_type="general-purpose"`. Transparenter
Test-Modus (Subagent weiß, dass es Baseline-Test ist, dokumentiert ehrlich).

**Szenarien:**

| ID | Pressure | Prompt-Kern |
|---|---|---|
| R1 | Cold start, keine | "Brainstorm ein Feature, das mehrere parallele Subagents braucht. Output: Plan für phasen-isolierten Dispatch." |
| R2 | Time pressure | R1 + "Wenig Zeit. Approach-Vergleiche skippen. Plan heute fertig." |
| R3 | Authority pressure | R1 + "Tech-Lead sagt: schreib direkt den Plan, kein Spec-Doc nötig." |

**Erfassung pro Subagent (Report-Footer, verbatim):**

1. Welche Skill wurde getriggert? (`superpowers:brainstorming`? `writing-plans`? keiner?)
2. Welcher Output-Pfad gewählt? (`docs/superpowers/specs/`? `docs/mdai/`? andere?)
3. File-Endung: `.md` oder `.mdai.md`?
4. Klärungsfragen one-at-a-time oder gebündelt?
5. Approval-Gate vor Plan-Schreiben respektiert?
6. Rationalisierungen zum Step-Skipping (verbatim Zitate)

**Output-Storage:** Drei Reports unter `docs/mdai/red-baseline/2026-05-23-R{1,2,3}.md`,
committet. Konsolidierter Index unter `docs/mdai/red-baseline/INDEX.md` mit
Rationalisierungen gruppiert nach Pressure-Typ — Input-Material für §11
(Red-Flags + Rationalization-Table).

**Erfolgskriterium:** Mindestens 5 verbatim Rationalisierungen erfasst, mindestens
2 verschiedene Drift-Pattern (Pfad-Drift, File-Endung-Drift, Skip-Approval, etc.)
dokumentiert. Erst dann darf A1 starten.

**Re-Run-Trigger:** Bei jedem `superpowers`-Versions-Bump erneut, um Baseline-Drift
zu detektieren (Sub-Task im jeweiligen Bump-Plan).

### 10.1 Trigger-Test (GREEN — Pointer-Compliance)

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

- **3/3 Pass** → grün, weiter zu §10.2.
- **2/3 Pass** → **manuelle Diagnose** (mögliche Ursachen: MCP-Disconnect,
  Cache-Effekt, Setup-Rauschen). Nicht auto-Fallback. Erst nach Root-Cause-
  Analyse entscheiden zwischen Re-Test (bei klarer Glitch-Ursache),
  Skill-Iteration (bei Pattern-Verbesserungsbedarf) oder §18-Fallback
  (wenn Root-Cause auf strukturelle Skill-Loader-Schwäche zeigt).
- **0/3 oder 1/3 Pass** → Fallback nach §18 (5-File-Layout) aktivieren,
  A2 re-do.

### 10.2 Discipline-Fidelity-Test (GREEN)

dialog-phase führt zu interaktivem Dialog mit Klärungsfragen (one-at-a-time), 2-3
Approach-Vorschlägen, Design-Sektion-by-Section. Verifiziere: kein
`Skill(superpowers:brainstorming)`-Invoke im Tool-Log, Claude folgt der hand-ported
Checklist ohne Upstream-Skill-Load. Red-Flags + Rationalization-Table aus §11
zur Skill-Laufzeit gefüllt und respektiert.

### 10.3 Output-Test (GREEN)

write-outputs erzeugt im Default-Pfad genau **zwei** Files:

- `docs/mdai/specs/<date>-<slug>-design.mdai.md` (Spec-Source, `consumer="ai"`)
- `docs/mdai/plans/<date>-<slug>.mdai.md` (Plan, consumer=ai)

Verifiziere via `git status` + `mcp__markdownai__list_phases` (Plan listet erwartete
Phase-IDs). Bei Opt-in `renderSpec(target="file")` kommt zusätzlich
`<date>-<slug>-design.md` dazu — separat verifizieren. Kein File unter
`docs/superpowers/specs/` (Default-Pfad existiert in unserem Flow nicht mehr).

### 10.4 GREEN-Re-Run der RED-Szenarien

Nach Skill-Implementation (A6 done): die drei RED-Szenarien aus §10.0 erneut
laufen lassen, diesmal **mit** installierter Skill. Erwartung:

- Subagent triggert `/mdai-brainstorm` (statt `superpowers:brainstorming`).
- Folgt der Discipline (HARD-GATE, one-at-a-time, Approval-Gates).
- Schreibt nach `docs/mdai/specs/` und `docs/mdai/plans/`.
- File-Endung `.mdai.md`.

Reports unter `docs/mdai/green-verification/2026-05-23-R{1,2,3}.md`, committet.
Diff zu §10.0-Reports beweist Skill-Wirkung. Wenn ein Pressure-Szenario
weiterhin driftet → §11 Rationalization-Table erweitern, Skill iterieren,
Re-Test.

### 10.5 Phase-Budget-Test

Pro Phase in `body.mdai.md` Worte-Budget gegen §12-Tabelle prüfen:

```bash
for phase in pre-context dialog write-outputs handoff; do
  count=$(mcp__markdownai__read_file path=skills/mdai/mdai-brainstorm/body.mdai.md \
            phase=$phase format=ai | wc -w)
  echo "$phase: $count words"
done
```

Pass-Kriterien: alle Phasen unter Budget aus §12. Bei Überschreitung Kürzung
oder Sub-Phase-Split, **nicht** Budget-Aufweichung.

## 11. Bulletproofing — Red-Flags + Rationalization-Table

Discipline-enforcing Skills brauchen explizite Anti-Rationalisierungs-Strukturen
(writing-skills/SKILL.md §"Bulletproofing"). Zwei Artefakte werden in
`body.mdai.md` dialog-phase verankert. Daten kommen aus §10.0 RED-Baseline,
nicht aus Spekulation.

### 11.1 Red-Flags-Liste (Position: direkt nach HARD-GATE-@constraint)

**Format** in `body.mdai.md` dialog-phase:

```markdown
## Red Flags — STOP and re-enter discipline

If any of these thoughts arise, STOP, re-read the HARD-GATE constraint, and
return to the checklist:

- [Sentence-shaped self-check, one per line]
```

**Source:** verbatim Rationalisierungen aus den drei RED-Subagent-Reports
(§10.0), in Satzform umformuliert. Beispiele KOMMEN aus RED-Daten, nicht aus
Spekulation. Erwartete Anzahl: 5–8 Einträge.

### 11.2 Rationalization-Table (Position: nach Process-Details, vor Visual-Companion-Dispatch)

**Format** in `body.mdai.md` dialog-phase:

```markdown
| Excuse | Reality |
|---|---|
| [verbatim oder leicht paraphrasierte Rationalisierung aus RED] | [Konter-Argument, 1 Satz] |
```

**Source:** dieselbe `docs/mdai/red-baseline/INDEX.md`. Jede Pressure-Kategorie
(cold/time/authority) liefert 2–4 Zeilen. Die 9 mdai-Discipline-Punkte (siehe
§11.4) sind Kategorie-Anchor — Tabelle deckt alle 9 ab, auch wenn RED nur für
einige Daten lieferte (Lücken markiert "no baseline data yet — reasoned counter").
Erwartete Anzahl: 8–12 Zeilen.

### 11.3 A2-Workflow (Sub-Schritte für §16 A2)

A2 schreibt `body.mdai.md` inkl. dialog-phase mit gefüllten Red-Flags +
Rationalization-Table. Voraussetzung: RED-Baseline §10.0 erfolgreich gelaufen,
`docs/mdai/red-baseline/INDEX.md` committet.

**A2-Sub-Schritte:**

1. `docs/mdai/red-baseline/INDEX.md` lesen, alle Rationalisierungen extrahieren.
2. Pro Pressure-Typ in 1-Satz-Self-Check umformulieren → Red-Flags-Liste füllen.
3. Pro Rationalisierung Konter-Argument formulieren → Rationalization-Table füllen.
4. 9-Discipline-Punkte cross-check: jeder Punkt in mindestens einer Tabellen-Zeile
   abgedeckt? Lücken mit "no baseline data yet — reasoned counter" markieren.
5. `wc -w` auf dialog-Phase (siehe §12) — falls Budget gerissen, Sub-Phase-Split.

### 11.4 Die 9 mdai-Discipline-Punkte (Cross-Check-Anchor)

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

## 12. Token-Budget pro Phase

Phase-Isolation aus §2 funktioniert nur, wenn jede einzelne Phase klein ist.
Hartes Cap pro Phase, Verifikation via `wc -w` auf phase-isolierten Output.

### 12.1 Budget-Tabelle

| Phase | Budget (Worte) | Begründung |
|---|---|---|
| pre-context | ≤300 | Statische Anweisungen knapp; `@query`-Outputs sind dynamisch (zur Render-Zeit eingefügt, nicht im Source) und zählen NICHT mit. |
| dialog | ≤600 | Hand-ported Checklist (~80) + Process (~120) + Principles (~50) + Red-Flags (~60) + Rationalization-Table (~120) + Visual-Companion-Conditional (~40) + Spec-Reviewer-Dispatch (~40) + Anti-Pattern-Intro (~50) + HARD-GATE-Constraint (~40) = ~600. Knapp über `writing-skills` Soft-Norm „<500 für andere Skills" — toleriert, da `mdai-brainstorm` nicht in jeder Session geladen wird. |
| write-outputs | ≤150 | Nur `@call`-Aufrufe + Output-Liste. |
| handoff | ≤100 | Phase-Inventory + Knowledge-Recall + Next-Step-Anweisung. |
| **Total body.mdai.md** | **≤1200 Worte** | Sum + ~50 für Header/Macros-Block. |

### 12.2 Verifikations-Befehl

```bash
for phase in pre-context dialog write-outputs handoff; do
  count=$(mcp__markdownai__read_file \
            path=skills/mdai/mdai-brainstorm/body.mdai.md \
            phase=$phase format=ai | wc -w)
  echo "$phase: $count words"
done
```

Identisch zu §10.5 Phase-Budget-Test.

### 12.3 Eskalation bei Budget-Überschreitung

**Reihenfolge:**

1. **Sub-Phase-Split:** Falls `dialog` >600 Worte, splitten in `dialog-rules`
   (HARD-GATE + Red-Flags + Rationalization-Table, „diszipliniert dich") und
   `dialog-process` (Checklist + Process + Principles + Companion + Reviewer-
   Dispatch, „so machst du es"). Jede Sub-Phase <400 Worte. Kostet einen
   extra `read_file`-Roundtrip pro Brainstorm-Session — toleriert.
2. **Inline-Kürzung:** Process-Details und Principles auf Bullet-Form ohne
   erläuternden Text. Risiko: hand-port verliert Disziplin-Bedeutung. Nur
   wenn Split nicht möglich.
3. **Budget-Aufweichung:** Letzte Option, braucht explizite Begründung in
   `body.mdai.md`-Kommentar + neuer Eintrag in §14 Risiken.

## 13. Non-Goals

1. **`superpowers/`-Skills modifizieren** — bleiben read-only. Kein Patch in Upstream-Body.
2. **Automatische `.md` → `.mdai.md`-Migration** alter Pläne — erstmal manuelles Re-Writing.
3. **Drift-Detection im aktuellen Scope** — separater Skill `mdai-drift-check` als Backlog-Item, kein Bestandteil dieser
   Spec.
4. **Plan-State-Persistenz** — `mdai-memory` (aus altem Design) bleibt zuständig, `mdai-brainstorm` ruft es nur am Ende
   einmal auf.
5. **`mdai-execution` und `mdai-memory` Migration** auf `skills/mdai/`-Layout — separate Spec.

## 14. Risiken

| Risiko                                                                                                             | Schweregrad | Mitigation                                                                                                                                                                    |
|--------------------------------------------------------------------------------------------------------------------|-------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Claude folgt der hand-ported Disziplin nicht so streng wie ein echter Skill-Invoke (kein Skill-Engine-Enforcement) | **Mittel**  | dialog-phase formuliert Anweisungen als `@constraint id="..." severity="high"` plus HARD-GATE. Discipline-Fidelity-Test §10.2 prüft Verhalten.                                |
| Upstream-`brainstorming/SKILL.md` ändert sich nach Bump → hand-ported Slices veralten                              | **Mittel**  | `mdai-drift-check`-Backlog: Hash-Vergleich + Diff-Report. Bis dahin: bei jedem `superpowers`-Versions-Bump manuelle Review der Source-Zeilen (16-20, 22-32, 70-104, 140-145). |
| `mcp__markdownai__read_file(phase=…)` Server disconnected mitten in Session                                        | Niedrig     | Reconnect via `/mcp`, headless-Fallback dokumentiert                                                                                                                          |
| `body.mdai.md` wird trotz pointer-Anweisung full gelesen                                                           | Mittel      | SKILL.md formuliert die Anweisung als Hard-Constraint („MUST"), nicht „SHOULD". Smoke-Test §10.1 prüft.                                                                       |
| `mcp__markdownai__read_file(consumer="human", format="standard")` liefert nicht das erwartete Format               | Niedrig     | Verifiziert mit mai-CLI v0.0.24: Flags `--consumer human` und `--format standard` existieren. Smoke-Test §10 prüft MCP-Output direkt; bei Diskrepanz Fallback auf CLI via `npx mai render`. |
| `mcp__lean-ctx__ctx_graph` ist nicht gebaut (kein Index) → pre-context liefert leeren Graph                        | Niedrig     | Pre-context-Phase führt `@if`-Check: bei leerem Graph fällt zurück auf `ctx_tree` + `ctx_overview`. `ctx_graph action="build"` einmalig in P0.                                |
| User-Global-Skills haben niedrigere Priorität als Plugin-Skills bei Description-Match                              | Niedrig     | Smoke-Test §10 prüft. Falls Konflikt: explizites `/mdai-brainstorm`-Trigger nutzen.                                                                                           |

## 15. Backlog (separate Specs)

Explizit-deferred Parking-List. Jeder Eintrag bekommt bei Bedarf eine eigene Spec via
`/mdai-brainstorm`. Backlog ≠ Open Items — diese hier sind bewusst aufgeschoben.

1. **`mdai-drift-check`** — Skill zum Upstream-Hash-Vergleich + Diff-Report. Hash-Store in
   `docs/mdai/upstream-hashes.json`. Manueller Trigger oder periodisch via `loop`/`schedule`.
   Trigger für separate Spec: spätestens beim ersten `superpowers`-Versions-Bump nach Release.
2. **`mdai-execution`-Migration** auf `skills/mdai/mdai-execution/`-Layout. Inhaltlich unverändert, nur Pfad-Umzug.
   Trigger: vor nächster substantieller mdai-execution-Änderung.
3. **`mdai-memory`-Migration** dito.
4. **Upstream-PR an markdownai** für `respondTool()`-Fix (separat, blockt nichts). Trigger: jederzeit.
5. **Plugin-Packaging** — Bündelung aller drei mdai-Skills als ein Claude-Code-Plugin (eigene `package.json`,
   `hooks.json` Stub). Trigger: nach mind. einem Monat stabiler Nutzung der drei Skills.
6. **Spec-Human-Render-Wrapper-Template** (Variante B oder C aus dem Vergleich §6/§7) — separate `.mdai.md`-Datei
   unter `skills/mdai/mdai-brainstorm/templates/` mit Layout-Macros (Cover-Page mit Branch/Date, automatische
   TOC, Constraint-Glossar). Bei aktiviertem Render zielt `renderSpec` auf den Wrapper, der das Source-Spec
   `@include`-ed und mit Human-Struktur anreichert. Aktuell nicht nötig — `renderSpec` rendert direkt das
   Source-Spec via MCP. Trigger für separate Spec: sobald ein Reviewer Cover/TOC/Glossar reproduzierbar verlangt.

## 16. Implementierungsschritte (high-level, der echte Plan kommt via writing-plans-Skill)

| Phase | Aufgabe                                                                                                                                                                                                                                            |
|-------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| RED   | **Iron Law — Baseline vor jedem Skill-Write (§10.0).** Drei parallele Subagents (`Agent`-Tool, `model="sonnet"`, `general-purpose`) für Szenarien R1/R2/R3 dispatchen. Reports + INDEX.md unter `docs/mdai/red-baseline/` committen. Erfolgskriterium: ≥5 verbatim Rationalisierungen + ≥2 verschiedene Drift-Pattern. Blockiert P0/A1 bis erfüllt. |
| P0    | Macro-Mirror `docs/mdai/macros/` verifizieren, Knowledge-Schema + Gotchas-Seed prüfen (sind aus altem P0 schon teilweise da). **Plus:** `mcp__lean-ctx__ctx_graph action="build"` einmalig laufen lassen, damit pre-context-Phase einen Index hat. **Plus:** `.superpowers/` in `.gitignore` aufnehmen (persistente Visual-Companion-Mockups via `--project-dir "$PWD"`). |
| A1    | `skills/mdai/mdai-brainstorm/SKILL.md` schreiben (~15 Z, pointer). Description aus §5 (Trigger-only, keine Workflow-Summary). |
| A2    | `skills/mdai/mdai-brainstorm/body.mdai.md` schreiben (~150 Z, alle Phasen + Macros). **Sub-Schritte (§11.3):** (a) `docs/mdai/red-baseline/INDEX.md` lesen, Rationalisierungen extrahieren; (b) Red-Flags-Liste füllen (5–8 sentence-form, §11.1); (c) Rationalization-Table füllen (8–12 Zeilen, §11.2); (d) 9-Discipline-Punkte-Cross-Check (§11.4); (e) `wc -w` pro Phase gegen §12.1-Budget — bei Überschreitung Sub-Phase-Split (§12.3). |
| A3    | `skills/mdai/mdai-brainstorm/scripts/install.sh` schreiben + `chmod +x` (Script bleibt im Repo, wird in A6 erst gerufen)                                                                                                                           |
| A4    | **Project-local Test-Setup:** `mkdir -p .claude/skills && ln -sf ../../skills/mdai/mdai-brainstorm .claude/skills/mdai-brainstorm`. Verifiziere: `ls -la .claude/skills/mdai-brainstorm/` zeigt Symlink.                                           |
| A5    | **Smoke-Tests gegen project-local Symlink (§10.1–§10.5).** Skill ist nur in diesem Repo aktiv, beeinflusst andere Projekte nicht. Iteriere am Source unter `skills/mdai/mdai-brainstorm/`, bis alle Tests grün sind. **Bei §10.1 Fail (0/3 oder 1/3 Pass):** A2 re-do mit Fallback-Architektur §18 (5-File-Layout). |
| A6    | **GREEN-Re-Run der RED-Szenarien (§10.4).** Drei Reports unter `docs/mdai/green-verification/` committen, Diff gegen RED-Baseline beweist Skill-Wirkung. |
| A7    | **Globaler Install (erst nach A5 + A6 grün):** `bash skills/mdai/mdai-brainstorm/scripts/install.sh`. Verifiziere via `ls ~/.claude/skills/mdai-brainstorm/`. Ab jetzt ist `/mdai-brainstorm` projekt-übergreifend verfügbar. |
| A8    | `mdai-plans` deinstallieren: `rm -rf ~/.claude/skills/mdai-plans/`, verifiziere via `/mdai-plans` triggert nicht mehr. |
| A9    | Self-Bootstrap: Plan-Datei selbst (`docs/mdai/plans/2026-05-23-mdai-brainstorm-impl.mdai.md`) wird per `/mdai-brainstorm` produziert — `mai render` + `list_phases` als finale Validation. |

## 17. Annahmen, die in Smoke-Tests zu verifizieren sind

1. `mcp__markdownai__read_file(consumer="human", format="standard")` rendert eine Spec für Human-Konsum korrekt
   (verifiziert mit mai-CLI v0.0.24, MCP wrapped denselben Renderer). Smoke-Test §10 prüft den MCP-Output direkt;
   bei Diskrepanz Fallback auf CLI-Variante: `cd markdownai && npx mai render --consumer human <path>`.
2. `mcp__markdownai__read_file(phase=…, format=ai)` returniert eine self-contained Phase-Auszug inkl. relevanter
   `@define`-Macros (geerbt von der Datei-Header). Falls nicht: Macros müssen in jeder Phase neu deklariert werden.
3. Claude Codes Skill-Loader respektiert die pointer-Anweisung „lies body.mdai.md nicht full". Verifikation §10.1
   Pointer-Compliance-Test (3 Runs, `jq`-Grep auf Session-Transcript). Bei 0/3 oder 1/3 Pass → **Fallback-Architektur
   §18** (5-File-Layout) aktivieren, A2 re-do.
4. Claude folgt der hand-ported Disziplin in `dialog`-phase ohne `Skill(superpowers:brainstorming)`-Invoke (
   Discipline-Fidelity-Test §10.2). Falls Claude die Disziplin lockerer interpretiert als ein echter Skill-Invoke:
   `@constraint id="hard-gate" severity="high"`-Block und HARD-GATE-Anweisung schärfen, oder als Fallback Hybrid (
   Skill-Invoke + hand-ported Pre-Briefing) zurück erwägen.
5. `mcp__lean-ctx__ctx_graph` ist im Projekt bereits gebaut (oder wird in P0 einmal indiziert). Falls Index leer:
   pre-context-Phase fällt auf `ctx_tree` + `ctx_overview` zurück.

Diese Annahmen sind die einzigen blockierenden Unbekannten. Alle anderen Risiken sind mitigierbar oder akzeptiert.

## 18. Fallback-Architektur (aktiviert bei §10.1 Pointer-Compliance-Fail)

Wenn Claude die pointer-Anweisung nicht respektiert (§17 Annahme 3, verifiziert
in §10.1), zwingt das File-System-Layout die Phase-Isolation auf — jedes File
ist klein genug, dass Full-Read OK ist.

### 18.1 5-File-Layout

```
skills/mdai/mdai-brainstorm/
  SKILL.md              ← pointer (~15 Z) → "Read phases/<id>.mdai.md in order"
  body.mdai.md          ← LEEREN STUB, nur "see phases/" Verweis
  macros.mdai.md        ← @define-Macros zentral (writeSpec, writeMdaiPlan, etc.)
  phases/
    pre-context.mdai.md
    dialog.mdai.md
    write-outputs.mdai.md
    handoff.mdai.md
  scripts/install.sh
```

### 18.2 SKILL.md-Anweisung im Fallback

```markdown
DO NOT read body.mdai.md (empty stub). Read phases/<id>.mdai.md in order,
starting with pre-context. macros.mdai.md is loaded once before phase-1
via mcp__lean-ctx__ctx_read(path=".../macros.mdai.md", mode=full).

Order: pre-context → dialog → write-outputs → handoff.
```

### 18.3 Migration-Aufwand (A2 re-do)

1. Inhalt aus `body.mdai.md` in 4 Phase-Files extrahieren (1:1-Split nach `@phase`-Marker).
2. `@define`-Block in `macros.mdai.md` auslagern.
3. `body.mdai.md` zum 3-Zeilen-Stub reduzieren (`"# Stub — see phases/ directory"`).
4. SKILL.md-Anweisung umschreiben.
5. install.sh anpassen (kopiert jetzt 6 Files statt 2).
6. §10.1 Re-Test gegen 5-File-Layout — Erwartung: 3/3 Pass, weil File-System die
   Isolation erzwingt.

### 18.4 Token-Impact

Pre-Phase-Load: macros.mdai.md (~200 Tok) + erste Phase (~200–600 Tok je nach
Phase). Pro Phase-Wechsel: ein zusätzlicher `ctx_read`-Call vs. einem
`mcp__markdownai__read_file(phase=…)`-Call — neutraler Tausch.

### 18.5 Begründung

Wenn pointer-Compliance scheitert, ist File-System-Layout der nächst-sichere
Mechanismus. Phase-File-Splits sind ein bewährtes Pattern (vgl. office-Skills
im upstream-superpowers, die nach Heavy-Reference in separate `.md`-Files
splitten).
