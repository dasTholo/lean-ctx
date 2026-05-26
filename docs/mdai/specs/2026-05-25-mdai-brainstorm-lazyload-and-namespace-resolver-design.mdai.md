---
title: mdai-brainstorm Lazy-Load Refactor + markdownai mdai/-Namespace-Resolver
slug: mdai-brainstorm-lazyload-and-namespace-resolver
target_skill: mdai/skills/mdai-brainstorm/
target_library: markdownai/packages/{engine,mcp,core}/
target_foundation: mdai/core/
skill_version_target: v0.1.x (kein Bump)
markdownai_version_target: minor-bump (additives Resolver-Feature)
date: 2026-05-25
status: superseded
superseded_by: docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md
superseded_reason: markdownai v1.0.0 release ersetzt Engine-Resolver-Patch durch security.json source_root/data_root config + ${MDAI_LIBRARY_ROOT}-Globs. Lazy-Load-Anteil (L1/L2/L3) wandert in die neue Spec mit MCP-first (call_macro statt @include für L2) und Wave-3–5-Direktiven-Adoption.
authors: kaitholo, claude
predecessor_spec: docs/mdai/specs/2026-05-24-mdai-brainstorm-design.mdai.md
predecessor_green_verification: docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md
next_step: see superseded_by
---

@markdownai v1.0

# Spec — mdai-brainstorm Lazy-Load Refactor + markdownai `mdai/`-Namespace-Resolver

@constraint id="hard-gate" severity="high"
Diese Spec ist ein DESIGN-DOCUMENT. Sie schreibt KEINEN Plan und KEINEN Code.
Plan-Erstellung erfolgt durch `/superpowers:writing-plans` als nächster
Skill-Invocation. Implementation erfolgt erst nach Plan-Approval.
@end

@constraint id="scope-coupling" severity="high"
Diese Spec koppelt zwei Themen bewusst: (a) skill-internen Lazy-Load-Refactor
(L1+L2+L3) und (b) markdownai-Engine-Erweiterung um `mdai/`-Namespace-Resolver.
Die Kopplung ist notwendig, weil L1+L2+L3 neue `@include mdai/core/...`-Pfade
einführen, die ohne den Resolver in der Install-Variante brechen würden.
Trennung in zwei Specs erzwingt einen brüchigen Zwischenzustand.
@end

## 1. Goal & Scope

### 1.1 Goal

`mdai-brainstorm` als Skill leaner machen (echte Lazy-Load-Wirkung, nicht nur
Source-Reorg) und gleichzeitig den Distribution-Pfad-Bug uniform fixen
(markdownai bekommt `mdai/`-Namespace-Resolver). Foundation-Layer um
`lean-context.md` erweitern. Spec-Reviewer entlang upstream-Form re-shapen.
Ein kohärenter v0.1.x-Patch ohne Skill-Version-Bump.

### 1.2 In-Scope

@prompt role="reference"
**Skill-Refactor (`mdai/skills/mdai-brainstorm/`):**

- L1: Direktiven-Konventions-Tabelle aus `body.mdai.md` dialog-process
  extrahieren nach `spec-directive-conventions.md`. Wandert in die
  `write-outputs`-Phase (real lazy: nicht geladen während Dialog).
