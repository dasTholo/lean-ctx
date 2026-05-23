---
id: mdai-macro-library
status: design
created: 2026-05-24
spec_version: 1
supersedes: docs/mdai/design-skill-integration.md §7a.1 (Macro-Pack)
---

@markdownai v1.0 consumer="ai"

# mdai-macro-library — Design (v0.1.0)

Status: Brainstorming abgeschlossen, wartet auf Spec-Review. Liefert die Macro-Library, auf der die drei mdai-Skills
(`mdai-brainstorm` = Skill A, `mdai-execution` = Skill B, `mdai-memory` = Skill C) ihre Tool-Calls und ihre
Skill-eigenen Composite-Macros aufbauen. Diese Spec definiert v0.1.0; spätere Versionen erweitern additive (siehe §15
Backlog).

## 1. Zielsetzung

Eine einzelne, versionierte Macro-Library bündelt drei Verantwortungen, die heute über drei ad-hoc Files unter
`docs/mdai/macros/` zerstreut sind und in den Skill-Bodies dupliziert werden:

1. **Cross-skill Tool-Wrapper** — `ctx_read`, `ctx_search`, `ctx_tree`, `ctx_shell`, `ctx_edit`,
   `read_phase`, `list_phases`, `get_constraints`, `remember_plan`, `recall_plan`, `add_gotcha`,
   `list_gotchas`. Diese MCP-Aufrufe werden von allen drei mdai-Skills sowie jedem generierten Plan benötigt.
2. **Variable, projekt-spezifische Macros** — `cargo_nextest`, `cargo_clippy`, `cargo_fmt` für Rust-Projekte,
   `reformat_file`, `step_reformat_commit` für JetBrains-MCP-Setups, `find_symbol` etc. für Serena. Werden
   opt-in pro Projekt geladen, gesteuert durch `ctx_session`-Flags die ein `mdai_bootstrap`-Macro setzt.
3. **Skill-eigene Composite-Macros** — pro Skill ein eigener Pack: für Skill A z.B. `write_spec`,
   `render_spec`, `write_mdai_plan`, `plan_phase`, `plan_frontmatter`, `plan_step`, `spec_reviewer_prompt`.
   Heute inline im Skill-Body, künftig im Pack-Folder.

**Erfolgskriterien:**

1. Library liegt im Repo unter `mdai/` mit klarer Schichtung: `core/` (always), `lang/` und `tooling/` (opt-in),
   `skills/<skill-name>/` (pro Skill-Pack).
2. Jeder mdai-Skill ruft in seiner `pre-context`-Phase exakt einmal `@call mdai_bootstrap()`. Das Macro prüft
   MCP-Liveness (lean-ctx + markdownai pflicht; jetbrains + serena optional), detektiert den Projekt-Typ
   (rust / python / node / unknown) und cached alle Flags in `ctx_session`.
3. Library-Wirkung wird durch Batch RED-GREEN-Pass pro Release verifiziert: 3 Subagents ohne Library
   (RED-Baseline) und 3 mit Library (GREEN-Discipline), Reports in `docs/mdai/red-baseline/library/` bzw.
   `docs/mdai/green-verification/library/`. Acceptance ist qualitativ (User-Entscheidung pro Macro), nicht
   harte Token-Schwelle.
4. Die Konsequenzen für die Skill-A-Spec (`docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md`) sind
   in dieser Spec §10 vollständig aufgelistet. Skill-A-Implementierung darf erst starten, nachdem die
   Skill-A-Patch-Session diese Punkte abgearbeitet hat (Bootstrap-Order A: Library zuerst).

## 2. Empirische Grundlage

Konkrete, beobachtbare Duplikation im aktuellen Stand `docs/mdai/macros/`:

- `hard-rules.md` Z. 6: prosaisch "Vor `git add`: `mcp__jetbrains__reformat_file` auf jede geänderte Datei."
- `step-reformat-commit.md`: dasselbe als `@define stepReformatCommit(file)`-Macro.

Dieselbe Anweisung steht zweimal — einmal als Lese-Regel, einmal als ausführbare Abstraktion. Beide Quellen
können auseinanderdriften. Library-Refactor löst das: `hard-rules.md` verweist textuell auf das Macro;
`tooling/jetbrains.md` enthält die Definition.

Skill-Body-Duplikation aus Skill-A-Spec §6.1 (heute inline): `@define writeSpec`, `@define renderSpec`,
`@define writeMdaiPlan`, `@define planFrontmatter`, `@define planPhase`, `@define planStep`, `@define
specReviewerPrompt`. Sieben Macros, alle in einem einzigen Skill-Body. Skill B + C wollen `read_phase` und
`remember_plan` ebenfalls — ohne Library werden sie kopiert. Library-Refactor: alle skill-eigenen Composites
wandern in `mdai/skills/<skill>/`, alle cross-skill Tool-Wrapper in `mdai/core/`.

Skizze-Quellen aus `docs/mdai/design-skill-integration.md`:

- §5 (lines:139-204): `mdai-execution` braucht `list_phases`, `get_constraints`, `read_file(phase=)`,
  `ctx_read`, `ctx_tree`, `ctx_search`, `ctx_shell` — alles wiederholend pro Plan.
- §6 (lines:205-263): `mdai-memory` braucht `ctx_knowledge.remember/recall`.
- §7a.1 (lines:324-341): vorgeschlagen `lean-ctx pack mdai-macros` für Cross-Projekt-Distribution; in
  dieser Spec als Backlog (Library bleibt zunächst repo-local).
- §7a.2 (lines:342-368): `remember_plan / recall_plan`-Wrapper.
- §7a.5 (lines:396-410): Gotchas-Tracking via `add_gotcha / list_gotchas`.

## 3. Architektur-Überblick

