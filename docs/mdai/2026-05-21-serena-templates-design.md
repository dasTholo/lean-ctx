# Serena-Templates als MDAI-Macros — Design

Datum: 2026-05-21 · Status: **Entwurf** 
Bezug: `docs/mdai/design-skill-integration.md`, `mdai-benchmark.md`

## 1. Zielsetzung

Eine MDAI-Macro-Bibliothek `docs/mdai/macros/serena.md`, die jedes weiterhin
nützliche Serena-Tool als `@define`-Macro kapselt. Pläne ziehen sie via
`@import macros/serena.md` und rufen sie mit `@call serenaXxx(...)`. Damit:

- Refactor-Operationen, die lean-ctx mit dem aktuellen v3.6.12-CLI nicht
  semantisch korrekt abbilden kann, bleiben über Serena verfügbar — aber
  einheitlich getemplated, nicht copy-paste-mässig.
- Pläne dokumentieren *implizit*, welche Serena-Tools die richtigen sind
  (Use-Case-Hinweise sind Teil jedes Macros).
- Eine Änderung an einem Tool-Aufruf (z. B. nach Serena-Update mit anderem
  Parameter-Namen) propagiert über die Macro-Datei in alle Pläne, die sie
  importieren — kein N-faches Update von Hand.

**Erfolgskriterien:**

1. `docs/mdai/macros/serena.md` enthält genau die unter §2 gelisteten 13 Tools
   als `@define`-Macros mit Use-Case-Doku.
2. `docs/mdai/macros/serena.demo.md` ruft jeden Macro einmal mit Beispielargs
   auf; `mai render serena.demo.md` zeigt alle Expansionen ohne Fehler.
3. In `lean-ctx gotchas` sind die zwei Companion-Policy-Pitfalls (siehe §3)
   mit Tag `mdai` registriert.
4. Bestehende Pläne werden nicht automatisch migriert — Macros sind ein
   *Opt-in*-Werkzeug für neue Pläne.

## 2. Scope — was rein, was raus

**Rein (13 Tools, kein gleichwertiges lean-ctx-Äquivalent):**

| Tool                                               | Zweck                                                   | Warum nicht durch lean-ctx                                                                                      |
|----------------------------------------------------|---------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------|
| `mcp__serena__jet_brains_find_symbol`              | Symbol projekt-weit finden (mit/ohne Body)              | `ctx_search` ist regex, kennt keine Symbol-Auflösung                                                            |
| `mcp__serena__jet_brains_find_referencing_symbols` | Wer ruft mich auf?                                      | `lean-ctx graph build` ist statisch, löst keine Generics                                                        |
| `mcp__serena__jet_brains_find_declaration`         | Wo ist das definiert?                                   | Tree-sitter findet das Token, nicht die Auflösung                                                               |
| `mcp__serena__jet_brains_find_implementations`     | Trait-/Interface-Impls projektweit                      | Statische Analyse löst Blanket-Impls nicht                                                                      |
| `mcp__serena__jet_brains_type_hierarchy`           | Super-/Subtypen                                         | nicht vorhanden in lean-ctx                                                                                     |
| `mcp__serena__jet_brains_get_symbols_overview`     | File-Outline mit Serena-internen Symbol-IDs             | `ctx_read mode=signatures` zeigt Signaturen, liefert aber keine IDs für `replace_symbol_body`/`insert_*_symbol` |
| `mcp__serena__jet_brains_replace_symbol_body`      | Funktion/Methode atomar ersetzen (Symbol-ID)            | `ctx_edit` ist string-basiert, bricht bei Duplikaten                                                            |
| `mcp__serena__jet_brains_insert_before_symbol`     | Vor Symbol einfügen (Symbol-ID)                         | string-basierte Inserts sind fehleranfällig                                                                     |
| `mcp__serena__jet_brains_insert_after_symbol`      | Nach Symbol einfügen (Symbol-ID)                        | string-basierte Inserts sind fehleranfällig                                                                     |
| `mcp__serena__jet_brains_rename`                   | Cross-file Symbol-Rename mit Re-Export-/Trait-Auflösung | `lean-ctx` hat im offiziellen CLI kein `refactor`-Command                                                       |
| `mcp__serena__jet_brains_move`                     | Symbol / Datei / Verzeichnis verschieben                | nicht vorhanden in lean-ctx                                                                                     |
| `mcp__serena__jet_brains_safe_delete`              | Symbol löschen mit Use-Site-Analyse                     | Tree-sitter sieht keine semantischen Use-Sites                                                                  |
| `mcp__serena__jet_brains_inline_symbol`            | Symbol-Aufrufe durch Definition ersetzen                | nicht vorhanden in lean-ctx                                                                                     |

