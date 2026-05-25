# MDAI Knowledge-Schema

Topics in `mcp__lean-ctx__ctx_knowledge` für MDAI-Pläne folgen diesem Schema.

## Topic-Pattern

`mdai-plan:<plan-id>`

Beispiel: `mdai-plan:mdai-skill-integration` (plan-id == `id` aus YAML-Frontmatter).

## Body-Schema (JSON)

```json
{
  "phases": [
    "P0-leanctx-setup",
    "A1-mdai-plans-skill",
    "..."
  ],
  "current_phase": "A1-mdai-plans-skill",
  "completed": [
    "P0-leanctx-setup"
  ],
  "started_at": "2026-05-21T10:00:00Z",
  "status": "in-progress"
}
```

## Pflichtfelder

- `phases` (str[]) — alle Phase-IDs aus `mcp__markdownai__list_phases`
- `current_phase` (str | null) — null wenn alle Phasen `completed`
- `completed` (str[]) — Reihenfolge der Fertigstellung
- `status` (enum: `planned` | `in-progress` | `done` | `aborted`)

## Optional

- `started_at` (ISO 8601)
- `notes` (str)
- `gain_run_log` (str) — Pfad zum `lean-ctx gain --json`-Audit nach Abschluss

## Operationen

| op          | Aufruf                                                                 |
|-------------|------------------------------------------------------------------------|
| `start`     | `remember(topic, body={phases, current_phase=phases[0], ...})`         |
| `<id>-done` | `recall` → `completed.append(<id>)`; `current_phase=next` → `remember` |
| `all-done`  | `recall` → `status="done"` → `remember`                                |
| `resume`    | `recall(topic)` liefert vollen Stand                                   |
