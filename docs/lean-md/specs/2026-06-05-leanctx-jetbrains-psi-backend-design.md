# Design-Spec: lean-ctx JetBrains-PSI-Backend (Serena-Ablösung / Backing B)

| Feld             | Wert                                                                          |
|------------------|-------------------------------------------------------------------------------|
| Status           | Draft (Design genehmigt 2026-06-05)                                           |
| Datum            | 2026-06-05                                                                    |
| Vorhaben         | Eigenständiges JetBrains-Plugin + Rust-Backend-Anbindung (Serena-Ablösung)    |
| Scope            | Kotlin/IntelliJ-Plugin (Backing B) + `LspBackend`-Refaktorierung im Rust-Kern |
| Nächster Schritt | `superpowers:writing-plans` (Implementierungsplan Phasen 0–5)                 |

---

## 1. Context — Warum

lean-ctx ist heute **read-only**; `ctx_refactor`/`ctx_symbol` fahren einen
**separaten** rust-analyzer (stdio-LSP). Gegenüber Serena fehlen: `type_hierarchy`,
IDE-Index-Konsistenz, symbolische PSI-Edits und Multi-Language ohne Standalone-LSP.

**Verifizierter Spike-Befund (2026-06-05):** Serena ist **kein** LSP-Client, sondern
ein IntelliJ-Platform-Plugin (`<depends>com.intellij.modules.platform</depends>`,
`de.oraios.serena`), das per `postStartupActivity` einen lokalen HTTP+JSON-Server
(`PostRequestHandler`/`SerenaBackendService`, gson) öffnet und die native **PSI-API**
nutzt. JetBrains exponiert PSI **nicht** extern → IDE-Genauigkeit
(`type_hierarchy`, PSI-genaue Edits, alle IDE-Sprachen) ist **nur** über ein
eigenes In-IDE-Plugin erreichbar.

**Ziel:** Ein eigenständig nachgebautes (nicht abgeleitetes) IntelliJ-Plugin in
`packages/jetbrains-lean-ctx`, das als **zweites Backend (B)** für lean-ctx dient
und so die Serena-Funktionslücken schließt — ohne Funktionsverlust gegenüber heute.

**Strategische Vision (bestätigt 2026-06-05):** lean-ctx soll künftig die
**alleinige Code-Intelligence-Schnittstelle** des Agenten sein — sowohl **Serena**
als auch das **offizielle JetBrains-MCP** (`mcp__jetbrains__*`) werden für
Code-Intelligence (Symbole, Navigation, Refactoring, Format, Inspektionen)
**entbehrlich**. Der Agent ruft nur `ctx_*`-Tools; lean-ctx-Rust spricht intern mit
Backing B (eigenes Plugin) oder Backing A (rust-analyzer). Deshalb **voller v1-Scope**
inkl. `format`/`inspections` (trotz Überlappung mit dem JetBrains-MCP) — eine
einheitliche, token-komprimierte, jail-geschützte Schnittstelle ohne Fremd-MCP.
(DB-/Run-/SQL-/Terminal-Funktionen des JetBrains-MCP sind **nicht** Scope — die deckt
lean-ctx über `ctx_shell` o. Ä. anders ab.)

---

## 2. Getroffene Architektur-Entscheidungen (vom User bestätigt)

| # | Frage                 | Entscheidung                                                                                                                                                                                                                                                                                                                     |
|---|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1 | Backend-Verhältnis    | **Koexistenz A+B, B-first.** Neues `LspBackend`-Trait. **Standard: zuerst `JetBrainsHttpBackend` (Backing B)** — nur wenn die IDE **nicht** erreichbar ist (keine/stale Port-Datei, `/health` schlägt fehl), Fallback auf `LspClient` (rust-analyzer, stdio = Backing A). A ist damit **Fallback** (CI/Headless), nicht Default. |
| 2 | Transport + Discovery | **HTTP/JSON auf 127.0.0.1** + **Port-Datei-Discovery**: Plugin schreibt Port+Token nach `~/.lean-ctx/jetbrains-<projecthash>.port`; Rust liest sie. Kein fester Port, kein Range-Scan.                                                                                                                                           |
| 3 | v1-Scope              | **Navigation + `type_hierarchy`** (+ Format/Inspections, read-only-artig). Edits (rename-apply/move/safe-delete/inline) = **v2-Ausblick**, nicht v1.                                                                                                                                                                             |
| 4 | Security/PathJail     | **Rust-PathJail (`jail_path`) ist alleiniger Validierungspunkt**, läuft VOR jedem HTTP-Request. Plugin re-validiert Pfade **nicht** (vertraut localhost-Caller), lauscht nur auf 127.0.0.1, verlangt Token.                                                                                                                      |

**Lizenz/Distribution (Frage 7 — vorgeschlagener Default, beim Spec-Review bestätigen):**
Eigenständiger Nachbau auf **Architektur-/Klassennamen-Ebene** (kein dekompilierter
Serena-Code). Lizenz = lean-ctx-Projektlizenz. Distribution v1: im lean-ctx-Repo
(`packages/jetbrains-lean-ctx`), **kein** JetBrains-Marketplace.

### 2.1 Abgrenzung gegen bereits verbundene MCPs (Befund 2026-06-05)

Vor dem Plugin-Bau wurde geprüft, was **bereits verbundene** MCPs an Code-Intelligence
liefern — das offizielle **JetBrains-MCP** (`mcp__jetbrains__*`) und **Serenas** MCP
(`mcp__serena__jet_brains_*`). Ergebnis (Evidenz = geladene Tool-Schemata):

| Op                                 | offiz. JetBrains-MCP                         | Serena-MCP                                 | echte Lücke    |
|------------------------------------|----------------------------------------------|--------------------------------------------|----------------|
| `find` (Symbol-Suche)              | `search_symbol` ✓                            | `jet_brains_find_symbol` ✓                 | nein           |
| `definition`                       | `get_symbol_info` ~teilw.                    | `find_symbol` ✓                            | teilweise      |
| `declaration`                      | `get_symbol_info` ~teilw.                    | `jet_brains_find_declaration` ✓            | teilweise      |
| **`references`**                   | ❌                                            | `jet_brains_find_referencing_symbols` ✓    | **nur Serena** |
| **`implementations`**              | ❌                                            | `jet_brains_find_implementations` ✓        | **nur Serena** |
| **`type_hierarchy`**               | ❌                                            | `jet_brains_type_hierarchy` ✓              | **nur Serena** |
| `overview`                         | `search_symbol` ~teilw.                      | `jet_brains_get_symbols_overview` ✓        | teilweise      |
| `format`                           | `reformat_file` ✓                            | (über IDE)                                 | gelöst         |
| `inspections`                      | `get_file_problems` + `run_inspection_kts` ✓ | `jet_brains_run_inspections` ✓             | gelöst         |
| `rename` (v2)                      | `rename_refactoring` ✓                       | `jet_brains_rename` ✓                      | gelöst         |
| `move`/`safe_delete`/`inline` (v2) | ❌                                            | `jet_brains_move`/`safe_delete`/`inline` ✓ | **nur Serena** |

**Schlussfolgerung (verändert die Motivation, nicht den Scope):**

1. Der **harte Kern** (`references`, `implementations`, `type_hierarchy` + symbolische
   Edits `move`/`safe_delete`/`inline`) fehlt dem offiziellen JetBrains-MCP **komplett**
   — heute löst ihn **nur Serena**. Das ist der eindeutige, einzigartige Mehrwert des
   eigenen Plugins und die Kern-Begründung des Vorhabens.
