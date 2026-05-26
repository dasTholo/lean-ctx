---
title: MDAI_LIBRARY_ROOT Audit — Variante A (eliminieren) vs. Variante B (Status quo)
date: 2026-05-26
status: ready-for-review
authors: claude
related_specs:
  - docs/mdai/specs/2026-05-26-mdai-v0.1.2-refit-v1.0-audit-design.mdai.md
related_plans:
  - docs/mdai/plans/2026-05-26-mdai-v0.1.2-part-a-adoption-audit.md
sourced_from:
  - ctx_session findings [t0.1-pre-sed] [t0.1-A-smoke1..4] [t0.1-B-smoke1..4]
---

# MDAI_LIBRARY_ROOT Audit — Variante A vs. Variante B

## Methodik

Zwei vergleichbare `mdai-brainstorm`-Engine-Pfade, je 4 Smokes:

- **Smoke 1:** `mcp__markdownai__list_phases` auf `mdai/skills/mdai-brainstorm/body.mdai.md` — Phasenmetadaten.
- **Smoke 2:** `mcp__markdownai__resolve_phase` mit Phase `pre-context` — Library-`@include`-Auflösung.
- **Smoke 3:** vollständiger `mdai-brainstorm`-Skill-Lauf (nicht `/superpowers:brainstorming` — siehe Plan-Korrektur unten), alle
  5 Phasen via `mcp__markdownai__read_file phase=...`. Spec-Self-Review + Spec-Write übersprungen (minimaler Run, nur
  Engine-Resolution-Evidence).
- **Smoke 4:** `mcp__markdownai__call_macro` auf 3 Bootstrap-Macros (`mdai_bootstrap`, `detect_mai_hook_version`,
  `lean_context_audit`).

**Variante A:** `source_root: "/home/tholo/Scripts/lean-ctx"` (absolut) + Sed-Substitution
`@include ${MDAI_LIBRARY_ROOT}/` → `@include mdai/` in allen `mdai/**/*.md` im Wegwerf-Branch `audit/no-mdai-root`.

**Variante B:** Status quo — `${MDAI_LIBRARY_ROOT}`-Globs in Library + env-var-Inheritance vom Claude-Code-Spawn an
den MCP-Server.

### Plan-Abweichungen (für Findings-v2 relevant)

1. **Plan sagt `/superpowers:brainstorming dummy-feature-{x,y}` für Smoke 3.** Dieser Skill ist aber ein regulärer
   Superpowers-Skill in `~/.claude/plugins/...`, ohne `body.mdai.md`, ohne `mcp__markdownai__*`-Engine-Calls — testet
   die zu auditierende Library-Resolution **nicht**. Korrektur: Smoke 3 wurde mit `mdai-brainstorm` (engine-basiert)
   durchgeführt — das ist der Skill mit `body.mdai.md` und `@include ${MDAI_LIBRARY_ROOT}/...`.

2. **Tool-Schemas haben kein `cwd`/`env`-Parameter.** Plan-Snippets wie `list_phases(file=..., cwd=..., env={...})`
   sind im v1.0 MCP-Schema **nicht möglich**. Der MCP-Server erbt CWD und env vom Spawn-Prozess (Claude Code). Damit
   funktioniert Variante B's env-Propagation automatisch — aber **nur** wenn der Spawn-Prozess MDAI_LIBRARY_ROOT
   gesetzt hat. Für isolierte CI-Läufe oder Subagent-Spawns ohne env-Vererbung wäre Variante B ohne explizites env
   nicht reproduzierbar.

3. **Smoke 3 Variante A: NOT-RUN** statt FAIL — wir haben den vollen Smoke-Lauf nicht erneut aufgesetzt (Branch +
   sed + security.json), nachdem Smoke 2 die Engine-Failure schon eindeutig demonstriert hat (User-Decision: nur
   Variante B mit korrektem Skill). Predicted=FAIL aus Smoke-2-Evidence.

## Vergleichstabelle

| Kriterium                                | Variante A (kein env)                        | Variante B (Status quo)                                     |
|------------------------------------------|----------------------------------------------|-------------------------------------------------------------|
| Setup-Cost (one-time)                    | `security.json filesystem`-Block neu (1x)    | `MDAI_LIBRARY_ROOT` in `~/.zshrc`/`~/.bashrc` exportieren   |
| Setup-Cost (neuer Dev / CI)              | nur `security.json`-Template kopieren        | env-var setzen + shell sourcen (oder CI-`env:`-Block)       |
| Portabilität Library → andere Repos      | **NEIN** — `source_root` absolut hardcoded   | **JA** — env-var portabel über Repos                        |
| Subagent-env-Vererbung (Finding §7)      | nicht betroffen (kein env in Resolution)     | weiterhin Stolperfalle ohne explizites env-Passthrough      |
| MCP-Tool-Schema-Eignung                  | passt (kein env-Param nötig)                 | passt nur via Inheritance — keine explizite `env=`-API      |
| Smoke 1 (list_phases)                    | **PASS** — 5 Phases korrekt                  | **PASS** — 5 Phases identisch                               |
| Smoke 2 (resolve_phase pre-context)      | **FAIL** — 9 warnings, davon 2 ENOENT critical | **PASS** — `warnings: []` (0 warnings)                    |
| Smoke 3 (full brainstorm, 5 Phasen)      | NOT-RUN (predicted FAIL aus Smoke-2-Evidence) | **PASS** — 5/5 Phases, 0 ENOENT, 16 nicht-kritische warns  |
| Smoke 4 (call_macro × 3 Bootstrap)       | mdai_bs=DEGRADED, detect_hook=PASS, lean_ctx_audit=PASS_param | identisch (no ENOENT in beiden — top-level macro calls) |
| Render-Output gegen Library funktionsfähig | **NEIN** — file-relative @include resolution bricht  | **JA** — `${MDAI_LIBRARY_ROOT}` expandiert korrekt          |

