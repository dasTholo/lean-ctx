# Design-Spec: Verpflichtende lean-ctx-Regeln für Subagent-Driven Execution

- **Datum:** 2026-06-01
- **Status:** Approved (Design)
- **Branch:** feat-lmd-v1
- **Quellen:** `docs/reference/08-multi-agent.md`, `docs/reference/03-memory-and-knowledge.md`, `docs/reference/05-advanced.md §7`
- **Skill-Kontext:** `superpowers:subagent-driven-development`

---

## 1. Problem & Motivation

Beim Ausführen von Plänen via `superpowers:subagent-driven-development` dispatcht der
Controller **pro Task einen frischen Subagenten** mit einem bewusst *isolierten*,
selbst gebauten Prompt (der Plan-Text wird hineingereicht; der Subagent liest die
Plan-Datei nicht selbst). Die lean-ctx-Tool-Disziplin steht heute in `CLAUDE.md` —
die wird von Subagenten zwar mitgeladen, aber der gecraftete Task-Prompt erinnert
nicht daran, und die **proaktiven** Multi-Agent-/Memory-Tools werden vom
superpowers-Flow gar nicht genutzt.

### Was bereits hart erzwungen ist (Bestandsaufnahme)

`.claude/settings.local.json` enthält bereits Hooks:

| Hook | Matcher | Kommando |
|------|---------|----------|
| PreToolUse | `Bash\|bash` | `lean-ctx hook rewrite` |
| PreToolUse | `Read\|Grep\|Search\|ListFiles\|...` | `lean-ctx hook redirect` |
| PostToolUse | `.*` | `lean-ctx hook observe` |
| PreCompact / UserPromptSubmit | — | `lean-ctx hook observe` |

**Verifiziert (Quelle: code.claude.com/docs — hooks.md, memory.md, subagents.md):**
Ein per Task-Tool dispatchter Subagent läuft in eigener Session, **lädt aber dieselben
Projekt-Dateien neu** — sowohl `.claude/settings.local.json` (→ die Hooks feuern auch
in der Subagent-Session) als auch projekt- und globale `CLAUDE.md`. Daraus folgt:

→ Die **native-Tool-Umleitung** (`Read/Grep/Bash/ListFiles` → `ctx_*`) ist für
Subagenten **bereits hart erzwungen**. Sie braucht keine zusätzliche Regel.

### Die eigentliche Lücke

Ein Hook kann nur *vorhandene* native Calls umleiten — er kann keine **proaktiven,
additiven** Verhaltensweisen *injizieren*. Genau diese fallen durch:

- **Koordination (`08`):** `ctx_agent` register/diary/status/sync/handoff, `ctx_share` push/pull
- **Memory (`03`):** `ctx_overview` (Start), `ctx_session` task/finding/decision, `ctx_knowledge` remember/recall

