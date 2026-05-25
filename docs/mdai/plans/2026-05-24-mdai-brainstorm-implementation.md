---
id: mdai-brainstorm-implementation
status: ready-to-execute
created: 2026-05-24
spec: docs/mdai/specs/2026-05-24-mdai-brainstorm-design.mdai.md
requires:
  mdai-library: ">=0.1.0"
  mcp__lean-ctx: ">=3.6.16"
  mcp__markdownai: ">=0.0.24"
---

@markdownai v1.0

# mdai-brainstorm — Implementation Plan (Phased)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan phase-by-phase, step-by-step. Steps use checkbox (`- [ ]`)
> syntax for tracking. The plan itself is structured around `@phase` blocks (per `markdownai/README.md` §"@phase,
> @on complete, and @graph") so it can be navigated via `mcp__markdownai__list_phases` and
> `mcp__markdownai__read_file(phase=<id>, format=ai)`.

**Goal:** Skill `mdai-brainstorm` produktionsreif machen — ein versionierter Spec-Writer (kein Plan-Writer), der
markdownai-Direktiven aktiv nutzt, hand-ported Brainstorming-Disziplin aus `superpowers:brainstorming` durchsetzt
und nach Spec-Approval explizit auf `/superpowers:writing-plans` als Next-Step zeigt.

**Architecture:** SKILL.md (Pointer, ~15 Z) + `body.mdai.md` (4 Phasen: pre-context | dialog | write-outputs |
handoff, phase-by-phase via MCP geladen) + Lazy-Import-Pattern für Skill-A-Packs (`write-spec.md`,
`spec-reviewer.md`) + project-local Symlink unter `.claude/skills/mdai-brainstorm`. Library v0.1.0 bleibt
unverändert (alle Additive bereits im Repo). Implementation deckt P0–A5.5 aus Spec §15 ab.

**Tech Stack:** `mcp__markdownai` (phase-aware read), `mcp__lean-ctx` (ctx_read/search/tree/shell, ctx_graph),
`@markdownai v1.0`-Direktiven (`@phase`, `@call`, `@import`, `@include`, `@constraint`, `@if file.exists`),
project-local Symlink für Skill-Loader-Discovery.

---

## State-of-Repo (Pre-Plan Snapshot, 2026-05-24)

Mehrere Spec-A2.5/A2.6-Sub-Schritte wurden bereits in der Patch-Session 2026-05-24 erledigt. Plan markiert die
betroffenen Tasks als **Verify-Only** statt **Create**:

| Spec-Schritt | Datei                                              | Status                                           |
| P0           | `.gitignore` Eintrag `.superpowers/`               | offen (0 matches in `.gitignore`)                |
| P0           | `mcp__lean-ctx__ctx_graph` Index                   | offen — einmalig bauen                           |
| A1           | `mdai/skills/mdai-brainstorm/SKILL.md`             | **fehlt — CREATE**                               |
| A2           | `mdai/skills/mdai-brainstorm/body.mdai.md`         | **fehlt — CREATE**                               |
| A2.5         | `mdai/skills/mdai-brainstorm/spec-reviewer.md`     | bereits gepatcht (§0/§4/§5 #1-#11/§6) — verify   |
| A2.6 (a)     | `mdai/core/lean-context.md`                        | exists (34 L, conventions+naming) — verify       |
| A2.6 (b)     | `mdai/core/ctx-tools.md`                           | hat `ctx_read_lines/map/signatures` — verify     |
| A2.6 (c)     | `mdai/core/tool-quick-ref.md`                      | listet 3 neue Wrapper — verify                   |
| A2.6 (d)     | `mdai/changelog.md`                                | hat Additive-Einträge — verify                   |
| A2.6 (e)     | `mdai/MACROS.md`                                   | bereits gelöscht — verify                        |
| A3           | `.claude/skills/mdai-brainstorm` Symlink           | **fehlt — CREATE**                               |
| A4           | Smoke-Tests §8.1–§8.6                              | offen — nach A3 fahren                           |
| A5           | `mdai-plans` Skill-Uninstall                       | offen — Existenz prüfen                          |
| A5.5         | `mdai/skills/mdai-brainstorm/README.md`            | **fehlt — CREATE**                               |

Verifiziert via `ctx_tree mdai/`, `ctx_read` (lean-context.md, ctx-tools.md, tool-quick-ref.md, changelog.md,
spec-reviewer.md, write-spec.md, file-utils.md), `Bash test -f` (SKILL.md, body.mdai.md, symlink), `ctx_search` auf
`.gitignore`. State-Snapshot ist eine Momentaufnahme — vor Phase-Start jeweils kurz re-checken.

---

## @phase p0-setup

**Pre-flight (Spec §15 P0): Index bauen, .gitignore-Eintrag, RED-Baseline-Skip dokumentieren.**

**Files:**
- Modify: `.gitignore` (add `.superpowers/`)

### Step P0.1 — `.gitignore` um `.superpowers/` ergänzen

- [ ] `ctx_search pattern="\.superpowers" path=".gitignore"` ausführen — muss `0 matches` liefern (Baseline).
- [ ] `.gitignore` öffnen und am Ende eine Zeile `.superpowers/` anhängen (Visual-Companion persistiert dort
      Mockups via `--project-dir "$PWD"`, Spec §5.3 Step 2).

```bash
echo '.superpowers/' >> .gitignore
```

- [ ] Verifikation: `ctx_search pattern="^\.superpowers/$" path=".gitignore"` muss `1 match` liefern.

### Step P0.2 — `mcp__lean-ctx__ctx_graph` Index einmalig bauen

- [ ] `mcp__lean-ctx__ctx_graph action="build"` aufrufen — baut den Dependency-Index, damit
      `pre-context`-Phase nicht leeren Graph zurückbekommt (Spec §13 Risiko "ctx_graph leer").
- [ ] Verifikation: `mcp__lean-ctx__ctx_graph action="diagram" kind="deps" depth=2` liefert ein nicht-leeres
      Mermaid-Diagramm.

### Step P0.3 — RED-Baseline-Skip dokumentieren (no-op)

- [ ] Spec §8 Intro liest "Bewusst kein RED-Baseline und kein GREEN-Re-Run" — Plan **erzeugt keine**
      RED-Baseline-Fixtures. Skip-Begründung ist im Spec verankert (explizit-invoke macht
      Trigger-Discovery-Drift irrelevant). Kein Filesystem-Output für diesen Schritt — reine
      Acknowledgement-Notiz.

@on complete
  P0 complete. Index built, .gitignore guarded. Proceeding to a1-skill-pointer.
@end

---

## @phase a1-skill-pointer

**Spec §4 + §15 A1: SKILL.md (Pointer ~15 Z) schreiben.**

**Files:**
- Create: `mdai/skills/mdai-brainstorm/SKILL.md`

### Step A1.1 — Pre-Check: Datei darf nicht existieren

- [ ] `ctx_shell cmd="test -f mdai/skills/mdai-brainstorm/SKILL.md && echo EXISTS || echo MISSING"`
      muss `MISSING` liefern. Falls `EXISTS`: STOP, manuelle Diagnose (gehört vermutlich aus einer früheren
      Session).

### Step A1.2 — `SKILL.md` schreiben

- [ ] Datei anlegen mit folgendem Inhalt (Spec §4 verbatim, Description-Diff aus §7):

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

### Step A1.3 — Verifikation Inhalt

- [ ] `ctx_search pattern="this skill does not write plans" path="mdai/skills/mdai-brainstorm/SKILL.md"` muss
      `1 match` liefern (Description-Wording aus Spec §7).
- [ ] `ctx_search pattern="phase-by-phase" path="mdai/skills/mdai-brainstorm/SKILL.md"` muss `1 match` liefern.
- [ ] Zeilenzahl: `ctx_shell cmd="wc -l mdai/skills/mdai-brainstorm/SKILL.md"` muss ≤ 22 ergeben (Spec §4
      ~15 Zeilen, Toleranz für Frontmatter).

@on complete
  SKILL.md staged. Pointer-only — no workflow leakage. Proceeding to a2-body-workflow.
@end

---

## @phase a2-body-workflow

**Spec §5 + §10.3 + §15 A2: `body.mdai.md` schreiben (~100 Z, 4 Phasen, Lazy-Imports, gefüllte Red-Flags +
Rationalization-Table, §5.6 Konventions-Block in dialog).**

**Files:**
- Create: `mdai/skills/mdai-brainstorm/body.mdai.md`

### Step A2.1 — Pre-Check: Datei darf nicht existieren

- [ ] `ctx_shell cmd="test -f mdai/skills/mdai-brainstorm/body.mdai.md && echo EXISTS || echo MISSING"` muss
      `MISSING` liefern.

### Step A2.2 — Header schreiben (minimal, keine globalen Imports per §5.1)

- [ ] Datei anlegen mit Header:

```markdown
@markdownai v1.0

<!--
  body.mdai.md — Skill A v3 live workflow (mdai-brainstorm)
  Spec: docs/mdai/specs/2026-05-24-mdai-brainstorm-design.mdai.md
  No global @import here — packs are lazy-loaded per phase (Spec §5.1).
-->
```

### Step A2.3 — `@phase pre-context` schreiben (Spec §5.2, Budget ≤250 Worte)

- [ ] Pre-context-Block anhängen:

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

@constraint id="tool-selection" severity="high"
File lesen → `@call ctx_read(path, mode)` (nicht `ctx_shell cmd="cat ..."`).
Verzeichnis listen → `@call ctx_tree(path, depth)` (nicht `ls`/`find`).
Pattern-Suche → `@call ctx_search(pattern, path)` (nicht `grep`/`rg`).
File-Edit ohne Read → `@call ctx_edit(path, old, new)`.
Plan-Phase lesen → `@call read_phase(plan, phase_id)`.
`@call ctx_shell` nur als Last-Resort (git-Ops, Shell-Skripte, Tools ohne Wrapper).
@end

Constraints for the dialog phase:

- Spec target: docs/mdai/specs/ (NOT docs/superpowers/specs/)
- NO plan target — plan-write is a separate skill invocation (handoff phase)
- Hard rules: see @include above
@end
```

### Step A2.4 — `@phase dialog` schreiben (Spec §5.3, Budget ≤580 Worte)

- [ ] Dialog-Phase anhängen mit HARD-GATE-Constraint, hand-ported Disziplin-Slices, **gefüllten** Red-Flags +
      Rationalization-Table (aus §10.3 Sub-Schritten 2+3 — reasoned-counter pro Discipline-Punkt §10.4), plus
      §5.6 Konventions-Block:

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

## Red Flags — STOP and re-enter discipline

If any of these thoughts arise, STOP, re-read the HARD-GATE constraint, and
return to the checklist:

- "Ich schreibe jetzt schnell den Plan dazu, der User wird sich freuen" → STOP. Skill schreibt KEINEN Plan.
  Plan-Write ist Aufgabe von `/superpowers:writing-plans` post-Spec. (Discipline §10.4 #1, Scope-Drift)
- "Diese Spec ist so klein, eine Plain-Markdown-Tabelle reicht" → STOP. Hauptziel §1 verlangt
  markdownai-Direktiven für Live-Inhalte (`@tree`, `@call ctx_overview`, `@constraint`). Plain-Markdown nur mit
  `markdownai_directives_omitted: <reason>` in Frontmatter. (Discipline §10.4 #9)
- "Ich speichere die Spec schnell unter `docs/superpowers/specs/`, das ist ja der Standard" → STOP. mdai-Specs
  gehören nach `docs/mdai/specs/`, Datei-Endung `.mdai.md`. (Discipline §10.4 #2 + #3)
- "Der User will schnell ein Approach-Listing, ich präsentiere direkt das Design" → STOP. Zuerst 2–3
  Approach-Alternativen mit Trade-offs, dann erst Design-Sektionen. One-question-at-a-time gilt auch hier.
  (Discipline §10.4 #4 + #5, time-pressure)
- "Die hat schon abgenickt nachdem ich Section 1 gezeigt hab, ich schreib die Spec jetzt" → STOP.
  Per-Section-Approval beim Design-Walkthrough — eine Section-Genehmigung ist kein Spec-Approval.
  (Discipline §10.4 #6, authority-pressure)
- "Self-Review kann ich überspringen, ich hab beim Schreiben aufgepasst" → STOP. Self-Review §7 ist
  MANDATORY vor User-Review-Gate. Vier Checks (Placeholders / Konsistenz / Scope / Ambiguity) plus #5
  mdai-Direktiven-Nutzung. (Discipline §10.4 #7)
- "Ich lade body.mdai.md kurz full, das ist effizienter" → STOP. Pointer-Anweisung in SKILL.md ist
  Hard-Constraint („MUST"). Phase-by-phase via `mcp__markdownai__read_file(phase=..., format=ai)`.
  (Discipline §10.4 #8, cold-start)

## Anti-Pattern: "This Is Too Simple To Need A Design"

[hand-ported from superpowers:brainstorming/SKILL.md, lines 16-20]

Even a one-paragraph feature deserves a brainstorm pass. The discipline is
about *not skipping the dialog*, not about scaling the output. A 100-line
spec for a 5-line change is fine. Skipping the dialog and going straight to
code is the failure mode.

## Process Checklist

**Each item MUST become a `TaskCreate` entry and be completed in order**
(Upstream §Checklist mandate).

1. Explore project context (already done in pre-context phase).
2. Offer visual companion (if visual) — own message (see Visual-Companion section).
3. Ask clarifying questions — one at a time.
4. Propose 2–3 approaches with trade-offs.
5. Present design sections, get approval after each.
6. Write design doc to `docs/mdai/specs/` (NOT `docs/superpowers/specs/`).
7. Spec Self-Review (5 checks — see "Spec Self-Review" below).
7.5 OPTIONAL: dispatch reviewer-subagent via `@call spec_reviewer_prompt` (mdai-Augmentation).
8. User reviews written spec (exact wording — see "User-Review-Gate").
9. Transition: invoke writing-plans skill (currently `superpowers:writing-plans`;
   future: `mdai-writing-plans` once that skill exists per Spec §14 Backlog #1)
   — THIS SKILL DOES NOT WRITE THE PLAN.

## The Process — Details

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

## Rationalization-Table

| Excuse | Reality |
|---|---|
| "Plan dazu schreib ich gleich mit, ist eh derselbe Kontext" [reasoned-counter] | Skill A v3 schreibt KEINEN Plan. Plan-Write ist ein separater Skill-Invoke nach Spec-Approval (`/superpowers:writing-plans`). (§10.4 #1) |
| "Andere Specs liegen unter `docs/superpowers/specs/`, ich folge der Konvention" [reasoned-counter] | mdai-Specs gehören nach `docs/mdai/specs/` mit Endung `.mdai.md` — bewusst getrennt von Upstream-Specs. (§10.4 #2 + #3) |
| "Eine Frage reicht, der User kennt sein Projekt schon" [reasoned-counter] | One-question-at-a-time ist Disziplin, nicht Ineffizienz. Batched Fragen führen zu Halb-Antworten und Re-Loops. (§10.4 #4) |
| "Ich habe schon eine klare Lösung im Kopf, ein Approach-Vergleich wäre Theater" [reasoned-counter] | Approach-Vergleich (2–3 Alternativen) ist Pflicht VOR Design-Präsentation. Ohne Alternativen ist die Lösung nicht begründet. (§10.4 #5) |
| "Section-by-Section-Approval ist umständlich, ich schicke die ganze Spec auf einmal" [reasoned-counter] | Per-Section-Approval verhindert spätes Veto auf bereits gefestigte Sektionen. Inkrementelle Validierung schlägt Big-Bang-Review. (§10.4 #6) |
| "Self-Review hab ich beim Schreiben implizit gemacht, das spar ich mir" [reasoned-counter] | Self-Review §7 hat 5 explizite Checks (Placeholders / Konsistenz / Scope / Ambiguity / mdai-Direktiven). Implizite Reviews missen mind. einen Check. (§10.4 #7) |
| "`body.mdai.md` ist nur 100 Z, full read ist effizienter als 4 Phase-Reads" [reasoned-counter] | Phase-Isolation hält Context klein (jede Phase <580 Worte). Full read pollutet Context für nachfolgende Steps. Pointer-Anweisung ist MUST. (§10.4 #8) |
| "Plain-Markdown ist lesbarer als ein Spec mit `@call`/`@tree`/`@constraint`-Direktiven" [reasoned-counter] | Hauptziel §1: Specs nutzen markdownai aktiv für Live-Inhalte. Statische Tabellen + Tree-Listings altern sofort. Direktiven liefern stets aktuellen State. (§10.4 #9) |
| "`mode='full'` macht überall Sinn, ich seh dann den ganzen Kontext" [reasoned-counter] | Lean-Context-Defaults aus `mdai/core/lean-context.md`: cross-file scan → `ctx_read_map`/`signatures`; after-search → `ctx_read_lines`. `mode='full'` nur mit `@note visible consumer="human"`-Justification. (§10.4 #9b) |

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
visual designs). Conceptual/text questions stay in the terminal.

Read the upstream guide for HTML-fragment patterns:

@note visible consumer="human"
visual-companion.md (upstream) hat keinen map/signatures-Pfad — full-read ist die einzig sinnvolle Variante.
Version auf 5.1.0 gepinnt (Spec §5.3 Versions-Pin); bei upstream-Bump aktualisieren. Reviewer-Check #10 passt
damit ohne weitere Note.
@end

{{ @call ctx_read(path="~/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/visual-companion.md", mode="full") }}

Start the companion server (persistent mockups under `.superpowers/brainstorm/`):
@call ctx_shell(cmd="~/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/start-server.sh --project-dir \"$PWD\"")

Capture `screen_dir` and `state_dir` from the server-info JSON for subsequent
screen pushes. Ensure `.superpowers/` is in `.gitignore` (Plan P0.1).
@endif

## Spec Self-Review (step 7, MANDATORY, Claude himself)

After the spec source (`.mdai.md`) is written, look at it with fresh eyes:

1. **Placeholder scan:** any "TBD", "TODO", incomplete sections, vague
   requirements? Fix inline.
2. **Internal consistency:** sections contradict each other? Architecture
   matches feature descriptions?
3. **Scope check:** focused enough for a single plan? Or needs decomposition
   into sub-projects?
4. **Ambiguity check:** any requirement interpretable two different ways?
   Pick one, make it explicit.
5. **mdai-Direktiven-Nutzung (Discipline §10.4 #9):** Enthält der Spec-Body
   markdownai-Direktiven für Live-Inhalte wo semantisch sinnvoll? Falls Spec
   rein Plain-Markdown ist: ist das gerechtfertigt mit
   `markdownai_directives_omitted: <reason>` in der Frontmatter? Wenn nein:
   Spec-Body um passende Direktiven ergänzen (z.B. `@tree mdai/` statt
   statischer Verzeichnis-Liste).

Fix issues inline. No re-review loop — fix and move on.

## Spec reviewer dispatch (step 7.5, OPTIONAL, mdai-Augmentation)

**Lazy-load** the reviewer macro just before dispatch:

@import mdai/skills/mdai-brainstorm/spec-reviewer.md

Then dispatch a reviewer subagent with `@call spec_reviewer_prompt(spec_path=<path>)`
as the prompt body. Returns Status (Approved | Needs-Revision | Needs-Clarification)
+ Strengths + Gaps + Concrete patches + Recommendations. Apply issues inline;
surface recommendations.

Trigger: spec touches MCP signatures, Library packs, or render flow. Skip
for pure-prose specs (Self-Review §7 reicht).

## User-Review-Gate (step 8, exact wording, MANDATORY)

After Self-Review (and optional reviewer dispatch), ask the user with this
exact wording:

> "Spec geschrieben und committed nach `<path>`. Bitte review und gib Feedback,
> ob du Änderungen willst, bevor du als nächsten Schritt
> `/superpowers:writing-plans <path>` aufrufst (oder `/mdai-writing-plans`
> sobald dieser Skill existiert)."

Wait for explicit response. If user requests changes → patch inline → re-run
Self-Review §7. Only proceed to write-outputs phase once user explicitly approves.

Collect for the next phase:

- `slug` — kebab-case topic name (e.g. "user-onboarding-flow").
- `design_content` — full design body as Markdown.

## Spec-Body mdai-Direktiven-Konventionen (Pflichtlektüre für Step 6)

Operationalisiert Discipline §10.4 #9. Pflicht beim "Write design doc"-Schritt.

| Use-Case                     | Best Practice                                                    | Anti-Pattern                                            |
| Datum in File-Pfaden         | `{{ @date format='YYYY-MM-DD' }}`                                | hartkodiertes `2026-05-24` im Spec-Body                 |
| Verzeichnis-Listing          | `@tree mdai/ depth=2`                                            | manuell zusammen-getippte Tree-Ausgabe                  |
| File-System-Status (Report)  | `@call file_check(path="...")` (aus `core/file-utils.md`)        | `ls -la` Output kopiert + committed                     |
| Branching auf File-Existenz  | inline `@if file.exists "..."` + `@else` + `@endif` am Call-Site | `@call file_check` (das ist nur Status, kein Flow)      |
| Strukturierte Daten          | `@list <file.yaml> \| @render type="table" columns="..."`        | Plain-Markdown-Tabelle bei >50 Zeilen oder externer SoT |
| Counts / Statistics          | `{{ @count ./src "*.ts" }}` (inline)                             | hartkodierte Zahlen, die altern                         |
| Cross-File-Content           | `@include ./CHANGELOG.md` oder `@include <file> lines=N-M`       | Copy-Paste zwischen Specs                               |
| Machine-Readable Constraints | `@constraint id="..." severity="high"` + body + `@end`           | Prosaische "Wichtig:"-Hinweise                          |
| Project-Context (live)       | `@call ctx_overview(task="...")` oder `@call ctx_tree(...)`      | manuell kopierte Projekt-Beschreibung                   |

**Anti-Pattern: `file_check` ist nicht Branching.** `@call file_check(path="x.md")`
rendert nur Status (`- x.md exists` / `- x.md MISSING`) — kein Control-Flow.
Für Branching IMMER inline am Call-Site:

```markdown
@if file.exists "x.md"
- do this when exists
@else
- do that when missing
@endif
```

**Ausnahme** (per §10.4 #9): Specs für rein algorithmische Themen ohne
File-/Tool-/Daten-Bezug dürfen plain Markdown sein — dann
`markdownai_directives_omitted: <reason>` in Frontmatter.

<!--
  Drift-Tracking: hand-ported from superpowers/5.1.0/.../brainstorming/SKILL.md,
  lines 16-20 (anti-pattern), 22-32 (checklist), 70-104 (process details),
  107-136 (after-the-design: documentation/self-review/user-review-gate/
  implementation-transition), 140-145 (key principles).
-->
@end
```

### Step A2.5 — `@phase write-outputs` schreiben (Spec §5.4, Budget ≤50 Worte)

- [ ] Write-outputs-Phase anhängen:

```markdown
@phase write-outputs

@import mdai/skills/mdai-brainstorm/write-spec.md

@call write_spec(slug={{ slug }}, body={{ design_content }})
@call render_spec(slug={{ slug }}, target={{ render_target | default("none") }})

Default output (one file staged in working tree):

- `docs/mdai/specs/<date>-<slug>-design.mdai.md` (spec source, consumer="ai")

Opt-in render targets (passed via `render_target` from dialog step 6):

- `target="none"` (default) → no render
- `target="chat"` → render inline via `mcp__markdownai__read_file`
- `target="file"` → adds `docs/mdai/specs/rendered/<date>-<slug>.rendered.md`
  via `npx mai render`

Verification:
@call ctx_tree(path="docs/mdai/specs/", depth=1)

Note: commit is left to the user (per CLAUDE.md — never auto-commit).
Note: NO plan file is written here. Plan-write is a separate skill invocation.
@end
```

### Step A2.6 — `@phase handoff` schreiben (Spec §5.5, Budget ≤80 Worte)

- [ ] Handoff-Phase anhängen:

```markdown
@phase handoff

Spec ready for plan-write. Next step (manual, separate skill invocation):

`/superpowers:writing-plans docs/mdai/specs/<date>-<slug>-design.mdai.md`

This skill does NOT write the plan. Plan-write is the responsibility of a
separate writing-plans skill:

- **Now:** `/superpowers:writing-plans <spec-path>` (upstream)
- **Future:** `/mdai-writing-plans <spec-path>` — once Spec §14 Backlog #1 is
  shipped, use that instead (produces `.mdai.md` plan with `@phase` markers,
  compatible with `mdai-execution`).

Verify spec file is in place:
@call ctx_read(path="docs/mdai/specs/<date>-<slug>-design.mdai.md", mode="map")
@end
```

### Step A2.7 — Phase-Budget-Test (§8.4 + §11.2)

- [ ] Worte-Count pro Phase prüfen:

```bash
for phase in pre-context dialog write-outputs handoff; do
  count=$(mcp__markdownai__read_file path=mdai/skills/mdai-brainstorm/body.mdai.md \
            phase=$phase format=ai | wc -w)
  echo "$phase: $count words"
done
```

- [ ] Erwartete Budgets (Spec §11.1):
  - `pre-context: ≤250` Worte
  - `dialog: ≤580` Worte
  - `write-outputs: ≤50` Worte
  - `handoff: ≤80` Worte
- [ ] **Bei Überschreitung:** Sub-Phase-Split (Spec §11.3 Stufe 1) — `dialog` splitten in `dialog-rules`
      (HARD-GATE + Red-Flags + Rationalization-Table) und `dialog-process` (Checklist + Process + Principles +
      Companion + Reviewer). Jede Sub-Phase <400 Worte. Inline-Kürzung (Stufe 2) nur wenn Split nicht möglich.
      Budget-Aufweichung (Stufe 3) braucht Risk-Eintrag in Spec §13.

### Step A2.8 — Drift-Annotation verifizieren

- [ ] `ctx_search pattern="hand-ported from superpowers" path="mdai/skills/mdai-brainstorm/body.mdai.md"` muss
      mind. 4 matches liefern (jeweils ein Verweis auf upstream-Lines: 16-20, 22-32, 70-104, 107-136, 140-145).
      Drift-Annotation enables `mdai-drift-check` (Spec §14 Backlog #2).

@on complete
  body.mdai.md staged with 4 phases, lazy imports, filled red-flags + rationalization-table,
  §5.6 conventions block, drift annotations. Proceeding to a2-5-spec-reviewer-verify.
@end

---

## @phase a2-5-spec-reviewer-verify

**Spec §15 A2.5 ist bereits in der Patch-Session 2026-05-24 erledigt (siehe State-of-Repo). Diese Phase ist
**Verify-Only** — kein Edit.**

**Files:**
- Read: `mdai/skills/mdai-brainstorm/spec-reviewer.md`

### Step A2.5.1 — Verifikation §0 Lean-Context-Discipline @include

- [ ] `ctx_search pattern="@include mdai/core/lean-context\.md" path="mdai/skills/mdai-brainstorm/spec-reviewer.md"`
      muss `1 match` liefern (Smoke-Test §8.6 expectation).

### Step A2.5.2 — Verifikation §4 Calibration-Paragraph

- [ ] `ctx_search pattern="Anti-Pedantry-Bremse" path="mdai/skills/mdai-brainstorm/spec-reviewer.md"` muss
      `1 match` liefern.

### Step A2.5.3 — Verifikation §5 Anti-Pattern-Checks #1–#11

- [ ] `ctx_search pattern="^[0-9]+\. \*\*" path="mdai/skills/mdai-brainstorm/spec-reviewer.md" max_results=20`
      muss mind. 11 nummerierte Bullets liefern (`1. **MCP signatures` bis `11. **Structured data`).

### Step A2.5.4 — Verifikation §6 Recommendations-Sektion

- [ ] `ctx_search pattern="Recommendations \(advisory, do not block approval" path="mdai/skills/mdai-brainstorm/spec-reviewer.md"`
      muss `1 match` liefern.

### Step A2.5.5 — Bei Fail: Stop und Patch nach Appendix A

- [ ] Falls einer der vier obigen Checks fehlschlägt: STOP. Spec-Appendix A (Z 1362-1402) als Patch-Referenz
      lesen und Live-Datei nachziehen. Wechsel zurück in A2.5 nach Patch-Commit.

@on complete
  spec-reviewer.md verified to match Appendix A. Proceeding to a2-6-library-verify.
@end

---

## @phase a2-6-library-verify

**Spec §15 A2.6 ist bereits in der Patch-Session 2026-05-24 erledigt. Diese Phase ist **Verify-Only** — kein
Edit. Dient als Backstop, falls einzelne Files versehentlich revertiert wurden.**

**Files:**
- Read: `mdai/core/lean-context.md`, `mdai/core/ctx-tools.md`, `mdai/core/tool-quick-ref.md`,
  `mdai/changelog.md`, `mdai/MACROS.md`

### Step A2.6.1 — `lean-context.md` existiert + Discipline-Tabelle vorhanden

- [ ] `ctx_shell cmd="test -f mdai/core/lean-context.md && echo OK || echo MISSING"` → `OK`.
- [ ] `ctx_search pattern="Defaults / Exceptions" path="mdai/core/lean-context.md"` → `1 match`.
- [ ] `ctx_search pattern="^---$" path="mdai/core/lean-context.md"` → `0 matches` (kein YAML-Frontmatter — Pflicht
      für `mode: include` per spec-reviewer Anti-Pattern-Check #4).

### Step A2.6.2 — `ctx-tools.md` hat 3 neue Wrapper

- [ ] `ctx_search pattern="@define ctx_read_(lines|map|signatures)" path="mdai/core/ctx-tools.md"` →
      `3 matches`.
- [ ] `ctx_search pattern="ctx_read_lines, ctx_read_map, ctx_read_signatures" path="mdai/core/ctx-tools.md"` →
      `1 match` (Frontmatter exports line).

### Step A2.6.3 — `tool-quick-ref.md` referenziert die 3 Wrapper

- [ ] `ctx_search pattern="ctx_read_(lines|map|signatures)" path="mdai/core/tool-quick-ref.md"` → `3 matches`.

### Step A2.6.4 — `changelog.md` hat drei Additive-Einträge

- [ ] `ctx_search pattern="Additive update 2026-05-24" path="mdai/changelog.md"` → `3 matches` (file-utils,
      MACROS-removal, Lean-Context-Discipline).

### Step A2.6.5 — `mdai/MACROS.md` ist gelöscht

- [ ] `ctx_shell cmd="test -e mdai/MACROS.md && echo EXISTS || echo MISSING"` → `MISSING`.

### Step A2.6.6 — Bei Fail: revert detection + redo

- [ ] Falls ein Check fehlschlägt: `git log --oneline -- mdai/core/<file>` zeigt letzten Touch. Wenn
      Revert-Commit identifizierbar: `git revert <commit>` (User-bestätigt). Sonst: Inhalte aus dieser Plan-Phase
      / Spec §15 A2.6 manuell rekonstruieren.

@on complete
  Library state matches v0.1.0 + 2026-05-24 additives. Proceeding to a3-symlink.
@end

---

## @phase a3-symlink

**Spec §15 A3: project-local Symlink, KEIN globaler Install (§14 Backlog #9 — globaler Install ist deferred).**

**Files:**
- Create: `.claude/skills/` directory
- Create: `.claude/skills/mdai-brainstorm` symlink → `../../mdai/skills/mdai-brainstorm`

### Step A3.1 — Verzeichnis und Symlink anlegen

- [ ] Pre-Check: `ctx_shell cmd="test -d .claude/skills && echo EXISTS || echo MISSING"` zeigt aktuellen State.
- [ ] Verzeichnis anlegen und Symlink setzen (separate Bash-Aufrufe per CLAUDE.md "Keine `&&`-Ketten"):

```bash
mkdir -p .claude/skills
```

```bash
ln -sf ../../mdai/skills/mdai-brainstorm .claude/skills/mdai-brainstorm
```

### Step A3.2 — Symlink-Target verifizieren

- [ ] `ctx_shell cmd="ls -la .claude/skills/mdai-brainstorm"` muss Symlink-Pfeil zeigen
      (`-> ../../mdai/skills/mdai-brainstorm`).
- [ ] `ctx_shell cmd="readlink .claude/skills/mdai-brainstorm"` muss `../../mdai/skills/mdai-brainstorm` zurück
      geben.
- [ ] `ctx_shell cmd="test -f .claude/skills/mdai-brainstorm/SKILL.md && echo OK || echo MISSING"` → `OK`
      (resolveiert über Symlink auf A1).
- [ ] `ctx_shell cmd="test -f .claude/skills/mdai-brainstorm/body.mdai.md && echo OK || echo MISSING"` → `OK`
      (resolveiert über Symlink auf A2).

### Step A3.3 — Skill-Loader-Discovery prüfen (manuell)

- [ ] Claude Code neu starten oder MCP-Reconnect (`/mcp`). Skill `/mdai-brainstorm` muss in der
      Slash-Command-Liste erscheinen. Falls nicht: Skill-Loader-Issue, siehe Spec §13 Risiko "User-Global-Skills
      haben niedrigere Priorität als Plugin-Skills bei Description-Match" — explizites
      `/mdai-brainstorm`-Trigger nutzen.

@on complete
  Project-local symlink live. Skill discoverable as /mdai-brainstorm. Proceeding to a4-smoke-tests.
@end

---

## @phase a4-smoke-tests

**Spec §8 + §15 A4: Smoke-Tests §8.1–§8.6 gegen project-local Symlink. Skill ist nur in diesem Repo aktiv,
beeinflusst andere Projekte nicht.**

**Files (none modified — only read for measurements):**

### Step A4.1 — Trigger-Test §8.1 (Pointer-Compliance, 3 Runs)

- [ ] **Drei frische Sessions** mit `/mdai-brainstorm` triggern. In jedem Run nach SKILL.md-Load muss der erste
      Tool-Call `mcp__markdownai__read_file(phase=pre-context, format=ai)` sein. Kein `read_file` ohne `phase=`.
      Kein `ctx_read` auf `body.mdai.md` mit `mode=full`.
- [ ] Mess-Befehl (Spec §8.1) nach jedem Run:

```bash
SESSION=$(ls -t ~/.claude/projects/-home-tholo-Scripts-lean-ctx/*.jsonl | head -1)
jq -r 'select(.type == "tool_use") |
       select(.input.path | tostring | contains("body.mdai.md")) |
       "\(.name) mode=\(.input.mode // "?") phase=\(.input.phase // "?")"' "$SESSION"
```

- [ ] Pass: nur Zeilen `mcp__markdownai__read_file mode=? phase=<id>`. Fail: irgendwo `mode=full` oder leeres
      `phase=`.
- [ ] Re-Architektur-Trigger (Spec §8.1):
  - **3/3 Pass** → weiter zu A4.2.
  - **2/3 Pass** → manuelle Diagnose (MCP-Disconnect / Cache-Effekt / Setup-Rauschen). Kein Auto-Fallback.
  - **0/3 oder 1/3 Pass** → File-System-Split (5-File-Layout: `phases/<id>.mdai.md`) als **separate Spec**
    aufsetzen, A2 re-do. Plan hier STOP.

### Step A4.1.1 — Gotcha-Capture bei 2/3-Partial-Pass (conditional, Recommendation R3)

- [ ] **Nur falls A4.1 Re-Architektur-Trigger 2/3 Pass ergeben hat:** nach abgeschlossener manueller
      Root-Cause-Diagnose den Befund als `mdai-gotcha` persistieren, damit künftige Sessions die Analyse
      als Seed haben:

```markdown
@call add_gotcha(
  key="mdai-brainstorm-smoke-8.1-partial",
  symptom="Pointer-Compliance 2/3 pass — body.mdai.md full-read in Run <X>",
  mitigation="Root-Cause war <…>. Mitigation: <…>."
)
```

- [ ] Bei 3/3 Pass oder 0/3-1/3 Pass: Step skippen. Bei 3/3 gibt es keinen Befund; bei 0/3-1/3 führt der
      File-System-Split-Pfad (separate Spec) den Gotcha-Capture dort.

### Step A4.2 — Discipline-Fidelity-Test §8.2

- [ ] In einer frischen Session `/mdai-brainstorm` triggern und durch dialog-phase durchspielen
      (clarifying questions → 2-3 approaches → design sections).
- [ ] Verifizieren: kein `Skill(superpowers:brainstorming)`-Invoke im Tool-Log (Claude folgt der hand-ported
      Checklist ohne Upstream-Skill-Load).
- [ ] **v3-Augmentation:** Verifizieren, dass am Ende der Session **kein** Plan-File geschrieben wird (kein
      `write` / `ctx_shell mkdir` / `ctx_edit` auf `docs/mdai/plans/`).
- [ ] Mess-Befehl:

```bash
SESSION=$(ls -t ~/.claude/projects/-home-tholo-Scripts-lean-ctx/*.jsonl | head -1)
jq -r 'select(.type == "tool_use") |
       select((.input.path // "") | tostring | contains("docs/mdai/plans/")) |
       "VIOLATION: \(.name) path=\(.input.path)"' "$SESSION"
```

- [ ] Pass: Output ist leer (keine Violations).

### Step A4.3 — Output-Test §8.3

- [ ] Nach Discipline-Fidelity-Test write-outputs-phase durchspielen.
- [ ] Verifizieren: genau **ein** File staged unter `docs/mdai/specs/<date>-<slug>-design.mdai.md`:

```bash
git status --short docs/mdai/specs/
```

- [ ] Pass: genau eine neue `*.mdai.md`-Zeile. Kein File unter `docs/superpowers/specs/`. **Kein File unter
      `docs/mdai/plans/`** (Plan-Write ist nicht Verantwortung dieses Skills).
- [ ] Bei Opt-in `render_spec(target="file")`: zusätzlich `docs/mdai/specs/rendered/<date>-<slug>.rendered.md`
      separat verifizieren.

### Step A4.4 — Phase-Budget-Test §8.4 (Re-Run von A2.7)

- [ ] Wiederholung des Wort-Counts aus A2.7. Erwartet: alle Phasen unter Budget aus §11.1. Bei Drift seit
      A2.7 (z.B. durch dazwischen liegende Edits): Eskalations-Reihenfolge §11.3.

### Step A4.5 — Library-Import-Smoke-Test §8.5

- [ ] Static-render via mai-CLI:

```bash
cd markdownai && npx mai render ../mdai/skills/mdai-brainstorm/body.mdai.md
```

- [ ] Pass-Kriterien: `exit 0`; alle `@call`s aus Skill-A-Pack (`write_spec`, `render_spec`,
      `spec_reviewer_prompt`) und Core-Wrappers (`ctx_read`, `ctx_shell`, `ctx_tree`, `list_phases`,
      `remember_plan`, `list_gotchas`) sind aufgelöst; keine `unknown directive` Errors; `mode: include` Text
      (hard-rules, tool-quick-ref) erscheint im Output; `mode: import-only` Source-Text erscheint NICHT.
      `@query`-Direktiven liefern leere Strings (mai-CLI blockiert Live-Execution per Spec §13).

### Step A4.5.1 — `@date`-Auflösungs-Test §8.5.1

- [ ] Fixture anlegen + rendern:

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

- [ ] Pass-Kriterien (Spec §8.5.1):
  1. `exit 0`.
  2. Output enthält **kein** literal `{{ @date format='YYYY-MM-DD' }}` (= alles aufgelöst).
  3. Output enthält Datum im Format `YYYY-MM-DD`:

```bash
cd markdownai && npx mai render ../tmp/mdai-date-resolve-test.mdai.md | \
  grep -E 'docs/mdai/specs/[0-9]{4}-[0-9]{2}-[0-9]{2}-smoke-test-design\.mdai\.md'
```

  4. `@query`-Direktiven literal gerendert, aber mit aufgelöstem `@date` und `{{ slug }}` im command-Body.
- [ ] Bei Fail: Fallback-Eskalation Spec §8.5.1 Stufen 2-4 (escaped double quotes → separate `@date label=`
      → `$(date)` shell-rollback). Entscheidung welcher Fallback hängt vom Smoke-Test-Output ab.

### Step A4.5.2 — `@if file.exists`-Conditional-Test §8.5.2

- [ ] Fixture A (Overwrite-Protection):

```bash
rm -f docs/mdai/specs/$(date -u +%Y-%m-%d)-smoke-overwrite-design.mdai.md
cat > /tmp/mdai-overwrite-test.mdai.md <<'EOF'
@markdownai v1.0

@import mdai/skills/mdai-brainstorm/write-spec.md

@call write_spec(slug="smoke-overwrite", body="first body")
@call write_spec(slug="smoke-overwrite", body="second body — should ABORT")
EOF
cd markdownai && npx mai render ../tmp/mdai-overwrite-test.mdai.md
```

- [ ] Pass A (Spec §8.5.2):
  1. Erster `@call` rendert `@query mcp lean-ctx ctx_shell` Body.
  2. Zweiter `@call` rendert `@if`-True-Body (ABORT message) — falls Live-Claude-Session (mai-CLI führt
     `@query` nicht aus, daher static-render eigentlich beide False-Branch → Plumbing-Check primär).
  3. `@if file.exists` parsed sauber, kein `unknown directive`-Error.
- [ ] Fixture B (Existence-Check in render_spec):

```bash
rm -f docs/mdai/specs/$(date -u +%Y-%m-%d)-smoke-render-design.mdai.md
cat > /tmp/mdai-render-missing-test.mdai.md <<'EOF'
@markdownai v1.0

@import mdai/skills/mdai-brainstorm/write-spec.md

@call render_spec(slug="smoke-render", target="chat")
EOF
cd markdownai && npx mai render ../tmp/mdai-render-missing-test.mdai.md
```

- [ ] Pass B (Spec §8.5.2):
  1. Output enthält `ERROR: Cannot render — spec file does not exist at
     docs/mdai/specs/<date>-smoke-render-design.mdai.md`.
  2. KEIN `@query mcp markdownai read_file` Direktiv im Output (`@if`-False-Branch wurde gewählt).
  3. `@if file.exists` + `@else` parsen sauber.

### Step A4.6 — Lean-Context-Discipline-Test §8.6

- [ ] Vier `ctx_search`-Anchors prüfen (Spec §8.6):

```bash
# Wrappers existieren in ctx-tools.md
mcp__lean-ctx__ctx_search pattern="@define ctx_read_(lines|map|signatures)" path="mdai/core/ctx-tools.md"
# Expected: 3 matches.

# Spec-reviewer included lean-context.md
mcp__lean-ctx__ctx_search pattern="^@include mdai/core/lean-context\.md" path="mdai/skills/mdai-brainstorm/spec-reviewer.md"
# Expected: 1 match.

# tool-quick-ref erwähnt die 3 neuen Wrapper
mcp__lean-ctx__ctx_search pattern="ctx_read_(lines|map|signatures)" path="mdai/core/tool-quick-ref.md"
# Expected: 3 matches.

# mode="full" Whitelist-Check
mcp__lean-ctx__ctx_search pattern='mode="full"' path="mdai/"
# Expected: 1 match (spec-reviewer.md §1 — Reviewer-Target-Read). Jeder weitere match muss
# eine @note visible consumer="human" Justification haben.
```

- [ ] Pass: alle 4 Suchen liefern expected counts.

### Step A4.7 — Iteration bei Fail

- [ ] Bei §8.1 Fail (0/3 oder 1/3) → File-System-Split als **separate Spec**. Plan hier STOP.
- [ ] Bei §8.2/8.3 Fail → Source unter `mdai/skills/mdai-brainstorm/` anpassen (bevorzugt
      `body.mdai.md`-Constraints schärfen), re-run.
- [ ] Bei §8.4 Fail → Sub-Phase-Split nach §11.3.
- [ ] Bei §8.5/8.5.1/8.5.2/8.6 Fail → Library-Asset patchen (zurück nach A2.6).

@on complete
  Smoke-tests green. Skill is production-ready. Proceeding to a4-5-green-verification.
@end

---

## @phase a4-5-green-verification

**Recommendation R2: Green-Verification-Artefakt analog zur Library-Konvention
(`docs/mdai/green-verification/library/v0.1.0-*.md`) anlegen, damit Smoke-Test-Outputs als versionierter
Anker für künftige Drift-Detection und Plan-Iterationen erhalten bleiben.**

**Files:**
- Create: `docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md`

### Step A4-5.1 — Verzeichnis anlegen

- [ ] `ctx_shell cmd="test -d docs/mdai/green-verification/skill && echo OK || echo MISSING"`. Bei `MISSING`:

```bash
mkdir -p docs/mdai/green-verification/skill
```

### Step A4-5.2 — Smoke-Test-Summary schreiben

- [ ] Datei anlegen mit konsolidierter Übersicht der 6 Smoke-Tests aus A4. Template:

```markdown
---
target: mdai/skills/mdai-brainstorm
version: v0.1.0
date: 2026-05-24
spec: docs/mdai/specs/2026-05-24-mdai-brainstorm-design.mdai.md
plan: docs/mdai/plans/2026-05-24-mdai-brainstorm-implementation.md
---

# Green Verification — mdai-brainstorm v0.1.0

## Summary

| Test  | Status | Notes                                                                                      |
| §8.1  | <pass|fail> | <3/3 Pass | 2/3 Pass + Diagnose | 0/3-1/3 Pass + File-System-Split-Spec>                |
| §8.2  | <pass|fail> | Discipline-Fidelity + no-plan-written check                                              |
| §8.3  | <pass|fail> | Output: genau ein File unter `docs/mdai/specs/`                                          |
| §8.4  | <pass|fail> | Phase-Budgets pre-context/dialog/write-outputs/handoff                                   |
| §8.5  | <pass|fail> | Static-Render via `npx mai render`                                                       |
| §8.5.1| <pass|fail> | `@date` Inline-Auflösung in `@query command="..."`-Strings                              |
| §8.5.2| <pass|fail> | `@if file.exists` Conditional-Parse (Plumbing-Check)                                    |
| §8.6  | <pass|fail> | Lean-Context-Discipline: 4 ctx_search-Anchors                                            |

## Phase-Budget (§8.4)

| Phase         | Budget | Actual | Status |
| pre-context   | ≤250   | <N>    | <pass|fail> |
| dialog        | ≤580   | <N>    | <pass|fail> |
| write-outputs | ≤50    | <N>    | <pass|fail> |
| handoff       | ≤80    | <N>    | <pass|fail> |

## Diagnose-Notes

<Pro Fail-Fall: Root-Cause, Mitigation, ggf. Verweis auf neue Spec / Iteration. Bei 100% Pass: leer
lassen oder "All tests green at v0.1.0".>

## Re-Verification-Trigger

Re-run dieser Verifikation bei:

- Bump der mdai-Library (>0.1.0).
- Patch in `mdai/skills/mdai-brainstorm/` (SKILL.md, body.mdai.md, write-spec.md, spec-reviewer.md).
- Upstream-Bump von `superpowers:brainstorming` (Versions-Pin 5.1.0 in body.mdai.md dialog phase).
- mai-CLI / `mcp__markdownai` Server-Update mit Render-Verhaltens-Änderungen.
```

- [ ] Pro tatsächlich gelaufenem A4-Step die `<pass|fail>` + `<N>`-Platzhalter mit echten Werten füllen.
- [ ] Bei Diagnose-relevanten Befunden (z.B. §8.5.1 musste auf Fallback-Stufe 2 wechseln) detaillierte Note
      unter "Diagnose-Notes" einfügen.

### Step A4-5.3 — Verifikation Artefakt

- [ ] `ctx_shell cmd="test -f docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md && echo OK || echo MISSING"`
      → `OK`.
- [ ] `ctx_search pattern="^## Summary" path="docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md"`
      → `1 match`.
- [ ] Inhaltliche Plausibilität: alle 8 Test-Zeilen + 4 Phase-Budget-Zeilen befüllt (kein `<pass|fail>` /
      `<N>`-Platzhalter mehr).

@on complete
  Smoke-Test-Summary committed. Drift-Anchor für künftige Iterationen vorhanden. Proceeding to
  a5-mdai-plans-uninstall.
@end

---

## @phase a5-mdai-plans-uninstall

**Spec §15 A5: alten `mdai-plans`-Skill deinstallieren, damit `/mdai-plans` nicht mehr triggert (Skill wurde
durch v3-Scope-Cut abgelöst).**

**Files:**
- Delete (conditional): `~/.claude/skills/mdai-plans/` or `.claude/skills/mdai-plans`

### Step A5.1 — Existenz prüfen

- [ ] `ctx_shell cmd="test -d ~/.claude/skills/mdai-plans && echo GLOBAL || echo NO-GLOBAL"`.
- [ ] `ctx_shell cmd="test -d .claude/skills/mdai-plans && echo LOCAL || echo NO-LOCAL"`.

### Step A5.2 — Deinstallation (conditional)

- [ ] Falls `GLOBAL`:

```bash
rm -rf ~/.claude/skills/mdai-plans/
```

- [ ] Falls `LOCAL`:

```bash
rm -rf .claude/skills/mdai-plans
```

- [ ] Falls `NO-GLOBAL` und `NO-LOCAL`: no-op. Skill existiert nicht.

### Step A5.3 — Trigger-Verifikation

- [ ] In frischer Claude-Code-Session prüfen: `/mdai-plans` darf **nicht** mehr triggern.
      Slash-Command-Liste darf keinen `mdai-plans` Eintrag enthalten.

@on complete
  mdai-plans deinstalled (or confirmed absent). Proceeding to a5-5-readme.
@end

---

## @phase a5-5-readme

**Spec §15 A5.5: Workflow-Übergang dokumentieren (klein, optional aber empfohlen). Übergang ist primär in
`body.mdai.md` handoff-Phase hartkodiert; README ist user-facing Sekundär-Doku.**

**Files:**
- Create: `mdai/skills/mdai-brainstorm/README.md` (~10 Z, **nicht** SKILL.md — SKILL.md bleibt schlank per §4)

### Step A5.5.1 — README schreiben

- [ ] Datei anlegen:

```markdown
# mdai-brainstorm — Skill Notes

Triggered as `/mdai-brainstorm`. Writes a versioned design spec under
`docs/mdai/specs/<date>-<slug>-design.mdai.md` with `consumer="ai"` default.

## Next step after spec approval

This skill does **not** write plans. After `/mdai-brainstorm` produces a spec
and the user approves it via the User-Review-Gate, invoke the plan-writing
skill manually:

```
/superpowers:writing-plans docs/mdai/specs/<date>-<slug>-design.mdai.md
```

Once `mdai-writing-plans` (Spec §14 Backlog #1) ships, switch to:

```
/mdai-writing-plans docs/mdai/specs/<date>-<slug>-design.mdai.md
```

## Source layout

- `SKILL.md` — pointer (~15 lines, do not put workflow content here).
- `body.mdai.md` — live workflow (4 phases, loaded phase-by-phase via MCP).
- `write-spec.md` — Skill-A pack (`write_spec`, `render_spec`).
- `spec-reviewer.md` — Skill-A pack (`spec_reviewer_prompt`).
```

### Step A5.5.2 — Verifikation

- [ ] `ctx_shell cmd="wc -l mdai/skills/mdai-brainstorm/README.md"` muss zwischen 10 und 30 Zeilen liegen
      (Spec §15 A5.5 sagt "~10 Z" als Orientierungspunkt, Frontmatter + Headers brauchen Spielraum).
- [ ] `ctx_search pattern="/superpowers:writing-plans" path="mdai/skills/mdai-brainstorm/README.md"` →
      `1 match`.
- [ ] `ctx_search pattern="/mdai-writing-plans" path="mdai/skills/mdai-brainstorm/README.md"` → `1 match`.

@on complete
  README.md staged. All implementation phases complete. Proceeding to verification-summary.
@end

---

## @phase verification-summary

**Final-State-Check vor User-Sign-Off. Kein neuer Output — nur konsolidierte Verifikation der gesamten
Implementation.**

### Step V.1 — File-Inventar

- [ ] `ctx_tree path="mdai/skills/mdai-brainstorm/" depth=1` zeigt:
  - `SKILL.md`
  - `body.mdai.md`
  - `README.md`
  - `spec-reviewer.md`
  - `write-spec.md`

### Step V.2 — Symlink-Inventar

- [ ] `ctx_tree path=".claude/skills/" depth=1` zeigt `mdai-brainstorm` als Symlink.

### Step V.3 — Spec-Erfolgskriterien (§1)

- [ ] Kriterium 1: `body.mdai.md` ruft `@call mdai_bootstrap()` als erste Zeile in `pre-context` auf →
      verifiziert in A2.3.
- [ ] Kriterium 2: Hand-ported Disziplin in dialog-phase (kein Skill-Invoke) → verifiziert in A4.2.
- [ ] Kriterium 3: Keine eigenen Macros in Skill A → verifiziert via `ctx_search pattern="^@define"
      path="mdai/skills/mdai-brainstorm/body.mdai.md"` → `0 matches`.
- [ ] Kriterium 4: Spec wird als `*.mdai.md` mit `consumer="ai"` committet → write-spec.md Frontmatter zeigt
      das (verifiziert in A2.5/A4.3).
- [ ] Kriterium 5: handoff-Phase zeigt auf `/superpowers:writing-plans` → verifiziert in A2.6.
- [ ] Kriterium 6: Datei-Operationen nutzen Library-Wrapper → verifiziert via
      `ctx_search pattern="ctx_shell\\(cmd=\"(cat|ls|grep|find)" path="mdai/skills/mdai-brainstorm/body.mdai.md"`
      → `0 matches` (kein Anti-Pattern).
- [ ] Kriterium 7: Produzierte Specs nutzen markdownai-Direktiven aktiv → Self-Review §7.5 + Reviewer
      Check #9 enforce.

### Step V.4 — Spec-Annahmen (§16)

- [ ] Annahme 1 (MCP read_file rendert für Human korrekt) — verifiziert in A4.3.
- [ ] Annahme 2 (Phase-isolated read_file returniert self-contained Phase) — verifiziert in A4.1 (3/3 Pass).
- [ ] Annahme 3 (Claude respektiert Pointer "kein full read") — verifiziert in A4.1.
- [ ] Annahme 4 (hand-ported Disziplin ohne Skill-Invoke, kein Plan-Write) — verifiziert in A4.2.
- [ ] Annahme 5 (`ctx_graph` Index existiert) — verifiziert in P0.2.
- [ ] Annahme 6 (lazy `@import` lädt Pack-`@define`s korrekt, 2 Packs) — verifiziert in A4.5.

### Step V.5 — Commit-Strategie (User-Entscheidung)

Per `CLAUDE.md`: **never auto-commit**. Plan endet ohne `git add`/`git commit`. User entscheidet, ob er die
Implementation als ein Commit oder phase-für-phase Commits führen will.

**Empfohlene Commit-Reihenfolge:**

1. P0 → `chore(mdai): gitignore .superpowers/, build ctx_graph index`
2. A1 + A2 + A5.5 → `feat(mdai-brainstorm): add SKILL.md, body.mdai.md, README.md`
3. A3 → `chore(mdai-brainstorm): symlink under .claude/skills`
4. A4.5 (Green-Verification-Artefakt) → `docs(mdai-brainstorm): green verification v0.1.0 smoke summary`
5. A5 → `chore(mdai): uninstall obsolete mdai-plans skill` (falls vorhanden war)

A2.5/A2.6 brauchen keine eigenen Commits (Verify-Only, kein neuer Content). A4.1.1
(Gotcha-Capture) ist ein `ctx_knowledge`-Aufruf und schreibt kein versioniertes File — kein Commit nötig.

**Pre-commit-Workflow** (CLAUDE.md): `mcp__jetbrains__reformat_file` auf jede geänderte Datei vor `git add`.
Einzeln aufrufen, **keine `&&`-Ketten**.

@on complete
  Implementation verified end-to-end. Skill mdai-brainstorm is production-ready.
  Hand off to user for commit + first /mdai-brainstorm session.
@end

---

## Backlog-Awareness (nicht Teil dieses Plans)

Folgende Items aus Spec §14 sind **explizit deferred** und werden durch separate `/mdai-brainstorm`-Sessions
adressiert — nicht hier:

1. `mdai-writing-plans`-Skill (hohe Priorität, separate Spec)
2. `mdai-drift-check` (Hash-Vergleich gegen Upstream `superpowers:brainstorming`)
3. `mdai-execution` / `mdai-memory` Migration auf `mdai/skills/`-Layout
4. Upstream-PR an markdownai für `respondTool()`-Fix
5. Plugin-Packaging (alle mdai-Skills bündeln)
6. Spec-Human-Render-Wrapper-Template
7. `mdai_bootstrap`-Cache (Library v0.2)
8. Globaler `install.sh` (nach 3 stabilen Sessions)

Wenn ein Schritt im Plan auf eines dieser Items zeigt, immer als Backlog-Pointer behandeln — niemals inline
implementieren.