`run_inspections` und `list_inspections` sind in §6 als **Borderline-Add-on**
notiert — sinnvoll, aber überlappen je nach Projekt mit `lean-ctx smells`.
Sie kommen rein, wenn die spätere Praxis zeigt, dass JetBrains-Inspections im
Plan-Workflow regelmässig gebraucht werden.

`jet_brains_debug` bleibt aussen vor — die Macro-Bibliothek soll für
*Plan-Subagents* nützlich sein, nicht für interaktives Debugging.

**Raus (lean-ctx deckt das ab):**

| Serena-Tool                                                                                     | lean-ctx-Ersatz                                            |
|-------------------------------------------------------------------------------------------------|------------------------------------------------------------|
| `write_memory`, `read_memory`, `list_memories`, `edit_memory`, `delete_memory`, `rename_memory` | `ctx_knowledge.remember/recall/search/remove/status`       |
| `replace_content`                                                                               | native `Edit` (bzw. `ctx_edit` falls Read nicht verfügbar) |
| `onboarding`, `initial_instructions`, `serena_info`                                             | `ctx_overview` + `ctx_knowledge`                           |

## 3. lean-ctx Companion-Policy

Templates sind nur die halbe Miete — sie müssen mit den richtigen Read- und
Shell-Defaults kombiniert werden, sonst geht die Token-Ersparnis verloren.

### 3.1 `ctx_read` — Mode-Selection-Order

| Situation                  | Default-Mode             | Begründung                                               |
|----------------------------|--------------------------|----------------------------------------------------------|
| Erkunden ohne Edit-Absicht | `map` oder `signatures`  | Deps/Exports bzw. Tree-sitter-API — Bruchteil der Tokens |
| Konkrete Zeilen brauchen   | `lines:N-M`              | nur das nötige Fenster                                   |
| Re-Read nach Edit          | `diff`                   | nur geänderte Zeilen                                     |
| Kontext-only, grosse Datei | `aggressive` / `entropy` | maximal komprimiert                                      |
| Datei wird gleich editiert | `full`                   | einziger legitimer `full`-Fall                           |

**Faustregel:** `full` ohne nachfolgenden Edit ist Token-Verschwendung.
Plan-Templates und Skills setzen `map` oder `signatures` als ersten Read-Mode.

### 3.2 `ctx_shell` — `raw=true` nur als Ausnahme

`raw=true` schaltet die 60+ Kompressions-Patterns aus → voller stdout-Dump.
Nur zulässig wenn:

- Pattern-Filter den Output kaputt-komprimiert (Bytes-roh-Vergleich nötig)
- Output maschinell weitergeleitet wird (JSON-Pipe, Parse)
- Die Kompression selbst gedebuggt wird

Default = **ohne `raw`**. Bei unklarem komprimiertem Output erst
`lean-ctx filter list` prüfen, ggf. `lean-ctx filter init` für
projekt-spezifische Filter — `raw=true` ist die letzte Stufe, nicht die erste.

### 3.3 Verankerung der Policy

| Ort                                                                           | Form                                                        |
|-------------------------------------------------------------------------------|-------------------------------------------------------------|
| Header-Kommentar in `macros/serena.md`                                        | Kurz-Hinweis mit Verweis auf dieses Design                  |
| `lean-ctx gotchas` (Tag `mdai`)                                               | „`ctx_read mode=full` ohne Edit → nutze `map`/`signatures`" |
| `lean-ctx gotchas` (Tag `mdai`)                                               | „`ctx_shell raw=true` als Default → Kompression aus"        |
| `mdai-plans`-Skill (sobald vorhanden, siehe `design-skill-integration.md` §4) | Drift-Checkliste im Skill-Body                              |

## 4. Macro-Konvention

Pro Tool ein `@define`-Block, davor ein Markdown-Block mit *Use-Case* und
*Beispiel*. Naming: `serena<ToolName>` in camelCase. Skizze:

````markdown
## serenaRename

**Use:** Cross-file Symbol-Rename mit Re-Export- und Trait-Impl-Auflösung.
NICHT durch `ctx_edit replace_all` ersetzen — das verfehlt Re-Exports und
benannte Re-Exports wie `pub use foo as bar`.

