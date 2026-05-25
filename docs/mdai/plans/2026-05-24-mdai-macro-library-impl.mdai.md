---
id: mdai-macro-library-impl
plan_for: docs/mdai/specs/2026-05-24-mdai-macro-library-design.mdai.md
created: 2026-05-24
lib_version_target: 0.1.0
language: de
---

@markdownai v1.0 consumer="ai"

# mdai-macro-library v0.1.0 — Implementierungs-Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (empfohlen) oder `superpowers:executing-plans`, um diesen Plan task-by-task abzuarbeiten. Steps nutzen Checkbox-Syntax (`- [ ]`) für Tracking.

**Ziel:** mdai-macro-library v0.1.0 implementieren — eine versionierte Macro-Library unter `mdai/` mit 13 Macro-Files (~28 Macros), die cross-skill Tool-Wrapper, projekt-spezifische Lang/Tooling-Packs und Skill-A-Composite-Macros bündelt. Eingangs RED-Baseline, Library-Build, Smoke-Tests, GREEN-Verification, Cleanup der alten `docs/mdai/macros/`-Files.

**Architektur:** Quelle der Wahrheit ist `mdai/MACROS.md` (Index, Frontmatter, Changelog). `mdai/core/` (7 Files, always-on) deckt Tool-Wrapper + Bootstrap ab; `mdai/lang/rust.md` + `mdai/tooling/jetbrains.md` + `mdai/tooling/serena.md` sind opt-in via `mdai_bootstrap`-Macro, das `ctx_session`-Flags setzt; `mdai/skills/mdai-brainstorm/*.md` bündelt drei skill-eigene Composite-Macros. Verifikation läuft per `npx mai render` (Smoke-Tests) und 3 RED + 3 GREEN Subagent-Reports.

**Tech-Stack:** `markdownai` v0.0.24 (`@define`/`@call`/`@import`/`@include`/`@query mcp`/`@if`), `mcp__lean-ctx` ≥3.6.16 (`ctx_read`, `ctx_search`, `ctx_tree`, `ctx_shell`, `ctx_edit`, `ctx_session`, `ctx_knowledge`), `mcp__markdownai` (`read_phase`, `list_phases`, `get_constraints`), optional `mcp__jetbrains` + `mcp__serena`. CLI: `npx mai render <file>.mdai.md` aus dem Repo-Root.

**Hard-Rule-Reminder (aus Project-CLAUDE.md):**
- Keine `&&`-Bash-Chains — jeden Befehl einzeln.
- Vor `git add`: `mcp__jetbrains__reformat_file` auf jede geänderte Datei.
- Keine Worktrees — direkt auf `feat-mdai`.
- lean-ctx Tools (`ctx_read`/`ctx_search`/`ctx_shell`/`ctx_tree`/`ctx_edit`) statt nativer Equivalents.

**Sprache:** Plan-Text/Interaktion = Deutsch. Code, Code-Kommentare, Macro-Namen, Frontmatter-Keys = Englisch (snake_case).

---

## File-Structure (Soll-Zustand nach diesem Plan)

**Neu angelegt:**

```
mdai/
├── MACROS.md                                 # Index + lib_version + Changelog
├── core/
│   ├── startup-check.md                      # mdai_bootstrap + service_check + detect_*
│   ├── hard-rules.md                         # @include text (refactored from docs/mdai/macros/)
│   ├── tool-quick-ref.md                     # @include text (refactored)
│   ├── ctx-tools.md                          # ctx_read, ctx_search, ctx_tree, ctx_shell, ctx_edit
│   ├── mcp-markdownai.md                     # read_phase, list_phases, get_constraints
│   ├── ctx-knowledge.md                      # remember_plan, recall_plan
│   └── gotchas.md                            # add_gotcha, list_gotchas
├── lang/
│   └── rust.md                               # cargo_nextest, cargo_clippy, cargo_fmt
├── tooling/
│   ├── jetbrains.md                          # reformat_file, step_reformat_commit
│   └── serena.md                             # find_symbol, replace_symbol_body, …
└── skills/mdai-brainstorm/
    ├── write-spec.md                         # write_spec, render_spec
    ├── write-mdai-plan.md                    # plan_frontmatter, plan_phase, plan_step, write_mdai_plan
    └── spec-reviewer.md                      # spec_reviewer_prompt

docs/mdai/red-baseline/library/               # RED-Reports (vor Library-Impl gefüllt)
docs/mdai/green-verification/library/         # GREEN-Reports (nach Library-Impl gefüllt)
tmp/library-smoke-test.mdai.md                # transientes Smoke-Test-File (nicht committen)
```

**Gelöscht (Task 21 / §11.A9):**

```
docs/mdai/macros/hard-rules.md
docs/mdai/macros/step-reformat-commit.md
docs/mdai/macros/tool-quick-ref.md
docs/mdai/macros/                              # leerer Folder bleibt ODER wird removed
```

**Verantwortung pro File:** siehe Spec §6 (Inventar-Tabelle). Jedes File hat Frontmatter (`lib_version`, `mdai-pack.{mode,exports}`), die Pack-Convention aus Spec §9.2.

---

## Task 1: RED-Baseline (Iron Law — vor Library-Impl)

**Phase-Mapping:** Spec §11 Phase RED, Detail in §8.1.

**Files:**
- Create: `docs/mdai/red-baseline/library/2026-05-24-RL1.md`
- Create: `docs/mdai/red-baseline/library/2026-05-24-RL2.md`
- Create: `docs/mdai/red-baseline/library/2026-05-24-RL3.md`
- Create: `docs/mdai/red-baseline/library/v0.1.0-INDEX.md`

**Hinweis:** Library existiert in diesem Task NOCH NICHT. Subagents schreiben Pläne mit inline-Tool-Calls. Drei parallele Dispatches via `Agent`-Tool in einer einzigen Nachricht.

- [ ] **Step 1.1: Setup-Folder für RED-Reports anlegen**

```bash
mkdir -p docs/mdai/red-baseline/library
```

- [ ] **Step 1.2: Drei Subagents parallel dispatchen (Agent-Tool, single message, 3 calls)**

Jeder Subagent bekommt `subagent_type="general-purpose"`, `model="sonnet"`. Prompt-Vorlage pro Subagent:

```
Du bist in einer kontrollierten Baseline-Erhebung (RED-Pass) für die mdai-macro-library v0.1.0.
Die Library existiert NOCH NICHT — du nutzt sie bewusst nicht.

Aufgabe: Schreibe einen `.mdai.md`-Plan mit 3 Phasen. Jede Phase enthält:
- ein `ctx_read` zum Lesen einer Datei,
- ein `ctx_search` für ein Pattern,
- ein `git status`-Equivalent via `ctx_shell`,
- den Composite "vor git add reformat_file + git add + git commit" (jetbrains).
Schreibe alle Tool-Calls direkt aus (inline `@query mcp ...` oder vergleichbare Strings),
KEINE Macro-Abstraktionen.

Pressure: <RL1=Cold | RL2=Time(5min, no optimisation) | RL3=Authority(Tech-Lead sagt: keine Abstraktionen)>

Erfassung (Report-Footer, verbatim):
1. Vollständiger generierter Plan (file-content).
2. Plan-Größe via `mcp__lean-ctx__ctx_read(path=<plan>, mode="map")` → LOC im Header.
3. Verbatim Code-Snippet, wie du recurring Tool-Calls geschrieben hast.
4. Beobachtbare Drift-Pattern (welche Tool-Strings wiederholen sich, welche
   Inkonsistenzen entstehen).

Speichere den Report unter `docs/mdai/red-baseline/library/2026-05-24-RL<N>.md`.
Lean-ctx-Tools (ctx_read/ctx_shell/ctx_edit/ctx_tree/ctx_search) bevorzugen für eigene
Arbeit. Keine `&&`-Chains. Keine Worktrees.
```

Drei Agent-Tool-Calls in einem einzigen Message-Block: RL1 (Cold), RL2 (Time), RL3 (Authority). Backgrund: nein (Reports werden in nächstem Step gelesen).

- [ ] **Step 1.3: Reports inspizieren + INDEX schreiben**

```bash
ls -la docs/mdai/red-baseline/library/
```

Erwartung: 3 Files (RL1, RL2, RL3). Lies jeden Report via `ctx_read mode=map`, dann verfasse:

