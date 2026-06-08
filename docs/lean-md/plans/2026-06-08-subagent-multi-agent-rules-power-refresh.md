# subagent-multi-agent Rules `power`-Refresh — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `.claude/rules/subagent-multi-agent.md` von `tool_profile = "standard"` auf `power` umstellen — alle Tools direkt, `ctx_call` nur Fallback, `ctx_task`-Beschreibung verbessert, `ctx_checkpoint` + `ctx_rules` ergänzt, `ctx_share`-Begründung präzisiert.

**Architecture:** Reine Doc-Edit-Aufgabe an **einer** Markdown-Datei. Fünf lokal begrenzte `ctx_edit`-Ersetzungen über vier Regionen (Tool-Set-Header, `ctx_share`-Block, Tool-Tabelle, Dispatch Contract). Keine Code-, Test- oder Build-Änderung. Verifikation = `ctx_read(mode="diff")` pro Edit + finaler `ctx_search`, der bestätigt, dass keine `standard`-Profil-/Gateway-Reste übrig sind.

**Tech Stack:** Markdown; lean-ctx `ctx_edit` (non-Rust → kein Serena), `ctx_read` (diff), `ctx_search`, `mcp__jetbrains__reformat_file` (Projekt-Hardrule vor `git add`), `git`.

**Spec:** `docs/lean-md/specs/2026-06-08-subagent-multi-agent-rules-power-refresh-design.md`

---

## File Structure

- **Modify:** `.claude/rules/subagent-multi-agent.md` (einzige Datei) — die Verhaltens-Contract-Datei für subagent-driven-development.
- Keine neuen Dateien.

**Werkzeug-Hinweis:** Es ist eine `.md`-Datei → **`ctx_edit`** (nicht Serena, das gilt nur für `.rs`). Vor jedem Edit die Zielregion mit `ctx_read` lesen; nach jedem Edit mit `ctx_read(mode="diff")` verifizieren.

---

### Task 1: Tool-Set-Header auf `power` umstellen

**Files:**
- Modify: `.claude/rules/subagent-multi-agent.md` (Abschnitt `## lean-ctx tool set`, ca. Z. 20–35)

- [ ] **Step 1: Zielregion lesen**

Run: `ctx_read(".claude/rules/subagent-multi-agent.md")`
Erwartet: Abschnitt `## lean-ctx tool set (3.7.x — use these proactively)` mit dem `standard`-Header und dem `ctx_call`-Gateway-Absatz ist sichtbar.

- [ ] **Step 2: Header + Gateway-Absatz ersetzen**

`ctx_edit` mit:

`old_string`:
```
## lean-ctx tool set (3.7.x — use these proactively)

Requires `tool_profile = "standard"`+ (`lean-ctx tools standard`). Standard tools
below are direct — **call them directly** (`ctx_read`, `ctx_search`, `ctx_shell`,
`ctx_tree`, `ctx_multi_read`, `ctx_delta`, …). If a standard tool shows up
**deferred** in an isolated subagent catalog, run `ToolSearch(query="select:<tool>")`
FIRST, then call it directly. **NEVER wrap a standard tool in `ctx_call`** (no
`ctx_call name=ctx_read`, no `ctx_call name=ctx_shell` — that is pure overhead).

Only **power-profile** tools (`ctx_task`, `ctx_handoff`, `ctx_workflow`) are NOT
exposed directly under `standard` — reach those via the `ctx_call` gateway:
`ctx_call name=ctx_task arguments={action:…}`. (Alt: `lean-ctx tools power`
exposes them directly but bloats the tool catalog.)
```