**Beispiel:**

```
@call serenaRename("crate::lock::BoundedLock", "crate::lock::AsyncBoundedLock")
```

@define serenaRename(symbol_path, new_name)
Tool: `mcp__serena__jet_brains_rename`
Args:

- `name_path`: `{{ symbol_path }}`
- `new_name`: `{{ new_name }}`
  @end
````

Der `@define`-Body ist **deklarativ** (Tool-Name + Args), nicht ausführbarer
Code. Der Subagent liest die Expansion und ruft das echte MCP-Tool selbst auf.
Das ist bewusst so:

- Sprach-agnostisch — keine Bindung an Python/JS-Sprach-Bridge.
- Idempotent reproduzierbar — die Args-Liste ist ein YAML-/Markdown-Vertrag.
- Audit-fähig — der Plan zeigt, *was* aufgerufen wurde, ohne Tool-Call-Logs.

**Argument-Konvention:**

- Symbol-Pfade als String, immer in doppelten Quotes (z. B. `"crate::foo::Bar"`)
- File-Pfade projekt-relativ (`"crates/rpc/src/lib.rs"`)
- Boolean-Flags als `true` / `false` ohne Quotes
- Listen als JSON-Array-Strings (`"[\"a\", \"b\"]"`) wenn das Tool sie braucht

## 5. Renderbarkeit

`@define`-Macros sind im Render **unsichtbar** by design (siehe
`mdai-benchmark.md` § Korrektur v1). Drei Sichten ergeben sich:

| Sicht                                  | Sieht der Mensch was?                                   |
|----------------------------------------|---------------------------------------------------------|
| `ctx_read full macros/serena.md`       | Doku + Macro-Bodies (Markdown-Quellcode)                |
| `mai render macros/serena.md` (direkt) | nur Doku-Markdown; `@define`-Bodies fehlen              |
| `mai render <plan-mit-@call>`          | Bodies werden an `@call`-Stellen vollständig expandiert |

Um auch beim direkten Render die vollständige Sicht zu haben:

**Demo-Datei `docs/mdai/macros/serena.demo.md`** — One-Pager mit
`@import macros/serena.md` und je einem `@call` pro Macro mit
Beispielargumenten. `mai render serena.demo.md` zeigt dann alle Expansionen
und dient gleichzeitig als Smoke-Test.

## 6. Speicherort & Integration

- Design-Doc: `docs/mdai/2026-05-21-serena-templates-design.md` (dieses Doku)
- Macro-Datei: `docs/mdai/macros/serena.md`
- Demo-Datei: `docs/mdai/macros/serena.demo.md`

**Konsistenz mit bestehender Struktur:**

- Header `@markdownai v1.0` in beiden Dateien (wie `tmp/mdai-bench/macros/*`)
- Naming-Konvention: `serena<ToolName>` camelCase, analog zu
  `stepReformatCommit` aus `tmp/mdai-bench/macros/step-reformat-commit.md`
- Verzeichnis `docs/mdai/macros/` ist neu — bisher liegt der einzige
  Macro-Ordner unter `tmp/mdai-bench/macros/`. Das Design legt das neue
  Verzeichnis als kanonischen Ort für *stabile* Macros fest; `tmp/mdai-bench/`
  bleibt für Benchmark-/Experiment-Macros.

**Spätere Migration in `lean-ctx pack`:**

Sobald `design-skill-integration.md` §7a.1 umgesetzt ist, wandert sowohl
`hard-rules.md`, `tool-quick-ref.md`, `step-reformat-commit.md` *als auch*
`serena.md` in das `mdai-macros`-Pack. Der Migrationsschritt ist trivial
(`lean-ctx pack create mdai-macros docs/mdai/macros/`), aber nicht Teil
dieses Designs — er ist Teil der Skill-Integration-Spec.

## 7. Non-Goals

- Keine Wrapper für Serena-Tools, die lean-ctx besser/gleichwertig macht.
- Keine LSP-Modus-Variante. Tool-Namen sind `mcp__serena__jet_brains_*`,
  weil hier das JetBrains-Plugin läuft. Bei Wechsel auf den LSP-Modus
  müssen die Präfixe einmalig per Find-Replace nachgezogen werden.
- Keine Auto-Generierung der Macros aus dem MCP-Tool-Schema. Manuell
  gepflegt, weil die Use-Case-Hinweise und Anti-Pattern-Notizen das
  Wertvolle sind, nicht die Argumentlisten.