### Variante-A Root-Cause-Analyse (kritisch)

Variante A's Sed-Substitution `@include ${MDAI_LIBRARY_ROOT}/...` → `@include mdai/...` setzt voraus, dass die Engine
relative `@include`-Pfade gegen `source_root` aus `security.json` resolved. Empirie aus Smoke 2:

```
@include: cannot read file "mdai/core/hard-rules.md":
Error: ENOENT: no such file or directory,
open '/home/tholo/Scripts/lean-ctx/mdai/skills/mdai-brainstorm/mdai/core/hard-rules.md'
```

Die Engine konkateniert: **directory of calling file** + relative path. `body.mdai.md` liegt in
`mdai/skills/mdai-brainstorm/` → relativer Pfad `mdai/core/hard-rules.md` resolved zu
`mdai/skills/mdai-brainstorm/mdai/core/hard-rules.md` (ENOENT).

`source_root` aus `security.json` ist offenbar **nur Security-Check-Root**, nicht **Resolution-Root**. Damit ist
Variante A in der aktuellen v1.0-Engine **nicht funktionsfähig** — egal welche `security.json`-Konfiguration.

### Nebenbefunde aus den Smokes (für Findings-v2 in Part-B)

- **`mdai_bootstrap` Output garbled (16 warnings)** in beiden Varianten — `@query`-Commands von der `shell.allow_patterns`-Allowlist
  geblockt. Output bricht mid-template. Separater Engine/Macro-Bug, nicht Library-Root-bezogen.
- **`detect_mai_hook_version` Output A vs. B unterschiedlich:** A=„not installed", B=„v0.x — RUN init". Variante A's
  security.json hat `~/.markdownai/hooks/**` nicht in `allowed_data_paths` → `file.containsLine` wahrscheinlich
  blockiert → false-Zweig. Variante B hat den Pfad whitelisted. Macht Sinn, ist erwartet.
- **Variante B write-outputs Phase:** zusätzlich zur erwarteten Template-Var-Unresolved-warnings (`slug`, `design_content`,
  `render_target_resolved`) ein **harter Engine-Error** `"@set" cannot be used as a pipe source`. Potenzielles
  Update zu Finding §2 oder §5 (Macro-Body-Edge-Cases).

## Empfehlung des Audits

**Variante A ist mit dem aktuellen v1.0-Engine-Verhalten nicht umsetzbar.** Die Annahme „`source_root` ersetzt
`${MDAI_LIBRARY_ROOT}` als Resolution-Root" trifft nicht zu — `@include`-Pfade werden weiterhin file-relative aufgelöst.
Eine Migration zu Variante A würde einen **Engine-Patch in markdownai** erfordern (z.B. neue Directive `@include-from-root`
oder `source_root`-aware-Resolution), nicht nur eine Sed-Substitution + `security.json`-Anpassung.

**Empfehlung: Variante B (Status quo) beibehalten** in v0.1.3. Variante A ist Engine-Roadmap-Material — nicht Adoption-Plan.
Wenn der Wunsch nach Repo-Portabilität bleibt: Engine-Issue in markdownai für `source_root`-relative `@include`-Resolution
öffnen, dann Variante A in einem späteren Release re-evaluieren.

**Mitigation für Variante-B-Stolperfalle (Finding §7 — env-Vererbung):** In `mdai/core/hard-rules.md` einen Hinweis
aufnehmen, dass MCP-Server `MDAI_LIBRARY_ROOT` vom Spawn-Prozess erben — bei isolierten Subagent-/CI-Spawns muss das
env explizit gesetzt werden.

## Wegwerf-Branch-Cleanup-Status

- Branch `audit/no-mdai-root`: gelöscht (T1 Step 9, war `86615739`).
- `~/.markdownai/security.json`: revertiert auf pre-audit-State (T1 Step 9, `diff` leer).
- Library-Files (`mdai/**/*.md`): unverändert nach Branch-Delete (T1 Step 9 verify, Treffer-Count = Pre-Flight 9
  Matches in 2 Files: `core/startup-check.md` (4) + `skills/mdai-brainstorm/body.mdai.md` (5)).