`new_string`:
```
## lean-ctx tool set (use these proactively)

Requires `tool_profile = power` (`lean-ctx tools power` → all 72 MCP tools
exposed). Under `power` **every** lean-ctx tool is direct — **call it directly**
(`ctx_read`, `ctx_search`, `ctx_shell`, `ctx_tree`, `ctx_multi_read`, `ctx_delta`,
`ctx_task`, `ctx_handoff`, `ctx_workflow`, `ctx_share`, `ctx_rules`, …). If a tool
shows up **deferred** in an isolated subagent catalog, run
`ToolSearch(query="select:<tool>")` FIRST, then call it directly. **NEVER wrap a
tool in `ctx_call`** (no `ctx_call name=ctx_read`, no `ctx_call name=ctx_task` —
that is pure overhead).

`ctx_call` is now only a **fallback**: use it solely if a tool stays deferred
after `ToolSearch`. (Profiles for reference — `minimal` = 6 tools, `standard` = 22,
`power` = all 72; this contract assumes `power`.)
```

- [ ] **Step 3: Edit verifizieren**

Run: `ctx_read(".claude/rules/subagent-multi-agent.md", mode="diff")`
Erwartet: Header ohne `3.7.x`/`standard`; neuer `power`-Absatz + `ctx_call`-Fallback-Satz vorhanden; kein „via the `ctx_call` gateway" mehr.

- [ ] **Step 4: Reformat (Projekt-Hardrule vor `git add`)**

Run: `mcp__jetbrains__reformat_file(path=".claude/rules/subagent-multi-agent.md", projectPath="/home/tholo/Scripts/lean-ctx")`
Erwartet: `ok`

- [ ] **Step 5: Commit**

```bash
git add .claude/rules/subagent-multi-agent.md
git commit -m "docs(rules): tool set header → tool_profile=power, all tools direct"
```

---

### Task 2: `ctx_share`-Begründung präzisieren (Cross-Prozess vs. Single-Prozess)

**Files:**
- Modify: `.claude/rules/subagent-multi-agent.md` (`> **No \`ctx_share\`:**`-Block, ca. Z. 37–43)

- [ ] **Step 1: `ctx_share`-Block ersetzen**

`ctx_edit` mit:

`old_string`:
```
> warm-cache push/pull via `ctx_share` is redundant ceremony and is intentionally
> NOT part of this contract. Subagents just `ctx_read` — **never `fresh`**
> (mtime auto-validation keeps cached entries current), **never `raw`**.
```

`new_string`:
```
> warm-cache push/pull via `ctx_share` is redundant ceremony and is intentionally
> NOT part of this contract. (`ctx_share` from Journey 8 §7 targets the
> **cross-process** case — separate Cursor/Claude/Codex processes — which does not
> apply to subagent-driven-development.) Subagents just `ctx_read` — **never
> `fresh`** (mtime auto-validation keeps cached entries current), **never `raw`**.
```

- [ ] **Step 2: Edit verifizieren**

Run: `ctx_read(".claude/rules/subagent-multi-agent.md", mode="diff")`
Erwartet: neuer Klammerzusatz „cross-process … does not apply" im `ctx_share`-Block; „No ctx_share"-Stance unverändert.

- [ ] **Step 3: Reformat**

Run: `mcp__jetbrains__reformat_file(path=".claude/rules/subagent-multi-agent.md", projectPath="/home/tholo/Scripts/lean-ctx")`
Erwartet: `ok`

- [ ] **Step 4: Commit**

```bash
git add .claude/rules/subagent-multi-agent.md
git commit -m "docs(rules): clarify no-ctx_share rationale (single-process vs cross-process)"
```

---

### Task 3: Tool-Tabelle — `ctx_task` verbessern + `ctx_checkpoint`/`ctx_rules` ergänzen