```
Repo-Wurzel
├── docs/mdai/                                  # Artefakte (specs, plans, etc.)
│   ├── specs/                                  ← Library-Spec landet hier
│   ├── plans/                                  ← Library-Impl-Plan landet hier
│   ├── red-baseline/library/                   ← NEU: RED-Reports pro Release
│   ├── green-verification/library/             ← NEU: GREEN-Reports pro Release
│   ├── KNOWLEDGE-SCHEMA.md
│   ├── GOTCHAS.md
│   └── design-skill-integration.md
│
├── mdai/                                       # Library-Wurzel (Quelle)
│   ├── MACROS.md                               # Index + Frontmatter (lib_version, requires)
│   │
│   ├── core/                                   # ALWAYS, immer in pre-context geladen
│   │   ├── startup-check.md                    # mdai_bootstrap, service_check, detect_*
│   │   ├── hard-rules.md                       # @include text (refactored)
│   │   ├── tool-quick-ref.md                   # @include text (Mapping-Tabelle)
│   │   ├── ctx-tools.md                        # ctx_read, ctx_search, ctx_tree, ctx_shell, ctx_edit
│   │   ├── mcp-markdownai.md                   # read_phase, list_phases, get_constraints
│   │   ├── ctx-knowledge.md                    # remember_plan, recall_plan
│   │   └── gotchas.md                          # add_gotcha, list_gotchas
│   │
│   ├── lang/                                   # OPT-IN nach MDAI_PROJECT_LANG
│   │   └── rust.md                             # cargo_nextest, cargo_clippy, cargo_fmt
│   │
│   ├── tooling/                                # OPT-IN nach MDAI_HAS_*
│   │   ├── jetbrains.md                        # reformat_file, step_reformat_commit
│   │   └── serena.md                           # find_symbol, replace_symbol_body, etc.
│   │
    └── skills/                                 # per-skill: Skill-Quelle + Library-Pack zusammen
        └── mdai-brainstorm/                    # Skill A — Quelle + Pack im selben Folder
            ├── SKILL.md                        # Skill-Quelle (pointer)
            ├── body.mdai.md                    # Skill-Quelle (live workflow, @imports Pack-Files unten)
            ├── scripts/install.sh              # Skill-Quelle (install nach ~/.claude/skills/)
            ├── write-spec.md                   # Pack: write_spec, render_spec
            ├── write-mdai-plan.md              # Pack: plan_frontmatter, plan_phase, plan_step, write_mdai_plan
            └── spec-reviewer.md                # Pack: spec_reviewer_prompt
```

Hinweis: kein separater Top-Level `skills/mdai/`-Tree mehr. Alle mdai-bezogenen Files (Library + Skill-Quellen) leben unter `mdai/`. Konsequenz für Skill-A-Spec: §4 Datei-Layout muss umgeschrieben werden (siehe §10).

**Verhältnis zu existierenden Komponenten:**

| Komponente                          | Behandlung                                                                                                |
|-------------------------------------|-----------------------------------------------------------------------------------------------------------|
| `docs/mdai/macros/*.md` (3 Files)   | **Physical-delete** mit Library-Impl. Skill-A wird durch dieselbe Patch-Session repariert (siehe §10).    |
| `superpowers/*` Skills              | Unverändert. Library kennt sie nicht, Skills kennen Library nicht.                                        |
| Skill A (`mdai-brainstorm`)         | Spec gepatcht in Follow-up-Session (§10). Body.mdai.md @import-iert Library statt inline @define.         |
| Skill B (`mdai-execution`)          | Eigener Pack `mdai/skills/mdai-execution/` als Backlog (eigene Spec).                                     |
| Skill C (`mdai-memory`)             | Eigener Pack `mdai/skills/mdai-memory/` als Backlog (eigene Spec).                                        |
| `mdai-drift-check` (Skill-A §15)    | Bleibt Backlog. Library kann durch `mdai-library-drift-check` (Backlog hier, §15.1) parallel erweitert.   |
| `lean-ctx pack mdai-macros` (§7a.1) | Cross-Projekt-Distribution als Backlog. v0.1.0 ist repo-local.                                            |

## 4. Bootstrap-Order und Library/Skill-A-Synchronisation

**Gewählter Pfad: A — Library zuerst, dann Skill A.**

Begründung: Skill-A-Spec referenziert die Library bereits durch `@include docs/mdai/macros/hard-rules.md`
und `@include docs/mdai/macros/tool-quick-ref.md` plus inline `@define`s für `writeSpec` etc. Eine saubere
Library-First-Reihenfolge bündelt alle Macro-Definitionen im Library-Repo, lässt Skill A einmal richtig
gebaut werden, und vermeidet einen Doppel-Refactor-Pass (erst inline, dann nach Library extrahieren).

**Konsequenz:** Skill-A-Spec wird in **dieser** Session NICHT geändert (Hard-Gate). Die nötigen Patches sind
in §10 vollständig dokumentiert. Eine Follow-up-Session bearbeitet die Skill-A-Spec, **bevor** Skill-A-A1
(Impl-Start) ausgeführt wird. Aufgrund der A9-Cleanup-Entscheidung (siehe §11.A9: physical-delete der alten
`docs/mdai/macros/*.md`) ist Skill-A nach Library-Impl bis zur Patch-Session render-broken. Das ist
beabsichtigt — es erzwingt den schnellen Patch-Turnaround und verhindert "vergessen".

## 5. Bootstrap-Mechanik (`mdai_bootstrap`)

Jeder mdai-Skill ruft in seiner `pre-context`-Phase exakt einmal `@call mdai_bootstrap()`. Das Macro
orchestriert die Liveness-Checks, die Projekt-Typ-Detection und das Caching.

### 5.1 `mdai/core/startup-check.md` (Skeleton)

```markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports:
    - service_check
    - detect_project_lang
    - detect_tooling
    - load_lang_pack
    - load_tooling_packs
    - mdai_bootstrap
---

@markdownai v1.0

@define service_check(service, mcp_tool, required)
@if @env MDAI_{{ service | upper }}_READY == "true"
  # cache hit, silent
@else
  @query mcp {{ service }} {{ mcp_tool }}
  @if @result.success
    [mdai-bootstrap OK] {{ service }} MCP reachable
    @query mcp lean-ctx ctx_session set "mdai_{{ service }}_ready" "true"
  @else
    @if {{ required }} == "true"
      [mdai-bootstrap FAIL] required service '{{ service }}' MCP unreachable.
        Reason: {{ @result.error | default("no response") }}
        Action: run `/mcp` to inspect connection, reconnect, then re-trigger skill.
        Blocking: skill cannot continue without '{{ service }}'.
      @query lean-ctx ctx_shell "exit 1"
    @else
      [mdai-bootstrap WARN] optional service '{{ service }}' MCP unreachable — skipping {{ service }} pack.
        Reason: {{ @result.error | default("no response") }}
        Impact: any later @call to {{ service }}-pack macros will be a no-op.
      @query mcp lean-ctx ctx_session set "mdai_{{ service }}_ready" "false"
    @endif
  @endif
@endif
@end

@define detect_project_lang()
@if @env MDAI_PROJECT_LANG != ""
  # cache hit, silent
@else
  @if @env MDAI_HAS_SERENA == "true"
    @query mcp serena serena_info topic="project"
    @if @result.language != ""
      [mdai-bootstrap] project lang detected via serena: {{ @result.language }}
      @query mcp lean-ctx ctx_session set "mdai_project_lang" "{{ @result.language | lower }}"
      @return
    @endif
  @endif
  @if @env MDAI_LEAN_CTX_READY == "true"
    @query mcp lean-ctx ctx_overview path="."
    @if @result.lang != ""
      [mdai-bootstrap] project lang detected via lean-ctx: {{ @result.lang }}
      @query mcp lean-ctx ctx_session set "mdai_project_lang" "{{ @result.lang | lower }}"
      @return
    @endif
  @endif
  # last-resort shell heuristic
  @query lean-ctx ctx_shell "
    if [ -f Cargo.toml ]; then echo rust
    elif [ -f pyproject.toml ] || [ -f setup.py ]; then echo python
    elif [ -f package.json ]; then echo node
    else echo unknown
    fi
  "
  [mdai-bootstrap] project lang detected via shell heuristic: {{ @result.stdout }}
  @query mcp lean-ctx ctx_session set "mdai_project_lang" "{{ @result.stdout }}"
@endif
@end

@define detect_tooling()
@if @env MDAI_TOOLING_DETECTED == "true"
  # cache hit, silent
@else
  @query mcp lean-ctx ctx_shell "claude mcp list | grep -E 'jetbrains|serena' || true"
  @if @result.stdout matches "jetbrains"
    @query mcp lean-ctx ctx_session set "mdai_has_jetbrains" "true"
  @else
    @query mcp lean-ctx ctx_session set "mdai_has_jetbrains" "false"
  @endif
  @if @result.stdout matches "serena"
    @query mcp lean-ctx ctx_session set "mdai_has_serena" "true"
  @else
    @query mcp lean-ctx ctx_session set "mdai_has_serena" "false"
  @endif
  @query mcp lean-ctx ctx_session set "mdai_tooling_detected" "true"
@endif
@end

@define load_lang_pack()
@if @env MDAI_PROJECT_LANG == "rust"
  @include mdai/lang/rust.md
@elseif @env MDAI_PROJECT_LANG == "python"
  @include mdai/lang/python.md
@elseif @env MDAI_PROJECT_LANG == "node"
  @include mdai/lang/node.md
@endif
@end

@define load_tooling_packs()
@if @env MDAI_HAS_JETBRAINS == "true"
  @include mdai/tooling/jetbrains.md
@endif
@if @env MDAI_HAS_SERENA == "true"
  @include mdai/tooling/serena.md
@endif
@end

@define mdai_bootstrap()
@call service_check(service="lean_ctx",   mcp_tool="ctx_session info",   required="true")
@call service_check(service="markdownai", mcp_tool="list_phases path=.", required="true")
@call detect_tooling()
@call detect_project_lang()
@end
```

### 5.2 Cache-Semantik

- `ctx_session`-State lebt session-scoped (laut lean-ctx-Doku). Bei Session-Ende oder explizitem
  `ctx_session clear` → Re-Detection beim nächsten Skill-Load.
- TTL: keine. Projekt-Typ und MCP-Liste ändern sich innerhalb einer Session nicht.
- Fail-Soft: `service_check(service="lean_ctx", ...)` läuft als allererstes. Wenn `ctx_session` selbst
  nicht erreichbar, schlägt dieser Check fehl, der `[mdai-bootstrap FAIL]`-Block erklärt warum, und
  `exit 1` blockt den Skill — der User weiß sofort, dass lean-ctx-MCP down ist.
- Cross-Subagent: jeder Subagent zahlt den Bootstrap einmal pro Dispatch. `ctx_session` ist
  subagent-spezifisch, kein cross-subagent-Cache in v0.1.0. Risiko in §14 akzeptiert.

### 5.3 Verwendung in einem Skill (`pre-context`)

```markdown
@phase pre-context

@call mdai_bootstrap()

@include mdai/core/hard-rules.md
@include mdai/core/tool-quick-ref.md
@include mdai/core/ctx-tools.md
@include mdai/core/mcp-markdownai.md
@include mdai/core/ctx-knowledge.md
@include mdai/core/gotchas.md

@call load_lang_pack()
@call load_tooling_packs()

@import mdai/skills/mdai-brainstorm/write-spec.md
@import mdai/skills/mdai-brainstorm/write-mdai-plan.md
@import mdai/skills/mdai-brainstorm/spec-reviewer.md
```

`@include` = Inline-Text + alle `@define`-Macros der Datei (für Regel-Files + Tool-Wrapper, deren Macros
zur Render-Zeit greifen sollen und deren Mapping-Tabelle als sichtbarer Text gewünscht ist).
`@import` = nur `@define`-Macros laden, kein sichtbarer Output (für Skill-Pack-Files).

## 6. Pflicht-Macro-Inventar v0.1.0

Alle Macro-Files tragen Frontmatter mit `lib_version`, `mdai-pack: { mode, exports }`. `mode` ∈
`{ import-only, include }`. Default beim Fehlen: `import-only`.

### 6.1 `mdai/MACROS.md` (Index)

```yaml
---
lib_version: "0.1.0"
released: 2026-05-24
status: pre-stable
requires:
  mcp__lean-ctx: ">=3.6.16"
  mcp__markdownai: ">=0.0.24"
---
```

Inhalt: Inventar-Tabelle (alle 13 Macro-Files + ihre Macros + mode + opt-in-condition), Conventions
(Frontmatter-Format, @import vs @include, snake_case-Naming), Changelog.

### 6.2 `mdai/core/` (ALWAYS, 7 Files)

| Datei                  | Mode        | Macros / Inhalt                                                                                              |
|------------------------|-------------|--------------------------------------------------------------------------------------------------------------|
| `startup-check.md`     | import-only | `service_check(service, mcp_tool, required)`, `detect_project_lang()`, `detect_tooling()`, `load_lang_pack()`, `load_tooling_packs()`, `mdai_bootstrap()` |
| `hard-rules.md`        | include     | Inline-Text (refactored: Z.6 raus, da in `tooling/jetbrains.md` als `step_reformat_commit`).                 |
| `tool-quick-ref.md`    | include     | Inline-Text Tool-Mapping-Tabelle (lean-ctx > native).                                                        |
| `ctx-tools.md`         | import-only | `ctx_read(path, mode)`, `ctx_search(pattern, path)`, `ctx_tree(path, depth)`, `ctx_shell(cmd)`, `ctx_edit(path, old, new)` |
| `mcp-markdownai.md`    | import-only | `read_phase(plan, phase_id)`, `list_phases(plan)`, `get_constraints(plan)`                                   |
| `ctx-knowledge.md`     | import-only | `remember_plan(plan_id, body)`, `recall_plan(plan_id)`                                                       |
| `gotchas.md`           | import-only | `add_gotcha(tag, title, body)`, `list_gotchas(tag)`                                                          |