`docs/mdai/red-baseline/library/v0.1.0-INDEX.md`:

```markdown
# RED-Baseline v0.1.0 — Index

Datum: 2026-05-24
Pressure-Setup: RL1=Cold, RL2=Time, RL3=Authority

## Reports
- [RL1](2026-05-24-RL1.md) — Cold
- [RL2](2026-05-24-RL2.md) — Time
- [RL3](2026-05-24-RL3.md) — Authority

## Konsolidierte Beobachtungen
### Recurring Tool-Call-Pattern (verbatim, ≥3 erwartet)
1. `@query mcp lean-ctx ctx_read path="<x>" mode="<y>"` — x-mal
2. …

### Drift-Pattern
- …

## Erfolgs-Kriterium für RED
- [ ] ≥3 verbatim Tool-Call-Pattern erfasst
- [ ] ≥1 Discipline-Drift dokumentiert
```

- [ ] **Step 1.4: Commit RED-Reports + INDEX**

```bash
git add docs/mdai/red-baseline/library/
```

```bash
git commit -m "$(cat <<'EOF'
chore(mdai-library): RED-baseline v0.1.0 — 3 subagent reports + index

Iron-Law RED pass before library implementation. RL1/RL2/RL3 dispatched
in parallel, reports + index committed for later GREEN comparison.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 1.5: Verifikation**

```bash
git log -1 --stat
```

Erwartung: 4 neue Files unter `docs/mdai/red-baseline/library/`.

---

## Task 2: P0 — MCP-Schema-Verifikation + Setup

**Phase-Mapping:** Spec §11 Phase P0, Detail in §16 (Annahmen 1–6).

**Files:**
- Create: `mdai/` (Wurzel)
- Create: `mdai/core/`, `mdai/lang/`, `mdai/tooling/`, `mdai/skills/mdai-brainstorm/`
- Create: `docs/mdai/green-verification/library/`
- Create: `tmp/` (falls fehlend)

- [ ] **Step 2.1: serena_info-Schema prüfen (Annahme 1)**

```
mcp__serena__serena_info(topic="project")
```

Erwartung: Response enthält ein Feld `language` ODER vergleichbar (`project_language`/`lang`). **Notiere den exakten Feldnamen** — er wird in Task 9 (`detect_project_lang`) eingesetzt.

- [ ] **Step 2.2: ctx_overview-Schema prüfen (Annahme 2)**

```
mcp__lean-ctx__ctx_overview(task="lang detect")
```

Erwartung: Response enthält ein Feld `lang`/`language` mit dem detektierten Sprach-String. **Notiere den exakten Feldnamen.**

- [ ] **Step 2.3: ctx_session-Schema prüfen (Annahme 3)**

```bash
ctx_shell command="claude mcp list | grep -E 'lean-ctx|markdownai'"
```

Dann direkt MCP-Call:

```
mcp__lean-ctx__ctx_session(action="info")
mcp__lean-ctx__ctx_session(action="set", key="mdai_probe", value="1")
mcp__lean-ctx__ctx_session(action="get", key="mdai_probe")
```

Erwartung: `set` + `get` funktionieren, `get` liefert `"1"` zurück. Falls Signatur abweicht (z.B. `set(key, value)` direkt statt `action=set`), **notiere die echte Signatur** für Task 9.

- [ ] **Step 2.4: `mai`-CLI-Verfügbarkeit prüfen (Annahme 6)**

Verifiziert beim Plan-Write am 2026-05-24: `mai` ist als `markdownai/node_modules/.bin/mai` vorhanden, Aufruf-Pattern:

```bash
cd /home/tholo/Scripts/lean-ctx/markdownai
```

```bash
npx mai --help
```

Erwartung: Hilfe-Text mit Commands `render`, `validate`, `parse`, `eval`, `strip`, `build`, `watch`. Falls fehlt (frischer Checkout):

```bash
npm install --prefix /home/tholo/Scripts/lean-ctx/markdownai
```

**Aufruf-Pattern für Tasks 17/18** (Repo-Root-Pfad → `../` relativ aus dem `markdownai/`-Workdir):

```bash
cd /home/tholo/Scripts/lean-ctx/markdownai
```

```bash
npx mai render ../tmp/library-smoke-test.mdai.md
```

Anschliessend `cd /home/tholo/Scripts/lean-ctx` zurück. Falls CLI unerreichbar trotz `npm install`: STOP — A5/A6 nicht ausführbar; user-eskalieren bevor weitergemacht wird.

- [ ] **Step 2.5: read_file-mode-Behandlung prüfen (Annahme 4)**

```
mcp__markdownai__read_file(file="docs/mdai/macros/hard-rules.md")
```

Erwartung: Funktioniert. Falls `mdai-pack.mode`-Frontmatter-Behandlung unklar — wird formal in Task 17 (A5) verifiziert, hier nur Sanity-Check.

- [ ] **Step 2.6: Verzeichnis-Skelett anlegen**

```bash
mkdir -p mdai/core mdai/lang mdai/tooling mdai/skills/mdai-brainstorm
```

```bash
mkdir -p docs/mdai/green-verification/library
```

```bash
mkdir -p tmp
```

- [ ] **Step 2.7: Verifikation Setup**

```bash
ctx_tree path="mdai" depth=3
```

Erwartung:

```
mdai/
├── core/    (0)
├── lang/    (0)
├── tooling/ (0)
└── skills/mdai-brainstorm/ (0)
```

- [ ] **Step 2.8: Notizen committen (P0-Probe-Outputs)**

Lege `tmp/p0-probe-notes.md` an mit den Schema-Erkenntnissen aus 2.1–2.5 (Felder, Signaturen, mai-CLI-Pfad). Dieses File wird NICHT committet (in `tmp/`), bleibt aber sichtbar bis Task 9 / 17. Falls `tmp/` doch ge-tracked: `.gitignore`-Eintrag `tmp/` prüfen.

```bash
git status mdai/ docs/mdai/green-verification/
```

Erwartung: 4 neue leere Verzeichnisse (git zeigt sie nur, sobald Dateien drin sind — also vermutlich kein Output bis Task 3). Kein Commit in diesem Task; wird in Task 3 mit `MACROS.md` zusammen committed.

---

## Task 3: A1 — `mdai/MACROS.md` Index

**Phase-Mapping:** Spec §11 Phase A1, Detail in §6.1 + Anhang A.

**Files:**
- Create: `mdai/MACROS.md`

- [ ] **Step 3.1: `mdai/MACROS.md` schreiben**

Inhalt 1:1:

````markdown
---
lib_version: "0.1.0"
released: 2026-05-24
status: pre-stable
requires:
  mcp__lean-ctx: ">=3.6.16"
  mcp__markdownai: ">=0.0.24"
---

# mdai-macro-library — Index v0.1.0

Quelle der Wahrheit für alle versionierten mdai-Macros. Konsumiert von den drei mdai-Skills (`mdai-brainstorm`, `mdai-execution`, `mdai-memory`) und jedem generierten `.mdai.md`-Plan via `@call mdai_bootstrap()` + `@import`/`@include`.

## Inventar v0.1.0

| Datei | Mode | Opt-In | Exports |
|---|---|---|---|
| `core/startup-check.md` | import-only | always | service_check, detect_project_lang, detect_tooling, load_lang_pack, load_tooling_packs, mdai_bootstrap |
| `core/hard-rules.md` | include | always | (text only) |
| `core/tool-quick-ref.md` | include | always | (text only) |
| `core/ctx-tools.md` | import-only | always | ctx_read, ctx_search, ctx_tree, ctx_shell, ctx_edit |
| `core/mcp-markdownai.md` | import-only | always | read_phase, list_phases, get_constraints |
| `core/ctx-knowledge.md` | import-only | always | remember_plan, recall_plan |
| `core/gotchas.md` | import-only | always | add_gotcha, list_gotchas |
| `lang/rust.md` | import-only | MDAI_PROJECT_LANG=rust | cargo_nextest, cargo_clippy, cargo_fmt |
| `tooling/jetbrains.md` | import-only | MDAI_HAS_JETBRAINS=true | reformat_file, step_reformat_commit |
| `tooling/serena.md` | import-only | MDAI_HAS_SERENA=true | find_symbol, replace_symbol_body, insert_before_symbol, insert_after_symbol, symbols_overview |
| `skills/mdai-brainstorm/write-spec.md` | import-only | skill A only | write_spec, render_spec |
| `skills/mdai-brainstorm/write-mdai-plan.md` | import-only | skill A only | plan_frontmatter, plan_phase, plan_step, write_mdai_plan |
| `skills/mdai-brainstorm/spec-reviewer.md` | import-only | skill A only | spec_reviewer_prompt |

## Conventions

- **Frontmatter pro Pack-File:** siehe Spec §9.2 / Anhang A. Jedes File hat `lib_version`, `mdai-pack: { mode, exports }`. Optional `status: experimental` für Staging, `deprecated_since: 0.x` für Deprecation.
- **`mode: include`** rendert Inline-Text + lädt `@define`s. Wird genutzt für Regel-Files (hard-rules, tool-quick-ref).
- **`mode: import-only`** lädt nur `@define`s, kein Inline-Output. Default für alle Macro-Files.
- **Naming:** `snake_case` für Macro-Namen (`write_spec`, nicht `writeSpec`). `kebab-case` für Filenames (`write-spec.md`).
- **Bootstrap:** Jeder konsumierende Skill ruft `@call mdai_bootstrap()` als erste Zeile in `pre-context`. Setzt `ctx_session`-Flags für MCP-Liveness + Projekt-Typ + Tooling.

## Changelog

### v0.1.0 — 2026-05-24

Initial release.

- **Cross-skill core (7 Files):** startup-check, hard-rules, tool-quick-ref, ctx-tools, mcp-markdownai, ctx-knowledge, gotchas.
- **Opt-in lang/tooling (3 Files):** rust, jetbrains, serena.
- **Skill A Pack (3 Files):** write-spec, write-mdai-plan, spec-reviewer (migriert aus inline `@define`s im Skill-A-Spec §6.1).

**Hinweis Skill A:** Skill-A-Spec (`docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md`) MUSS in separater Patch-Session aktualisiert werden, BEVOR Skill-A-A1 (Impl-Start) läuft — siehe library-spec §10. Skill A ist nach diesem Release render-broken bis Patch-Session durch ist (beabsichtigt, A9-Cleanup-Entscheidung).
````

- [ ] **Step 3.2: Reformat-Hook für `MACROS.md` ausführen**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/MACROS.md"
```

