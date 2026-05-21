# MDAI Gotchas — Initiale Seed-Liste

`lean-ctx gotchas` unterstützt aktuell `list|clear|export|stats` (siehe
`lean-ctx gotchas --help`). Solange `gotchas add --tag mdai` fehlt, werden
Gotchas via `ctx_knowledge` mit Category `mdai-gotcha` registriert.

| Key                    | Symptom                                               | Mitigation                                                                                                                     |
|------------------------|-------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------|
| `g-import-vs-include`  | `@import` vs `@include` verwechselt                   | `@import` lädt nur `@define`-Macros ohne sichtbaren Output. Für Inline-Content `@include` nutzen.                              |
| `g-italic-header-hang` | `_..._` in Header-Zeilen triggert `ctx_read`-Hang     | Erste 5 Zeilen niemals mit `_…_` umschließen. `*…*` nutzen oder weglassen. Verifiziert 2026-05-21.                             |
| `g-rel-paths`          | MCP-Calls brauchen relative Pfade ab Projekt-Root     | `mcp__markdownai__read_file path="docs/mdai/plans/foo.mdai.md"` — kein absoluter Pfad.                                         |
| `g-respondtool-patch`  | `respondTool()`-Patch im markdownai-Server            | Bei `npm install` im markdownai-Repo kann der Patch verloren gehen. Backup in `tmp/mdai-bench/patches/respondtool-fix.patch`.  |
| `g-cache-stale`        | Macro-Edits invalidieren MDAI-Cache nicht automatisch | Nach Macro-Edit zwei Calls: `mcp__lean-ctx__ctx_shell command="lean-ctx cache clear"` und `mcp__markdownai__invalidate_cache`. |