- L2: Spec-Self-Review (5 Checks + neuer Check #6 "lean-context spot") +
  Reviewer-Dispatch + User-Review-Gate aus dialog-process extrahieren nach
  `spec-self-review.md`. Wandert in die `handoff`-Phase (real lazy: nicht
  geladen während Dialog/Write).
- L3: Process-Details + Key-Principles aus dialog-process extrahieren nach
  `process-principles.md`. `@include` an gleicher Position in dialog-process
  (token-neutral, aber Drift-Anker für hand-portierte upstream-Slices).
- `body.mdai.md` pre-context: `@include mdai/core/lean-context.md` neu;
  `@constraint id="tool-selection"` auf Pointer-Form geschrumpft (delegiert an
  tool-quick-ref + lean-context); `@call detect_mdai_root()` neu nach
  `@call mdai_bootstrap()`.
- `body.mdai.md` Process-Checklist Schritt 6-9: explizite Phase-Transition-Hints
  (`read_file(phase="write-outputs")` / `read_file(phase="handoff")`).
- `spec-reviewer.md`: komplett re-shape zur Lean-Form (~45 Z statt 168 Z) entlang
  upstream `superpowers/brainstorming/spec-document-reviewer-prompt.md`.
  `@prompt role="reference"` für What-to-Check, `@prompt role="calibration"`
  für Calibration-Block.
- `write-spec.md`: neue Macro `write_review_report(spec_path, status, strengths,
  issues, recommendations)` für Reviewer-Output-Writing.

**Foundation-Erweiterung (`mdai/core/`):**

- Neue Datei `lean-context-audit.md` (`mode: import-only`): exportiert
  `lean_context_audit(spec_path)` als 6-Anchor-Sweep (war §5 Check #10 im alten
  Reviewer). Reusable für künftige mdai-Reviewer.
- Neue Datei `library-spec-audit.md` (`mode: import-only`): exportiert
  `library_spec_audit(spec_path)` mit den 7 Library-Spec-Checks (alt #1, #2, #3,
  #4, #5, #7, #8). Conditional vom Reviewer aufgerufen.
- `startup-check.md`: neue Macro `detect_mdai_root()` emittiert
  `[mdai-bootstrap] MDAI_LIBRARY_ROOT=<path>` analog zum
  MDAI_HAS_JETBRAINS-Pattern.

**markdownai-Engine + MCP + CLI:**

- `engine/src/engine-include.ts`: neue Fn `resolveMdaiRoot(ctx)` mit
  Reihenfolge env-var → walk-up → null. `executeInclude` und `executeImport`
  bekommen `mdai/`-Prefix-Detection. Bei Resolver-Hit: Path wird strip+resolve
  via `mdaiRoot`. Bei Miss: klare Error-Message.
- `mcp/src/tools/read_file.ts` (+ Geschwister `call_macro`, `list_phases`,
  `next_phase`, `resolve_phase`, `get_constraints`): kein API-Bruch nötig —
  `args.env` propagiert MDAI_LIBRARY_ROOT schon korrekt. Audit-Sweep zur
  Sicherheit.
- `core/src/cli.ts`: explizite Propagation von `process.env.MDAI_LIBRARY_ROOT`
  in `envFiles`.
- Neue Tests: `engine/__tests__/resolve-mdai-root.test.ts`,
  `mcp/__tests__/server.test.ts` (Erweiterung),
  `core/__tests__/cli-mdai-namespace.test.ts`.
  @end

### 1.3 Out-of-Scope

@prompt role="reference"

- **ctx_session-Adoption über Visual-Gate (S2) hinaus.** S3 (Skill-Handoff-Path),
  S1 (Cross-Session-Resume) bleiben Backlog.
- **Skill-Version-Bump.** `mdai-brainstorm` bleibt v0.1.x. lib_version-Felder in
  betroffenen `mode: import-only` Files (z.B. `write-spec.md`,
  `spec-reviewer.md`) bumpen zu 0.1.1 nur falls Patch-Tracking gewünscht.
- **markdownai-Release-Mechanik.** Engine + MCP-Code-Change ist in-scope;
  Release-Notes und Versionierungs-Strategie ist markdownai-Maintainer-Job
  (vermutlich minor-bump, additives Feature).
- **Migration anderer mdai-Skills.** Resolver greift automatisch wenn andere
  Skills `mdai/...`-Pfade nutzen. Audit-Sweep nach Release ist Backlog.
- **Approach C Self-Contained-Bundle.** Library wird ohnehin co-installiert.
- **6-Phasen-Variante** (separater `post-write` zwischen write-outputs und
  handoff). 5-Phasen-Lösung mit erweitertem handoff reicht.
- **Promotion `spec-self-review.md` zu `mdai/core/`.** YAGNI bis 2. Konsument
  auftaucht.
- **`process-principles.md` weglassen** (V1 in Brainstorm verworfen).
- **`spec-directive-conventions.md` ins `mdai/core/` promoten** (V2 in Brainstorm
  verworfen, weil dieser Skill der einzige Spec-Produzent ist und L1 skill-local
  bleibt).
  @end

### 1.4 Success Criteria

@prompt role="reference"

1. `body.mdai.md` dialog-process-Phase Source schrumpft von 986 W → ≤ 600 W (mit
   Reserve, Ziel ~510 W).
2. pre-context-Phase lädt `mdai/core/lean-context.md` zusätzlich erfolgreich
   ohne ENOENT-Warning.
3. `mcp__markdownai__read_file` mit Pfad-Argument `.claude/skills/mdai-brainstorm/body.mdai.md`
   (Symlink-Variante, Claude-Code-default) liefert **0 ENOENT-Warnings** für alle
   `@include mdai/...`-Direktiven.
4. Smoke-Test §8.5 nicht mehr `tooling-discrepancy`, sondern `pass` für beide
   Pfade (Symlink + kanonisch + temp-dir mit env-var).
5. Smoke-Test §8.4 dialog-process im Budget (kein `pass-with-concern`).
6. markdownai mai-CLI `npx mai render` liefert für `mdai/...`-Includes
   konsistente Resolution (gleiche Logik wie MCP, sonst Verifikations-Gap).
7. spec-reviewer-Prompt-Body bei dispatch reduziert sich von ~1533 W auf ~340 W
   für Design-Doc-Pfad (Library-Spec-Pfad mit `@call library_spec_audit`:
   zusätzlich +700 W, also ~1040 W — immer noch unter dem ursprünglichen 1533 W).
   @end

## 2. File-Layout-Diff

### 2.1 Aktueller Stand

@tree mdai/skills/mdai-brainstorm depth=1
@tree mdai/core depth=1

### 2.2 Skill-Files (`mdai/skills/mdai-brainstorm/`)

**Neu (3 Files):**

| File                            | Mode      | Inhalt                                                                                                          | ~Wörter |
|---------------------------------|-----------|-----------------------------------------------------------------------------------------------------------------|--------:|
| `spec-directive-conventions.md` | `include` | Direktiven-Konventions-Tabelle (9 Use-Cases × 3 Spalten) + `file_check`-Anti-Pattern + Plain-Markdown-Exception |    ~360 |
| `spec-self-review.md`           | `include` | 5 existierende Checks + neuer Check #6 (lean-context spot) + Reviewer-Dispatch-Block + User-Review-Gate-Wording |    ~290 |
| `process-principles.md`         | `include` | hand-portierte Process-Details (5 Bullets) + Key-Principles (6 Bullets) + Drift-Tracking-Kommentar              |    ~250 |

Alle drei: keine YAML-Frontmatter (für `mode: include` safe), kein `@define`,
kein eigener `@markdownai v1.0` Header.

**Modifiziert:**

- `body.mdai.md`: pre-context (+ `@call detect_mdai_root()`,
    + `@include mdai/core/lean-context.md`, ~ `tool-selection`-Constraint
      geschrumpft); dialog-process (~ Process-Checklist Schritt 6-9 mit
      Phase-Transitions, ~ Z 137-160 → `@include ./process-principles.md`,
      − Z 188-220 + Z 222-238 + Z 240-282 alle raus); write-outputs
      (+ `@include ./spec-directive-conventions.md` vor write_spec); handoff
      (+ `@include ./spec-self-review.md` + User-Gate-Wording aus dialog-process).
- `write-spec.md`: + Macro `write_review_report(spec_path, status, strengths,
  issues, recommendations)`.
- `spec-reviewer.md`: komplett re-shape — siehe §3.5.
- `SKILL.md`: optional Pointer-Hinweis "Pass `mdai/skills/mdai-brainstorm/body.mdai.md`
  als path-Argument an mcp__markdownai__read_file; der Symlink unter
  `.claude/skills/` funktioniert mit Namespace-Resolver auch, ist aber kein
  Pflicht-Pfad".

**Unangetastet:**

- `visual-companion-offer.md`, `README.md`.

### 2.3 Foundation-Files (`mdai/core/`)

**Neu (2 Files):**

| File                    | Mode          | Inhalt                                                          | ~Wörter |
|-------------------------|---------------|-----------------------------------------------------------------|--------:|
| `lean-context-audit.md` | `import-only` | `@define lean_context_audit(spec_path)` — 6-Anchor-Sweep        |    ~200 |
| `library-spec-audit.md` | `import-only` | `@define library_spec_audit(spec_path)` — 7 Library-Spec-Checks |    ~700 |

**Modifiziert:**

- `startup-check.md`: + `@define detect_mdai_root()` analog zu
  `detect_tooling`. Optional aufgerufen aus `mdai_bootstrap()`. Persistierung
  via ctx_session-Cache (Pattern wie MDAI_HAS_JETBRAINS).

**Unangetastet:**

- `hard-rules.md`, `tool-quick-ref.md`, `lean-context.md`, `ctx-tools.md`,
  `file-utils.md`.

### 2.4 markdownai (`markdownai/packages/`)

**Modifiziert (Engine-Kern):**

- `engine/src/engine-include.ts`: + Helper `resolveMdaiRoot(ctx)` mit
  env-var → walk-up → null Reihenfolge; ~ `executeInclude` und `executeImport`
  detect `mdai/`-Prefix und routen.
- `core/src/cli.ts`: ~ explicit MDAI_LIBRARY_ROOT-Propagation aus `process.env`.

**Modifiziert (MCP, evtl. nur Audit):**

- `mcp/src/tools/read_file.ts`, `call_macro.ts`, `list_phases.ts`,
  `next_phase.ts`, `resolve_phase.ts`, `get_constraints.ts`: Audit-Sweep ob
  `args.env` überall konsistent durchgereicht wird. Falls ja: kein Patch.

**Neu (Tests):**

| File                                             | Coverage                                                                |
|--------------------------------------------------|-------------------------------------------------------------------------|
| `engine/src/__tests__/resolve-mdai-root.test.ts` | env-var-hit, walk-up-hit (parent-1, parent-N), jailRoot-respect, no-hit |
| `mcp/src/__tests__/server.test.ts` (Erweiterung) | read_file mit MDAI_LIBRARY_ROOT, mit walk-up, mit invalid env           |
| `core/src/__tests__/cli-mdai-namespace.test.ts`  | mai-CLI render gegen Fixture mit `@include mdai/...` (env + walk-up)    |

**Unangetastet:**

- `parser` (Syntax `@include mdai/...` ist schon valid), `renderer`, `vscode`.

### 2.5 Docs (`docs/mdai/`)

**Erzeugt durch diesen Brainstorm + Folge-Workflow:**

- `docs/mdai/specs/2026-05-25-mdai-brainstorm-lazyload-and-namespace-resolver-design.mdai.md`
  — diese Datei.
- `docs/mdai/plans/2026-05-25-mdai-brainstorm-lazyload-and-namespace-resolver-implementation.md`
  — kommt aus `/superpowers:writing-plans`.
- `docs/mdai/reviews/2026-05-25-mdai-brainstorm-lazyload-and-namespace-resolver-design-review.md`
  — wenn `spec_reviewer_prompt` dispatched wird.
- `docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.1-smoke.md` — Re-Run
  der Smoke-Suite nach Implementierung, analog v0.1.0.

## 3. Lazy-Load-Mechanik

### 3.1 Architektur-Entscheidung: was bedeutet "lazy" wirklich?

`@include` rendert die Ziel-Datei zum Render-Zeitpunkt **inline** in die
aktuelle Phase. Drift-Tracking-Wert (Inhalt in einer Datei statt vier), aber
**keine Agent-Context-Token-Ersparnis** — bei Phase-Load ist der Include mit drin.

Echte Token-Ersparnis nur durch **Phase-Verschiebung**: L1 nach `write-outputs`,
L2 nach `handoff`. L3 bleibt `@include`-only (Drift-Tracking-Wert, hand-portierte
upstream-Slices).

### 3.2 Phase-Struktur (5 Phasen, handoff erweitert)

```
Vorher (5 Phasen):              Nachher (5 Phasen):
  pre-context                     pre-context              (+ lean-context include + detect_mdai_root)
  dialog-rules                    dialog-rules             (unverändert)
  dialog-process                  dialog-process           (shrunk: L3 inline, L1+L2+UserGate raus)
  write-outputs                   write-outputs            (+ L1 conventions)
  handoff                         handoff                  (+ L2 self-review + reviewer-dispatch + user-gate)
```

Kein API-Bruch (Phase-Namen identisch). 5-Phasen-Variante mit erweitertem
handoff bevorzugt gegenüber 6-Phasen (separater `post-write`) wegen
API-Kontinuität. handoff wächst von 69 W auf ~430 W — noch in vernünftigem
Rahmen.

### 3.3 dialog-process: Shrink-Diff

**Raus (in andere Files / Phasen):**

- Z 188-220 (`## Spec Self-Review` + `## Spec reviewer dispatch`) → komplett
  raus, wandert nach `handoff` als `@include ./spec-self-review.md`.
- Z 222-238 (`## User-Review-Gate`) → komplett raus, wandert nach `handoff`
  inline (Wording verbatim).
- Z 240-282 (`## Spec body mdai directive conventions`) → komplett raus,
  wandert nach `write-outputs` als `@include ./spec-directive-conventions.md`.

**Bleibt (verschlankt):**

- Process Checklist (Z 117-133) leicht modifiziert: Schritt 6 sagt "switch to
  write-outputs phase", Schritte 7-9 sagen "in handoff phase".
- Visual-Companion-Block (Z 162-184) bleibt.
- `## The Process — Details` + `## Key Principles` (Z 137-160) → ersetzt durch
  `@include ./process-principles.md` (L3, token-neutral).
- Neuer Übergangshinweis am Ende: `Next: read_file(phase="write-outputs")`.

**Wort-Bilanz:** dialog-process Source 990 W → ~510 W (−48 %, §1
Success-Criterion #1 erfüllt mit 90 W Reserve).

### 3.4 write-outputs: Grow-Diff

**Neu (vor `@call write_spec`):**

```markdown
@phase write-outputs

@import mdai/skills/mdai-brainstorm/write-spec.md
@include ./spec-directive-conventions.md

Apply conventions while finalizing design_content, then call write_spec below.

@call write_spec(slug={{ slug }}, body={{ design_content }})
@call render_spec(slug={{ slug }}, target={{ render_target | default("none") }})

# ... existing verification ...

Next: read_file(phase="handoff")
@end
```

**Wort-Bilanz:** write-outputs Source 95 W → ~470 W. Wirkt nur wenn schreiben
tatsächlich passiert (Abbruch im Dialog: 0 Tokens für die Conventions-Tabelle).

### 3.5 handoff: Grow-Diff

**Neu (komplette Phase):**

```markdown
@phase handoff

@include ./spec-self-review.md

# ... User-Review-Gate-Wording aus dialog-process Z 222-238 verbatim hier ...

Spec ready for plan-write. Next step (manual, separate skill invocation):
`/superpowers:writing-plans docs/mdai/specs/<date>-<slug>-design.mdai.md`

# ... existing handoff message ...

@end
```

`spec-self-review.md` Inhalt enthält die 5 Checks + Check #6 + den
Reviewer-Dispatch-Block. User-Gate-Wording bleibt verbatim außerhalb des
Includes (weil es kein Self-Review-Inhalt ist, sondern Brainstorm-Workflow-Gate).

**Wort-Bilanz:** handoff Source 69 W → ~430 W.

### 3.6 pre-context: Kleine Erweiterung

```diff
@phase pre-context

@call mdai_bootstrap()
+ @call detect_mdai_root()

@include mdai/core/hard-rules.md
@include mdai/core/tool-quick-ref.md
+ @include mdai/core/lean-context.md

# ... existing project-context section ...

@constraint id="tool-selection" severity="high"
- Read file → `@call ctx_read(path, mode)` (not `ctx_shell cmd="cat ..."`).
- List directory → `@call ctx_tree(path, depth)` (not `ls`/`find`).
- Pattern search → `@call ctx_search(pattern, path)` (not `grep`/`rg`).
- File edit without read → `@call ctx_edit(path, old, new)`.
- Read plan phase → `@call read_phase(plan, phase_id)`.
- `@call ctx_shell` only as a last resort (git ops, shell scripts, tools without a wrapper).
+ Task → Macro: see `mdai/core/tool-quick-ref.md` (included above).
+ Read-Mode + Defaults / Exceptions: see `mdai/core/lean-context.md` (included above).
+ `@call ctx_shell` only as a last resort (git ops, shell scripts, tools without a wrapper).
@end
```

**Wort-Bilanz:** pre-context Source 165 W → ~170 W
(Constraint-Schrumpfung kompensiert lean-context-Include-Zeile + detect_mdai_root-Call).

### 3.7 Process-Checklist-Update (kritisch für UX)

Schritte 6-9 müssen explizit Phase-Übergänge benennen, sonst weiß der Agent
nicht dass er `read_file(phase=...)` für die nächsten Schritte aufrufen muss:

```markdown
1. Explore project context (already done in pre-context phase).
2. Offer visual companion (if visual) — own message (see Visual-Companion section).
3. Ask clarifying questions — one at a time.
4. Propose 2–3 approaches with trade-offs.
5. Present design sections, get approval after each.
6. Switch to `write-outputs` phase: `read_file(phase="write-outputs")`.
    - Apply spec-directive-conventions while finalizing design_content.
    - `@call write_spec(slug, design_content)`.
7. Switch to `handoff` phase: `read_file(phase="handoff")`.
    - Spec Self-Review (5 + 1 checks).
    - 7.5 OPTIONAL: dispatch `spec_reviewer_prompt(spec_path)`.
8. User-Review-Gate (in same handoff phase, exact wording).
9. Transition: invoke writing-plans skill — THIS SKILL DOES NOT WRITE THE PLAN.
```

### 3.8 Lean Spec-Reviewer (~45 Z statt 168 Z)

Komplette Re-Shape entlang upstream `superpowers/brainstorming/spec-document-reviewer-prompt.md`:

```markdown
---
lib_version: 0.1.1
mdai-pack:
  mode: import-only
  exports: [spec_reviewer_prompt]
---

@markdownai v1.0

@import mdai/skills/mdai-brainstorm/write-spec.md
@import mdai/core/lean-context-audit.md
@import mdai/core/library-spec-audit.md

@define spec_reviewer_prompt(spec_path)
You are a spec doc reviewer. Verify {{ spec_path }} is complete and ready
for planning.

## 1. Read the spec

`@call ctx_read(path={{ spec_path }}, mode="full")` — the one allowed
`mode="full"` (the review IS the read).

## 2. What to Check

@prompt role="reference"
| Category | What to Look For |
|---|---|
| Completeness | TODOs, placeholders, "TBD", incomplete sections |
| Consistency | Internal contradictions, conflicting requirements |
| Clarity | Requirements ambiguous enough to cause someone to build the wrong thing |
| Scope | Focused enough for a single plan — not covering multiple independent subsystems |
| YAGNI | Unrequested features, over-engineering |
@end

## 3. Calibration

@prompt role="calibration"
**Only flag issues that would cause real problems during impl planning.**
A missing section, a contradiction, or a requirement so ambiguous it could be
interpreted two different ways — those are issues. Minor wording improvements,
stylistic preferences, and "sections less detailed than others" are not.
Approve unless there are serious gaps that would lead to a flawed plan.
@end

## 4. mdai-Augmentations (universal)

a. **Language convention** (CLAUDE.md): spec body German, code/snippets English.
b. **mdai directives in body** (Discipline §10.4 #9): ≥3 distinct directive
types in body, OR frontmatter has `markdownai_directives_omitted: <reason>`.
c. **Lean-context audit:** `@call lean_context_audit(spec_path={{ spec_path }})`.

## 5. Heavy library-spec checks (conditional)

If spec touches MCP signatures / library packs / `mode: include` frontmatter /
render-flow tests: `@call library_spec_audit(spec_path={{ spec_path }})`.
Skip for plain design-docs.

## 6. Output

`@call write_review_report(spec_path={{ spec_path }}, status=..., strengths=..., issues=..., recommendations=...)`.
(Macro defined in `mdai/skills/mdai-brainstorm/write-spec.md`.)
@end
```

Note: §Tools-Sektion ist absichtlich entfernt — irrelevant für Design-Doc-Review.

### 3.9 write_review_report Macro (in `write-spec.md`)

```markdown
@define write_review_report(spec_path, status, strengths, issues, recommendations)
@query mcp lean-ctx ctx_shell cmd="
mkdir -p docs/mdai/reviews &&
REPORT_PATH=docs/mdai/reviews/$(basename {{ spec_path }} .mdai.md)-review.md &&
cat > \"$REPORT_PATH\" <<'EOF'
---
target: {{ spec_path }}
reviewer: spec_reviewer_prompt
date: {{ @date format='YYYY-MM-DD' }}
---

# Review — $(basename {{ spec_path }} .mdai.md)

## Status: {{ status }}

## Strengths

{{ strengths }}

## Issues

{{ issues }}

## Recommendations

{{ recommendations }}
EOF
echo \"wrote $REPORT_PATH\""
@end
```

### 3.10 Self-Review Check #6 (in `spec-self-review.md`)

Neuer Check zusätzlich zu den 5 existierenden:

```markdown
6. **Lean-context defaults spot check** (light variant of Reviewer-Check #10):
   Schnell-Scan auf häufigste lean-context-Violations:
    - `@call ctx_search(pattern="mode=\"full\"", path="<spec_path>")` → flag jeden
      match ohne adjacent `@note visible consumer="human"` Block.
    - `@call ctx_search(pattern="raw=true", path="<spec_path>")` → flag ohne
      `@note visible`.
    - `@call ctx_search(pattern="fresh=true", path="<spec_path>")` → flag wenn
      nicht IMMEDIATELY nach einem write/edit zum gleichen Pfad.
      Fix inline. Heavy-Variante (6-Anchor-Sweep) bleibt im optional dispatched
      Reviewer (`@call lean_context_audit`).
```

## 4. markdownai `mdai/`-Namespace-Resolver

### 4.1 Engine: `resolveMdaiRoot(ctx)` Helper

Neue Function in `markdownai/packages/engine/src/engine-include.ts`:

```typescript
import { existsSync } from 'node:fs'
import { dirname, resolve } from 'node:path'

const MDAI_MARKER = 'core/lean-context.md'

export function resolveMdaiRoot(ctx: EngineContext): string | null {
  // (1) Explicit env-var override
  const envRoot = ctx.envFiles['MDAI_LIBRARY_ROOT']
  if (envRoot && existsSync(resolve(envRoot, MDAI_MARKER))) {
    return resolve(envRoot)
  }

  // (2) Walk-up from docDir looking for an `mdai/` dir with the marker.
  //     Stops at jailRoot (security) or filesystem root.
  const stop = ctx.security.jailRoot ?? '/'
  let dir = ctx.docDir
  while (dir.startsWith(stop) && dir !== '/') {
    const candidate = resolve(dir, 'mdai')
    if (existsSync(resolve(candidate, MDAI_MARKER))) return candidate
    const parent = dirname(dir)
    if (parent === dir) break  // root
    dir = parent
  }

  return null
}
```

Marker-Wahl `core/lean-context.md` weil neu (mit v0.1.1 hinzugefügt, stabiles
Anchor). Alternative `core/hard-rules.md` ginge auch.

Walk-Up-Grenze: `ctx.security.jailRoot` respektiert (Filesystem-Confinement
bleibt intakt). Sicher gegen Path-Traversal via parent-dirs.

### 4.2 Engine: `@include` / `@import` Namespace-Detection

Patch in `executeInclude` (engine-include.ts:74) und `executeImport` (Z 38):

```typescript
let full: string
if (node.path.startsWith('mdai/')) {
  ctx.mdaiRootCache ??= resolveMdaiRoot(ctx)
  const mdaiRoot = ctx.mdaiRootCache
  if (mdaiRoot === null) {
    ctx.warnings.push(
      `@include: cannot resolve mdai/-namespace path "${node.path}": ` +
      `MDAI_LIBRARY_ROOT not set and no mdai/ library found in ancestors of ${ctx.docDir}. ` +
      `Place the mdai/ library at any ancestor or set MDAI_LIBRARY_ROOT env var.`
    )
    return ''  // (for include; for import: just return)
  }
  full = resolve(mdaiRoot, node.path.slice(5))  // strip "mdai/" prefix
} else {
  full = resolve(ctx.docDir, node.path)
}
```

Eigenschaft: `node.path.slice(5)` strippt `mdai/`-Prefix, weil `mdaiRoot` schon
auf den `mdai/`-Dir zeigt (sonst doppelte `mdai/`-Komponente).

Security: `checkFilePath` Check bleibt davor, gegen `jailRoot`. Wenn `mdaiRoot`
außerhalb `jailRoot` liegt: blocked.

Idempotenz: Cache der `resolveMdaiRoot`-Resolution per Render-Context
(`ctx.mdaiRootCache ??=`). Trivial-Optimierung, vermeidet repeated walks pro
Render.

### 4.3 MCP-Server: Env-Var-Passthrough

Status quo (`read_file.ts:79`): `args.env` wird schon an `execute()`
durchgereicht via `execOpts.ctx.envFiles`. **Keine Code-Change nötig**, nur
Konvention dokumentieren.

Caller-Disziplin: Claude Code's Skill-Loader (oder Test-Harness) kann
`args.env = { MDAI_LIBRARY_ROOT: "/abs/path/to/mdai" }` mitgeben. Wenn nicht:
Walk-Up greift.

Audit-Sweep für Symmetrie: `@call ctx_search(pattern="envFiles", path="markdownai/packages/mcp/src/tools")`
— wenn überall konsistent `args.env ?? {}` steht: kein Patch. Sonst
symmetrisieren.

### 4.4 mai-CLI: Parity

`markdownai/packages/core/src/cli.ts` ruft Engine direkt. Muss dieselbe
`resolveMdaiRoot`-Logik triggern — das passiert automatisch wenn Engine die Fn
exportiert und include/import sie nutzen. **Keine separate CLI-Logik nötig**,
nur Env-Propagation:

```typescript
// cli.ts (im render-command-Handler):
const ctx = {
  envFiles: {
    ...loadEnvFiles(),
    MDAI_LIBRARY_ROOT: process.env.MDAI_LIBRARY_ROOT ?? '',
  },
  // ...
}
```

User-Disziplin: `MDAI_LIBRARY_ROOT=/path/to/mdai npx mai render <skill>.md` ODER
auf Walk-Up vertrauen.

### 4.5 Error-Messages + Diagnose

**ENOENT bei `@include mdai/...` ohne Resolver-Hit:**

```
@include: cannot resolve mdai/-namespace path "mdai/core/hard-rules.md":
MDAI_LIBRARY_ROOT not set and no mdai/ library found in ancestors of
/home/user/.claude/skills/mdai-brainstorm/. Place the mdai/ library at any
ancestor or set MDAI_LIBRARY_ROOT env var.
```

**Diagnose-Direktive für `startup-check.md`:** der neue `detect_mdai_root()`
emittiert dieselbe Info früh, damit der User sofort sieht ob Resolver greift:

```
[mdai-bootstrap] MDAI_LIBRARY_ROOT=/home/user/mdai (via env)
# oder
[mdai-bootstrap] MDAI_LIBRARY_ROOT=/home/user/repo/mdai (via walk-up from docDir)
# oder
[mdai-bootstrap WARN] MDAI_LIBRARY_ROOT not detected — engine will fall back to walk-up at @include time
```

### 4.6 Tests

**`engine/__tests__/resolve-mdai-root.test.ts`:**

- env-var-hit (set MDAI_LIBRARY_ROOT to valid dir, verify resolution)
- env-var-set-but-invalid (set to dir without marker, verify walk-up fallback)
- walk-up-hit at parent-1
- walk-up-hit at parent-N
- walk-up jailRoot-respect (marker exists OUTSIDE jailRoot, expect no-hit)
- no-hit (no env, no marker anywhere): null + warning

**`mcp/__tests__/server.test.ts` (Erweiterung):**

- read_file mit `env.MDAI_LIBRARY_ROOT` setzt: include funktioniert
- read_file ohne env: walk-up findet repo-local `mdai/`
- read_file mit ungültigem MDAI_LIBRARY_ROOT: walk-up fallback ODER warning

**`core/__tests__/cli-mdai-namespace.test.ts`:**

- `mai render fixture.md` mit `@include mdai/core/hard-rules.md` aus temp-dir
  → walk-up findet repo-local `mdai/`
- mit env `MDAI_LIBRARY_ROOT` → env-var wins

Fixtures: temp-dir-Struktur mit `mdai/core/lean-context.md` als Marker,
`body.md` mit `@include mdai/core/hard-rules.md`.

### 4.7 Backwards Compatibility

**Was bricht NICHT:**

- Bestehende `@include relative/path.md` ohne `mdai/`-Prefix: unverändert
  (alter docDir-Resolve-Pfad).
- Bestehende `@import relative/path.md` ohne Prefix: unverändert.
- Files die `@include mdai/...` nutzen AUS einem Verzeichnis WO der alte
  docDir-resolve zufällig funktionierte: funktionieren weiter, weil neuer
  Code-Pfad das gleiche Ergebnis liefert (wenn Walk-Up `mdai/` im
  aktuellen-cwd-ancestor findet).

**Was bricht POTENZIELL:**

- Files mit `@include mdai-irgendwas/...` (Prefix `mdai-`, nicht `mdai/`): keine.
  Safe.
- Files die einen lokalen `mdai/`-Subdir HABEN und darauf zeigen wollen (NICHT
  die Library): bricht. Workaround: `./mdai/...` statt `mdai/...`. Selten-Case,
  in Doku erwähnt.

### 4.8 Out-of-Scope (Engine-Feature-Backlog)

- Andere Namespaces (`superpowers/...`, `claude/...`): gleiche Mechanik möglich,
  YAGNI.
- Mehrere mdai-Libraries (local-override + global-fallback chain): one-shot
  Resolution reicht aktuell.
- Caching über Render-Boundaries hinaus (ctx_session-Persistenz des
  `mdaiRoot`-Werts): YAGNI — Walk-Up ist billig.

## 5. Verification

### 5.1 Re-Run der bestehenden §8.x-Suite

Alle Tests aus `docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md`
werden gegen den refactorierten Skill erneut ausgeführt.

@if file.exists "docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md"
v0.1.0-Smoke-Artefakt vorhanden — Re-Run ist 1:1 Vergleich.
@else
v0.1.0-Smoke-Artefakt FEHLT — Plan muss Re-Run gegen aktuelle Skill-Realität neu definieren.
@endif

**Erwartete Status-Änderungen:**

| Test                         | v0.1.0              | v0.1.1 erwartet                | Begründung                                          |
|------------------------------|---------------------|--------------------------------|-----------------------------------------------------|
| §8.1 Pointer-Compliance      | deferred            | deferred                       | User-Action                                         |
| §8.2 Discipline-Fidelity     | deferred            | deferred                       | User-Action                                         |
| §8.3 Output-Test             | deferred            | deferred                       | User-Action                                         |
| §8.4 Phase-Budget            | pass-with-concern   | **pass**                       | dialog-process Source ~510 W << 600 W Target        |
| §8.5 Library-Import          | tooling-discrepancy | **pass**                       | Namespace-Resolver löst `@include mdai/...` uniform |
| §8.5.1 `@date`-Resolve       | pass-structural     | pass-structural                | Unverändert                                         |
| §8.5.2 False-Branch          | fail                | **fail (unchanged) — Backlog** | markdownai-Upstream-Edge-Case                       |
| §8.6 Lean-Context-Discipline | pass                | **pass** + 2 neue Anchors      | erweiterte Discipline durch lean-context-audit      |

### 5.2 §8.7 (NEU): Namespace-Resolver Tests

**§8.7.1 — Env-var-Pfad:**

```bash
MDAI_LIBRARY_ROOT=$PWD/mdai npx mai render mdai/skills/mdai-brainstorm/body.mdai.md
```

Pass: 0 ENOENT-Warnings für die 3 `@include mdai/core/...`-Direktiven.

**§8.7.2 — Walk-Up-Pfad (kein env-var):**

```bash
unset MDAI_LIBRARY_ROOT
npx mai render mdai/skills/mdai-brainstorm/body.mdai.md
```

Pass: 0 ENOENT-Warnings, walk-up findet `<repo-root>/mdai/`.

**§8.7.3 — Symlink-Pfad (Production via Claude Code):**

```
mcp__markdownai__read_file(
  path="/abs/path/.claude/skills/mdai-brainstorm/body.mdai.md",
  phase="pre-context"
)
```

Pass: 0 ENOENT-Warnings (Resolver walks up from symlink-target via Marker; oder
findet via env wenn gesetzt).

**§8.7.4 — Negativ-Test:**
Aus temp-dir OHNE mdai/-Bibliothek im Ancestry und ohne env-var:

```bash
cd /tmp/empty && unset MDAI_LIBRARY_ROOT
npx mai render <synthetic-fixture-with-@include-mdai>
```

Pass: klare ENOENT-Warning mit Error-Message aus §4.5.

### 5.3 §8.8 (NEU): Phase-Transition-Workflow

Verifiziert dass die neue Phase-Reihenfolge funktioniert. Manueller Walk durch
eine fresh-Session-Brainstorm:

1. pre-context: `grep "Lean-Context-Discipline"` im Render → match.
2. dialog-rules: Word-Count ≈ 706 W, unverändert.
3. dialog-process: `grep "read_file(phase=\"write-outputs\")"` → match.
4. write-outputs: `grep "directive conventions"` AND `@call write_spec` → beide
   match.
5. handoff: `grep "Spec Self-Review"` AND `Please review and give feedback` →
   beide match.

Pass: alle 5 Phasen rendern ohne ENOENT/unresolved-Warnings, alle `grep`-Marker
matchen.

### 5.4 §8.9 (NEU): Lean-Reviewer Dispatch-Render

```bash
npx mai render --macro spec_reviewer_prompt \
  --args spec_path=docs/mdai/specs/2026-05-25-mdai-brainstorm-lazyload-and-namespace-resolver-design.mdai.md
```

Pass:

- Output enthält die 6 Sektionen (Read / What to Check / Calibration /
  mdai-Augmentations / Heavy library-spec checks / Output)
- `@prompt role="reference"` + `@prompt role="calibration"` Blöcke im
  consumer=ai-Output sichtbar (consumer=human gestrippt — separater Sub-Test)
- Word-Count ≤ 400 W (Lean-Threshold; bei Library-Trigger via
  `@call library_spec_audit` ≤ 1100 W)
- Keine `writing-plans`-Referenz im Reviewer-Body

### 5.5 §8.10 (NEU): Audit-Macro Composability

**§8.10.1 — lean-context-audit:**

Fixture mit künstlichem Spec der `mode="full"` ohne `@note` enthält:

```bash
npx mai render <fixture-spec.mdai.md>
```

Expected: 1 flagged match für `mode="full"`.

**§8.10.2 — library-spec-audit:**

Fixture mit MCP-Signatur-Lock im Spec-Body:

```bash
npx mai render <library-spec-fixture.mdai.md>
```

Expected: Check #1 flag MCP-Signatur-Verifikation needed.

Pass: beide Audit-Macros laden via `@import mdai/core/...` (Namespace-Resolver
greift) und produzieren strukturierten Output.

### 5.6 §8.11 (NEU): write_review_report Integration

Dispatch Reviewer mit `@call spec_reviewer_prompt(spec_path=<test-spec>)`,
Subagent ruft `@call write_review_report(spec_path=<test-spec>, status="Approved", ...)`.

Pass:

- Datei `docs/mdai/reviews/<test-spec-basename>-review.md` existiert
- Frontmatter enthält korrekt `target`, `reviewer`, `date`
- Body enthält Status-Sektion, Strengths, Issues, Recommendations
- `@date format='YYYY-MM-DD'` resolved zur tatsächlichen Datum-String

### 5.7 Erwartetes Phase-Budget (Source-Wörter)

| Phase          | v0.1.0 Actual | v0.1.1 Budget |          v0.1.1 Erwartet |
|----------------|--------------:|--------------:|-------------------------:|
| pre-context    |           162 |          ≤250 |                     ~170 |
| dialog-rules   |           703 |          ≤750 |                      703 |
| dialog-process |           986 |      **≤600** | **~510** (mit L3-inline) |
| write-outputs  |            92 |          ≤500 |                     ~470 |
| handoff        |            66 |          ≤500 |                     ~430 |
| **Σ Source**   |      **2009** |             — |                    ~2283 |

Source-Total wächst leicht (+274 W) durch Phase-Transitions + Foundation-Includes.
Aber: dialog-process-Render schrumpft drastisch — und das ist die Phase, die der
Agent typischerweise länger sieht.

### 5.8 Re-Verification-Trigger

Re-Run der Smoke-Suite bei:

- Patch in `mdai/skills/mdai-brainstorm/` (alle 9 Files: SKILL.md, body.mdai.md,
  spec-reviewer.md, spec-self-review.md, spec-directive-conventions.md,
  process-principles.md, write-spec.md, visual-companion-offer.md, README.md)
- Patch in `mdai/core/` (lean-context.md, lean-context-audit.md,
  library-spec-audit.md, startup-check.md, tool-quick-ref.md, hard-rules.md,
  ctx-tools.md)
- markdownai-Engine-Bump mit Resolver-Verhaltens-Änderungen
- Upstream-Bump von `superpowers:brainstorming` (Versions-Pin in
  `visual-companion-offer.md`)
- §8.1/§8.2/§8.3 nachgeholt (User-Action)

### 5.9 Outstanding (für künftige Iteration)

1. §8.1/§8.2/§8.3 User-driven Smoke-Tests (deferred aus v0.1.0)
2. §8.5.2 False-Branch markdownai-Upstream-Bug-Report
3. Migration anderer mdai-Skills auf Namespace-Resolver (Audit-Sweep nach Release)
4. mdai-drift-check Skill für hand-portierte upstream-Slices

### 5.10 Green-Verification-Artefakt

Erzeugt nach v0.1.1-Implementation:
`docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.1-smoke.md`

Format analog v0.1.0:

- Summary-Tabelle (§8.1-§8.11 mit Status, Notes)
- Phase-Budget-Tabelle (Vorher/Nachher/Budget/Δ)
- Diagnose-Notes pro non-pass-Test
- Re-Verification-Trigger-Liste
- Outstanding-Liste

## 6. Risks, Migration, Backlog

### 6.1 Risiken

@prompt role="reference"
| Risk | Severity | Mitigation |
|---|---|---|
| Walk-Up findet falsche `mdai/`-Library (mehrere Forks im Filesystem-Ancestry) | mittel | env-var `MDAI_LIBRARY_ROOT`
als expliziter Override dokumentiert. `detect_mdai_root` Output zeigt resolved path beim Bootstrap. |
| markdownai-Engine-Bump nötig → koordinierter Release | hoch | Engine-Change additiv (`@include mdai/...` ohne
Resolver = ENOENT wie bisher = kein Regress). Konsumenten migrieren stufenweise. |
| Phase-Split bricht UX wenn Agent Phase-Transitions ignoriert | mittel | Process-Checklist Schritt 6-9 enthält
explizite `read_file(phase=...)`-Aufrufe. dialog-process render endet mit deutlicher Next-Zeile. |
| Lean-Reviewer wirft Library-Spec-Checks raus die bei meta-Specs nötig waren | niedrig | `@call library_spec_audit`
bleibt aufrufbar; Reviewer-Trigger-Sektion erklärt wann. Self-Review-Check #6 fängt häufigste Verletzungen ab. |
| mai-CLI vs MCP-Server Behavior-Divergence beim Resolver | mittel | §8.7 Smoke-Tests prüfen beide Pfade. Engine-Logik
ist single-source-of-truth, beide Consumer rufen dieselbe Fn. |
| `@prompt`-Blöcke in spec-reviewer.md werden bei consumer=human gestrippt | niedrig | Reviewer-Dispatch via Skill geht
IMMER über consumer=ai. Doc-String weist explizit darauf hin. |
| Bestehende Skills mit lokalem `mdai/`-Subdir brechen wegen Namespace-Capture | sehr niedrig | Workaround `./mdai/...`
dokumentiert. Audit-Sweep nach Release. |
| `library-spec-audit.md` enthält 7 Checks — Engine-Bump kann Check #3 brechen | niedrig | Drift-Anker im File-Kopf:
`# Last verified against markdownai vX.Y.Z`. |
@end

### 6.2 Migration

**In diesem Repo (`lean-ctx`):**

- `body.mdai.md` ist Drop-in-Replacement — Skill-User-Workflow unverändert von
  außen.
- Existierende Specs unter `docs/mdai/specs/` brauchen keine Anpassung.
- Bestehender Reviewer-Output `docs/mdai/reviews/...` hat dieselbe Struktur
  (`write_review_report` erzeugt kompatibles Format).

**Andere mdai-Skills (Backlog-Awareness):**

- Wenn ein anderer Skill `@include mdai/...` nutzt: profitiert automatisch vom
  Resolver, keine Code-Change.
- Wenn ein anderer Skill schwer-Library-Reviewer braucht:
  `@import mdai/core/library-spec-audit.md` + `@call library_spec_audit(spec_path)`.
- Promotion-Trigger für `spec-self-review.md` zu `mdai/core/`: sobald 2. Skill
  spec-Self-Review-Pattern braucht.

**Install-Variante (`.claude/skills/mdai-brainstorm/` ohne `mdai/`-Tree daneben):**

- User installiert `mdai/`-Library separat.
- `MDAI_LIBRARY_ROOT=/path/to/mdai` env-var ODER `mdai/` in Ancestor-Dir
  (Walk-Up).
- `detect_mdai_root` emittiert klare Diagnose.

**Falls Walk-Up zu langsam wird (sehr tiefe Filesystem-Hierarchien):**
Future-Optimierung `MDAI_LIBRARY_ROOT_CACHE` in Engine-Context (cached
per-Process). Aktuell out-of-scope.

### 6.3 Backlog (out-of-scope, getrackt)

**Aus dieser Spec abgeleitet:**

1. **Migration anderer mdai-Skills** auf Namespace-Resolver — Audit-Sweep nach
   v0.1.1-Release.
2. **Promotion `spec-self-review.md`** zu `mdai/core/` wenn 2. Skill auftaucht
   (YAGNI bis dahin).
3. **§8.5.2 markdownai-Upstream-Bug-Report** für False-Branch in `mode: import-only`
   Pack-Render. Nicht durch unsere Spec blockierend, aber bekannt.

**Aus Vorgänger-Spec übernommen (bleibt offen):**

4. **mdai-writing-plans Skill** (Vorgänger-Spec §14 #1) — eigene Spec.
5. **mdai-drift-check Skill** (Vorgänger-Spec §14 #2) — eigene Spec, würde
   hand-portierte upstream-Slices automatisch auditen.

**Aus Brainstorm-Diskussion abgeleitet (nicht adoptiert):**

6. **ctx_session S3 Skill-Handoff** — eigene kleine Spec, wenn
   `mdai-writing-plans` shippt.
7. **ctx_session S1 Cross-Session-Resume** — größere Spec, wenn
   Resume-Use-Cases häufig werden.
8. **Andere Namespaces im Resolver** (`superpowers/`, `claude/`, etc.) — YAGNI.
9. **Multi-Source-Resolver** (local-override + global-fallback chain) — YAGNI.
10. **mai-CLI `--cwd`-Flag** für Render aus Sub-Dirs — Walk-Up macht's überflüssig.

**Out-of-scope erwogen, bewusst nicht adoptiert:**

11. **Approach C Self-Contained-Bundle** für Skill-Distribution — Library wird
    co-installiert.
12. **Phase-Split in 6 Phasen** (separater `post-write` zwischen write-outputs
    und handoff) — 5-Phasen-Lösung reicht.
13. **L1+L2 ins `mdai/core/` promoten** — Skill produziert die Specs, also
    Skill-Authoring-Files bleiben skill-local.

## 7. Annahmen

@prompt role="reference"

1. markdownai-Repo (`markdownai/packages/`) ist Teil dieses Workspaces und kann
   im selben Plan-Zyklus mit-implementiert werden. Falls nicht: Spec muss in
   zwei Phasen umgesetzt werden (Engine-PR zuerst, dann Skill-Refactor wenn
   markdownai released).
2. `mdai/core/lean-context.md` existiert bereits als stabile Datei (verifiziert
   in dieser Spec — Marker-File-Wahl für Walk-Up).
3. Claude Code's Skill-Loader übergibt `args.env` korrekt an `mcp__markdownai__read_file`
   wenn env-vars gesetzt sind. Falls nicht: Walk-Up greift trotzdem.
4. Der Symlink `.claude/skills/mdai-brainstorm → ../../mdai/skills/mdai-brainstorm`
   ist Dev-Convenience und wird in Install-Varianten durch direkte Datei-Kopien
   ersetzt.
5. mdai-Library wird in Install-Varianten parallel zum Skill-Verzeichnis
   bereitgestellt (entweder im Projekt-Repo oder global). Skill ist ohne
   Library nicht funktional — das ist intendiert.
   @end

## 8. Open Questions

@prompt role="reference"

1. Soll `detect_mdai_root` aus `mdai_bootstrap()` automatisch aufgerufen
   werden oder optional? — Empfehlung: automatisch, aber mit Cache (analog
   detect_tooling).
2. Soll der Resolver bei mehreren Walk-Up-Hits die NÄCHSTE oder die ENTFERNTESTE
   wählen? — Empfehlung: nächste (häufigster Use-Case: project-local Lib hat
   Vorrang vor global-installed Lib).
3. Soll der Walk-Up-Marker fest `core/lean-context.md` sein oder konfigurierbar?
   — Empfehlung: fest, einfach, dokumentiert. Konfigurierbarkeit ist YAGNI.
4. Soll `lib_version` in den modifizierten Pack-Files (`write-spec.md`,
   `spec-reviewer.md`) zu 0.1.1 bumpen? — Empfehlung: ja, Patch-Tracking. Skill
   selbst bleibt v0.1.x.
   @end

---

**Spec-Ende.** Nächster Schritt:
`/superpowers:writing-plans docs/mdai/specs/2026-05-25-mdai-brainstorm-lazyload-and-namespace-resolver-design.mdai.md`
(oder `/mdai-writing-plans <path>` wenn dieser Skill mal existiert).