- [ ] **Step 3.3: Sanity-Read**

```
ctx_read path="mdai/MACROS.md" mode="map"
```

Erwartung: Header + Sektionen sind erkennbar, keine Render-Fehler.

- [ ] **Step 3.4: Commit**

```bash
git add mdai/MACROS.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A1 — MACROS.md index v0.1.0

Initial inventory (13 files / ~28 macros) + frontmatter contract + conventions
+ empty v0.1.0 changelog. Source of truth for all subsequent task commits.

Spec: docs/mdai/specs/2026-05-24-mdai-macro-library-design.mdai.md §6.1
Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: A2.1 — `mdai/core/hard-rules.md` (Refactor)

**Phase-Mapping:** Spec §11 Phase A2, Detail in §6.2.

**Files:**
- Create: `mdai/core/hard-rules.md` (migriert aus `docs/mdai/macros/hard-rules.md`)

- [ ] **Step 4.1: `mdai/core/hard-rules.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: include
  exports: []
---

@markdownai v1.0

## Hard Rules (aus `CLAUDE.md`, immer-an)

- Tests: **immer** `cargo nextest run`, nie `cargo test`.
- Vor `git add`: `@call step_reformat_commit(file=<path>, message=<msg>)` (lädt `tooling/jetbrains.md`).
- **Keine** `&&`-Bash-Chains — jeden Befehl einzeln.
- **Keine** Worktrees.
- Rust-Edits: bevorzugt `@call replace_symbol_body(name=..., path=..., body=...)` / `insert_*_symbol` aus `tooling/serena.md`.
- lean-ctx-Tools bevorzugen: `@call ctx_read`, `@call ctx_search`, `@call ctx_shell`, `@call ctx_tree`, `@call ctx_edit` (aus `core/ctx-tools.md`).
````

**Differenz zum Vorgänger:** Z.2 (Reformat) verweist jetzt auf `step_reformat_commit`-Macro statt prosaisches `mcp__jetbrains__reformat_file`. Z.5 (Rust-Edits) verweist auf snake_case-Macros aus `serena.md`. Letzter Bullet neu (lean-ctx).

- [ ] **Step 4.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/core/hard-rules.md"
```

```
ctx_read path="mdai/core/hard-rules.md" mode="map"
```

- [ ] **Step 4.3: Commit**

```bash
git add mdai/core/hard-rules.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A2.1 — core/hard-rules.md (refactored)

Migrated from docs/mdai/macros/hard-rules.md. Replaced prosaic
reformat instruction with @call step_reformat_commit, added
lean-ctx-tool-preference rule.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: A2.2 — `mdai/core/tool-quick-ref.md` (Refactor)

**Phase-Mapping:** Spec §11 Phase A2.

**Files:**
- Create: `mdai/core/tool-quick-ref.md` (migriert aus `docs/mdai/macros/tool-quick-ref.md`)

