---
target: docs/mdai/plans/2026-05-24-mdai-brainstorm-implementation.md
reviewer: spec_reviewer_prompt (mdai/skills/mdai-brainstorm/spec-reviewer.md)
date: 2026-05-24
---

# Review — mdai-brainstorm Implementation Plan

## Status: Approved

Plan ist umsetzbar wie geschrieben. Keine planning-blocker, zwei advisory-Empfehlungen.

## Strengths

1. **Vollständige Spec-§15-Abdeckung** — Jede Phase (P0, A1, A2, A2.5, A2.6, A3, A4, A5, A5.5) hat einen dedizierten
   `@phase`-Block mit bite-sized Checkbox-Steps und exakten Commands/Paths. Plus `verification-summary`-Phase mit
   konsolidierter Final-Kontrolle der Erfolgskriterien §1 und Annahmen §16.
2. **Filesystem-State-aware** — Pre-Plan-Snapshot (Z 42–60) identifiziert A2.5/A2.6 als bereits-erledigt
   (Patch-Session 2026-05-24) und markiert sie als **Verify-Only**, statt sie redundant zu re-implementieren.
   Verifiziert via `ctx_tree`/`ctx_read`/`Bash test -f` vor Plan-Start.
3. **Plan ist selbst markdownai-stilisiert** — `@markdownai v1.0`-Header + `@phase ... @on complete ... @end`-Blöcke
   matchen README §"@phase, @on complete, and @graph" exakt. Plan ist damit phase-navigierbar via
   `mcp__markdownai__list_phases` / `read_file(phase=…)` (User-Anforderung).
4. **Smoke-Test-Coverage matcht Spec §8** — §8.1 Pointer-Compliance (3 Runs, jq-Mess-Befehl, 3/3-2/3-0/3
   Re-Architektur-Trigger), §8.2 Discipline-Fidelity inkl. v3-"no plan written"-Check, §8.3 Output, §8.4
   Phase-Budget, §8.5 Library-Import + Sub-Tests §8.5.1 (`@date`-Resolve) + §8.5.2 (`@if file.exists`), §8.6
   Lean-Context-Discipline. Eskalations-Pfade explizit in A4.7.