2. `format`/`inspections`/`rename`/`find`/`def`/`decl` sind vom JetBrains-MCP (teils
   Serena) **schon abgedeckt**. Sie werden im Plugin **trotzdem** gebaut (Entscheidung
   „voller v1"), damit lean-ctx die **alleinige** Code-Intelligence-Schnittstelle wird
   und **beide** Fremd-MCPs eingespart werden können — nicht weil eine Lücke besteht,
   sondern um Vereinheitlichung + Token-Kompression + Jail unter einem Dach zu haben.
3. Das JetBrains-MCP **beweist** den In-IDE-PSI-MCP-Weg, exponiert aber kein
   `find_references`/`type_hierarchy` → ein eigenes Plugin bleibt der einzige Weg zu
   diesen Ops **ohne** Serena.

---

## 3. Architektur

```
Claude/Agent → lean-ctx MCP (Rust) → ctx_refactor → router::select_backend
                                                        │
                          ┌─────────────────────────────┴───────────────┐
                          │ (IDE erreichbar?)                            │ (sonst)
                          ▼                                              ▼
              JetBrainsHttpBackend (B)                          LspClient (A)
              HTTP/JSON 127.0.0.1                               stdio rust-analyzer
                          │                                     (CI/headless-Fallback)
                          ▼
              IntelliJ-Plugin (Kotlin)
              HttpServer → RequestRouter → endpoint/* → PSI (ReadAction)
```

**Kernprinzip:** Die MCP-Tool-Schnittstelle (`ctx_refactor`/`ctx_symbol`) bleibt
stabil. Backend-Wahl ist intern und transparent. PathJail validiert vor dem Backend.

---

## 4. Rust-Seite — `LspBackend`-Trait-Refaktorierung

**Befund:** `lsp/router.rs` (111 Z.) ist hart an `LspClient` gekoppelt
(`static CLIENTS: HashMap<String, LspClient>` L11; `with_client` L45-81). Kein Trait.
`LspClient` (client.rs) bietet nur `references/definition/rename/implementations`
(L209-297) — **kein** `type_hierarchy/format/inspections/symbols_overview`.

### 4.1 Neuer Trait (`rust/src/lsp/backend.rs`)

- Pflicht-Methoden (in beiden Backings): `open_file`, `references`, `definition`,
  `implementations`, `rename` (bleibt — heutiger Pfad darf nicht brechen).
- **Default-degradierende** Methoden (Backing-B-bevorzugt, Default = klarer
  „nicht unterstützt"-`Err`): `declaration`, `type_hierarchy(direction)`,
  `symbols_overview`, `format`, `inspections`.
- Begleittypen: `HierarchyDirection {Subtypes, Supertypes}`, `TypeHierarchyNode`,
  `SymbolOverviewItem`, `InspectionDiag`.

### 4.2 Impls

- `impl LspBackend for LspClient` (client.rs): delegiert die 4+1 vorhandenen
  Methoden; Rest = Default-Err. `LspClient` selbst unverändert.
- `JetBrainsHttpBackend` (neu `lsp/jetbrains_backend.rs`): `base_url`/`token`,
  HTTP via **`ureq`** (synchron — passt zur synchronen `McpTool::handle`-Signatur,
  blockiert nicht den Tokio-Runtime). Serialisiert Request-DTO → POST → Response-DTO
  → mappt auf `lsp_types`.

### 4.3 Router-Umbau

- `HashMap<String, Box<dyn LspBackend>>`; `with_client` → `with_backend`
  (Closure-Param `&mut dyn LspBackend`). Die 4 Call-Sites in `ctx_refactor.rs`
  (L55/72/88/117) ändern nur den Param-Typ.
- **Factory `select_backend(language, project_root)`:**
    1. Port-Datei `~/.lean-ctx/jetbrains-<projecthash>.port` vorhanden + `pid`-Live
        + `/health`-Ping (Token, ~300 ms Timeout) → **Backing B**.
    2. Config erzwingt B aber IDE nicht erreichbar → sauberer Fehler.
    3. sonst → **Backing A** (`LspClient::start`, wie heute).
- **Config:** `cfg.lsp` ist `HashMap<String,String>` (config/mod.rs:272). **Default
  (kein Eintrag) = `"auto"` = B-first** (zuerst JetBrains, Fallback A) — entspricht
  Entscheidung 1. Magic-Value `"jetbrains"` = nur B (Fehler statt Fallback, wenn IDE
  fehlt); expliziter Binary-Pfad = nur A (erzwingt das heutige rust-analyzer-Verhalten).
  Kein Schema-Migrationszwang.
- **Latenz-Hinweis:** Da B-first jeden ersten Zugriff pro `(language, project_root)`
  einen `/health`-Ping kostet, wird das Selektionsergebnis in der `BACKENDS`-HashMap
  **gecacht** (nicht pro Call neu geprüft). Ohne IDE entsteht so genau **ein**
  Ping-Timeout (~300 ms), danach steht Backing A. Stale-Erkennung invalidiert den
  Cache-Eintrag und löst Re-Selektion aus.

### 4.4 `type_hierarchy` & Co. in den Tools

- **Neue Actions auf `ctx_refactor`** (kein neues Tool — vermeidet Nachziehen in
  `tool_profiles.rs`/`dynamic_tools.rs`/`workflow/types.rs`): `type_hierarchy`
  (`direction: subtypes|supertypes`, default supertypes), `overview`, `format`,
  `inspections`. Match-Block ctx_refactor.rs L33-46 + Hilfetext erweitern;
  `tool_def`-Schema um Actions + `direction` erweitern. **Hinweis (Changelog 3.7.4 #141 —
  Tool-Registry = single schema source):** `registered/ctx_refactor.rs` pflegt das Schema
  nicht mehr inline (alte L19-24), sondern ruft `crate::tool_defs::tool_def(...)`; die
  neuen Actions/`direction` werden über diese **eine** Registry ergänzt (Drift-Regression-Test),
  nicht als zweite handgepflegte Schema-Kopie.
- **Degradierung:** Backing A → Default-Trait-`Err` → sauberer `ERROR: …`-String
  (Muster wie heute L57), z. B. „type_hierarchy requires the JetBrains backend".
- `ctx_symbol` bleibt **unberührt** (nutzt graph_provider, nicht LSP).

### 4.5 ⚠ Sicherheitskritische Naht (Entscheidung 4)

`ctx_refactor::handle` baut `abs_path` heute **selbst** aus `project_root + path`
(L20-23) statt `ctx.resolved_path("path")` zu verwenden, das durch
`tool_trait.rs::resolve_path_sync` → `jail_path` (pathjail.rs L88-179) läuft.
Da das Plugin bewusst **nicht** re-validiert, MUSS sichergestellt sein, dass
`jail_path` greift, **bevor** der (relative) Pfad über die Wire geht — entweder
`ctx_refactor` auf `ctx.resolved_path` umstellen **oder** `jail_path` explizit in
`with_backend`/`select_backend` aufrufen. Sonst umgeht der selbstgebaute `abs_path`
das Jail. **Diese Umstellung ist Pflicht-Bestandteil von Phase 0.**

> **Re-Verifikation 2026-06-06 (Changelog 3.7.4 #145 — unified path resolution).** Der
> `abs_path`-Selbstbau in `ctx_refactor.rs:20-23` (`format!("{project_root}/{path}")`)
> besteht **weiterhin** — die Naht ist real noch offen, dieser §4.5-Befund bleibt gültig.
> `jail_path` liegt unverändert in `core/pathjail.rs:88`. **Neu/zuträglich:** die
> konsolidierte Pfadauflösung lebt jetzt zentral in `core::path_resolve::resolve_tool_path
> (project_root, shell_cwd, raw)` (`core/path_resolve.rs:32`). Bevorzugtes Phase-0-Fix-Ziel
> ist daher, `ctx_refactor` auf **`resolve_tool_path`** (bzw. `ctx.resolved_path` →
> `resolve_path_sync`, `tool_trait.rs:137`) umzustellen, statt den `abs_path` weiter selbst
> zu bauen — ein kanonischer Resolver statt einer dritten Ad-hoc-Variante.

### 4.6 Änderungsstellen

| Datei                                       | Änderung                                                |
|---------------------------------------------|---------------------------------------------------------|
| `rust/src/lsp/backend.rs`                   | NEU: Trait + Begleittypen                               |
| `rust/src/lsp/jetbrains_backend.rs`         | NEU: `JetBrainsHttpBackend` (ureq)                      |
| `rust/src/lsp/port_discovery.rs`            | NEU: projecthash, Port-Datei, Token, `/health`          |
| `rust/src/lsp/client.rs`                    | `impl LspBackend for LspClient`                         |
| `rust/src/lsp/router.rs`                    | `Box<dyn LspBackend>`, `with_backend`, `select_backend` |
| `rust/src/lsp/mod.rs`                       | Modul-Exporte                                           |
| `rust/src/tools/ctx_refactor.rs`            | neue Actions + **§4.5-Pfad-Fix**                        |
| `rust/src/tools/registered/ctx_refactor.rs` | Schema-Erweiterung                                      |

---

## 5. Plugin-Seite (Kotlin) — Komponentenschnitt

**Befund (korrigiert 2026-06-06):** `packages/jetbrains-lean-ctx` ist **nicht leer**
(die frühere Annahme „Gerüst leer" war falsch). Es existiert bereits ein
**Companion-Plugin** (`com.leanctx.plugin`, Version `1.0.0`) mit einem **anderen
Concern** — Token-Ersparnis anzeigen, Binary finden: `LeanCtxStartupActivity`
(bereits `ProjectActivity`, Coroutine-Form), Statusbar-Widget
(`LeanCtxStatusBarFactory` + `StatsReader`), `BinaryResolver`, Tools-Menü-Actions
(Setup/Doctor/Gain/Dashboard, `actions/`). Das PSI-HTTP-Backend wird **additiv** in
denselben Plugin-Modul (`com.leanctx.plugin`) integriert — es **koexistiert**, ersetzt
nichts. Build aktuell veraltet (IC 2024.1, Kotlin 1.9.25, IntelliJ-Platform-Gradle
2.14.0, jvmTarget 17) → wird in Phase 2 mit auf IC 2026.1 / Kotlin 2.3.20 modernisiert
(§15.7). plugin.xml deklariert `LeanCtxStartupActivity` via `postStartupActivity`-Tag
(mappt auf `ProjectActivity`) + `statusBarWidgetFactory`. **Konsequenz:** Startup ist
schon modern (keine Code-Modernisierung nötig); Phase 2 erweitert
`LeanCtxStartupActivity.execute` um den Server-Boot und legt neue Sub-Packages
`server/`, `dto/` an.

### 5.1 Packages (`com.leanctx.plugin`)

- `LeanCtxStartupActivity` → bootet HTTP-Server **pro `Project`**.
- `server/`: `BackendHttpServer` (Lifecycle, Bind 127.0.0.1), `RequestRouter`
  (Dispatch + Token-Check), `PortFileWriter` (atomar temp+rename), `JsonCodec` (gson).
- `endpoint/`: je v1-Op ein Handler — `FindReferences/Definition/Implementations/
  Declaration`, `GetSymbolsOverview`, `TypeHierarchy`, `Format`, `RunInspections`,
  `Health`.
- `psi/`: `PsiLocator` (Pfad→VirtualFile→PsiFile, line/col↔offset via `Document`),
  `ReferenceFinder`, `ImplementationFinder`, `SymbolStructureReader`,
  `TypeHierarchyReader`.
- `dto/`: Position, TextRange, SymbolDTO, TypeHierarchyNodeDTO, …

### 5.2 PSI-APIs (konkret)

- Pfad→PSI: `LocalFileSystem.findFileByPath` → `PsiManager.findFile`; line/col→offset
  via `PsiDocumentManager.getDocument` + `document.getLineStartOffset`.
- References: `ReferencesSearch.search(element, scope)`.
- Definition/Declaration: `reference.resolve()` / `TargetElementUtil` /
  `getNavigationElement()`.
- Implementations: `DefinitionsScopedSearch` / `OverridingMethodsSearch` /
  `ClassInheritorsSearch`.
- Overview: `PsiStructureViewFactory` / `StructureViewModel` (sprachneutral).
- **type_hierarchy:** `TypeHierarchyProvider` (EP `com.intellij.typeHierarchyProvider`,
  `Subtypes/SupertypesHierarchyTreeStructure`); JVM-Fallback `ClassInheritorsSearch` /
  `psiClass.getSupers()`.

### 5.3 Threading

- Alle PSI-Reads in `ReadAction.compute {}` bzw. `ReadAction.nonBlocking{}.
  executeSynchronously()`. HTTP-Handler laufen off-EDT (HttpServer-Pool) → **nie**
  EDT blockieren. Index-Schutz: `DumbService.runReadActionInSmartMode`, sonst
  `error: INDEXING` mit Retry-Hinweis. v1 read-only → keine WriteAction.

### 5.4 HTTP-Stack

- **`com.sun.net.httpserver.HttpServer` (JDK-eingebaut)**, wie Serena. Begründung:
  null Extra-Dependency, kein ClassLoader-Konflikt mit IDE-Runtime (Ktor/Netty-Drift),
  read-only-JSON braucht keinen async Stack. gson `compileOnly` (IDE bündelt gson).

### 5.5 Port/Token-Datei

- Pfad: `<data_dir>/jetbrains-<projecthash>.port`, Permissions `0600`, atomar.
- **`<data_dir>` = `lean_ctx_data_dir()`-Parität (NICHT hardcoded `~/.lean-ctx`).**
  Rust und Kotlin MÜSSEN dieselbe Auflösungspriorität nutzen (`core/data_dir.rs`):
    1. `LEAN_CTX_DATA_DIR` (env-Override),
    2. `~/.lean-ctx` **nur wenn Daten vorhanden** (Marker `stats.json`/`config.toml`/`sessions`),
    3. `$XDG_CONFIG_HOME/lean-ctx` (default `~/.config/lean-ctx`).
  **Token bleibt inline** in der `.port`-Datei (Entscheidung 2026-06-06: 1 atomarer
  Write, 1 Cleanup, keine Zwei-Datei-Staleness; Phase 1 `PortFile.token` liest bereits
  inline). **⚠ Phase-1-Begleit-Fix (Pflicht):** `port_discovery.rs::port_file_path`
  (aktuell hardcoded `dirs::home_dir().join(".lean-ctx")`, `rust/src/lsp/port_discovery.rs:41`)
  auf `core::data_dir::lean_ctx_data_dir()` umstellen + Test — sonst divergieren Rust-
  und Kotlin-Pfad bei XDG-/Override-Setups.
- **`projecthash` = `sha256(canonical(projectRoot))[..16]`** — Rust und Kotlin müssen
  identisch canonicalisieren (Symlink/Trailing-Slash-Falle). Rust: `std::fs::canonicalize`
  (Fallback = roher Pfad). Kotlin: `Path.toRealPath()` (Fallback = roher Pfad), SHA-256
  der UTF-8-Bytes, erste 8 Bytes → 16 lowercase-hex. Naht-Test: gleicher Input → gleicher
  16-hex-Output auf beiden Seiten.
- Inhalt (JSON-Keys **snake_case** = Rust `PortFile`-serde, `port_discovery.rs`):
  `{port, token (32-byte hex via SecureRandom), pid, project_root, ide_version, started_at}`.
  ⚠ NICHT camelCase — Rust liest `project_root`/`ide_version` per serde; `started_at` ist
  Zusatzfeld (von Rust ignoriert).
- Token als Header `X-LeanCtx-Token`; ohne/falsch → 401. Bei `projectClosing`/`dispose` löschen.

---

## 6. Wire-Protokoll (DTO)

- **Pfade relativ zu `project_root`** (Rust joint zurück; PathJail validiert absolut
  vorher).
- **Positionen 0-basiert (Zeile + Spalte)** auf der Wire = LSP-Konvention → Backing-B-
  Adapter symmetrisch zu Backing A (rust-analyzer `lsp_types::Position` ist 0-basiert;
  `ctx_refactor` rechnet heute genau **einmal** um, L24 rein / `+1` raus). Tool-Eingabe
  nach außen bleibt 1-basierte Zeile (unverändert). PSI ist offset-basiert → Kotlin
  rechnet via `Document`.
- Endpoints: `POST /references|/definition|/implementations|/declaration`
  (`{path, line, character}` → `{locations:[{path, range:{start,end}}]}`),
  `/symbolsOverview`, `/typeHierarchy` (`{…, direction}`), `/format` (`{edits}` —
  v1 liefern, nicht anwenden), `/inspections` (`{diagnostics}`), `GET /health`.
- **Fehler:** `{error:{code, message}}` mit `code ∈ {UNSUPPORTED_LANGUAGE, INDEXING,
  FILE_NOT_FOUND, POSITION_OUT_OF_RANGE, NO_SYMBOL_AT_POSITION, UNAUTHORIZED,
  INTERNAL}`. HTTP 200 für fachliche Negativfälle (einheitliches JSON-Parsen),
  401 nur Token, 500 nur echte Exceptions. Rust mappt `code` → `ERROR: …`-String.

---

## 7. Serena- & JetBrains-MCP-Neueinordnung

- **Tool-Schnittstelle stabil:** `ctx_refactor` (refs/def/impl + neue Actions
  type_hierarchy/overview/format/inspections) und `ctx_symbol` (find/overview) bleiben
  unverändert. Mit Backing B aktiv kommen `type_hierarchy` + IDE-Genauigkeit
  **transparent** dazu — gleiche Aufrufe, anderes Backend. Backing A bleibt der
  CI-/Headless-Fallback.
- **Serena- UND JetBrains-MCP-Ablösung nach v1:** Read/Navigation/Format/Inspections
  vollständig durch Backing B (oder A) abgelöst → das **offizielle JetBrains-MCP** wird
  für Code-Intelligence entbehrlich (seine `references`/`implementations`/`type_hierarchy`
  fehlen ohnehin, siehe §2.1). Nach v2 (symbolische Edits rename-apply/move/safe-delete/
  inline + `insert_*`/`replace_symbol_body`) ist auch **Serena** als Edit-Engine
  entbehrlich → lean-ctx wird die **alleinige** Code-Intelligence-Schnittstelle,
  serena- und fremd-MCP-frei.
  (Out of scope bleiben die DB-/Run-/SQL-/Terminal-Tools des JetBrains-MCP.)
- **Abgrenzung textuelle vs. symbolische Edits:** Textuelle Edits laufen unverändert
  über `ctx_edit` (search-and-replace, read-only-Kern bleibt). Symbolische PSI-Edits
  (rename-apply/move/safe-delete/inline/insert) sind eine **andere Klasse** und kommen
  als eigener v2-Edit-Spec (§9 v2-Ausblick) — nicht über `ctx_edit`.

---

## 8. Risiken

1. **IDE-Lifecycle:** nicht offen/stale → `select_backend` **immer** mit Fallback A.
   Erreichbarkeit = Port-Datei + `pid`-Live + `/health`-Ping (nicht nur Datei-Existenz).
2. **Stale Port-Datei:** IDE-Crash → toter Port. Mehrstufig: pid-Check + Ping-Timeout →
   stale → Fallback A. Atomar schreiben, bei `projectClosing` löschen.
3. **Mehrere Projekte/IDEs:** Port pro Project + eigene Hash-Datei → kollisionsfrei.
   Zwei IDEs, gleiches Projekt → last-writer-wins + `/health` entscheidet (v1-akzeptabel).
4. **0-vs-1-Basierung:** wahrscheinlichstes Bug-Cluster → dedizierte Tests an beiden
   Nähten (Rust↔Wire, Wire↔PSI).
5. **type_hierarchy Sprach-Lücken:** nur Sprachen mit `typeHierarchyProvider`-EP
   (Java/Kotlin sicher), sonst `UNSUPPORTED_LANGUAGE`. Backing A kann es nie.
6. **§4.5-Sicherheitsnaht:** selbstgebauter `abs_path` muss durch `jail_path` (Phase 0).
7. **Versions-Drift IC 2024.1:** Sprach-EPs defensiv über nullable `LanguageExtension`-
   Lookup, nicht harte Imports.
8. **Build/CI:** Plugin braucht JDK 17 + Gradle (separater CI-Job); Rust-CI läuft
   weiter ohne Plugin (Backing A deckt LSP-Pfade headless).
9. **gson-Konflikt:** IDE-gebündelte gson → `compileOnly`/shaden, sonst Laufzeitfehler.

---

## 9. Implementierungs-Phasen (Grobschnitt für writing-plans)

> **Serena-Referenzklassen pro Phase (dekompiliert 2026-06-06 aus
> `tmp/serena-jetbrains-plugin/lib/serena-jetbrains-plugin-2023.2.16.jar`,
> Paket `de.oraios.serena.*`).** Architektur-/Namens-Referenz, **keine** Code-Quelle.

- **Phase 0 — Trait-Extraktion (Rust, refactor-only):** `LspBackend` +
  `impl für LspClient`; Router auf `Box<dyn LspBackend>`; **§4.5-Pfad-Fix**.
  *Gate:* bestehende ctx_refactor-Tests grün, Verhalten identisch, clippy sauber.
  *(Serena-Ref: keine — reine Rust-Seite.)*
- **Phase 1 — Port-Discovery + HTTP-Backend-Skeleton (Rust):** `port_discovery.rs`,
  `jetbrains_backend.rs` (refs/def/impl via ureq), `select_backend` mit Fallback.
  *Gate:* gegen Mock-Server parsebar; ohne Port-Datei deterministischer Fallback A.
  *(Serena-Ref: Wire-Gegenstück zu `service/request/*Request` + `service/dto/*` —
  Rust spiegelt deren JSON-Shape.)*
- **Phase 2 — Plugin-Kern (Kotlin):** `BackendHttpServer` + `PortFileWriter` +
  `LeanCtxPaths` (data-dir/hash-Parität) + `HealthHandler` + `RequestRouter`
  (Token-Check); `LeanCtxStartupActivity.execute` erweitert → bootet/stoppt pro Project.
  Token inline in `.port`, `<data_dir>` = `lean_ctx_data_dir()`-Parität (§5.5).
  *Gate:* IDE auf → Port-Datei unter korrektem Data-Dir mit Token & `0600`;
  `/health` mit Token = 200, ohne = 401; `projectClosing`/`dispose` löscht; Kotlin-Unit
  für Resolver-Priorität + `projectHash`==Rust; Phase-1-Begleit-Fix (`port_file_path`
  → `lean_ctx_data_dir()`) grün via `cargo nextest`. Verifikation: manuelles `runIde`-Gate.
  *(Serena-Ref: `SerenaBackendService` [HttpServer-Lifecycle/`startService`/`dispose`/gson],
  `PluginStartupActivity` [`ProjectActivity.execute`], `PostRequestHandler`
  [`handleExchange`-Dispatch-Basis], `HttpExchangeUtils`
  [`readRequestBody`/`sendResponse`/`sendErrorResponse`], `service/HttpStatus`. **Abweichung:**
  Serena hat **kein** Token (nur localhost) und nutzt **Range-Scan** `findFreePort` ab
  `START_PORT` — wir: Token-Header + ephemerer OS-Port `:0` + Port-Datei-Discovery.)*
- **Phase 3 — Nav-Endpoints PSI + E2E:** Find*-Handler + `psi/` unter ReadAction.
  *Gate:* references/definition stimmen mit IDE-„Find Usages"; 0/1-Naht getestet.
  *(Serena-Ref: `endpoint/FindReferencesHandler`, `FindDeclarationHandler`,
  `FindImplementationsHandler`, `FindSymbolHandler`; `symbol/SymbolFinder`, `Symbol`,
  `util/ProjectContext` [`getAbsolutePath`/ReadAction-Smart-Mode],
  `service/dto/SymbolDTO`/`TextRangeDTO`/`PositionDTO`.)*
- **Phase 4 — type_hierarchy + symbols_overview (B-only):** neue Actions + Handler +
  Degradierung. *Gate:* korrekte Super/Subtypes (Java/Kotlin); rust-only → sauberer
  ERROR; unsupported → `UNSUPPORTED_LANGUAGE`.
  *(Serena-Ref: `endpoint/TypeHierarchyHandler` + `GetSubtypesHandler`/`GetSupertypesHandler`,
  `symbol/TypeHierarchy`(`.Node`)/`SubtypeHierarchy`/`SupertypeHierarchy`,
  `exception/TypeHierarchyNotSupportedException`; `endpoint/GetSymbolsOverviewHandler`,
  `symbol/FileStructure`; DTOs `TypeHierarchyNodeDTO`/`TypeHierarchyResponse`/
  `GetSymbolsOverviewResponse`/`FileStructureDTO`.)*
- **Phase 5 — format + inspections + Härtung:** read-only Handler; stale/PID/401/
  atomare Writes; Plugin-CI-Job. *Gate:* strukturierte Ergebnisse; stale → Fallback
  ohne Hänger; Plugin-CI grün.
  *(Serena-Ref: `endpoint/FormatCodeHandler`/`FormatSymbolHandler`;
  `endpoint/InspectionRunner` + `RunInspectionsOnFileHandler`/`ListInspectionsHandler`;
  DTOs `InspectionInfoDTO`/`InspectionProblemDTO`/`InspectionsResponse`. **Nicht
  übernommen:** Serenas Edit-/Debug-Pfade `RenameSymbolHandler`/`MoveHandler`/
  `SafeDeleteHandler`/`InlineSymbolHandler`/`ApplyQuickFixHandler` + `debugging/*` →
  v2-Edit-Spec bzw. out of scope.)*
- **v2-Ausblick (eigener Spec, nicht hier):** Die **symbolischen Edit-Ops** —
  Serena-Äquivalente `replace_symbol_body`, `insert_before_symbol`,
  `insert_after_symbol`, `jet_brains_rename` (apply), `jet_brains_move`,
  `jet_brains_safe_delete`, `jet_brains_inline_symbol` — werden **NICHT** in diesem
  v1-Spec behandelt, sondern in einem **separaten v2-Edit-Spec**. Begründung: sie
  brauchen ein fundamental anderes Modell (`WriteCommandAction` auf EDT,
  Transaktionalität, Undo, Konflikt-Handling, Cache-Kohärenz mit dem Session-Cache
  von `ctx_edit`) als die read-only-v1-Ops. Sie kommen additiv als Default-`Err`-Trait-Methoden
  hinzu (kein Breaking Change an v1). Erst dieser v2-Spec macht Serena auch als
  Edit-Engine entbehrlich.

---

## 10. Verifikation (End-to-End)

- **Rust-Einheit:** `cargo nextest run` (lsp/router, backend-Selektion, port_discovery,
  ctx_refactor-Actions inkl. Degradierung). Niemals `cargo test`.
- **Naht-Tests:** 0/1-Basierung an Rust↔Wire und Wire↔PSI explizit.
- **Plugin:** Gradle-Build + `runIde`; manuell IDE öffnen, Port-Datei prüfen,
  `/health` + `ctx_refactor action=references` E2E gegen Java/Kotlin-Testprojekt,
  Abgleich mit IDE-„Find Usages".
- **Fallback:** ohne laufende IDE → `ctx_refactor` nutzt Backing A unverändert
  (Regressionsschutz).
- **Security:** Pfad außerhalb project_root → `jail_path`-Fehler **vor** HTTP-Call
  (verifiziert, dass Plugin nie einen Jail-fremden Pfad sieht).

---

## 11. Referenz-Artefakte

- Serena-Plugin (Architektur-Referenz, **nicht** Code-Quelle): `tmp/serena-jetbrains-plugin/lib/`,
  extrahiert `tmp/serena_extract/META-INF/plugin.xml`.
- Plugin-Gerüst: `packages/jetbrains-lean-ctx/`.
- Rust-Backend-Pfad: `rust/src/lsp/{router,client,config}.rs`,
  `rust/src/tools/ctx_refactor.rs`, `rust/src/core/pathjail.rs`,
  `rust/src/server/tool_trait.rs`.

---

## 12. Branch- & Release-Strategie (von `feat-lmd-v1`, lmd-frei)

**Anforderung:** Ein eigener Branch `feat-jetbrains-plugin`, der **von `feat-lmd-v1`
abzweigt** und damit **alle** dort gemachten rust/src-Änderungen erbt — **außer** dem
lmd-Modul, das entfernt wird. **Kein** worktree (Projekt-Rule „No worktrees"),
**kein** `main`/`origin`-Umweg (der frühere Plan, von `origin/main` neu aufzusetzen,
war falsch — die Arbeit liegt auf `feat-lmd-v1`).

### 12.1 Ausgangslage (verifiziert 2026-06-05)

- **Basis = `feat-lmd-v1`** (Version `3.7.4-lmd`, Stand 2026-06-06 — der **Changelog-3.7.4-
  Stand inkl. #141** (`tool_def`-Registry) **und #145** (`resolve_tool_path`) ist damit
  **bereits in der Branch-Basis**; ein 3.7.4-Rebase entfällt). Sollte `main` bis zum finalen
  Squash-Merge-PR (§12.4) noch weiter vorausgelaufen sein, vor dem PR darauf rebasen; für die
  Branch-Neuanlage selbst bleibt `feat-lmd-v1` die korrekte Basis.
  Durch das Abzweigen sind **alle**
  rust/src-Änderungen (z. B. `rust/src/graph::get_forward_deps`) **automatisch** auf dem
  Branch — kein „Übertragen" nötig.
- **Deps bereits vorhanden:** `ureq = "3.3.0"` (`Cargo.toml:140`) + `sha2 = "0.10"`
  (`Cargo.toml:159`) → keine `Cargo.toml`-Dependency-Änderung in Phase 1. (ureq **3.x**;
  kein `json`-Feature — JSON via `serde_json` + `ureq`-Body-API, Muster `cloud_client.rs`.)
- **Stale-Branch:** Der alte lokale `feat-jetbrains-plugin` saß auf veraltetem `main`
  (3.6.11, 231 Commits zurück) und wird **gelöscht** und neu von `feat-lmd-v1` angelegt.

### 12.2 Branch-Neuanlage + lmd-Entfernung (erster Commit)

- **`feat-jetbrains-plugin` neu von `feat-lmd-v1` (HEAD)** anlegen (alten stale Branch
  vorher löschen): `git branch -D feat-jetbrains-plugin` → `git switch -c feat-jetbrains-plugin`.
- **lmd als erster Commit entfernen** (vollständiger, verifizierter Footprint — lmd ist
  sauber isoliert, einzige externe Referenz = `lib.rs:36`):
    1. `rust/src/lmd/` — gesamtes Modul löschen.
    2. `rust/src/lib.rs:36` → `pub mod lmd;` entfernen (**das** ist die „mod"-Anpassung —
       es ist `lib.rs`, **kein** `rust/src/mod.rs`).
    3. `rust/tests/lmd_phase1_gate.rs` + `rust/tests/lmd_rushdown_spike.rs` löschen
       (beide `use lean_ctx::lmd::…` → würden den Build sonst brechen).
    4. *(optional, kosmetisch)* `rust/Cargo.toml:3` Version `"3.7.4-lmd"` → `"3.7.4"`.
- **Bleibt drin (verifiziert):** `ctx_compile` (`registry.rs:175`) hat **keine**
  lmd-Abhängigkeit — die frühere Behauptung „lmd-Render-Tool" war falsch; bleibt
  registriert. Alle übrigen rust/src-Änderungen bleiben.
- *Gate:* nach der Entfernung `cargo build` + `cargo nextest run` grün (kein dangling
  `lmd`-Verweis), clippy sauber.
- Dieses Spec + der Phase-0/1-Plan liegen bereits auf `feat-lmd-v1` und wandern damit
  automatisch mit (keine separate Datei-Übernahme nötig).

### 12.3 Implementierung — ein Commit pro Phase

- Phasen 0–5 (§9) werden **frisch** auf `feat-jetbrains-plugin` umgesetzt: **je ein
  Commit pro Phase**, jeweils erst nach erfülltem Phasen-Gate (§9/§10). Saubere,
  reviewbare Feature-Historie; **kein** Squash während der Entwicklung.
- Direkt auf dem Branch — **kein worktree**.

### 12.4 Release/Merge

- Finaler Merge nach `main` via **Squash-Merge-PR**: Das Squashing passiert **am
  Schluss beim Merge**, nicht währenddessen. So bleibt die Phasen-Historie bis zum PR
  erhalten, `main` erhält genau **einen** sauberen Feature-Commit.

## 13. Follow-up — Vollständige Serena-/JetBrains-MCP-Tool-Abdeckung (Backlog)

**Ziel:** Damit lean-ctx die *alleinige* Code-Intelligence-Schnittstelle wird (§1, §7),
müssen die **restlichen relevanten** Serena- und JetBrains-MCP-Tools als `ctx_*`-Ops
(Backing B, Fallback A) nachgezogen werden. Diese Liste ist das Backlog dazu — sie speist
den separaten **v2-Edit-Spec** (§9 v2-Ausblick) und etwaige Folge-Ops. Nicht-Code-
Intelligence (DB/Run/SQL/Terminal des JetBrains-MCP) bleibt **out of scope** (§1).

### 13.1 Navigation / Symbol-Lesen (read-only)

| Fremd-Tool                                   | lean-ctx-Ziel                                       | Status               |
|----------------------------------------------|-----------------------------------------------------|----------------------|
| `serena.jet_brains_find_symbol`              | `ctx_symbol action=find`                            | v1 (vorhanden)       |
| `serena.jet_brains_find_declaration`         | `ctx_refactor action=declaration`                   | v1                   |
| `serena.jet_brains_find_implementations`     | `ctx_refactor action=implementations`               | v1                   |
| `serena.jet_brains_find_referencing_symbols` | `ctx_refactor action=references`                    | v1                   |
| `serena.jet_brains_get_symbols_overview`     | `ctx_refactor action=overview`                      | v1                   |
| `serena.jet_brains_type_hierarchy`           | `ctx_refactor action=type_hierarchy`                | v1                   |
| `jetbrains.search_symbol`                    | `ctx_symbol action=find`                            | v1 (abgedeckt)       |
| `jetbrains.get_symbol_info`                  | `ctx_refactor action=definition/declaration`        | v1 (teilw.)          |
| `jetbrains.generate_psi_tree`                | NEU `ctx_refactor action=psi_tree` (PSI-Dump/Debug) | Follow-up (optional) |

### 13.2 Symbolische Edits (write — Kern des v2-Edit-Specs)

| Fremd-Tool                                                  | lean-ctx-Ziel                                   | Status |
|-------------------------------------------------------------|-------------------------------------------------|--------|
| `serena.replace_symbol_body`                                | NEU `ctx_refactor action=replace_symbol_body`   | v2     |
| `serena.insert_before_symbol`                               | NEU `ctx_refactor action=insert_before_symbol`  | v2     |
| `serena.insert_after_symbol`                                | NEU `ctx_refactor action=insert_after_symbol`   | v2     |
| `serena.jet_brains_rename` / `jetbrains.rename_refactoring` | NEU `ctx_refactor action=rename_apply`          | v2     |
| `serena.jet_brains_move`                                    | NEU `ctx_refactor action=move`                  | v2     |
| `serena.jet_brains_safe_delete`                             | NEU `ctx_refactor action=safe_delete`           | v2     |
| `serena.jet_brains_inline_symbol`                           | NEU `ctx_refactor action=inline`                | v2     |
| `serena.replace_content`                                    | bereits `ctx_edit` (textuell, kein Symbol-Edit) | gelöst |

Diese Ops brauchen ein anderes Modell als read-only-v1 — `WriteCommandAction` auf EDT,
Transaktionalität/Undo, Konflikt-Handling, Cache-Kohärenz mit `ctx_edit` — Details im
**v2-Edit-Spec** (§9). Sie kommen additiv als Default-`Err`-Trait-Methoden (kein Breaking
Change an v1).

### 13.3 Format / Inspektionen / Diagnostik

| Fremd-Tool                                               | lean-ctx-Ziel                       | Status       |
|----------------------------------------------------------|-------------------------------------|--------------|
| `jetbrains.reformat_file`                                | `ctx_refactor action=format`        | v1           |
| `jetbrains.get_file_problems` / `run_inspection_kts`     | `ctx_refactor action=inspections`   | v1           |
| `serena.jet_brains_run_inspections` / `list_inspections` | `ctx_refactor action=inspections`   | v1           |
| `serena.jet_brains_debug`                                | (kein Code-Intelligence-Äquivalent) | out of scope |

### 13.4 Akzeptanzkriterium

Sobald 12.1–12.3 (außer „out of scope") als `ctx_*`-Ops vorliegen und gegen ein
Java/Kotlin-Testprojekt verifiziert sind (Abgleich mit IDE-Verhalten), können
**Serena-MCP** und das **offizielle JetBrains-MCP** für Code-Intelligence aus der
Agent-Konfiguration entfernt werden — lean-ctx ist dann die alleinige Schnittstelle (§7).

## 14. Implementierungs-Status & Befunde — Phase 0 + 1 (2026-06-06)

Phase 0 + 1 wurden via `superpowers:subagent-driven-development` umgesetzt. Branch
`feat-jetbrains-plugin` (von `feat-lmd-v1`, §12). Drei Commits, Commit-Disziplin §12.3
(genau ein Commit pro Phase) eingehalten:

| Commit      | Inhalt                                                                          |
|-------------|---------------------------------------------------------------------------------|
| `6ed981da`  | lmd-Modul entfernt (lmd-freie Basis, §12.2)                                      |
| `211e594f`  | **Phase 0**: `LspBackend`-Trait (§4.1) + `impl` für `LspClient` (§4.2) + Router auf `Box<dyn LspBackend>` (§4.3) + §4.5-PathJail-Härtung in `ctx_refactor` |
| `3bdb5a23`  | **Phase 1**: `port_discovery.rs` (§5.5) + `jetbrains_backend.rs`-Skeleton (§6, refs/def/impl via `ureq` 3.3, ohne `json`-Feature) + B-first `select_backend` (§4.3) |

**Gate (§9):** Build ok, `cargo fmt --check` clean, 5 neue Tests grün
(`inner_handle_uses_provided_abs_path_not_raw_args`, `project_hash_is_stable_and_16_hex`,
`port_file_absent_for_unlikely_root`, `references_parses_wire_locations`,
`no_port_file_means_no_backing_b`), clippy 0 Errors + 0 Warnungen in allen neuen Dateien.
Finaler Opus-Gesamt-Review (Spec + Quality): **READY TO MERGE**, 0 Critical/Important.

### 14.1 Offene Follow-ups (nicht-blockierend, spätere Phasen)

1. **`project_root`-Kanonisierung im HTTP-Backend (§5.5-Trap).** In
   `jetbrains_backend.rs` leiten `position_body` (`strip_prefix(project_root)`) und
   `rel_to_uri` (`format!("{root}/{rel}")`) relative bzw. absolute Pfade ab, **ohne**
   `project_root` zu kanonisieren. Bei symlinktem Root oder Trailing-Slash schlägt
   `strip_prefix` fehl → `.unwrap_or(abs)` schickt den **absoluten** Pfad als vermeintlich
   relativen an die IDE; `rel_to_uri` kann Double-Slash erzeugen. **Fix:** `project_root`
   einmalig in `JetBrainsHttpBackend::new` (oder in `select_backend`) kanonisieren +
   Trailing-`/` trimmen, sodass Rust- und Kotlin-Seite **byte-identisch** kanonisieren
   (deckt sich mit der §5.5-Forderung). Ziel: **Phase 2/3** (beim Plugin-Bau gegen
   `project_hash` verifizieren). Kein Live-IDE in Phase 1 → derzeit latent.

2. **Stale-Cache-Invalidierung in `select_backend` (§4.3).** `with_backend` cached den
   gewählten Backend in `BACKENDS` **ohne** Stale-Re-Prüfung. Schließt die IDE nach dem
   Cachen eines Backing-B-Eintrags, laufen Folge-Aufrufe gegen den toten HTTP-Endpoint
   bis zum Prozess-Neustart. §4.3 fordert „Stale-Erkennung invalidiert den Cache-Eintrag" —
   in Phase 1 (Skeleton) **noch nicht implementiert**. Ziel: **Phase 5** (PID-/Health-
   basiertes Cache-Invalidieren).

### 14.2 Prozess-Hinweis (für Phase-3+-Ausführung)

Während der Phase-0-Ausführung wurden die formalen **Spec-/Quality-Subagenten-Reviews**
pro Task (Tasks 0.3/0.4) **übersprungen** — nur Controller-Code-Verifikation + grünes
Gate. Der finale Opus-Gesamt-Review hat beide Phasen nachträglich abgedeckt (READY TO
MERGE). Für Folgephasen: die Zwei-Stufen-Review (Spec-Compliance **vor** Code-Quality)
pro Task nicht überspringen — sie ist Teil des `subagent-driven-development`-Vertrags.

---

## 15. Phase-2-Detaildesign — Plugin-Kern (HTTP-Lifecycle) — genehmigt 2026-06-06

**Ziel:** Die Kotlin-Seite startet beim Projektöffnen einen localhost-HTTP-Server **pro
`Project`**, meldet ihn via Port-/Token-Datei an die (Phase-1-)Rust-Seite, beantwortet
token-geschütztes `/health`, und räumt beim Schließen sauber ab. **Noch keine PSI-Logik**
— nur die erreichbare, authentifizierte Hülle, gegen die Phase 1 bereits spricht.

### 15.1 Fixierte Entscheidungen (User, 2026-06-06)

| # | Entscheidung | Begründung |
| 1 | **Scope minimal** — 5 Bausteine + `/health`, **keine** Settings-UI/Configurable | schlankster reviewbarer Phase-Commit; Settings später additiv |
| 2 | **Token inline** in `.port` (kein separater `http-tokens`-Store) | 1 atomarer Write/Cleanup, keine Zwei-Datei-Staleness; Phase 1 liest `token` bereits inline |
| 3 | **`<data_dir>` = `lean_ctx_data_dir()`-Parität** (nicht hardcoded `~/.lean-ctx`) | korrekt unter `LEAN_CTX_DATA_DIR`/XDG; Rust+Kotlin müssen identisch auflösen |
| 4 | **Tests: manuelles `runIde`-Gate** + reine Kotlin-Unit (Resolver/Hash, ohne IDE) | IntelliJ-Plugin-Testframework erst ab Phase 3 (PSI-E2E) |
| 5 | **Additive Koexistenz** im bestehenden `com.leanctx.plugin` Companion-Plugin | PSI-Backend ist anderer Concern als Statusbar/Binary — ersetzt nichts |
| 6 | **Build-Modernisierung auf IC 2026.1 / Kotlin 2.3.20** (jvmTarget 21) gleich mitziehen (§15.7) | aktuelle IDE-Baseline; Kotlin an gebündelte Runtime gekoppelt; kein Marketplace → keine Alt-IDE-Kompat nötig |

### 15.2 Neue Komponenten (Sub-Packages in `com.leanctx.plugin`, unter
`packages/jetbrains-lean-ctx`)

| Datei (neu/~erweitert) | Aufgabe | Serena-Ref |
| `server/BackendHttpServer.kt` | `Disposable` Project-Service; bindet `com.sun.net.httpserver.HttpServer` auf `127.0.0.1:0` (ephemerer OS-Port); off-EDT-Pool; `dispose()` → stop + Port-Datei löschen | `SerenaBackendService` |
| `server/RequestRouter.kt` | `HttpHandler`-Dispatch + `X-LeanCtx-Token`-Check → sonst 401; Phase 2 registriert nur `/health` | `PostRequestHandler.handleExchange` + `HttpExchangeUtils` |
| `server/PortFileWriter.kt` | `<data_dir>/jetbrains-<hash>.port` atomar (temp+rename), `0600`, JSON snake_case `{port, token, pid, project_root, ide_version, started_at}` (= Rust `PortFile`-serde) | (Serena: Range-Scan — wir: Datei) |
| `server/LeanCtxPaths.kt` | Data-Dir-Resolver **identisch zu Rust** (`LEAN_CTX_DATA_DIR` → `~/.lean-ctx` mit Daten → `$XDG_CONFIG_HOME/lean-ctx`) + `projectHash = sha256(realpath(root))[..16]` | — (Parität-Naht §5.5) |
| `server/HealthHandler.kt` | `GET /health` → `{status:"ok", ideVersion, project}` | `endpoint/*Handler`-Muster |
| `dto/Health.kt`, `dto/ErrorResponse.kt` | gson-DTOs (gson `compileOnly`) | `service/dto/*`, `ErrorResponse` |
| `LeanCtxStartupActivity.kt` (~erweitern) | nach Binary-Check zusätzlich `BackendHttpServer` für das `Project` booten | `PluginStartupActivity.execute` |

### 15.3 Lebenszyklus

```
Project open → LeanCtxStartupActivity.execute (existiert)
   → BackendHttpServer(project) als Disposable Project-Service
   → bind 127.0.0.1:0 ; token = SecureRandom 32-byte hex
   → PortFileWriter.write(<data_dir>/jetbrains-<hash>.port, 0600, atomar temp+rename)
Rust select_backend → read_port_file → pid_alive + GET /health (X-LeanCtx-Token) → Backing B
Project close (Disposable.dispose / projectClosing) → server.stop(0) + Port-Datei delete
```

### 15.4 plugin.xml-Delta

- Kein neuer `postStartupActivity` nötig — bestehende `LeanCtxStartupActivity` wird
  erweitert. `BackendHttpServer` als **project-level service** (Disposable, an `Project`
  gebunden → automatischer `dispose` bei `projectClosing`). Statusbar/Actions unberührt.

### 15.5 Begleit-Fix Phase 1 (Pflicht-Bestandteil dieses Phasen-Commits)

`rust/src/lsp/port_discovery.rs:41` `port_file_path`: hardcoded
`dirs::home_dir().join(".lean-ctx")` → `core::data_dir::lean_ctx_data_dir()` + Test.
(Sonst Pfad-Divergenz Rust↔Kotlin bei XDG-/Override-Setups.)

### 15.6 Gate (Verifikation)

1. `runIde` → Port-Datei erscheint unter korrektem `<data_dir>` mit Token & `0600`.
2. `curl -H "X-LeanCtx-Token: <tok>" http://127.0.0.1:<port>/health` = **200**;
   ohne/falscher Token = **401**.
3. Projekt schließen → Port-Datei **gelöscht**.
4. Kotlin-Unit: Resolver-Priorität (`LEAN_CTX_DATA_DIR`/XDG/legacy) + `projectHash`
   byte-identisch zu Rust `project_hash` (gleicher Input → gleicher 16-hex-Output).
5. `cargo nextest run` grün inkl. Phase-1-Begleit-Fix (§15.5).
6. Companion-Plugin (Statusbar/Actions) weiterhin funktional (keine Regression).

### 15.7 Build-Modernisierung (Teil des Phase-2-Commits)

**Befund (Web-Recherche 2026-06-06):** Aktueller Plugin-Build ist veraltet **und nutzt
die alte DSL**. Maßgeblich = offizielle JetBrains-Vorlage
[`intellij-platform-plugin-template`](https://github.com/JetBrains/intellij-platform-plugin-template)
(Stand `main`): Kotlin **2.1.20**, IntelliJ-Platform-Gradle **2.16.0**, Ziel
`intellijIdea("2025.2.6.2")` via **neuer Dependency-DSL**, Changelog-Plugin 2.5.0,
`pluginManagement` in `settings.gradle.kts`, Config-/Build-Cache an. **Kopplung:** Ein
JetBrains-Plugin läuft zur Laufzeit gegen die **IDE-gebündelte** Kotlin-Runtime → die
kompilierte Kotlin-Version muss ≤ gebündelt sein. Das Template zielt 2025.2.6.2 →
Kotlin 2.1.20; **wir** zielen (User-Entscheidung) auf **IC 2026.1** (bündelt
**Kotlin 2.3.20**) → Kotlin **2.3.20**, NICHT 2.4.0. **Vorgehen: bestehendes Plugin
retrofitten** (nicht neu scaffolden) — Build-Dateien an die Template-Konventionen
angleichen, Companion-Code behalten.

**Versions-Änderungen:**

| Setting | Alt | Neu |
| Kotlin (`org.jetbrains.kotlin.jvm`) | `1.9.25` | `2.3.20` (= gebündelt IC 2026.1) |
| IntelliJ-Platform-Gradle | `2.14.0` | `2.16.0` |
| Ziel-IDE | `create("IC", "2024.1")` (alte DSL) | `intellijIdea("2026.1.3")` (**neue Dependency-DSL**) |
| `ideaVersion.sinceBuild` | `241` | `261` |
| `ideaVersion.untilBuild` | `261.*` | **entfernen** (offen — bricht nicht bei IDE-Minor-Update; OK für Privat-Plugin ohne Marketplace) |
| JVM-Target | `kotlinOptions.jvmTarget = "17"` | `compilerOptions { jvmTarget = JvmTarget.JVM_21 }` (`kotlinOptions` in Kotlin 2.x deprecated; IC 2026.1 läuft auf JBR 21) |

**Struktur-Angleichung an die Vorlage (Retrofit der Build-Dateien):**

- `settings.gradle.kts`: `pluginManagement { plugins { kotlin.jvm 2.3.20; changelog 2.5.0 } }`
  + `plugins { foojay-resolver-convention 1.0.0; org.jetbrains.intellij.platform.settings 2.16.0 }`
  + `dependencyResolutionManagement { repositories { mavenCentral(); intellijPlatform { defaultRepositories() } } }`.
- `build.gradle.kts`: `plugins {}` ohne Versions-Literale (Versionen aus `pluginManagement`);
  `intellijPlatform { intellijIdea("2026.1.3"); testFramework(TestFrameworkType.Platform) }`.
- `gradle.properties`: `kotlin.stdlib.default.dependency=false` +
  `org.gradle.configuration-cache=true` + `org.gradle.caching=true`.
- Gradle-Wrapper auf aktuelle Version anheben (vom Template übernehmen).

**Nicht in Phase 2 nötig** (erst ab Phase 3, Kotlin-PSI): `<depends>org.jetbrains.kotlin</depends>`
+ K2-/Analysis-API-Deklaration. Phase 2 nutzt nur `com.sun.net.httpserver` + reine JVM —
keine Kotlin-Compiler-/Analysis-APIs. Changelog-/Qodana-/Kover-/Verifier-CI aus dem
Template sind **optional** und gehören frühestens in die Phase-5-Härtung (CI-Job), nicht
in den minimalen Phase-2-Schnitt.

*Gate-Ergänzung:* `./gradlew build` + `./gradlew runIde` grün auf dem neuen Stack
(Config-Cache aktiv); Companion-Plugin lädt weiterhin in IC 2026.1.

**Quellen:** [intellij-platform-plugin-template](https://github.com/JetBrains/intellij-platform-plugin-template) ·
[Kotlin 2.4.0 Released](https://blog.jetbrains.com/kotlin/2026/06/kotlin-2-4-0-released/) ·
[IntelliJ Platform Gradle Plugin Releases](https://github.com/JetBrains/intellij-platform-gradle-plugin/releases) ·
[IntelliJ IDEA 2026.1.3 Is Out](https://blog.jetbrains.com/idea/2026/06/intellij-idea-2026-1-3/) ·
[Configuring Kotlin Support](https://plugins.jetbrains.com/docs/intellij/using-kotlin.html)

---

## 16. Companion-Track (Issue #246) — Abgrenzung

[Issue #246](https://github.com/yvgude/lean-ctx/issues/246) ("Integration: JetBrains
native plugin for lean-ctx") ist der **Ursprung** des Plugin-Vorhabens und beschreibt
einen **UX/Companion-Track**, der vom PSI-Backend-Track **dieses** Specs zu trennen ist.
Beide teilen sich **ein** Plugin-Modul (`com.leanctx.plugin`, `packages/jetbrains-lean-ctx`),
verfolgen aber **verschiedene Zwecke** und **entgegengesetzte Kommunikationsrichtungen**.

| | **Companion/UX-Track (#246)** | **PSI-Backend-Track (dieser Spec)** |
| Zweck | Ersparnis anzeigen, Auto-Setup, read-mode Hints, Settings-UI, One-Click-Toggle | Serena-Ablösung: `ctx_refactor`-Backend B (refs/def/impl, type_hierarchy, …) |
| Plugin-Rolle | **Host/Client** — ruft das lean-ctx-Binary (`BinaryResolver.runCommand`) bzw. liest `stats.json` | **HTTP-Server** — lean-ctx Rust ruft das Plugin auf (`X-LeanCtx-Token`) |
| Bestehender Code | `StatsReader`, `LeanCtxStatusBarFactory`, `BinaryResolver`, `actions/` | NEU ab Phase 2 (`server/`, `dto/`) |
| Status | tracked (#246), teilweise vorhanden | Phase 0+1 fertig, Phase 2 in Arbeit |

**#246-Companion-Features** (eigener späterer Mini-Spec, **nicht** Teil von Phase 0–5):
Statusbar-„wrapped"-Card via `lean-ctx gain --wrapped`/`--json` (on-demand, nicht im
30s-Timer; `gain --json` als stabiler Parsing-Contract statt Text-Scraping; reiche Card
via `--svg`/`--html` in JCEF-Panel), per-File read-mode Hints, Settings-UI für
`config.toml`, One-Click-Enable/Disable pro Projekt, Auto-Setup.

**Überholt aus #246** (Architektur seither weiterentwickelt): Der #246-Design-Kommentar
empfiehlt ein **separates Repo** `lean-ctx-jetbrains` + **MCP über stdio** (Plugin spawnt
lean-ctx als Child) bzw. Proxy-Routing auf Port 4444. **Dieser Spec überschreibt das:**
**in-repo** `packages/jetbrains-lean-ctx` (der Companion-Code liegt bereits dort) und für
den PSI-Track ist das Plugin ein **HTTP-Server**, den lean-ctx Rust aufruft (umgekehrte
Richtung). Die beiden Richtungen koexistieren konfliktfrei im selben Modul.