**Files:**
- Modify: `.claude/rules/subagent-multi-agent.md` (Tabellenzeile „Task Liste", ca. Z. 55–56)

**Hinweis Markdown-Escape:** Innerhalb einer Tabellenzelle müssen literale `|` als `\|` geschrieben werden, sonst brechen sie die Spalten. Die State-Liste `completed|failed|canceled` wird daher zu `completed\|failed\|canceled`.

- [ ] **Step 1: `ctx_task`-Zeile durch drei Zeilen ersetzen**

`ctx_edit` mit:

`old_string`:
```
| Task Liste                        | `ctx_task`                     | task create need: to_agent, States: "working,input-required,completed,failed,canceled" | 
```

`new_string`:
```
| A2A task board                    | `ctx_task`       | actions: create(needs `to_agent`)/update(needs `task_id`+`state`)/list/get/message/cancel/info. State machine: created(implicit)→working→{input-required↔working}→completed\|failed\|canceled (last 3 terminal). NOTE: `in_progress` is NOT valid (08-multi-agent.md §6 typo) |
| Shadow-git of own edits           | `ctx_checkpoint` | snapshot/log/diff/restore — separate from the user's `.git`; snapshot before+after a change to capture exactly what you modified |
| Rule consistency across agents    | `ctx_rules`      | sync (distribute rules) / diff (drift) / lint (consistency) / status / init |
```

- [ ] **Step 2: Edit verifizieren**

Run: `ctx_read(".claude/rules/subagent-multi-agent.md", mode="diff")`
Erwartet: drei Tabellenzeilen (`ctx_task` verbessert + `ctx_checkpoint` + `ctx_rules`); kein `\|`-Leak außerhalb der Zelle; alte „Task Liste"-Zeile weg.

- [ ] **Step 3: Reformat**

Run: `mcp__jetbrains__reformat_file(path=".claude/rules/subagent-multi-agent.md", projectPath="/home/tholo/Scripts/lean-ctx")`
Erwartet: `ok`

- [ ] **Step 4: Commit**

```bash
git add .claude/rules/subagent-multi-agent.md
git commit -m "docs(rules): improve ctx_task row + add ctx_checkpoint & ctx_rules"
```

---

### Task 4: Dispatch Contract — Power-Tools direkt statt via `ctx_call`

**Files:**
- Modify: `.claude/rules/subagent-multi-agent.md` (Block ```` ```text ````-Dispatch Contract, ca. Z. 100–110)

Zwei Ersetzungen im Dispatch-Contract-Block.

- [ ] **Step 1: „DIRECT standard tools" generalisieren**

`ctx_edit` mit:

`old_string`:
```
- ctx_read / ctx_search / ctx_shell / ctx_tree / ctx_multi_read / ctx_delta are
  DIRECT standard tools — call them DIRECTLY. If one shows up deferred, run
  ToolSearch(query="select:<tool>") FIRST, then call it. NEVER wrap a standard
  tool in ctx_call (no ctx_call name=ctx_read / name=ctx_shell — pure overhead).
```

`new_string`:
```
- Under tool_profile=power ALL lean-ctx tools are DIRECT — call them DIRECTLY
  (ctx_read / ctx_search / ctx_shell / ctx_tree / ctx_multi_read / ctx_delta /
  ctx_task / ctx_handoff / ctx_workflow / ctx_share / ctx_rules / …). If one shows
  up deferred, run ToolSearch(query="select:<tool>") FIRST, then call it. NEVER
  wrap a tool in ctx_call (no ctx_call name=ctx_read / name=ctx_task — pure overhead).
```

- [ ] **Step 2: „Power tools ONLY … via ctx_call"-Zeile ersetzen**

`ctx_edit` mit:

`old_string`:
```
- Power tools ONLY (ctx_task, ctx_handoff, ctx_workflow) → via ctx_call name=<tool>
```

`new_string`:
```
- ctx_call is ONLY a deferred-fallback (after ToolSearch); ctx_task / ctx_handoff /
  ctx_workflow / ctx_share / ctx_rules are called DIRECTLY under power
```

- [ ] **Step 3: Beide Edits verifizieren**

Run: `ctx_read(".claude/rules/subagent-multi-agent.md", mode="diff")`
Erwartet: Dispatch Contract nennt „ALL lean-ctx tools are DIRECT under power"; keine „Power tools ONLY … via ctx_call name=<tool>"-Zeile mehr.

- [ ] **Step 4: Reformat**

Run: `mcp__jetbrains__reformat_file(path=".claude/rules/subagent-multi-agent.md", projectPath="/home/tholo/Scripts/lean-ctx")`
Erwartet: `ok`

- [ ] **Step 5: Commit**

```bash
git add .claude/rules/subagent-multi-agent.md
git commit -m "docs(rules): dispatch contract — power tools direct, ctx_call as fallback"
```

---

### Task 5: Residual-Sweep — keine `standard`-Profil-/Gateway-Reste

**Files:**
- Modify: nur falls der Sweep noch Treffer findet: `.claude/rules/subagent-multi-agent.md`

- [ ] **Step 1: Nach veralteten Formulierungen suchen**

Run: `ctx_search(pattern='tool_profile = "standard"|via ctx_call gateway|via the .ctx_call. gateway|Power tools ONLY|standard tools|3\\.7\\.x', path=".claude/rules/subagent-multi-agent.md")`
Erwartet: **0 Treffer**. Falls Treffer (außer bewusst belassener Profil-Referenz „`standard` = 22" in Task 1): per `ctx_edit` analog zu Tasks 1/4 angleichen, dann reformat + committen.

- [ ] **Step 2: Gegenprobe — `power` & neue Tools präsent**

Run: `ctx_search(pattern='tool_profile = power|ctx_checkpoint|ctx_rules|in_progress is NOT valid', path=".claude/rules/subagent-multi-agent.md")`
Erwartet: Treffer für `tool_profile = power`, `ctx_checkpoint`, `ctx_rules`, `in_progress is NOT valid`.

- [ ] **Step 3: Gesamtdiff sichten**

Run: `git diff main -- .claude/rules/subagent-multi-agent.md`
Erwartet: nur die in Tasks 1–4 geplanten Änderungen; keine ungewollten Zeilen.

- [ ] **Step 4 (nur falls Step 1 etwas geändert hat): Commit**

```bash
git add .claude/rules/subagent-multi-agent.md
git commit -m "docs(rules): sweep residual standard-profile/gateway wording"
```

---

## Out of Scope (separat, nicht Teil dieses Plans)

- `docs/reference/08-multi-agent.md §6`: `ctx_task action=update state=in_progress` → der State `in_progress` existiert nicht; auf `working` korrigieren. Eigener Fix außerhalb der Rules-Datei.

---

## Self-Review

**Spec coverage:**
- Spec §1 (Header → power) → Task 1 ✓
- Spec §2 („NEVER wrap"-Regel generalisiert) → Task 1 (Header) + Task 4 (Dispatch Contract) ✓
- Spec §3 (`ctx_share`-Begründung) → Task 2 ✓
- Spec §4 (`ctx_task` verbessert + `ctx_checkpoint` + `ctx_rules`) → Task 3 ✓
- Spec §5 (Dispatch Contract) → Task 4 ✓
- Spec §6 (Residual-Gateway-Verweise angleichen) → Task 4 + Task 5 (Sweep) ✓
- Spec „Nicht geändert": `ctx_task`-States bleiben korrekt (Task 3 setzt exakt `working/input-required/completed/failed/canceled`); `ToolSearch`-Reflex, `ctx_shell`-Regeln, Serena-für-Rust, Test-Runner-Regeln werden von keinem Edit berührt ✓

**Placeholder scan:** Keine TBD/TODO; jeder Edit zeigt vollständige `old_string`/`new_string`. ✓

**Type/String consistency:** `old_string`-Anker stammen wörtlich aus dem aktuellen Dateiinhalt; `power` durchgängig als `tool_profile = power`; Tool-Namen `ctx_checkpoint`/`ctx_rules` in Task 3 (Tabelle) und Task 1/Task 4 (Listen) identisch geschrieben. Markdown-Pipe-Escape (`\|`) in Task 3 berücksichtigt. ✓