Zusätzlicher Effekt: Ein frischer Subagent hat einen **kalten Cache** — die
persistenten File-Refs (`F1…`) des Controllers sind für ihn wertlos. Entweder
pusht der Controller Kontext via `ctx_share` (→ warmer Cache, kein `fresh` nötig,
die Projektregel „no `fresh`" bleibt heil), oder der Subagent müsste `fresh=true`
lesen (Regelverstoß). Sauber nur über den Push-Weg.

---

## 2. Gewählte Lösung

**Koordinationstiefe:** *Standard + warmer Cache-Handoff* (standard-Tool-Profil).
Kein `ctx_task`-Board, keine deterministischen `ctx_handoff`-Bundles (= „Volles A2A",
bewusst abgewählt).

**Architektur: B + C**

- **B — Zuhause:** neues `.claude/rules/subagent-multi-agent.md`, via `@` aus
  projekt-`CLAUDE.md` importiert (Muster wie `@rules/lean-ctx.md`). Gespiegelt in
  `AGENTS.md` für Nicht-Claude-Agenten.
- **C — Dispatch-Contract:** ein kopierbarer Pflichtblock in derselben Datei, den
  der Controller **jedem** Subagent-Prompt voranstellen MUSS. Das ist der harte
  Hebel, weil der gecraftete Task-Prompt das dominante Signal im isolierten
  Subagenten ist.

---

## 3. Komponenten — drei Rollen-Verträge

### 3.1 Controller-Vertrag (Haupt-Agent, fährt den Plan)

Hard rules:

1. **Plan-Start:** `ctx_overview "<plan-thema>"`; Session-Restore prüfen.
2. Einmalig `ctx_agent action=register agent_type=claude role=plan`.
3. Plan-Fakten persistieren: `ctx_knowledge action=remember category=decision …`
   **und** `ctx_agent action=share_knowledge msg="key=val;…"` (fürs Team).
4. **Pro Task, VOR dem Dispatch:** relevante Quelldateien per `ctx_read` warm lesen,
   dann `ctx_share action=push to_agent=<sub-id> paths=[…]` (= warmer Cache-Handoff).
5. Den **Dispatch-Contract (§3.4)** jedem Subagent-Prompt voranstellen.
6. **Nach jedem Task:** `ctx_session action=task value="<task> [N%]"`; dauerhafte
   Fakten via `ctx_knowledge remember`.
7. Team-Status via `ctx_agent action=sync` (statt manuellem Nachfragen).

### 3.2 Implementer-Subagent-Vertrag

1. **Start:** `ctx_agent action=register agent_type=subagent role=dev` +
   `ctx_share action=pull` (warmen Cache des Controllers ziehen) → **kein `fresh`**.
2. Reads/Search/Shell explizit als `ctx_read`/`ctx_search`/`ctx_shell` ohne `fresh`
   (Hook leitet native ohnehin um; explizit hält den Cache konsistent).
3. **Rust-Edits via Serena** (`replace_symbol_body`, `insert_*`, `rename`/`move`/
   `safe_delete`) — Projektregel, nie native `Edit`/`ctx_edit` auf `*.rs`.
4. **Während:** `ctx_agent action=diary category=<discovery|decision|blocker|progress|insight>`
   bei signifikanten Schritten.
5. **Ende:** `ctx_agent action=post category=status msg="…"` + Status
   (DONE / DONE_WITH_CONCERNS / NEEDS_CONTEXT / BLOCKED);
   `ctx_agent action=handoff to_agent=<controller>` als Baton.
6. Gotchas/dauerhafte Fakten: `ctx_knowledge action=remember`.

### 3.3 Reviewer-Subagent-Vertrag (spec-reviewer + code-quality-reviewer)

1. **Start:** `ctx_agent action=register agent_type=subagent role=review` +
   `ctx_share action=pull`.
2. Findings zusätzlich via `ctx_agent action=post category=finding` (nicht nur
   Text-Return an den Controller).
3. `ctx_agent action=diary` für nicht-triviale Urteile.

### 3.4 Dispatch-Contract (Block C — vom Controller vorangestellt)

Kopierbarer Block, der jedem Implementer-/Reviewer-Prompt vorangestellt wird:

```text
## lean-ctx Subagent Contract (MANDATORY)
You run in an isolated context. Before any other action:
1. ctx_agent action=register agent_type=subagent role=<dev|review>
2. ctx_share action=pull          # warm cache from controller — DO NOT use fresh=true
Tool discipline:
- Reads/search/shell → ctx_read / ctx_search / ctx_shell (never fresh, never raw)
- Rust (*.rs) edits → Serena tools only (never native Edit / ctx_edit)
During work: ctx_agent action=diary category=<discovery|decision|blocker|progress>
On finish:
- ctx_agent action=post category=<status|finding> msg="<summary>"
- ctx_agent action=handoff to_agent=<controller-id> msg="<baton>"
- ctx_knowledge action=remember for any durable fact/gotcha
Report final status: DONE | DONE_WITH_CONCERNS | NEEDS_CONTEXT | BLOCKED
```

---

## 4. Konflikt-Auflösung (gegen bestehende Mechanismen geprüft)

| Bestehender Mechanismus | Spannung | Auflösung |
|-------------------------|----------|-----------|
| **Hooks** (`redirect`/`rewrite`) | Würde Regel native→`ctx_*` duplizieren | Regeln **verweisen** nur, duplizieren nicht. Hooks bleiben die Durchsetzung. |
| **superpowers-Skill** („Controller liefert vollen Task-Text, Subagent liest Plan nicht") | Scheinbar gegen `ctx_share` | Kein Widerspruch: `ctx_share` betrifft **Quelldateien**, nicht den Plan. Voller Task-Text bleibt im Prompt. |
| **Projektregel „no `fresh`/`raw`"** | Kalter Subagent-Cache | Gelöst durch `ctx_share push→pull`. Einzige Ausnahme: kein Push erfolgt → `fresh=true` beim **allerersten** Read, explizit dokumentiert. |
| **lean-ctx `rules sync`** (managed `<!-- lean-ctx -->`-Blöcke) | Würde Hand-Inhalt überschreiben | Unseren Inhalt **außerhalb** der `<!-- lean-ctx -->`-Marker platzieren. |
| **Plugin-Cache-Skill-Dateien** | Direktes Patchen wäre fragil | **Nicht** patchen; Verhalten über Dispatch-Contract injizieren. |

---

## 5. Betroffene Dateien

1. **NEU** `.claude/rules/subagent-multi-agent.md` — drei Verträge (§3.1–3.3) +
   Dispatch-Contract (§3.4).
2. `CLAUDE.md` (projekt) — neuer Abschnitt „## Subagent-Driven Execution" mit
   2–3 Sätzen (wann es greift) + `@rules/subagent-multi-agent.md`-Import.
3. `AGENTS.md` — gespiegelter Abschnitt für Nicht-Claude-Agenten, **außerhalb** der
   `<!-- lean-ctx -->`-Marker.

`lean-ctx rules` wird **nicht** aufgerufen (Inhalt ist handgeschrieben, nicht Teil
der auto-generierten Blöcke; zentrale `rules.toml` existiert noch nicht).

---

## 6. Erfolgskriterien

- Ein dispatchter Implementer ruft nachweislich `ctx_agent register` +
  `ctx_share pull` + `ctx_agent diary` + `ctx_agent handoff` auf.
- `ctx_agent action=sync` zeigt nach einem Plan-Lauf: `plan`/`dev`/`review`-Agenten
  + Diaries + shared knowledge.
- Keine `fresh`-Reads in Subagent-Transkripten (außer dokumentierter Ausnahme).
- `lean-ctx rules diff` meldet keine Drift (Inhalt außerhalb der Marker).

---

## 7. Nicht-Ziele (YAGNI)

- Kein `ctx_task`-Board, keine deterministischen `ctx_handoff`-Bundles (Volles A2A).
- Kein `lean-ctx rules init` / zentrale `.leanctx/rules.toml` (separate Entscheidung).
- Keine Änderungen an globaler `~/.claude/CLAUDE.md` (projekt-scoped genügt).
- Keine Patches an superpowers-Skill-Dateien im Plugin-Cache.
