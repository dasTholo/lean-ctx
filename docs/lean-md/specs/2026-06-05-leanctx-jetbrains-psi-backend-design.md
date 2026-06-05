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
  `tool_def`-Schema (registered/ctx_refactor.rs L19-24) um Actions + `direction`.
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

**Befund:** Gerüst leer. IC 2024.1, Kotlin 1.9.25, IntelliJ-Platform-Gradle 2.14.0,
`com.leanctx`. plugin.xml deklariert `LeanCtxStartupActivity` (postStartupActivity).

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

- Pfad: `~/.lean-ctx/jetbrains-<projecthash>.port`, Permissions `0600`, atomar.
- **`projecthash` = `sha256(canonical(projectRoot))[..16]`** — Rust und Kotlin müssen
  identisch canonicalisieren (Symlink/Trailing-Slash-Falle).
- Inhalt: `{port, token (32-byte hex), pid, projectRoot, ideVersion, startedAt}`.
- Token als Header `X-LeanCtx-Token`; ohne/falsch → 401. Bei `projectClosing` löschen.

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

- **Phase 0 — Trait-Extraktion (Rust, refactor-only):** `LspBackend` +
  `impl für LspClient`; Router auf `Box<dyn LspBackend>`; **§4.5-Pfad-Fix**.
  *Gate:* bestehende ctx_refactor-Tests grün, Verhalten identisch, clippy sauber.
- **Phase 1 — Port-Discovery + HTTP-Backend-Skeleton (Rust):** `port_discovery.rs`,
  `jetbrains_backend.rs` (refs/def/impl via ureq), `select_backend` mit Fallback.
  *Gate:* gegen Mock-Server parsebar; ohne Port-Datei deterministischer Fallback A.
- **Phase 2 — Plugin-Kern (Kotlin):** `BackendHttpServer` + `PortFileWriter` +
  `Health` + `RequestRouter` (Token); StartupActivity bootet/stoppt pro Project.
  *Gate:* IDE auf → Port-Datei mit Token; `/health`-Ping ok; `projectClosing` löscht.
- **Phase 3 — Nav-Endpoints PSI + E2E:** Find*-Handler + `psi/` unter ReadAction.
  *Gate:* references/definition stimmen mit IDE-„Find Usages"; 0/1-Naht getestet.
- **Phase 4 — type_hierarchy + symbols_overview (B-only):** neue Actions + Handler +
  Degradierung. *Gate:* korrekte Super/Subtypes (Java/Kotlin); rust-only → sauberer
  ERROR; unsupported → `UNSUPPORTED_LANGUAGE`.
- **Phase 5 — format + inspections + Härtung:** read-only Handler; stale/PID/401/
  atomare Writes; Plugin-CI-Job. *Gate:* strukturierte Ergebnisse; stale → Fallback
  ohne Hänger; Plugin-CI grün.
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

## 12. Branch- & Release-Strategie (worktree-frei, sauber von `main`)

**Anforderung:** Ein eigener Branch `feat-jetbrains-plugin`, der **von `main` abgeht**
und **ausschließlich** JetBrains-Plugin-Arbeit trägt — **kein** lmd, **keine**
Baum-Übernahme aus `feat-lmd-v1`, **kein** worktree (Projekt-Rule „No worktrees").

### 12.1 Ausgangslage (verifiziert 2026-06-05)

- **`feat-jetbrains-plugin` existiert bereits** auf sauberer Basis: `main` (v3.6.11)
  + **1 Commit** (lmd-freies Spec + Projekt-Rules). Kein lmd auf dem Branch → **nichts
  zu entfernen**.
- **Kein Plugin-Code auf `feat-lmd-v1` zu übertragen:** `rust/src/lsp` Δ = 1 Zeile,
  `packages/jetbrains-lean-ctx` Δ = 0. Die Phasen 0–5 (§9) sind **alle offen** und
  werden frisch auf diesem Branch implementiert.
- **Die alte „Baum-Übernahme + lmd-Entfernung"-Strategie entfällt ersatzlos.**
  `feat-lmd-v1` ist 393 Commits vor `main` und vermischt weit über lmd hinaus (CI-
  Workflows, `.bak`-Dateien, README, VS-Code-Publish …). Diesen Wust per Baum-Übernahme
  mitzuschleppen, nur um lmd wieder herauszuoperieren, ist unnötig und unsauber.

### 12.2 Spec-Sync (einmalig)

- Dieses Spec (inkl. neuer §12/§13) ist das **einzige** mitwandernde Dokument. Es wird
  als **ein** Update-Commit von `feat-lmd-v1` → `feat-jetbrains-plugin` übernommen
  (Datei-Inhalt, kein Cherry-pick der Historie).
- Allgemeine Projekt-Rules (`CLAUDE.md`, `rust/CLAUDE.md`, lean-ctx-/Serena-Tool-
  Discipline) wandern mit, **lmd-spezifische Referenzen dabei bereinigen** (z. B.
  lmd-Spec-Verweise in `.claude/rules/subagent-multi-agent.md`). lmd-Specs/-Pläne unter
  `docs/lean-md/` bleiben auf `feat-lmd-v1`.

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