Beispiel `core/ctx-tools.md`:

```markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [ctx_read, ctx_search, ctx_tree, ctx_shell, ctx_edit]
---

@markdownai v1.0

@define ctx_read(path, mode)
@query mcp lean-ctx ctx_read path="{{ path }}" mode="{{ mode | default('auto') }}"
@end

@define ctx_search(pattern, path)
@query mcp lean-ctx ctx_search pattern="{{ pattern }}" path="{{ path | default('.') }}"
@end

@define ctx_tree(path, depth)
@query mcp lean-ctx ctx_tree path="{{ path | default('.') }}" depth="{{ depth | default(3) }}"
@end

@define ctx_shell(cmd)
@query mcp lean-ctx ctx_shell command="{{ cmd }}"
@end

@define ctx_edit(path, old, new)
@query mcp lean-ctx ctx_edit path="{{ path }}" old_string="{{ old }}" new_string="{{ new }}"
@end
```

### 6.3 `mdai/lang/` (OPT-IN nach MDAI_PROJECT_LANG)

| Datei     | Mode        | Opt-In                  | Macros                                                                                |
|-----------|-------------|-------------------------|---------------------------------------------------------------------------------------|
| `rust.md` | import-only | `MDAI_PROJECT_LANG=rust` | `cargo_nextest()`, `cargo_clippy()`, `cargo_fmt()` — gemäß `~/.claude/CLAUDE.md` mandated |

`python.md`, `node.md` sind Backlog (siehe §15).

### 6.4 `mdai/tooling/` (OPT-IN nach MDAI_HAS_*)

| Datei          | Mode        | Opt-In                  | Macros                                                                                                                |
|----------------|-------------|-------------------------|-----------------------------------------------------------------------------------------------------------------------|
| `jetbrains.md` | import-only | `MDAI_HAS_JETBRAINS=true` | `reformat_file(file)`, `step_reformat_commit(file, message)` (composite: reformat + git add + git commit)             |
| `serena.md`    | import-only | `MDAI_HAS_SERENA=true`    | `find_symbol(name, path, include_body)`, `replace_symbol_body(name, path, body)`, `insert_before_symbol(name, path, body)`, `insert_after_symbol(name, path, body)`, `symbols_overview(path)` |

### 6.5 `mdai/skills/mdai-brainstorm/` (Pack für Skill A, 3 Files)

| Datei              | Mode        | Macros                                                                                                                     |
|--------------------|-------------|----------------------------------------------------------------------------------------------------------------------------|
| `write-spec.md`    | import-only | `write_spec(slug, body)` — schreibt `docs/mdai/specs/<date>-<slug>-design.mdai.md` via ctx_shell heredoc                   |
|                    |             | `render_spec(slug, target)` — target ∈ `{none, chat, file}`, conditional render via markdownai MCP                          |
| `write-mdai-plan.md` | import-only | `plan_frontmatter(id, spec)` — YAML-Block-Macro                                                                            |
|                    |             | `plan_phase(id, title, files, steps)` — @phase-Block-Macro                                                                  |
|                    |             | `plan_step(check, body)` — Checklist-Step                                                                                   |
|                    |             | `write_mdai_plan(slug, phases)` — schreibt `docs/mdai/plans/<date>-<slug>.mdai.md`                                          |
| `spec-reviewer.md` | import-only | `spec_reviewer_prompt(spec_path)` — generischer Reviewer-Prompt (hand-ported aus upstream)                                  |

**Total v0.1.0: 14 Files (1 Index `MACROS.md` + 7 core + 1 lang + 2 tooling + 3 skill-pack = 13 Macro-Files), ~28 Macros.**

## 7. Versionierung und Lifecycle

### 7.1 Schema

- **Library-Level:** semver in `mdai/MACROS.md` (`lib_version: "0.1.0"`).
- **Per-File:** jede `*.md` trägt im Frontmatter `lib_version: 0.1` (string ohne patch — wird beim Edit
  bumped, dient als Drift-Anker).
- **Bump-Regeln pre-1.0:**
  - **minor** (`0.1.0 → 0.2.0`): Breaking ODER additive Changes (in 0.y.z ist die API per Konvention
    unstabil, jede Minor-Bump darf brechen).
  - **patch** (`0.1.0 → 0.1.1`): Bug-Fixes nur (Render-Output funktional identisch).
- **Stabil:** Erste `1.0.0`-Release erst wenn mindestens 3 Skills die Library konsumieren UND ein
  Quartal kein Breaking-Change anfiel.

### 7.2 Lifecycle — Batch RED-GREEN pro Release

1. **Staging-Phase:** Neue Macros werden in die jeweiligen Pack-Files hinzugefügt mit Frontmatter-Marker
   `status: experimental`. Index `mdai/MACROS.md` listet sie in Sektion "Pending v0.x".
2. **Release-Trigger:** User entscheidet "Library v0.x ist reif" (z.B. mind. 3 neue Macros oder ein
   Skill-Pack komplett).
3. **RED-Pass:** 3 Subagents (Gruppe RED) bekommen Plan-Generation-Auftrag, der die experimental-Macros
   normalerweise nutzen würde — aber ohne sie. Tool-Strings werden inline geschrieben. Reports unter
   `docs/mdai/red-baseline/library/v0.x-RED-{1,2,3}.md`.
4. **GREEN-Pass:** Selbe 3 Subagents bekommen denselben Auftrag mit Library v0.x verfügbar. Reports
   unter `docs/mdai/green-verification/library/v0.x-GREEN-{1,2,3}.md`.
5. **Acceptance:** Qualitativ. Pro Macro entscheidet User basierend auf (a) LOC-Reduktion RED vs.
   GREEN, (b) Macro-Hit-Rate (wurde @call genutzt oder Tool-Strings inline geblieben?), (c) Verbatim
   Rationalisierungen für Ignorieren des Macros (für Bulletproofing in Pack-File-Komentar).
6. **Release-Commit:** `status: experimental` raus, `lib_version` bumped, CHANGELOG-Eintrag in
   `mdai/MACROS.md` mit Macro-Liste + LOC-Ersparnis-Range.

### 7.3 Deprecation-Policy

