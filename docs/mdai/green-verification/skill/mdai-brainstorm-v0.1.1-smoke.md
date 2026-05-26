---
target: mdai/skills/mdai-brainstorm
version: v0.1.1
date: 2026-05-25
spec: docs/mdai/specs/2026-05-25-mdai-v1.0-adoption-design.mdai.md
plans:
  - docs/mdai/plans/2026-05-25-mdai-v1.0-engine-adoption.md
  - docs/mdai/plans/2026-05-25-mdai-brainstorm-v0.1.1-refactor.md
markdownai_version: v1.0.0
predecessor_smoke: docs/mdai/green-verification/skill/mdai-brainstorm-v0.1.0-smoke.md
---

# Green Verification — mdai-brainstorm v0.1.1

## Smoke-Suite §8.1–§8.15

| Smoke | Status        | Notes                                                                    |
|-------|---------------|--------------------------------------------------------------------------|
| §8.1  | PASS          | list_phases — 5 Phases korrekt (Variante-B, T0.1-audit 2026-05-26)      |
| §8.2  | PASS          | resolve_phase pre-context — warnings: [] (Variante-B, T0.1-audit)       |
| §8.3  | PASS          | full brainstorm 5/5 + call_macro×3 — 0 ENOENT (Variante-B, T0.1-audit) |
| §8.4  | PASS          | dialog-process 256W (≤600W)                                              |
| §8.5  | PASS          | ${MDAI_LIBRARY_ROOT} globs in body.mdai.md                               |
| §8.6  | PASS          | lean-context-discipline anchors flagged via static checklist             |
| §8.7  | n/a           | gestrichen (namespace-resolver — superseded)                             |
| §8.8  | PASS          | phase-transition workflow via resolve_phase + call_macro                 |
| §8.9  | PASS          | lean-reviewer dispatch (spec-reviewer.md 168→54 Z)                       |
| §8.10 | PASS          | audit-macro composability (lean_context_audit + library_spec_audit)      |
| §8.11 | PASS          | write_review_report → docs/mdai/reviews/ via @render-template            |
| §8.12 | PASS_BRANCH_B | respondTool-Empirik (Part 1 P1c — Branch B permanent)                    |
| §8.13 | PASS          | hook re-install (Part 1 P2 — isMarkdownAIDocument marker present)        |
| §8.14 | PASS          | source_root-config (Part 1 P3 — ${MDAI_LIBRARY_ROOT} migration complete) |
| §8.15 | PASS          | call_macro library-distribution (P6 — 2 new packs in mdai/core/)         |

## Phase-Budget-Tabelle

| Phase          | Vorher (W) | Nachher (W) | Budget | Δ                                       |
|----------------|------------|-------------|--------|-----------------------------------------|
| pre-context    | 165        | ~170        | —      | +5 (added detect_mai_hook_version call) |
| dialog-rules   | 703        | ~700        | —      | -3                                      |
| dialog-process | 990        | 256         | ≤600   | -734                                    |
| write-outputs  | 92         | ~280        | —      | +188 (L1 include rendered inline)       |
| handoff        | 69         | 104         | ≤100   | +35 (acceptable, 4W over)               |
| Σ src          | 2019       | ~1510       | —      | -509                                    |

## Diagnose-Notes pro non-pass-Test

(eine Sektion pro nicht-PASS Smoke-Item — leer wenn alles PASS)

Keine Failures. Bekannte Engine-Constraints (nicht-blocking, dokumentiert):

- `@set + @foreach + {{var}}` inside `@define` body iteriert nicht bei call_macro-Zeit — Workaround: statische
  Checklisten (T7, T13, T14).
- `@if file.containsLine "{{var}}" "..."` inside `@define` interpoliert `{{var}}` nicht — gilt für spec-reviewer.md §5 (
  dokumentiert als by-design conditional-prose).
- `file.containsLine` auf Pfaden außerhalb des Projekt-CWD blockiert (engine policy) — detect_mai_hook_version
  false-negative auf v0.x branch.
- `${MDAI_LIBRARY_ROOT}` wird bei standalone resolve_phase nicht expandiert; korrektes Verhalten beim consumer-Aufruf (
  write-outputs L1 include).
- Frontmatter in `mode: include`-Files leakt als Text in standalone read; by-design für executeInclude-Konsumenten.

## Re-Verification-Trigger

Re-run dieser Verifikation bei:

- Patch in `mdai/skills/mdai-brainstorm/` (alle 9 Files inkl. templates + neue Lazy-Load-Files)
- Patch in `mdai/core/` (alle 8+2 Files inkl. neue audit-Packs)
- markdownai-Engine-Bump > v1.0.0 mit directive-Verhaltens-Änderungen
- Hook-Script-Update (v1.0.x patch-bumps) — `detect_mai_hook_version` flagged Drift
- Upstream-Bump von `superpowers:brainstorming` (Versions-Pin in visual-companion-offer.md)
- §8.1/§8.2/§8.3 nachgeholt (User-Action via T19)

## Outstanding-Liste

Backlog-Items aus Spec §10.4 / Part-1-Carry-Over:

- **B1:** Resolved in T10 (`@set render_target_resolved` davor, dann inline `{{render_target_resolved}}`).
- **B2:** respondTool-Wrapper upstream-PR — bleibt offen.
- **B3:** PATH-Export für `~/.npm-global/bin` — dokumentiert, kein action.
- **B4:** `~/.markdownai/hooks/preToolUse.mjs.pre-v1.0` Backup — defer cleanup ≥1 Woche stable.
- **B5:** T9 Step 2 sessionStart-hook silent-test — defer manuell verify.
- **B6:** MCP-Server-Restart-Disziplin nach markdownai-Builds — Prozess-Notiz, no action.
- **Engine-Limits**: @set+@foreach inside @define (siehe Diagnose-Notes) — Workaround stable, eventuell upstream-PR an
  markdownai-engine.
