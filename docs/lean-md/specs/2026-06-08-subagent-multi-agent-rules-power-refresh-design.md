# Design: Refresh `.claude/rules/subagent-multi-agent.md` für `tool_profile = power`

**Datum:** 2026-06-08
**Status:** Approved (brainstorming)
**Scope:** Voller Refresh der Rules-Datei gegen aktualisierte Referenz-Docs.

## Problem

`.claude/rules/subagent-multi-agent.md` ist durchgängig auf `tool_profile = "standard"`
gebaut: Power-Tools (`ctx_task`, `ctx_handoff`, `ctx_workflow`) werden als „nicht direkt
verfügbar — via `ctx_call`-Gateway" beschrieben. Das lean-ctx Tool-Profil ist jetzt auf
**`power`** umgestellt. Unter `power` sind **alle** Tools direkt exponiert, wodurch die
gesamte Gateway-Logik der Datei invertiert/obsolet wird.

Zusätzlich haben die Referenz-Docs (`docs/reference/07-context-engineering.md`,
`08-multi-agent.md`, `docs/reference/generated/mcp-tools.md`) Änderungen erhalten, die
einen Drift gegenüber der Rules-Datei erzeugen.

## Faktenbasis (gegen die Quelle verifiziert)

- `rust/src/core/tool_profiles.rs:51` — `Power => is_tool_enabled = true` für **alle**
  Tools. `Standard` = 22 balanced tools; `Power` = „All tools exposed".
- `tool_profiles.rs:78-79` — bestehende Installs defaulten auf `power` (backward compat);
  neue Installs setzen `standard` beim Setup.
- `docs/reference/generated/mcp-tools.md` (Header) — lean-ctx registriert **72 MCP tools**
  (granular profile).
- `ctx_task` State-Machine, verifiziert in `rust/src/core/a2a/task.rs:7-65` +
  `rust/src/tools/ctx_task.rs:44-100`:
    - 5 setzbare States: `working, input-required, completed, failed, canceled`.
      Die Rules-Datei ist hier **korrekt**.
    - Impliziter Start-State `created` (von `create_task` gesetzt, nicht per `state=`).
    - Transitions: `created → working|canceled|failed` · `working →
    input-required|completed|failed|canceled` · `input-required ↔ working` ·
      `completed/failed/canceled` = terminal.
    - **`in_progress` ist kein gültiger State** — der Wert in `08-multi-agent.md §6` ist
      ein Doc-Bug (out of scope für die Rules-Datei, separat zu fixen).
- `ctx_share` (08 §7) ist ein Power-Tool für den **Cross-Prozess**-Fall (Cursor + Claude +
  Codex als getrennte Prozesse). Im subagent-driven-Szenario teilen alle Subagents **einen**
  MCP-Prozess → der geteilte Cache macht `ctx_share` redundant. Die „No ctx_share"-Stance
  der Rules-Datei bleibt korrekt; nur die Begründung wird präzisiert.

## Änderungen an `subagent-multi-agent.md`

### 1. Tool-Set-Header (≈ Z. 20–35)

- „Requires `tool_profile = "standard"`+" → **„Requires `tool_profile = power`
  (`lean-ctx tools power` → all 72 tools exposed)"**.
- Den Absatz „Only power-profile tools (`ctx_task`, `ctx_handoff`, `ctx_workflow`) … via
  `ctx_call` gateway" **streichen** und ersetzen durch:
  > Unter `power` sind **alle** Tools direkt — `ctx_task`/`ctx_handoff`/`ctx_workflow`/
  > `ctx_share`/`ctx_rules` inklusive. `ctx_call` nur noch als **Fallback**, falls ein Tool
  > ausnahmsweise deferred auftaucht → `ToolSearch(query="select:<tool>")` zuerst.

### 2. „NEVER wrap in ctx_call"-Regel

Bleibt, wird generalisiert: gilt jetzt für **jedes** lean-ctx Tool (alle direkt unter
`power`), nicht nur die frühere Standard-Teilmenge.

### 3. `ctx_share`-Block (≈ Z. 39–43)

Stance „No ctx_share" beibehalten; Begründung präzisieren:
> Gilt, weil subagent-driven-development einen einzigen MCP-Prozess teilt (geteilter Cache).
> Das Cross-IDE-`ctx_share` aus Journey 8 §7 adressiert getrennte Prozesse — hier N/A.

### 4. Tool-Tabelle (≈ Z. 45–56)

- **`ctx_task`-Zeile ersetzen** durch verbesserte Beschreibung:
  > `| A2A task board | ctx_task | actions: create(needs to_agent)/update(needs
  > task_id+state)/list/get/message/cancel/info. State machine: created(implicit)→working→
  > {input-required↔working}→completed|failed|canceled (last 3 terminal). NOTE: in_progress
  > is NOT valid (08-multi-agent.md §6 typo) |`
- **Zwei Zeilen ergänzen:**
    -
    `| Shadow-git der eigenen Änderungen | ctx_checkpoint | snapshot/log/diff/restore — getrennt von User-.git; snapshot vor+nach einer Änderung |`
    -
    `| Rules-Konsistenz über Agents | ctx_rules | sync (verteilt Rules) / diff (Drift) / lint (Konsistenz) / status / init |`

### 5. Dispatch Contract (≈ Z. 108–110)

Zeile „Power tools ONLY (ctx_task, ctx_handoff, ctx_workflow) → via ctx_call name=<tool>"
ersetzen durch:
> All lean-ctx tools are DIRECT under `power` — call `ctx_task`/`ctx_handoff`/`ctx_workflow`
> directly. `ctx_call` only as deferred-fallback after `ToolSearch(query="select:<tool>")`.

### 6. Controller/Implementer-Contracts

Verbleibende `ctx_call`-Gateway-Verweise für Power-Tools auf „direkt aufrufen" angleichen
(Konsistenz mit §1/§5).

## Nicht geändert (bewusst)

- `ctx_task`-State-Liste (ist korrekt).
- `ToolSearch`-Reflex-Regel bei deferred Tools.
- `ctx_shell` bare-command + `cwd=` / kein `2>&1` Regeln.
- Serena-für-Rust-Edits.
- Test-Runner-Regeln (verbatim output).

## Out of scope (separat flaggen)

- `docs/reference/08-multi-agent.md §6` — `in_progress` → muss auf gültigen State
  (`working`) korrigiert werden. Eigener Fix, nicht Teil dieser Rules-Datei-Änderung.