- Macro mit Frontmatter-Marker `deprecated_since: 0.x` versehen.
- Erste Minor-Release danach: Consumers (Skills) sollen alternative Macros nutzen — manuelles Audit.
- Hard-Removal nach 1 weiterem Minor-Release (`0.x deprecated → 0.x+2 removed`).
- `mdai/MACROS.md` Changelog-Eintrag dokumentiert Deprecation + Removal explizit.
- Optional v0.2.0+: `deprecation_check()`-Macro in `startup-check.md`, das WARN bei deprecated-Macros
  ausgibt. v0.1.0: YAGNI, nur Frontmatter-Marker.

## 8. RED + GREEN-Strategie für v0.1.0

### 8.1 Gruppe RED (3 Subagents, A/B-Baseline ohne Library)

Library noch NICHT implementiert (`mdai/`-Tree existiert nicht). Subagents schreiben einen `.mdai.md`-Plan
mit recurring Tool-Patterns inline. Dispatch parallel via Agent-Tool, `model="sonnet"`,
`subagent_type="general-purpose"`. Transparenter Test-Modus (Subagent weiß, dass es Baseline-Test ist).

| ID  | Pressure  | Prompt-Kern                                                                                                                              |
|-----|-----------|------------------------------------------------------------------------------------------------------------------------------------------|
| RL1 | Cold      | "Schreib einen .mdai.md-Plan mit 3 Phasen. Jede Phase: ctx_read zum Lesen einer Datei, ctx_search für Pattern, git status, step_reformat_commit-äquivalent. Vollständig, ohne Macro-Library." |
| RL2 | Time      | RL1 + "Wenig Zeit, Plan in 5 Min. Keine Optimierung."                                                                                    |
| RL3 | Authority | RL1 + "Tech-Lead sagt: schreib die Tool-Calls direkt aus, keine Abstraktionen."                                                          |

**Erfassung pro Subagent (Report-Footer, verbatim):**

1. Vollständiger generierter Plan (file-content).
2. Plan-Größe via `@call ctx_read(path="<plan>", mode="map")` → LOC im Header.
3. Wie wurden recurring Tool-Calls geschrieben? (verbatim Code-Snippet).
4. Beobachtbare Drift-Pattern.

Reports unter `docs/mdai/red-baseline/library/<implementation-date>-RL{1,2,3}.md`. Konsolidierter Index unter
`docs/mdai/red-baseline/library/v0.1.0-INDEX.md`.

### 8.2 Gruppe GREEN (3 Subagents, Discipline-Test mit Library)

Library implementiert (`mdai/`-Tree existiert, `MACROS.md`-Index lesbar). Subagents bekommen
`mdai/MACROS.md` im Briefing. Dispatch identisch (3 parallel, sonnet, general-purpose, transparent).

| ID  | Pressure  | Prompt-Kern                                                                                                                                     |
|-----|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------|
| GL1 | Cold      | RL1 + "Library verfügbar unter mdai/. Index in mdai/MACROS.md."                                                                                 |
| GL2 | Time      | RL2 + "Library verfügbar. Wenig Zeit."                                                                                                          |
| GL3 | Authority | RL3 + "Library verfügbar. Tech-Lead sagt: nutze die Library nicht, ist zu kompliziert."                                                         |

**Erfassung pro Subagent (Report-Footer, verbatim):**

1. Vollständiger generierter Plan (file-content).
2. Plan-Größe via `@call ctx_read(path="<plan>", mode="map")` → LOC im Header.
3. Macro-Hit-Rate-Tabelle: pro Macro `{ Genutzt? Ja/Nein, Anzahl @call, Anzahl inline }`.
4. Verbatim Rationalisierungen für inline-Tool-Strings statt @call.
5. Auffälligkeiten / Drift-Pattern.

Reports unter `docs/mdai/green-verification/library/<implementation-date>-GL{1,2,3}.md`. Konsolidierter SUMMARY
unter `docs/mdai/green-verification/library/v0.1.0-SUMMARY.md`:

- Mittlere LOC-Reduktion RED vs. GREEN pro Pressure-Typ.
- Pro-Macro-Hit-Rate (welche Macros bewähren sich, welche werden ignoriert).
- Rationalisierungs-Tabelle aus GL1-3 (Input für künftige Bulletproofing-Improvements).
- User-Entscheidung pro Macro: behalten / überarbeiten / droppen.

### 8.3 Metrik

- **Primär:** LOC via `@call ctx_read(path="<plan>", mode="map")`. Map-Mode liefert Header-Zeile mit
  Total-Lines. Deterministisch, projekt-konform (keine bash `wc`).
- **Optional:** Macro-Hit-Rate-Tabelle (GREEN only) — qualitativer Indikator.
- **Acceptance:** keine harte Schwelle. User entscheidet pro Macro basierend auf Reports.

## 9. Skill-Integration (wie konsumiert ein Skill die Library)

### 9.1 Pre-Context-Phase (Minimal-Template)

```markdown
@phase pre-context

@call mdai_bootstrap()

@include mdai/core/hard-rules.md
@include mdai/core/tool-quick-ref.md
@include mdai/core/ctx-tools.md
@include mdai/core/mcp-markdownai.md
@include mdai/core/ctx-knowledge.md
@include mdai/core/gotchas.md

@call load_lang_pack()
@call load_tooling_packs()

@import mdai/skills/<skill-name>/<pack-file-1>.md
@import mdai/skills/<skill-name>/<pack-file-2>.md
...
```

### 9.2 Convention: Pack-File-Frontmatter

Jede Pack-Datei deklariert ihre `mode` im Frontmatter (Default `import-only`). Skills lesen das beim
Generieren ihres pre-context und wählen `@import` vs. `@include`. Konvention wird in `mdai/MACROS.md`
Conventions-Sektion dokumentiert.

```yaml
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only        # oder: include
  exports: [write_spec, render_spec]
---
```

### 9.3 Aufruf der Macros in den Skill-Phasen

```markdown
@phase write-outputs

@call write_spec(slug={{ slug }}, body={{ design_content }})
@call write_mdai_plan(slug={{ slug }}, phases={{ phase_list }})

Verification:
@call ctx_shell(cmd="git status docs/mdai/")
```

## 10. Konsequenzen für Skill-A-Spec (Patches in Follow-up-Session)

Skill-A-Spec (`docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md`) MUSS in einer separaten Session
gepatcht werden, **bevor** Skill-A-A1 (Impl-Start) ausgeführt wird. Aufgrund A9-Cleanup (§11) ist Skill A
zwischen Library-Impl und Patch-Session render-broken — das ist beabsichtigt.

