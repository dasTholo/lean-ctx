# Appendix — JetBrains-Plugin (Agent-Referenz)

> Knappe Lookup-Tabellen für Agents: jede `ctx_refactor`-Action, ihr HTTP-Endpunkt,
> Schlüssel-Parameter, Backing. Ausführliche Beschreibung (curl, Responses, Guards,
> Architektur, E2E): **[Journey 18 — JetBrains-Plugin](18-jetbrains-plugin-de.md)**.
>
> Sprache: Deutsch; Tool-/Endpunkt-/Parameter-Namen und Error-Codes englisch.
> Serena-Abgrenzung: eigenständiger Nachbau (nicht abgeleitet), lean-ctx-Lizenz —
> löst Serena + offizielles JetBrains-MCP als Code-Intelligence-Schnittstelle ab.

## Koordinaten & Aufruf

- Aufruf: `ctx_refactor action=<action> …` (MCP) oder `POST 127.0.0.1:<port><endpoint>`
  mit Header `X-LeanCtx-Token: <token>`, Body = JSON.
- `ctx_refactor`-Ebene: `line` **1-indexed**, `column` **0-indexed**.
  Wire-Ebene: `line`/`character` **0-based**; `line` in `type_hierarchy`/
  `symbols_overview`/`inspections`-Antworten **1-based**.
- Fachlicher Negativfall: HTTP 200 + `{"error":{"code","message"}}`.

## Funktionen

| Action                 | HTTP-Endpunkt                                  | Zweck                             | Key-Parameter                                          | Backing                     |
|------------------------|------------------------------------------------|-----------------------------------|--------------------------------------------------------|-----------------------------|
| `references`           | `POST /references`                             | semantische Verwendungen          | `path`, `line`, `column`, `scope`                      | B (+A-Fallback)             |
| `definition`           | `POST /definition`                             | Sprung zur Definition             | `path`, `line`, `column`                               | B (+A-Fallback)             |
| `implementations`      | `POST /implementations`                        | Implementierungen/Overrides       | `path`, `line`, `column`, `scope`                      | B (+A-Fallback)             |
| `declaration`          | `POST /declaration`                            | Deklaration                       | `path`, `line`, `column`                               | B-only                      |
| `type_hierarchy`       | `POST /type_hierarchy`                         | Super-/Subtypen-Baum              | `path`, `line`, `column`, `direction`                  | B-only                      |
| `symbols_overview`     | `POST /symbols_overview`                       | Top-Level-Symbole der Datei       | `path`                                                 | B (+headless tree-sitter)   |
| `inspections`          | `POST /inspections`, `POST /list_inspections`  | Inspektionen ausführen/auflisten  | `path`, `mode=run\|list`                               | B-only                      |
| `replace_symbol_body`  | `POST /replaceSymbolBody`                      | Symbol-Rumpf ersetzen             | `name_path`/`path`+`line`, `new_body`, `expected_hash` | B (+headless)               |
| `insert_before_symbol` | `POST /insertBeforeSymbol`                     | Geschwister davor einfügen        | `name_path`, `text`, `expected_hash`                   | B (+headless)               |
| `insert_after_symbol`  | `POST /insertAfterSymbol`                      | Geschwister danach einfügen       | `name_path`, `text`, `expected_hash`                   | B (+headless)               |
| `rename`               | `POST /renamePreview` → `/renameApply`         | Symbol + alle Usages umbenennen   | `new_name`, `force`, `search_comments`                 | B-only (`BACKEND_REQUIRED`) |
| `reformat`             | `POST /reformat`                               | Datei in-place formatieren        | `path`                                                 | B-only                      |
| `move`                 | `POST /movePreview` → `/moveApply`             | Symbol verschieben + Referenzen   | Ziel, `force`                                          | B-only (`BACKEND_REQUIRED`) |
| `safe_delete`          | `POST /safeDeletePreview` → `/safeDeleteApply` | löschen, wenn keine Blocker       | `force`                                                | B-only (`BACKEND_REQUIRED`) |
| `inline`               | `POST /inlinePreview` → `/inlineApply`         | Symbol an Aufrufstellen einsetzen | `force`                                                | B-only (`BACKEND_REQUIRED`) |

Backing: **B** = JetBrains-IDE (Plugin via HTTP); **A** = rust-analyzer (headless);
**headless** = tree-sitter / `local_range_write` ohne IDE. Refactoring-Engine
(`rename`/`move`/`safe_delete`/`inline`) ist Two-Phase (`*Preview`→`*Apply`,
`plan_hash`-geschützt) und hat keinen headless-Pfad.

> `find_symbol` (Serena) → `ctx_symbol` / `ctx_outline`, nicht `ctx_refactor`.

## Guards (Kurzform)

- **BLAKE3-Conflict-Guard** (`expected_hash` Edits / `plan_hash` Refactoring) —
  Rust-zentral; Plugin hasht nicht. Mismatch → `CONFLICT`.
- **PathJail** — jede Mutation + jeder zurückgemeldete Pfad gegen `project_root`.
- **Smart-Mode** — Dumb-Mode → `INDEXING` (kein Teilergebnis).
- **Auth** — `X-LeanCtx-Token` pro Projekt; Fehlen → 401. Nur `127.0.0.1`.

## Fehler-Codes

| Code                    | Auslöser                                | Behebung                          |
|-------------------------|-----------------------------------------|-----------------------------------|
| `UNAUTHORIZED` (401)    | Token fehlt/falsch                      | gültigen `X-LeanCtx-Token` senden |
| `NOT_FOUND` (404)       | unbekannte Route                        | Endpunkt-Pfad prüfen              |
| `FILE_NOT_FOUND`        | Datei nicht lesbar                      | Pfad mit `ctx_tree` verifizieren  |
| `POSITION_OUT_OF_RANGE` | Zeile/Spalte hinter EOF                 | Range neu auflösen                |
| `CONFLICT`              | Hash-Mismatch oder Konflikte ∧ `!force` | frisch lesen; ggf. `force`        |
| `AMBIGUOUS_SYMBOL`      | `name_path` > 1 Treffer                 | qualifizieren (`Class/method`)    |
| `NO_SYMBOL`             | `name_path` 0 Treffer                   | Name/Pfad korrigieren             |
| `INDEXING`              | IDE im Dumb-Mode                        | warten, erneut                    |
| `UNSUPPORTED_LANGUAGE`  | kein LSP/PSI-Processor                  | Sprache nicht unterstützt         |
| `BACKEND_REQUIRED`      | Refactoring ohne IDE                    | IDE mit Projekt öffnen            |
| `INTERNAL`              | sonstiger Fehler                        | `message` prüfen                  |

## Siehe auch

- [Journey 18 — JetBrains-Plugin](18-jetbrains-plugin-de.md) — Vollreferenz
- [MCP-Tool-Map](appendix-mcp-tools.md) · [Per-IDE-Quickstarts](appendix-ide-quickstarts.md)