- Keine automatische Migration bestehender Pläne. Macros sind ein
  Opt-in-Werkzeug für neue Pläne; alte Pläne bleiben wie sie sind, bis
  jemand sie aktiv umstellt.
- Kein Wrapper für `jet_brains_debug`. Interaktives Debugging gehört nicht
  in eine Plan-Macro-Bibliothek.

## 8. Risiken

| Risiko                                                            | Schweregrad | Mitigation                                                                                                            |
|-------------------------------------------------------------------|-------------|-----------------------------------------------------------------------------------------------------------------------|
| JetBrains-Plugin-Tool-Namen ändern sich bei Serena-Update         | Mittel      | Macro-Datei zentralisiert — ein Find-Replace genügt; Macro-Doku verweist auf Serena-Release-Notes                     |
| `@define`-Unsichtbarkeit verwirrt User                            | Niedrig     | Demo-Datei (§5) plus Hinweisblock im Header der Macro-Datei                                                           |
| Pläne nutzen `@call serenaXxx` ohne `@import macros/serena.md`    | Niedrig     | MDAI-Renderer warnt bei unaufgelöstem `@call`; im späteren `mdai-plans`-Skill als Lint-Check                          |
| Serena hier ist JetBrains-Modus — bei LSP-Wechsel droht Bruch     | Mittel      | §7 dokumentiert die Migration; Tool-Namen-Präfix ist die einzige Stelle, die sich ändert                              |
| `run_inspections`/`list_inspections` werden später doch gebraucht | Niedrig     | Nachträgliches Hinzufügen ist trivial (zwei weitere `@define`-Blöcke); Scope-Erweiterung als kleines Follow-up-Update |
| Companion-Policy (§3) wird ignoriert, weil sie nur Doku ist       | Mittel      | `gotchas`-Einträge wirken zur Plan-Laufzeit; im `mdai-plans`-Skill (sobald da) als Hard-Check verankert               |

## 9. Umsetzungs-Schritte (high-level)

1. Verzeichnis `docs/mdai/macros/` anlegen.
2. `docs/mdai/macros/serena.md` schreiben:
    - `@markdownai v1.0`-Header plus Companion-Policy-Hinweis (§3)
    - 13 `@define`-Blöcke gemäss §2, je mit Use-Case-Doku gemäss §4
3. `docs/mdai/macros/serena.demo.md` schreiben:
    - `@import macros/serena.md`
    - 13 `@call`-Beispiele mit Demo-Argumenten
4. `mai render docs/mdai/macros/serena.demo.md` ausführen, Expansionen visuell verifizieren.
5. `lean-ctx gotchas add` für die zwei §3-Pitfalls mit Tag `mdai`.
6. Im nächsten echten Refactor-Plan einmal verwenden (z. B. `rust/src/server/bounded_lock.rs`-Refactor), Erfahrung in
   `mdai-benchmark.md` als Update-Eintrag dokumentieren.
7. Optional, nach Praxiserfahrung: Scope-Erweiterung um `run_inspections`/`list_inspections` (siehe §2 Anmerkung).

## 10. Offene Punkte

- **Symbol-Pfad-Syntax in `name_path`:** Serena akzeptiert sowohl absolute
  (`crate::foo::Bar`) als auch dateibasierte (`src/lib.rs::Bar`) Pfade.
  Welche Form als Macro-Default? Vorschlag: absolute Modul-Pfade
  (`crate::...`) für Rust, Datei-Pfade für TS/JS. Wird im Macro-Body
  per Sprach-Heuristik entschieden — im ersten Wurf einheitlich
  absolute Pfade, Sprach-Spezifik nachziehen wenn nötig.
- **Mehrsprachen-Projekte:** Bei einem Repo mit Rust + TS müssen Calls
  je nach Ziel-Sprache evtl. unterschiedliche Argumente nutzen. Im
  ersten Wurf nicht modelliert — bei Bedarf zwei Macro-Varianten
  (`serenaRenameRust`, `serenaRenameTs`).
- **Auto-Lint im `mdai-plans`-Skill:** Soll der Skill bei einem `@call serenaXxx`
  prüfen, ob `@import macros/serena.md` im Plan-Header steht? Erst
  klären, wenn `mdai-plans` umgesetzt wird (siehe `design-skill-integration.md` §4).