| Skill-A-Sektion                | Aktueller Zustand                                                                                              | Patch                                                                                                                                                                                                       |
|--------------------------------|----------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| §3 Architektur-Diagramm        | Zeigt nur SKILL.md + body.mdai.md mit inline @define + @include `docs/mdai/macros/*.md`                        | Erweitern um `mdai/`-Tree (core/, lang/, tooling/, skills/mdai-brainstorm/). Pfeil "body.mdai.md @import mdai/skills/mdai-brainstorm/*" + "body.mdai.md @include mdai/core/*"                                |
| §3 Verhältnis-Tabelle          | Listet superpowers, mdai-plans, mdai-execution, mdai-memory, mdai-drift-check                                  | Zeile dazu: `mdai-macro-library` — Library für cross-skill + skill-pack Macros, konsumiert in pre-context-Phase                                                                                              |
| §4 Datei-Layout                | Zeigt `skills/mdai/mdai-brainstorm/SKILL.md + body.mdai.md + scripts/install.sh` (Top-Level `skills/`-Tree)    | Komplett umschreiben: Skill-Quellen wandern nach `mdai/skills/mdai-brainstorm/` (gleicher Folder wie Library-Pack). `skills/mdai/`-Tree entfällt. Plus install.sh-Pfad-Update: `mdai/skills/mdai-brainstorm/scripts/install.sh` muss `$SRC`-Pfad neu berechnen. Symlink-Target aus §4 (`.claude/skills/mdai-brainstorm → ../../skills/mdai/mdai-brainstorm`) ändert sich zu `.claude/skills/mdai-brainstorm → ../../mdai/skills/mdai-brainstorm`. |
| §6.1 Macros                    | Inline `@define writeSpec / renderSpec / writeMdaiPlan / planFrontmatter / planPhase / planStep / specReviewerPrompt` — alle camelCase | Komplett raus aus body.mdai.md. Ersetzen durch drei `@import mdai/skills/mdai-brainstorm/*.md`. Macro-Namen werden snake_case (writeSpec → write_spec, planPhase → plan_phase etc.). Sed-Skript für ~10 Renames. |
| §6.2 pre-context               | `@include docs/mdai/macros/hard-rules.md` + `@include docs/mdai/macros/tool-quick-ref.md`                      | Erste Zeile NEU: `@call mdai_bootstrap()`. Pfade umstellen: `@include mdai/core/hard-rules.md`, plus `@include mdai/core/ctx-tools.md`, `@include mdai/core/mcp-markdownai.md`, `@include mdai/core/ctx-knowledge.md`, `@include mdai/core/gotchas.md`. Dann `@call load_lang_pack()` + `@call load_tooling_packs()` |
| §6.4 write-outputs             | `@call writeSpec(...)`, `@call writeMdaiPlan(...)` (camelCase)                                                 | snake_case-Rename: `@call write_spec(...)`, `@call write_mdai_plan(...)`                                                                                                                                    |
| §6.5 handoff                   | `@query mcp markdownai list_phases path=...` direkt                                                            | Optional: ersetzen durch `@call list_phases(plan=...)` (kosmetisch, kleiner Boilerplate-Gewinn)                                                                                                             |
| §16 P0                         | "Macro-Mirror docs/mdai/macros/ verifizieren"                                                                  | Ersetzen durch "mdai-macro-library v0.1.0 implementiert (siehe library-spec §11) — Pfade mdai/core/, mdai/skills/mdai-brainstorm/ vorhanden, mdai/MACROS.md present"                                        |
| §16 A1                         | SKILL.md schreiben ohne Library-Referenz                                                                       | Keine Änderung am SKILL.md (Pointer). Aber A2 (body.mdai.md) ändert sich.                                                                                                                                   |
| §16 A2                         | body.mdai.md schreiben mit inline @define + alten @include-Pfaden                                              | A2 Sub-Schritte erweitern: vor body.mdai.md-Write `mdai/MACROS.md` lesen, alle benötigten @import-Pfade in pre-context einsetzen. Alle inline `@define`s aus §6.1 raus.                                     |
| §17 Annahmen                   | Annahmen 1-5 zu MCP-Tools                                                                                      | Annahme 6 dazu: "mdai-macro-library v0.1.0 ist im Repo unter mdai/, `mdai_bootstrap`-Macro funktioniert, MACROS.md-Inventar matched §6"                                                                     |

**Verifikation der Skill-A-Patch-Session:** `mai render` auf gepatchtem `body.mdai.md` (Library muss
existieren) → keine Render-Fehler, alle Macros expandieren.

## 11. Implementierungsschritte (high-level, der echte Plan kommt via writing-plans-Skill)

| Phase | Aufgabe                                                                                                                                                                                                                                                                                                                                                                                              |
|-------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| RED   | **Iron Law — Baseline vor Library-Impl (§8.1).** 3 parallele Subagents für RL1/RL2/RL3 dispatchen. Reports + INDEX unter `docs/mdai/red-baseline/library/` committen. Erfolgs-Kriterium: ≥3 verbatim Tool-Call-Pattern erfasst, ≥1 Discipline-Drift dokumentiert. Blockt P0 bis erfüllt.                                                                                                              |
| P0    | **MCP-Verifikation:** `mcp__serena__serena_info topic="project"` und `mcp__lean-ctx__ctx_overview path="."` einmal callen, Output-Schema inspizieren. `detect_project_lang`-Macro auf reale Field-Namen tunen. **Setup:** `mkdir -p mdai/core mdai/lang mdai/tooling mdai/skills/mdai-brainstorm docs/mdai/red-baseline/library docs/mdai/green-verification/library`.                                  |
| A1    | `mdai/MACROS.md` schreiben (Index + Frontmatter + leerer Changelog v0.1.0).                                                                                                                                                                                                                                                                                                                          |
| A2    | `mdai/core/*` schreiben (7 Files). Reihenfolge: hard-rules.md + tool-quick-ref.md (Refactor existierender Files) → ctx-tools.md + mcp-markdownai.md + ctx-knowledge.md + gotchas.md → startup-check.md (am Schluss, weil er die anderen orchestriert).                                                                                                                                                |
| A3    | `mdai/lang/rust.md` + `mdai/tooling/jetbrains.md` + `mdai/tooling/serena.md` schreiben. `step_reformat_commit` aus existing `docs/mdai/macros/step-reformat-commit.md` migrieren (Pfad + snake_case-Rename, Composite erweitern um git add + git commit).                                                                                                                                              |
| A4    | `mdai/skills/mdai-brainstorm/*` schreiben (3 Files). Macros aus Skill-A-Spec §6.1 wörtlich übernehmen, snake_case-Rename.                                                                                                                                                                                                                                                                            |
| A5    | **Smoke-Tests pro File:** `mai render` auf einer Test-`.mdai.md`-Datei, die alle 13 Files via @import / @include + @call referenziert. Erwartung: keine Render-Fehler, alle Macros expandieren. Pro-Pack-File-Mode-Verifikation: prüfe dass `mode: include` Inline-Text rendert, `mode: import-only` nicht.                                                                                            |
| A6    | **Live-Test mdai_bootstrap:** Eine Test-`.mdai.md`, die nur `@call mdai_bootstrap()` plus einen `@call ctx_read(...)` enthält. Render in Claude-Session. Erwartung: `[mdai-bootstrap OK]`-Outputs sichtbar (lean_ctx + markdownai), `[mdai-bootstrap WARN]` falls jetbrains/serena nicht da. Bei zweitem Aufruf: silent (Cache-Hit verifiziert). `ctx_session`-Persistenz wird hier nebenbei geprüft. |
| A7    | **GREEN-Verification (§8.2):** 3 Subagents (GL1-3) dispatchen, Reports + SUMMARY nach `docs/mdai/green-verification/library/`.                                                                                                                                                                                                                                                                       |
| A8    | **Lookup-Tabellen committen:** `v0.1.0-INDEX.md` (RED) + `v0.1.0-SUMMARY.md` (GREEN) committen.                                                                                                                                                                                                                                                                                                       |
| A9    | **Cleanup alte Macros:** `rm docs/mdai/macros/hard-rules.md docs/mdai/macros/tool-quick-ref.md docs/mdai/macros/step-reformat-commit.md`. Skill-A ist ab jetzt render-broken bis Skill-A-Patch-Session durch ist. Verifiziere: `ls docs/mdai/macros/` ist leer.                                                                                                                                       |