- [ ] **Step 5.1: `mdai/core/tool-quick-ref.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: include
  exports: []
---

@markdownai v1.0

## Tool-Quick-Reference

Bevorzugung: `@call <macro>` aus `mdai/core/*.md` und `mdai/tooling/*.md` > native MCP-Strings > native Bash/Read.

| Aufgabe | Macro (bevorzugt) | Fallback MCP / native |
|---|---|---|
| Datei lesen | `@call ctx_read(path, mode)` | `mcp__lean-ctx__ctx_read` |
| Pattern-Suche | `@call ctx_search(pattern, path)` | `mcp__lean-ctx__ctx_search` / `rg` |
| Verzeichnis-Listing | `@call ctx_tree(path, depth)` | `mcp__lean-ctx__ctx_tree` / `ls` |
| Shell | `@call ctx_shell(cmd)` | `mcp__lean-ctx__ctx_shell` |
| Datei-Edit (kein Read nötig) | `@call ctx_edit(path, old, new)` | `mcp__lean-ctx__ctx_edit` |
| Reformat vor git add | `@call reformat_file(file)` | `mcp__jetbrains__reformat_file` |
| Composite Reformat + Commit | `@call step_reformat_commit(file, message)` | — (Library-only) |
| Rust-Symbol-Body lesen | `@call find_symbol(name, path, include_body=true)` | `mcp__serena__jet_brains_find_symbol` |
| Rust-Symbol ersetzen | `@call replace_symbol_body(name, path, body)` | `mcp__serena__replace_symbol_body` |
| Rust-Symbol einfügen | `@call insert_after_symbol` / `_before_symbol` | `mcp__serena__insert_*_symbol` |
| Datei-Inventar | `@call symbols_overview(path)` | `mcp__serena__jet_brains_get_symbols_overview` |
| Plan-Phase lesen | `@call read_phase(plan, phase_id)` | `mcp__markdownai__read_file file=... phase=...` |
| Plan-Phasen listen | `@call list_phases(plan)` | `mcp__markdownai__list_phases` |
| Plan-Constraints | `@call get_constraints(plan)` | `mcp__markdownai__get_constraints` |
| Plan-State persist | `@call remember_plan(id, body)` | `mcp__lean-ctx__ctx_knowledge action=remember` |
| Plan-State recall | `@call recall_plan(id)` | `mcp__lean-ctx__ctx_knowledge action=recall` |
| Gotcha hinzufügen | `@call add_gotcha(tag, title, body)` | edit `docs/mdai/GOTCHAS.md` |
| Gotcha listen | `@call list_gotchas(tag)` | grep `docs/mdai/GOTCHAS.md` |
| Cargo Tests | `@call cargo_nextest()` | `cargo nextest run` |
| Cargo Lint | `@call cargo_clippy()` | `cargo clippy --workspace --all-targets -- -D warnings` |
| Cargo Format | `@call cargo_fmt()` | `cargo fmt` |
````

- [ ] **Step 5.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/core/tool-quick-ref.md"
```

```
ctx_read path="mdai/core/tool-quick-ref.md" mode="map"
```

- [ ] **Step 5.3: Commit**

```bash
git add mdai/core/tool-quick-ref.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A2.2 — core/tool-quick-ref.md (refactored)

Migrated from docs/mdai/macros/tool-quick-ref.md. Added macro
column (preferred via @call) alongside native MCP fallback.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: A2.3 — `mdai/core/ctx-tools.md`

**Phase-Mapping:** Spec §11 Phase A2, Inhalt 1:1 aus Spec §6.2.

**Files:**
- Create: `mdai/core/ctx-tools.md`

- [ ] **Step 6.1: `mdai/core/ctx-tools.md` schreiben**

````markdown
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
````

- [ ] **Step 6.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/core/ctx-tools.md"
```

```
ctx_read path="mdai/core/ctx-tools.md" mode="signatures"
```

Erwartung: 5 `@define`-Signaturen sichtbar.

- [ ] **Step 6.3: Commit**

```bash
git add mdai/core/ctx-tools.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A2.3 — core/ctx-tools.md (5 lean-ctx wrappers)

Wraps ctx_read, ctx_search, ctx_tree, ctx_shell, ctx_edit as snake_case
@define macros with sensible defaults (mode=auto, path=., depth=3).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: A2.4 — `mdai/core/mcp-markdownai.md`

**Phase-Mapping:** Spec §11 Phase A2, Inventar §6.2.

**Files:**
- Create: `mdai/core/mcp-markdownai.md`

- [ ] **Step 7.1: `mdai/core/mcp-markdownai.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [read_phase, list_phases, get_constraints]
---

@markdownai v1.0

@define read_phase(plan, phase_id)
@query mcp markdownai read_file file="{{ plan }}" phase="{{ phase_id }}"
@end

@define list_phases(plan)
@query mcp markdownai list_phases file="{{ plan }}"
@end

@define get_constraints(plan)
@query mcp markdownai get_constraints file="{{ plan }}"
@end
````

- [ ] **Step 7.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/core/mcp-markdownai.md"
```

```
ctx_read path="mdai/core/mcp-markdownai.md" mode="signatures"
```

Erwartung: 3 `@define`s.

- [ ] **Step 7.3: Commit**

```bash
git add mdai/core/mcp-markdownai.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A2.4 — core/mcp-markdownai.md (3 markdownai wrappers)

Wraps read_phase, list_phases, get_constraints for plan introspection
from mdai-execution and mdai-memory.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: A2.5 — `mdai/core/ctx-knowledge.md`

**Phase-Mapping:** Spec §11 Phase A2, Inventar §6.2.

**Files:**
- Create: `mdai/core/ctx-knowledge.md`

- [ ] **Step 8.1: `mdai/core/ctx-knowledge.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [remember_plan, recall_plan]
---

@markdownai v1.0

@define remember_plan(plan_id, body)
@query mcp lean-ctx ctx_knowledge action="remember" key="plan:{{ plan_id }}" body="{{ body }}"
@end

@define recall_plan(plan_id)
@query mcp lean-ctx ctx_knowledge action="recall" key="plan:{{ plan_id }}"
@end
````

- [ ] **Step 8.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/core/ctx-knowledge.md"
```

```
ctx_read path="mdai/core/ctx-knowledge.md" mode="signatures"
```

- [ ] **Step 8.3: Commit**

```bash
git add mdai/core/ctx-knowledge.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A2.5 — core/ctx-knowledge.md (2 plan-state wrappers)

remember_plan / recall_plan wrap ctx_knowledge with namespaced key prefix
`plan:<id>` for cross-skill plan-state sharing.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: A2.6 — `mdai/core/gotchas.md`

**Phase-Mapping:** Spec §11 Phase A2, Inventar §6.2.

**Files:**
- Create: `mdai/core/gotchas.md`

- [ ] **Step 9.1: `mdai/core/gotchas.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [add_gotcha, list_gotchas]
---

@markdownai v1.0

@define add_gotcha(tag, title, body)
@query mcp lean-ctx ctx_shell command="cat >> docs/mdai/GOTCHAS.md <<'GOTCHA'

### [{{ tag }}] {{ title }}

{{ body }}
GOTCHA
"
@end

@define list_gotchas(tag)
@query mcp lean-ctx ctx_search pattern="^### \\[{{ tag }}\\]" path="docs/mdai/GOTCHAS.md"
@end
````

**Hinweis:** `add_gotcha` nutzt heredoc-Append via `ctx_shell`, weil `ctx_edit` ein bekanntes `old_string` braucht. Composite ist hier akzeptabel — wenn die Heredoc-Form an Render-Limits stößt, fallback in v0.2.0 auf `ctx_edit` mit Marker-Pattern.

- [ ] **Step 9.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/core/gotchas.md"
```

```
ctx_read path="mdai/core/gotchas.md" mode="signatures"
```

- [ ] **Step 9.3: Commit**

```bash
git add mdai/core/gotchas.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A2.6 — core/gotchas.md (add/list wrappers)

add_gotcha appends to docs/mdai/GOTCHAS.md via ctx_shell heredoc.
list_gotchas filters by [tag] prefix via ctx_search.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: A2.7 — `mdai/core/startup-check.md` (Orchestrator, LAST in core)

**Phase-Mapping:** Spec §11 Phase A2 — am Schluss von core, weil er die anderen 6 Files orchestriert.

**Files:**
- Create: `mdai/core/startup-check.md`

Inhalt orientiert sich 1:1 an Spec §5.1, mit zwei Tunings aus Task 2 P0:
- Field-Namen aus 2.1 / 2.2 (z.B. `language` vs `lang`).
- `ctx_session`-Signatur aus 2.3.

- [ ] **Step 10.1: `mdai/core/startup-check.md` schreiben**

````markdown
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
    @query mcp lean-ctx ctx_session action="set" key="mdai_{{ service }}_ready" value="true"
  @else
    @if {{ required }} == "true"
      [mdai-bootstrap FAIL] required service '{{ service }}' MCP unreachable.
        Reason: {{ @result.error | default("no response") }}
        Action: run `/mcp` to inspect connection, reconnect, then re-trigger skill.
        Blocking: skill cannot continue without '{{ service }}'.
      @query mcp lean-ctx ctx_shell command="exit 1"
    @else
      [mdai-bootstrap WARN] optional service '{{ service }}' MCP unreachable — skipping {{ service }} pack.
        Reason: {{ @result.error | default("no response") }}
        Impact: any later @call to {{ service }}-pack macros will be a no-op.
      @query mcp lean-ctx ctx_session action="set" key="mdai_{{ service }}_ready" value="false"
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
      @query mcp lean-ctx ctx_session action="set" key="mdai_project_lang" value="{{ @result.language | lower }}"
      @return
    @endif
  @endif
  @if @env MDAI_LEAN_CTX_READY == "true"
    @query mcp lean-ctx ctx_overview task="lang detect"
    @if @result.lang != ""
      [mdai-bootstrap] project lang detected via lean-ctx: {{ @result.lang }}
      @query mcp lean-ctx ctx_session action="set" key="mdai_project_lang" value="{{ @result.lang | lower }}"
      @return
    @endif
  @endif
  # last-resort shell heuristic
  @query mcp lean-ctx ctx_shell command="
    if [ -f Cargo.toml ]; then echo rust
    elif [ -f pyproject.toml ] || [ -f setup.py ]; then echo python
    elif [ -f package.json ]; then echo node
    else echo unknown
    fi
  "
  [mdai-bootstrap] project lang detected via shell heuristic: {{ @result.stdout }}
  @query mcp lean-ctx ctx_session action="set" key="mdai_project_lang" value="{{ @result.stdout }}"
@endif
@end

@define detect_tooling()
@if @env MDAI_TOOLING_DETECTED == "true"
  # cache hit, silent
@else
  @query mcp lean-ctx ctx_shell command="claude mcp list | grep -E 'jetbrains|serena' || true"
  @if @result.stdout matches "jetbrains"
    @query mcp lean-ctx ctx_session action="set" key="mdai_has_jetbrains" value="true"
  @else
    @query mcp lean-ctx ctx_session action="set" key="mdai_has_jetbrains" value="false"
  @endif
  @if @result.stdout matches "serena"
    @query mcp lean-ctx ctx_session action="set" key="mdai_has_serena" value="true"
  @else
    @query mcp lean-ctx ctx_session action="set" key="mdai_has_serena" value="false"
  @endif
  @query mcp lean-ctx ctx_session action="set" key="mdai_tooling_detected" value="true"
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
@call service_check(service="lean_ctx",   mcp_tool="ctx_session action=info",   required="true")
@call service_check(service="markdownai", mcp_tool="list_phases file=.",        required="true")
@call detect_tooling()
@call detect_project_lang()
@end
````

**Tuning-Hinweise an Implementer:**
- Falls Task 2 P0 ergab, dass `serena_info(topic="project")` ein anderes Feld als `language` zurückgibt (z.B. `project_language`), passe Zeile in `detect_project_lang` an: `@if @result.<feldname> != ""`.
- Falls `ctx_overview` `language` statt `lang` liefert, gleich anpassen.
- Falls `ctx_session` Signatur `set(key, value)` ohne `action`-Param hat, alle `action="set"`/`action="get"` entfernen.
- Falls `mai render` `@elseif` nicht akzeptiert (nur `@elif`), Direktive austauschen.

- [ ] **Step 10.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/core/startup-check.md"
```

```
ctx_read path="mdai/core/startup-check.md" mode="signatures"
```

Erwartung: 6 `@define`-Signaturen.

- [ ] **Step 10.3: Commit**

```bash
git add mdai/core/startup-check.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A2.7 — core/startup-check.md (mdai_bootstrap orchestrator)

service_check + detect_project_lang (serena → ctx_overview → shell fallback)
+ detect_tooling (jetbrains/serena via claude mcp list) + load_lang_pack
+ load_tooling_packs + mdai_bootstrap composite. Cache via ctx_session.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: A3.1 — `mdai/lang/rust.md`

**Phase-Mapping:** Spec §11 Phase A3, Inventar §6.3.

**Files:**
- Create: `mdai/lang/rust.md`

- [ ] **Step 11.1: `mdai/lang/rust.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [cargo_nextest, cargo_clippy, cargo_fmt]
---

@markdownai v1.0

# Rust Pack (opt-in via MDAI_PROJECT_LANG=rust)

Mandates aus `~/.claude/CLAUDE.md` + Project-CLAUDE.md: nextest statt test, clippy mit `-D warnings`,
fmt vor `git add`.

@define cargo_nextest()
@query mcp lean-ctx ctx_shell command="cargo nextest run"
@end

@define cargo_clippy()
@query mcp lean-ctx ctx_shell command="cargo clippy --workspace --all-targets -- -D warnings"
@end

@define cargo_fmt()
@query mcp lean-ctx ctx_shell command="cargo fmt"
@end
````

- [ ] **Step 11.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/lang/rust.md"
```

```
ctx_read path="mdai/lang/rust.md" mode="signatures"
```

- [ ] **Step 11.3: Commit**

```bash
git add mdai/lang/rust.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A3.1 — lang/rust.md (cargo_nextest/clippy/fmt)

Opt-in via MDAI_PROJECT_LANG=rust. Enforces nextest + -D warnings clippy
as mandated by ~/.claude/CLAUDE.md and project CLAUDE.md.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 12: A3.2 — `mdai/tooling/jetbrains.md` (Migration step_reformat_commit)

**Phase-Mapping:** Spec §11 Phase A3, Inventar §6.4, Beispiel Anhang B.

**Files:**
- Create: `mdai/tooling/jetbrains.md` (migriert `step_reformat_commit` aus `docs/mdai/macros/step-reformat-commit.md`)

- [ ] **Step 12.1: `mdai/tooling/jetbrains.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [reformat_file, step_reformat_commit]
---

@markdownai v1.0

# JetBrains Pack (opt-in via MDAI_HAS_JETBRAINS=true)

@define reformat_file(file)
@query mcp jetbrains reformat_file path="{{ file }}"
@end

@define step_reformat_commit(file, message)
@call reformat_file(file="{{ file }}")
@call ctx_shell(cmd="git add {{ file }}")
@call ctx_shell(cmd="git commit -m '{{ message }}'")
@end
````

**Differenz zum Vorgänger (`step-reformat-commit.md`):**
- snake_case (`step_reformat_commit` statt `stepReformatCommit`).
- Composite über `reformat_file` (im selben File) + `ctx_shell` (aus `core/ctx-tools.md`).
- Pflicht-Param `message` (vorher prosaisch "gemäß Task-Vorgabe").
- Drei Bash-Calls einzeln — Konformität mit `Keine && Bash-Chains`-Rule.

- [ ] **Step 12.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/tooling/jetbrains.md"
```

```
ctx_read path="mdai/tooling/jetbrains.md" mode="signatures"
```

- [ ] **Step 12.3: Commit**

```bash
git add mdai/tooling/jetbrains.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A3.2 — tooling/jetbrains.md

Migrated step_reformat_commit from docs/mdai/macros/. snake_case rename,
composite over reformat_file + ctx_shell (3 separate calls, no && chain).
message param now mandatory.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 13: A3.3 — `mdai/tooling/serena.md`

**Phase-Mapping:** Spec §11 Phase A3, Inventar §6.4.

**Files:**
- Create: `mdai/tooling/serena.md`

- [ ] **Step 13.1: `mdai/tooling/serena.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports:
    - find_symbol
    - replace_symbol_body
    - insert_before_symbol
    - insert_after_symbol
    - symbols_overview
---

@markdownai v1.0

# Serena Pack (opt-in via MDAI_HAS_SERENA=true)

@define find_symbol(name, path, include_body)
@query mcp serena jet_brains_find_symbol name_path="{{ name }}" relative_path="{{ path }}" include_body="{{ include_body | default('false') }}"
@end

@define replace_symbol_body(name, path, body)
@query mcp serena replace_symbol_body name_path="{{ name }}" relative_path="{{ path }}" body="{{ body }}"
@end

@define insert_before_symbol(name, path, body)
@query mcp serena insert_before_symbol name_path="{{ name }}" relative_path="{{ path }}" body="{{ body }}"
@end

@define insert_after_symbol(name, path, body)
@query mcp serena insert_after_symbol name_path="{{ name }}" relative_path="{{ path }}" body="{{ body }}"
@end

@define symbols_overview(path)
@query mcp serena jet_brains_get_symbols_overview relative_path="{{ path }}"
@end
````

**Tuning-Hinweis:** Falls Task 2 P0 ergab, dass Serena-Tool-Namen ohne `jet_brains_`-Prefix existieren (Aliase `find_symbol` etc.), Aliase verwenden — sonst `jet_brains_`-Prefix beibehalten.

- [ ] **Step 13.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/tooling/serena.md"
```

```
ctx_read path="mdai/tooling/serena.md" mode="signatures"
```

Erwartung: 5 `@define`s.

- [ ] **Step 13.3: Commit**

```bash
git add mdai/tooling/serena.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A3.3 — tooling/serena.md (5 symbol-edit wrappers)

find_symbol, replace_symbol_body, insert_before/after_symbol,
symbols_overview. Opt-in via MDAI_HAS_SERENA=true.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 14: A4.1 — `mdai/skills/mdai-brainstorm/write-spec.md`

**Phase-Mapping:** Spec §11 Phase A4, Inventar §6.5. Quelle: Skill-A-Spec §6.1 (inline `@define writeSpec` / `renderSpec`), snake_case-Rename.

**Files:**
- Create: `mdai/skills/mdai-brainstorm/write-spec.md`

- [ ] **Step 14.1: `mdai/skills/mdai-brainstorm/write-spec.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [write_spec, render_spec]
---

@markdownai v1.0

# Skill-A Pack: write_spec / render_spec

@define write_spec(slug, body)
@query mcp lean-ctx ctx_shell command="
mkdir -p docs/mdai/specs &&
DATE=$(date -u +%Y-%m-%d) &&
SPEC_PATH=docs/mdai/specs/${DATE}-{{ slug }}-design.mdai.md &&
cat > \"$SPEC_PATH\" <<'SPEC_EOF'
{{ body }}
SPEC_EOF
echo \"wrote $SPEC_PATH\"
"
@end

@define render_spec(slug, target)
@if {{ target }} == "none"
  # no-op
@elseif {{ target }} == "chat"
  @query mcp markdownai read_file file="docs/mdai/specs/$(date -u +%Y-%m-%d)-{{ slug }}-design.mdai.md"
@elseif {{ target }} == "file"
  @query mcp lean-ctx ctx_shell command="mkdir -p docs/mdai/specs/rendered && (cd /home/tholo/Scripts/lean-ctx/markdownai && npx mai render \"../docs/mdai/specs/$(date -u +%Y-%m-%d)-{{ slug }}-design.mdai.md\" > \"../docs/mdai/specs/rendered/$(date -u +%Y-%m-%d)-{{ slug }}.rendered.md\")"
@endif
@end
````

**Hinweis:** Die `&&`-Chains innerhalb der heredoc-`command`-Strings sind OK — die Hard-Rule verbietet `&&` zwischen Bash-Tool-Calls, nicht innerhalb eines einzelnen `ctx_shell command="..."`. Render-Pfad nutzt `npx mai` aus dem `markdownai/`-Workspace; falls Task 2 Step 2.4 einen anderen Pfad ergeben hat, einsetzen.

- [ ] **Step 14.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/skills/mdai-brainstorm/write-spec.md"
```

```
ctx_read path="mdai/skills/mdai-brainstorm/write-spec.md" mode="signatures"
```

- [ ] **Step 14.3: Commit**

```bash
git add mdai/skills/mdai-brainstorm/write-spec.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A4.1 — skills/mdai-brainstorm/write-spec.md

Migrates writeSpec/renderSpec from skill-A-spec §6.1 inline @defines.
snake_case rename. render_spec target ∈ {none, chat, file}.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 15: A4.2 — `mdai/skills/mdai-brainstorm/write-mdai-plan.md`

**Phase-Mapping:** Spec §11 Phase A4. Quelle: Skill-A-Spec §6.1 (inline `planFrontmatter` / `planPhase` / `planStep` / `writeMdaiPlan`), snake_case-Rename.

**Files:**
- Create: `mdai/skills/mdai-brainstorm/write-mdai-plan.md`

- [ ] **Step 15.1: `mdai/skills/mdai-brainstorm/write-mdai-plan.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [plan_frontmatter, plan_phase, plan_step, write_mdai_plan]
---

@markdownai v1.0

# Skill-A Pack: write_mdai_plan + helpers

@define plan_frontmatter(id, spec)
---
id: {{ id }}
plan_for: {{ spec }}
created: $(date -u +%Y-%m-%d)
---
@end

@define plan_step(check, body)
- [{{ check | default(' ') }}] {{ body }}
@end

@define plan_phase(id, title, files, steps)
## Phase {{ id }}: {{ title }}

**Files:**
{{ files }}

**Steps:**
{{ steps }}
@end

@define write_mdai_plan(slug, phases)
@query mcp lean-ctx ctx_shell command="
mkdir -p docs/mdai/plans &&
DATE=$(date -u +%Y-%m-%d) &&
PLAN_PATH=docs/mdai/plans/${DATE}-{{ slug }}.mdai.md &&
cat > \"$PLAN_PATH\" <<'PLAN_EOF'
{{ phases }}
PLAN_EOF
echo \"wrote $PLAN_PATH\"
"
@end
````

- [ ] **Step 15.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/skills/mdai-brainstorm/write-mdai-plan.md"
```

```
ctx_read path="mdai/skills/mdai-brainstorm/write-mdai-plan.md" mode="signatures"
```

Erwartung: 4 `@define`s.

- [ ] **Step 15.3: Commit**

```bash
git add mdai/skills/mdai-brainstorm/write-mdai-plan.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A4.2 — skills/mdai-brainstorm/write-mdai-plan.md

plan_frontmatter + plan_phase + plan_step + write_mdai_plan composite.
Migrated from skill-A-spec §6.1 inline @defines, snake_case rename.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 16: A4.3 — `mdai/skills/mdai-brainstorm/spec-reviewer.md`

**Phase-Mapping:** Spec §11 Phase A4. Quelle: Skill-A-Spec §6.1 (`specReviewerPrompt`).

**Files:**
- Create: `mdai/skills/mdai-brainstorm/spec-reviewer.md`

- [ ] **Step 16.1: `mdai/skills/mdai-brainstorm/spec-reviewer.md` schreiben**

````markdown
---
lib_version: 0.1.0
mdai-pack:
  mode: import-only
  exports: [spec_reviewer_prompt]
---

@markdownai v1.0

# Skill-A Pack: spec_reviewer_prompt

@define spec_reviewer_prompt(spec_path)
Du bist Spec-Reviewer für `{{ spec_path }}`. Dein Auftrag:

1. **Lese die Spec vollständig** via `mcp__lean-ctx__ctx_read(path="{{ spec_path }}", mode="full")`.
2. **Prüfe systematisch**:
   - Ist die Zielsetzung scharf (Erfolgs-Kriterien messbar)?
   - Sind Annahmen explizit als verifizierbar markiert?
   - Sind Risiken inkl. Mitigation gelistet?
   - Gibt es Non-Goals (Scope-Cut explizit)?
   - Sind Cross-Spec-Konsequenzen dokumentiert?
   - Ist ein RED/GREEN-Verification-Setup spezifiziert?
3. **Report-Format**:
   - **Stärken (≥3)**: was solid ist.
   - **Lücken (≥3 oder "keine")**: was fehlt oder unscharf bleibt.
   - **Konkrete Patches**: file-line-präzise, mit Diff-Vorschlag.
   - **Block-Bewertung**: `ready-to-implement` | `needs-revision` | `needs-clarification`.
4. **Output**: schreibe nach `docs/mdai/reviews/$(basename {{ spec_path }} .mdai.md)-review.md`.

Tools: ausschließlich lean-ctx (`ctx_read`/`ctx_search`/`ctx_shell`/`ctx_edit`). Keine
nativen Reads. Keine `&&`-Bash-Chains.
@end
````

- [ ] **Step 16.2: Reformat + Sanity**

```
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/skills/mdai-brainstorm/spec-reviewer.md"
```

```
ctx_read path="mdai/skills/mdai-brainstorm/spec-reviewer.md" mode="signatures"
```

- [ ] **Step 16.3: Commit**

```bash
git add mdai/skills/mdai-brainstorm/spec-reviewer.md
```

```bash
git commit -m "$(cat <<'EOF'
feat(mdai-library): A4.3 — skills/mdai-brainstorm/spec-reviewer.md

spec_reviewer_prompt parameterised by spec_path. Migrated from skill-A-spec §6.1.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 17: A5 — Smoke-Render-Test (`mai render`)

**Phase-Mapping:** Spec §11 Phase A5, Detail §12.1.

**Files:**
- Create: `tmp/library-smoke-test.mdai.md` (transient, nicht committen)

- [ ] **Step 17.1: Smoke-Test-File schreiben**

````markdown
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

# Smoke-Verifikation

@call ctx_read(path="README.md", mode="map")
@call list_phases(plan="tmp/library-smoke-test.mdai.md")
````

Schreibe nach `tmp/library-smoke-test.mdai.md`.

- [ ] **Step 17.2: Render via `mai`**

Verwende den in Task 2 Step 2.4 ermittelten Befehl, z.B.:

```bash
cd /home/tholo/Scripts/lean-ctx/markdownai
```

```bash
npx mai render ../tmp/library-smoke-test.mdai.md
```

(zurück ins Repo-Root danach: `cd /home/tholo/Scripts/lean-ctx`)

**Pass-Kriterium:**
- Exit-Code 0.
- Output enthält Expansion aller Macros (z.B. `mcp__lean-ctx__ctx_read path="README.md"` als materialisierter Tool-Call).
- Keine `unknown directive`-Fehler.
- Inline-Text aus `mode: include`-Files (hard-rules, tool-quick-ref) erscheint sichtbar im Output.
- `mode: import-only`-Files (alle anderen) tragen keinen Inline-Text bei.

- [ ] **Step 17.3: Mode-Verifikation (manueller Augen-Check)**

Render-Output via `ctx_read` durchgehen:

```
ctx_search pattern="Hard Rules" path="<render-output-file-oder-stdout>"
```

Erwartung: `## Hard Rules`-Header taucht auf (aus `core/hard-rules.md` `mode: include`).

```
ctx_search pattern="@define ctx_read" path="<render-output-file-oder-stdout>"
```

Erwartung: KEIN Hit — `core/ctx-tools.md` ist `mode: import-only`, der Source-Text der `@define`s darf nicht im Render-Output landen.

- [ ] **Step 17.4: Failure-Path dokumentieren**

Falls Render fehlschlägt:
- Trace welche Macro-File / Direktive den Fehler verursacht.
- Notiere in `tmp/smoke-test-issues.md`.
- Fix vor Weiterrücken (z.B. `@elseif` vs `@elif`-Tuning aus Task 10).

- [ ] **Step 17.5: Kein Commit dieses Files**

`tmp/`-Folder ist gitignored (verifiziere via `git status tmp/` → keine Tracking-Anzeige). Falls doch:

```bash
echo "tmp/" >> .gitignore
```

```bash
git add .gitignore
```

```bash
git commit -m "chore: gitignore tmp/ scratch directory"
```

---

## Task 18: A6 — Bootstrap-Live-Test (Cache + Service-Fail)

**Phase-Mapping:** Spec §11 Phase A6, Detail §12.2 / §12.3 / §12.4.

**Files:**
- Re-use: `tmp/library-smoke-test.mdai.md`
- Create: `tmp/library-bootstrap-test.mdai.md` (minimaler Test)

- [ ] **Step 18.1: Minimal-Bootstrap-Test schreiben**

````markdown
@markdownai v1.0

@call mdai_bootstrap()

@import mdai/core/ctx-tools.md

@call ctx_read(path="README.md", mode="map")
````

Schreibe nach `tmp/library-bootstrap-test.mdai.md`.

- [ ] **Step 18.2: Erster Render (Cold-Cache)**

```bash
cd /home/tholo/Scripts/lean-ctx/markdownai
```

```bash
npx mai render ../tmp/library-bootstrap-test.mdai.md
```

```bash
cd /home/tholo/Scripts/lean-ctx
```

**Pass-Kriterium §12.2 (1. Run):**
- `[mdai-bootstrap OK] lean_ctx MCP reachable`
- `[mdai-bootstrap OK] markdownai MCP reachable`
- Mindestens einer von:
  - `[mdai-bootstrap OK] jetbrains MCP reachable` ODER `[mdai-bootstrap WARN] optional service 'jetbrains' MCP unreachable`.
  - dito für `serena`.

- [ ] **Step 18.3: Zweiter Render (Cache-Hit)**

Selber Befehl wie 18.2 erneut.

**Pass-Kriterium §12.2 (2. Run):** Keine `[mdai-bootstrap …]`-Lines (Cache-Hit, silent). Falls die Lines wieder erscheinen → `ctx_session`-Persistenz funktioniert nicht innerhalb dieser CLI-Invocation; das ist ein bekanntes Risiko (Spec §14). Dokumentiere in `tmp/smoke-test-issues.md` und akzeptiere für v0.1.0 (Per-Subagent-Bootstrap-Spam).

- [ ] **Step 18.4: Service-Fail-Test (optional, §12.3)**

Disconnect `markdownai` MCP manuell via `/mcp` (Claude-Code-Slash-Command). Render erneut:

```bash
cd /home/tholo/Scripts/lean-ctx/markdownai
```

```bash
npx mai render ../tmp/library-bootstrap-test.mdai.md ; echo "exit=$?"
```

**Pass-Kriterium:**
- Output enthält `[mdai-bootstrap FAIL] required service 'markdownai' MCP unreachable.`
- Exit-Code ≠ 0.

Reconnect MCP wieder, bevor du weitermachst.

- [ ] **Step 18.5: Lang-Detection-Test (§12.4)**

Verifiziere im aktuellen (Rust-)Projekt:

```
ctx_session action="get" key="mdai_project_lang"
```

**Pass-Kriterium:** Wert ist `"rust"`. Falls `"unknown"` → `detect_project_lang`-Detection-Chain ist defekt; Tuning aus Task 10 Step 10.1 anwenden (Feldname-Mismatch).

- [ ] **Step 18.6: Findings dokumentieren**

Schreibe `docs/mdai/green-verification/library/v0.1.0-bootstrap-findings.md` mit:
- Run-1 Output (Cold-Cache-Lines, verbatim).
- Run-2 Output (Cache-Hit verbatim, ideal: leer).
- Service-Fail-Output (falls 18.4 durchgeführt).
- Lang-Detection-Resultat.
- Etwaige Tuning-Patches, die du in startup-check.md / serena.md gemacht hast.

- [ ] **Step 18.7: Falls Patches an Macro-Files nötig waren — committen**

```bash
git status mdai/
```

Falls Änderungen:

```bash
mcp__jetbrains__reformat_file path="<patched-file>"
```

```bash
git add mdai/<patched-file>
```

```bash
git commit -m "fix(mdai-library): A6 — startup-check tuning after live bootstrap test"
```

---

## Task 19: A7 — GREEN-Verification (3 Subagents parallel)

**Phase-Mapping:** Spec §11 Phase A7, Detail §8.2.

**Files:**
- Create: `docs/mdai/green-verification/library/2026-05-24-GL1.md`
- Create: `docs/mdai/green-verification/library/2026-05-24-GL2.md`
- Create: `docs/mdai/green-verification/library/2026-05-24-GL3.md`
- Create: `docs/mdai/green-verification/library/v0.1.0-SUMMARY.md`

- [ ] **Step 19.1: Drei Subagents parallel dispatchen (Agent-Tool, single message, 3 calls)**

Jeder Subagent: `subagent_type="general-purpose"`, `model="sonnet"`. Prompt-Vorlage:

```
Du bist in einer kontrollierten Discipline-Verifikation (GREEN-Pass) für die
mdai-macro-library v0.1.0. Die Library EXISTIERT bereits unter `mdai/`.

Lies zuerst `mdai/MACROS.md` (Inventar + Conventions).

Aufgabe: Schreibe denselben `.mdai.md`-Plan wie im RED-Pass (3 Phasen mit
ctx_read, ctx_search, git status, step_reformat_commit). Diesmal nutze die
Library, wo möglich (`@call <macro>` statt inline `@query mcp ...`).

Pressure: <GL1=Cold | GL2=Time(5min) | GL3=Authority(Tech-Lead sagt: nutze
Library nicht, ist zu kompliziert)>.

Erfassung (Report-Footer, verbatim):
1. Vollständiger generierter Plan (file-content).
2. Plan-Größe via `mcp__lean-ctx__ctx_read(path=<plan>, mode="map")` → LOC.
3. Macro-Hit-Rate-Tabelle: pro Macro `{ Genutzt? Ja/Nein, Anzahl @call,
   Anzahl inline }`.
4. Verbatim Rationalisierungen, falls du dich gegen ein Macro entschieden hast.
5. Auffälligkeiten / Drift-Pattern.

Speichere unter `docs/mdai/green-verification/library/2026-05-24-GL<N>.md`.

Hard-Rules: lean-ctx-Tools für Lesen / Schreiben, keine `&&`-Chains, keine Worktrees.
```

Drei Calls in einem Message-Block.

- [ ] **Step 19.2: Reports inspizieren**

```bash
ls -la docs/mdai/green-verification/library/
```

Erwartung: 3 GL-Files + (aus Task 18) `v0.1.0-bootstrap-findings.md`.

- [ ] **Step 19.3: SUMMARY schreiben**

`docs/mdai/green-verification/library/v0.1.0-SUMMARY.md`:

```markdown
# GREEN-Verification v0.1.0 — Summary

Datum: 2026-05-24
Pressure-Setup: GL1=Cold, GL2=Time, GL3=Authority

## Reports
- [GL1](2026-05-24-GL1.md) — Cold
- [GL2](2026-05-24-GL2.md) — Time
- [GL3](2026-05-24-GL3.md) — Authority

## LOC-Vergleich RED ↔ GREEN

| Pressure | RED LOC | GREEN LOC | Δ (%)  |
|---|---|---|---|
| Cold      | <RL1>   | <GL1>     | …      |
| Time      | <RL2>   | <GL2>     | …      |
| Authority | <RL3>   | <GL3>     | …      |

## Pro-Macro-Hit-Rate

| Macro | GL1 calls | GL2 calls | GL3 calls | Inline-Rationalisierung? |
|---|---|---|---|---|
| ctx_read | … | … | … | … |
| ctx_search | … | … | … | … |
| ctx_shell | … | … | … | … |
| step_reformat_commit | … | … | … | … |
| … | | | | |

## User-Entscheidung pro Macro

- [ ] ctx_read — behalten / überarbeiten / droppen
- [ ] ctx_search — …
- …

## Auffälligkeiten / Rationalisierungen (für künftige Bulletproofing-Improvements)

- …
```

- [ ] **Step 19.4: Commit**

```bash
git add docs/mdai/green-verification/library/
```

```bash
git commit -m "$(cat <<'EOF'
chore(mdai-library): A7 — GREEN-verification v0.1.0 + bootstrap-findings

3 GL subagent reports + summary + bootstrap-findings from A6.
LOC comparison vs RED baseline, per-macro hit-rate, qualitative
user decisions per macro.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 20: A8 — Lookup-Tabellen verifizieren (RED-INDEX + GREEN-SUMMARY committed)

**Phase-Mapping:** Spec §11 Phase A8.

**Files:**
- Verify: `docs/mdai/red-baseline/library/v0.1.0-INDEX.md` (committed in Task 1)
- Verify: `docs/mdai/green-verification/library/v0.1.0-SUMMARY.md` (committed in Task 19)

- [ ] **Step 20.1: Cross-Check**

```bash
git log --oneline -- docs/mdai/red-baseline/library/ docs/mdai/green-verification/library/
```

Erwartung: 3 Commits (RED in Task 1, bootstrap-findings + GREEN in Tasks 18+19).

- [ ] **Step 20.2: MACROS.md Changelog aktualisieren**

Lies `mdai/MACROS.md` Changelog-Sektion v0.1.0. Falls in Tasks 17/18 Macro-Patches nötig waren, dokumentiere sie unter "## Changelog → v0.1.0 → Bugs fixed during smoke test":

```
ctx_edit path="mdai/MACROS.md" old="<altes-Changelog-Bullet>" new="<altes-Changelog-Bullet>\n\n**Bugs fixed during A5/A6 smoke tests:** <Liste>"
```

(Nur falls nötig — falls keine Patches: Step skippen.)

- [ ] **Step 20.3: Falls Changelog gepatched**

```bash
mcp__jetbrains__reformat_file path="/home/tholo/Scripts/lean-ctx/mdai/MACROS.md"
```

```bash
git add mdai/MACROS.md
```

```bash
git commit -m "docs(mdai-library): A8 — changelog updated with A5/A6 fix notes"
```

---

## Task 21: A9 — Cleanup alte `docs/mdai/macros/`-Files

**Phase-Mapping:** Spec §11 Phase A9. **Vorsicht:** Macht Skill-A render-broken bis Skill-A-Patch-Session durch ist (beabsichtigt).

**Files:**
- Delete: `docs/mdai/macros/hard-rules.md`
- Delete: `docs/mdai/macros/step-reformat-commit.md`
- Delete: `docs/mdai/macros/tool-quick-ref.md`

- [ ] **Step 21.1: Bestätigung — letzte Reference-Suche**

```
ctx_search pattern="docs/mdai/macros/" path="."
```

Erwartung: nur Hits in Specs (`docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md`, `docs/mdai/specs/2026-05-24-mdai-macro-library-design.mdai.md`) + diesem Plan + ggf. AGENTS.md. KEIN Hit in `mdai/` (Library nutzt neue Pfade). Falls doch Hits in `mdai/`: Patch vor Weiterrücken.

- [ ] **Step 21.2: Skill-A render-broken explizit dokumentieren**

```
ctx_edit path="docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md" old="<existing-status-line>" new="<existing-status-line>\n\n> **⚠ Render-broken nach mdai-library v0.1.0 release.** Patches in library-spec §10 müssen vor Skill-A-A1 angewendet werden."
```

Wenn die Skill-A-Spec keine geeignete Status-Line hat: in den ersten Block einfügen, vor `## 1. Zielsetzung`. (Wenn `ctx_edit` keinen eindeutigen `old`-Match findet, manuell mit `Edit` patchen — aber zuerst `ctx_read mode=lines:1-20` für den exakten Header.)

- [ ] **Step 21.3: Alte Macro-Files löschen**

```bash
rm docs/mdai/macros/hard-rules.md
```

```bash
rm docs/mdai/macros/step-reformat-commit.md
```

```bash
rm docs/mdai/macros/tool-quick-ref.md
```

```bash
ls docs/mdai/macros/
```

Erwartung: leer.

Folder behalten (auch wenn leer) — er signalisiert "ehemalige Macro-Location" und bleibt als Stub für ggf. künftige Skill-spezifische Mirror. Falls explizit removed werden soll: `rmdir docs/mdai/macros/`. Default: behalten.

- [ ] **Step 21.4: Commit Cleanup + Skill-A-Warnung**

```bash
git add docs/mdai/macros/ docs/mdai/specs/2026-05-23-mdai-brainstorm-design.mdai.md
```

```bash
git commit -m "$(cat <<'EOF'
chore(mdai-library): A9 — remove legacy docs/mdai/macros/ + warn on skill-A spec

Deletes hard-rules.md, step-reformat-commit.md, tool-quick-ref.md.
Library v0.1.0 (mdai/core/, mdai/tooling/) is now the source of truth.

Adds render-broken warning to skill-A-spec — patches in library-spec §10
must be applied before skill-A implementation can resume.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 21.5: Verifikation gesamt**

```bash
git log --oneline -20
```

Erwartung: ~14 Commits in diesem Plan (Task 1 RED + Tasks 3–16 je 1 Commit + Tasks 17/18 ggf. Patches + Task 19 GREEN + Task 20 Changelog + Task 21 Cleanup). Library-Spec-Frontmatter (`lib_version_target: 0.1.0`) ist nun in `mdai/MACROS.md` und in jeder Pack-File Frontmatter materialisiert.

```bash
ctx_tree path="mdai" depth=3
```

Erwartung: Voller Tree wie in File-Structure-Sektion am Anfang des Plans.

```bash
ctx_read path="mdai/MACROS.md" mode="map"
```

Erwartung: Inventory-Tabelle mit 13 Files + Changelog v0.1.0.

---

## Backlog-Reminder (nicht in v0.1.0 — siehe Spec §15)

Diese Tasks sind explizit out-of-scope für diesen Plan, brauchen eigene Specs:

1. `mdai-library-drift-check`-Skill (Spec §15.1)
2. `mdai/skills/mdai-execution/` Pack (Spec §15.2)
3. `mdai/skills/mdai-memory/` Pack (Spec §15.3)
4. Multi-Lang-Detection (Spec §15.4)
5. `lean-ctx pack mdai-macros` (Spec §15.5)
6. Auto-Generated MACROS.md (Spec §15.6)
7. Runtime `deprecation_check()` (Spec §15.7)
8. `lang/python.md`, `lang/node.md` (Spec §15.8)
9. Cross-Subagent-Bootstrap-Cache (Spec §15.9)

**Skill-A-Patch-Session** (Spec §10) ist parallel, eigener Plan, läuft NACH diesem hier.

---

## Self-Review (post-write — checked, gaps fixed inline)

- **Spec coverage:** §11 Phasen RED/P0/A1–A9 → Tasks 1–21 vollständig. §8.1/§8.2 RED/GREEN-Dispatch in Tasks 1/19. §12.1/§12.2/§12.3/§12.4 Smoke-Tests in Tasks 17/18. §10 Skill-A-Patches: dokumentiert in Backlog-Reminder + Task 21.2 Render-Broken-Warnung — die echte Patch-Session ist ein separater Plan (Spec §17 + §10). §16 Annahmen 1–6: verifiziert in Task 2 P0. §15 Backlog: gespiegelt im Backlog-Reminder.
- **Placeholder scan:** Keine "TBD" / "implement later". Alle Macro-Bodies vollständig (Tasks 3–16). Commit-Messages vollständig. Pass/Fail-Kriterien je Smoke-Step expliziert.
- **Type consistency:** Macro-Namen snake_case durchgehend (`write_spec`, `step_reformat_commit`, `mdai_bootstrap`). File-Names kebab-case (`write-spec.md`, `tool-quick-ref.md`). Frontmatter-Keys konsistent (`lib_version`, `mdai-pack.{mode, exports}`). `ctx_session` mit `action=set/get/info`-Param (Tuning-Hinweis in Task 10 falls Real-Signatur abweicht — Task 2 verifiziert).