5. **Drift-Tracking erhalten** — Hand-ported Disziplin-Slices in A2.4 mit Verweis auf upstream-Lines (16-20, 22-32,
   70-104, 107-136, 140-145). A2.8 Verification-Step prüft `≥4 matches` für `hand-ported from superpowers` — Anchor
   für künftiges `mdai-drift-check` (Spec §14 Backlog #2).
6. **Lean-Context-Discipline durchgehend respektiert** — `fresh=true` / `raw=true` 0 matches in Plan (verifiziert).
   `ctx_read mode="full"` außerhalb §1 nur an einer kontrollierten Stelle (Z 362, siehe Recommendations).

## Gaps

Keine — Plan ist ready-to-execute.

## Concrete patches

Keine planning-blocker.

## Recommendations (advisory, non-blocking)

### R1 — `@note visible consumer="human"` Justification für `visual-companion.md mode="full"`

**Position:** Plan Z 362, inside Step A2.4 (`body.mdai.md` dialog phase content).

Der eingebettete Aufruf

```markdown
{{ @call ctx_read(path="~/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/visual-companion.md", mode="full") }}
```

ist per Spec §5.3 als bewusste Versions-Pin-Ausnahme dokumentiert (Anti-Pattern-Check 5, `mdai-drift-check`-Backlog
§14.1). Da der Aufruf in der Live-Skill-`body.mdai.md` landet, würde Reviewer Check #10 (lean-context-defaults) bei
einer Re-Review der Live-Datei eine `@note visible consumer="human"`-Justification fordern. Vorschlag: A2.4 ergänzen,
sodass `body.mdai.md` direkt vor dem `@call` eine Justification rendert:

```markdown
@note visible consumer="human"
visual-companion.md (upstream) hat keinen map/signatures-Pfad — full-read ist die nur sinnvolle Variante. Version
auf 5.1.0 gepinnt (Spec §5.3 Versions-Pin); bei upstream-Bump aktualisieren.
@end
```

Effekt: Live-Skill passt Check #10 ohne manuelle Note. Plan selbst ist nicht betroffen (Aufruf liegt in code-fence,
Reviewer-Check #10 zielt auf den Live-Output).

### R2 — Green-Verification-Artefakt als Spiegel zur Library-Konvention

Repo hat bereits Konvention `docs/mdai/green-verification/library/v0.1.0-*.md` (siehe `docs/mdai/green-verification/`).
Empfehlung: Nach A4-Completion ein analoges Artefakt
`docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md` anlegen, das die 6 Smoke-Test-Outputs (§8.1–§8.6)
zusammenfasst. Nicht blocking — strukturierter Anker für künftige Drift-Detection und Plan-Iterationen. Falls Plan
mit hoher Frequenz iteriert wird (z.B. nach `mdai-writing-plans`-Bootstrap), wird das Artefakt schnell wertvoll;
falls Skill stabil läuft, kann es bewusst weggelassen werden.

### R3 — Gotcha-Capture-Pfad für Smoke-Test-Partial-Pass-Fälle

Spec §8.1 sieht "2/3 Pass → manuelle Diagnose" vor (kein Auto-Fallback). Empfehlung: A4.1 um optionalen Step
ergänzen — falls 2/3-Diagnose abgeschlossen ist, `@call add_gotcha(key, symptom, mitigation)` aufrufen, damit
künftige Sessions die Root-Cause-Analyse als seed haben. Beispiel:

```markdown
@call add_gotcha(
  key="mdai-brainstorm-smoke-8.1-partial",
  symptom="Pointer-Compliance 2/3 pass — body.mdai.md full-read in Run X",
  mitigation="Root-Cause war <…>. Mitigation: <…>."
)
```

Nicht blocking — Plan hält sich an Spec §8.1-Wording exakt. Add_gotcha ist Convenience.

## Anti-Pattern-Check-Result-Summary (§5 #1-#11)

| # | Check                                                | Result | Notes                                                                                  |
| 1 | MCP signatures verified against source               | Pass   | `ctx_graph`/`read_file`/`ctx_*` Aufrufe matchen Spec §5.2 Pattern; keine ctx_session-Pitfall |
| 2 | Existing-store check                                 | Pass   | Plan führt keinen neuen State-Persisting-Wrapper ein                                  |
| 3 | mai-CLI does not execute `@query`                    | Pass   | A4.5/4.5.1/4.5.2 explizit als Plumbing-Check formuliert, Live-MCP-Hinweis dort         |
| 4 | Frontmatter convention for `mode: include`           | Pass   | A2.6.1 prüft `lean-context.md` auf `^---$` 0-matches                                   |
| 5 | Repo-relative paths only                             | Pass   | `~/.claude/.../superpowers/5.1.0/...` ist Spec-§5.3-genehmigte Versions-Pin-Ausnahme  |
| 6 | Language convention enforced                         | Pass   | Plan-Body Deutsch (Plan-Konvention CLAUDE.md), Code/Snippets Englisch                  |
| 7 | Parameter names match MCP source                     | Pass   | Plan nutzt ctx_tools-Macro-Konvention (`cmd=`); Library-Decision Out-of-Scope          |
| 8 | Smoke-render-test mandatory pre-release              | Pass   | A4.5 deckt explizit `npx mai render` gegen `body.mdai.md` ab                           |
| 9 | Plan-Body uses markdownai directives actively        | Pass   | `@phase`/`@on complete`/`@end`/`@markdownai v1.0` Header; eingebettete Specs zeigen `@call`/`@include`/`@constraint`/`@if file.exists` |
| 10 | Lean-Context-Defaults                                | Pass*  | `ctx_read mode="full"` 1× in code-fence (R1 Recommendation für Live-Skill); `fresh=true`/`raw=true` 0 matches  |
| 11 | Structured data via `@read`/`@list`                  | Pass   | Tabellen sind State-Snapshot/Rationalization-Counter (kein externer SoT)              |

*Pass mit Recommendation R1 für Live-Skill-Artefakt.

## Output

Plan kann unverändert zur Execution gegeben werden. Empfohlene Sub-Skills aus Plan-Header: `superpowers:subagent-driven-development`
(parallele Phasen, fresh-context pro Task) oder `superpowers:executing-plans` (inline, Batch + Checkpoints). Da
Phasen weitestgehend sequentiell abhängig sind (P0 → A1 → A2 → A3 → A4) und A2.5/A2.6 nur Verify-Only sind, ist
inline-Execution (Option 2) etwas effizienter als Subagent-Driven. Falls Phase A4 Smoke-Tests parallelisiert werden
sollen, lohnt sich Subagent-Driven für A4.5/A4.5.1/A4.5.2/A4.6 (4 unabhängige Static-Render-Tests).