**Hinweis Reihenfolge:** RED **vor** P0 — konsistent mit Iron Law aus `superpowers:writing-skills`. Library
existiert während RED noch nicht. P0 macht erst Setup, dann läuft Library-Impl.

## 12. Smoke-Tests (Detail)

### 12.1 Render-Test pro Macro-File (A5)

Eine `tmp/library-smoke-test.mdai.md` referenziert alle 13 Files:

```markdown
@markdownai v1.0

@call mdai_bootstrap()

@include mdai/core/hard-rules.md
@include mdai/core/tool-quick-ref.md
@include mdai/core/ctx-tools.md
@include mdai/core/mcp-markdownai.md
@include mdai/core/ctx-knowledge.md
@include mdai/core/gotchas.md

@call load_lang_pack()
@call load_tooling_packs()

@import mdai/skills/mdai-brainstorm/write-spec.md
@import mdai/skills/mdai-brainstorm/write-mdai-plan.md
@import mdai/skills/mdai-brainstorm/spec-reviewer.md

@call ctx_read(path="README.md", mode="map")
@call list_phases(plan="tmp/library-smoke-test.mdai.md")
@call write_spec(slug="smoke-test", body="dummy")
```

Befehl: `npx mai render tmp/library-smoke-test.mdai.md`. Pass-Kriterium: kein Render-Fehler, alle Macros
expandieren, `[mdai-bootstrap OK]`-Lines erscheinen.

### 12.2 Bootstrap-Cache-Test (A6)

Render dieselbe Datei zweimal in einer Session:

- **1. Run:** `[mdai-bootstrap OK] lean_ctx MCP reachable` + `[mdai-bootstrap OK] markdownai MCP reachable`.
- **2. Run:** keine `[mdai-bootstrap ...]`-Lines (Cache-Hit, silent).

### 12.3 Service-Fail-Test (A6, optional)

Manuell markdownai-MCP disconnecten (`/mcp` → disconnect), Render erneut:

- Erwartung: `[mdai-bootstrap FAIL] required service 'markdownai' MCP unreachable.` + `exit 1`.

### 12.4 Lang-Detection-Test (A6)

In einem Rust-Projekt (dieses Repo): `MDAI_PROJECT_LANG` muss `rust` werden, `mdai/lang/rust.md` muss
geladen werden. Test-Render in einem `tmp/non-rust-project/` (kein `Cargo.toml`): `MDAI_PROJECT_LANG` muss
`unknown` werden, `load_lang_pack()` läuft als no-op.

## 13. Non-Goals (v0.1.0)

1. `mdai/skills/mdai-execution/` und `mdai/skills/mdai-memory/` Packs — separate Specs nach Skill-B/C-Brainstorm.
2. `lang/python.md`, `lang/node.md` — Backlog, kommen wenn ein Projekt sie braucht.
3. Automatische Pack-Distribution via `lean-ctx pack` — Library bleibt repo-local in v0.1.0. Pack-Distribution
   als Backlog (siehe §15.5, parallel zu Skill-A-Spec §15 Backlog #5 Plugin-Packaging).
4. Auto-Generated Inventory (`mdai/MACROS.md` aus Macro-Files generieren) — manuell pflegen in v0.1.0.
5. Runtime `deprecation_check()` — YAGNI bis erste Deprecation auftritt.
6. Cross-Subagent-Cache für `mdai_bootstrap` — siehe §14, jeder Subagent zahlt Bootstrap einmal pro Dispatch.
7. Multi-Lang-Detection (Monorepo mit Rust + Node) — siehe §14, single-lang only in v0.1.0.

## 14. Risiken

| Risiko                                                                                                                  | Schweregrad | Mitigation                                                                                                                                                                                                                       |
|-------------------------------------------------------------------------------------------------------------------------|-------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `mcp__markdownai__read_file` behandelt `mode`-Frontmatter nicht konsistent (import-only vs. include)                    | Mittel      | A5 Smoke-Test prüft jeden Pack-File-Mode. Bei Fail: alle Pack-Files temporär auf `mode: include` stellen (mehr Token, aber sicher). Drift-Report.                                                                                |
| `mdai_bootstrap` läuft pro Subagent-Dispatch erneut → bootstrap-Output spammt jeden Subagent-Chat                       | Mittel      | `ctx_session` ist subagent-spezifisch laut lean-ctx-Doku. Akzeptieren: pro Subagent ein `[mdai-bootstrap OK]`-Block in pre-context. v0.2.0-Backlog: cross-subagent-Cache via `ctx_knowledge`.                                    |
| MCP-Tool-Signatur-Drift (z.B. `ctx_read` kriegt neuen Pflicht-Parameter) bricht Wrapper-Macros stumm                    | Mittel      | A5 Smoke-Test als CI-Step (nach jedem MCP-Bump). Backlog: `mdai-library-drift-check`-Skill (§15.1) für Hash-Vergleich Tool-Signaturen vs. Wrapper.                                                                               |
| Skill A nutzt Library nicht (User vergisst Patch-Session, implementiert Skill A mit inline @define)                     | Hoch        | A9 Cleanup macht Skill A render-broken bis Patch-Session durch ist (BLOCKING). Plus expliziter Hinweis im Library-CHANGELOG v0.1.0: "Skill-A-Spec muss vor Skill-A-A1 gepatcht werden, siehe library-spec §10."                  |
| snake_case-Naming-Switch bricht alle existierenden camelCase-Referenzen im Skill-A-Spec auf einmal                      | Niedrig     | Single-Session-Rewrite: ein PR ändert §6.1, §6.4, §6.5 atomar. Sed-skript für die ~10 Renames.                                                                                                                                  |
| `detect_project_lang` Multi-Lang-Edge-Case (Monorepo mit Rust + Node) → wählt nur erstes Match                          | Niedrig     | v0.1.0 single-lang-only akzeptiert. Multi-Lang in v0.2.0 Backlog. Workaround: `MDAI_PROJECT_LANG` env-Override manuell setzen.                                                                                                  |
| `serena_info(topic="project")` returniert nicht `language`-Field (Annahme aus §5.1)                                      | Niedrig     | P0 verifiziert Schema. Falls Field fehlt: Detection-Chain fällt automatisch auf `ctx_overview` und dann auf Shell-Heuristik zurück (siehe §5.1). Macro funktioniert robust.                                                       |

## 15. Backlog (separate Specs)

Explizit-deferred Parking-List. Jeder Eintrag bekommt bei Bedarf eine eigene Spec via `/mdai-brainstorm`.

1. **`mdai-library-drift-check`** — Skill für Hash-Vergleich Macro-Files vs. MCP-Tool-Signaturen. Speicher
   in `mdai/upstream-hashes.json`. Trigger: erster MCP-Version-Bump nach v0.1.0-Release.
2. **`mdai/skills/mdai-execution/` Pack** — `phase_dispatch`, `subagent_briefing`. Trigger: Skill-B-Brainstorm.
3. **`mdai/skills/mdai-memory/` Pack** — `plan_start`, `plan_phase_done`, `plan_all_done`. Trigger: Skill-C-Brainstorm.
4. **Multi-Lang-Detection + `MDAI_PROJECT_LANG`-Override** — siehe §14. Trigger: Multi-Lang-Monorepo-Bedarf.
5. **`lean-ctx pack mdai-macros`** — Cross-Projekt-Distribution (parallel zu Skill-A-Spec §15 #5 Plugin-Packaging).
   Trigger: ≥2 weitere Projekte wollen die Library nutzen.
6. **Auto-Generated MACROS.md** — Index aus Frontmatter-Scan der Pack-Files. Trigger: Inventar ≥30 Files.
7. **`deprecation_check()`** — Runtime-WARN für deprecated Macros. Trigger: erste Deprecation in v0.x.
8. **`lang/python.md`, `lang/node.md`** — analog zu rust.md. Trigger: Projekt in Python oder Node.
9. **Cross-Subagent-Cache für mdai_bootstrap** — via `ctx_knowledge` statt `ctx_session`. Trigger: Bootstrap-
   Spam wird in Reports als störend dokumentiert.

## 16. Annahmen, die in Smoke-Tests / P0 zu verifizieren sind

1. `mcp__serena__serena_info(topic="project")` returniert ein `language`-Field. Verifiziert in P0; bei Fail
   fällt `detect_project_lang` auf Stufe 2 (ctx_overview) zurück.
2. `mcp__lean-ctx__ctx_overview(path=".")` returniert ein `lang`-Field. Verifiziert in P0; bei Fail fällt
   `detect_project_lang` auf Shell-Heuristik zurück.
3. `mcp__lean-ctx__ctx_session.set(key, value)` und `ctx_session info` existieren mit der hier
   angenommenen Signatur. Verifiziert in P0 via direktem MCP-Call.
4. `mcp__markdownai__read_file` respektiert `mdai-pack.mode`-Frontmatter (`import-only` lädt nur @define,
   kein Inline-Text). Verifiziert in A5.
5. `ctx_session` ist subagent-spezifisch (jeder Subagent hat seinen eigenen State). Verifiziert in A6.
6. `mai render` (`npx mai render`) ist im Projekt-Repo verfügbar (`markdownai/`-Paket ist gebaut).
   Verifiziert in A5 via direktem CLI-Call.

## 17. Output-Erwartung dieser Spec

Diese Spec ist der Input für die Library-Implementierung. Der echte Implementierungs-Plan wird in einer
separaten Session via `superpowers:writing-plans` aus dieser Spec generiert und unter
`docs/mdai/plans/2026-05-24-mdai-macro-library-impl.mdai.md` committet. Der Plan enthält die in §11
skizzierten Phasen als `@phase`-Blöcke für parallel-dispatch via `mdai-execution` (sobald Skill B verfügbar)
oder via `superpowers:subagent-driven-development` (heute).

**Skill-A-Patch-Session** ist ein separater Plan und wird parallel via dieselbe Mechanik geplant.

---

## Anhang A: Frontmatter-Konventionen (Quick-Reference)

### Macro-File-Frontmatter

```yaml
---
lib_version: 0.1.0                    # bumped bei Edit
mdai-pack:
  mode: import-only | include         # default: import-only
  exports:                            # Liste der @define-Namen
    - macro_one
    - macro_two
  status: experimental | stable       # default: stable
  deprecated_since: 0.x               # optional
---
```

### Index-File-Frontmatter (`mdai/MACROS.md`)

```yaml
---
lib_version: "0.1.0"                  # semver string
released: 2026-05-24                  # ISO date
status: pre-stable | stable
requires:
  mcp__lean-ctx: ">=3.6.16"
  mcp__markdownai: ">=0.0.24"
---
```

## Anhang B: Beispiel-Pack-File (`mdai/tooling/jetbrains.md`)

```markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [reformat_file, step_reformat_commit]
---

@markdownai v1.0

@define reformat_file(file)
@query mcp jetbrains reformat_file path="{{ file }}"
@end

@define step_reformat_commit(file, message)
@call reformat_file(file="{{ file }}")
@call ctx_shell(cmd="git add {{ file }}")
@call ctx_shell(cmd="git commit -m '{{ message }}'")
@end
```

Hinweis: `step_reformat_commit` ist composite über `reformat_file` (im selben File) und `ctx_shell` (aus
`mdai/core/ctx-tools.md`). Funktioniert nur, wenn `mdai_bootstrap` `MDAI_HAS_JETBRAINS=true` gesetzt hat —
sonst wurde dieses File via `load_tooling_packs()` gar nicht erst geladen.
