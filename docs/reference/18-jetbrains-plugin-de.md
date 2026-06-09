# Journey 18 — JetBrains-Plugin

> Du willst Code-Intelligenz (Navigation, Struktur, Inspektionen, symbolische
> Edits, Refactoring) direkt aus einer laufenden JetBrains-IDE — token-komprimiert
> unter `ctx_refactor`, mit headless-Fallback für CI. Dieses Journey erklärt jede
> Funktion ausführlich: was sie tut, wie der Agent sie aufruft, der rohe
> HTTP-Endpunkt und das Verhalten unter der Haube.

> Sprache: Deutsch. Code, Parameter, Endpunkt-/Tool-Namen und Error-Codes bleiben
> englisch. Knappe Tabellen-Referenz für Agents:
> [appendix-jetbrains-plugin-de.md](appendix-jetbrains-plugin-de.md).

Autoritative Quellen:

- Plugin: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/{server,endpoint,psi,dto}/…`
- Rust-Backend: `rust/src/lsp/{backend,jetbrains_backend,router,edit_apply,port_discovery}.rs`
- MCP-Tool-Schema: `rust/src/tools/registered/ctx_refactor.rs`

---

## 0. Serena als Inspiration

Das lean-ctx JetBrains-Plugin ist konzeptionell von **Serena** (Oraios' IntelliJ-
Platform-MCP-Tool) inspiriert. Serena war das Vorbild, weil es als einziges Werkzeug
den semantischen Kern — `references`, `implementations`, `type_hierarchy` **und**
symbolische Edits — direkt aus der IDE liefert; das offizielle JetBrains-MCP
(`mcp__jetbrains__*`) hat diese Lücke nie geschlossen.

**Eindeutige Abgrenzung:** Das Plugin ist ein **eigenständiger Nachbau auf
Architektur- und Klassennamen-Ebene — keine Ableitung, kein dekompilierter
Serena-Code**. Es steht unter der lean-ctx-Projektlizenz und wird im Repository
(`packages/jetbrains-lean-ctx`) ausgeliefert. Ziel: Serena (und das offizielle
JetBrains-MCP) als Code-Intelligence-Abhängigkeit **entbehrlich** machen, sodass
lean-ctx die alleinige Schnittstelle für Symbole, Navigation und Refactoring wird.

### 0.1 Abgrenzung Serena ↔ lean-ctx-Plugin

| Aspekt         | Serena                     | lean-ctx JetBrains-Plugin                                                |
|----------------|----------------------------|--------------------------------------------------------------------------|
| Hosting        | externe Oraios-Komponente  | im lean-ctx-Repo (`packages/jetbrains-lean-ctx`)                         |
| Schnittstelle  | mehrere separate MCP-Tools | gebündelt unter `ctx_refactor` (Token-Kompression)                       |
| Backend-Modell | nur laufende IDE           | Backing B (IDE) **+** Backing A (rust-analyzer) **+** Headless           |
| Headless / CI  | nein                       | ja — tree-sitter-Fallback für `symbols_overview` + Edits                 |
| Conflict-Guard | keiner                     | BLAKE3 `expected_hash` (Edits) / `plan_hash` (Refactoring), Rust-zentral |
| Sicherheit     | —                          | PathJail (Project-Root-Validierung) + Token-Auth pro Projekt             |
| Lizenz         | proprietär (Oraios)        | lean-ctx-Projektlizenz                                                   |

### 0.2 Mapping: Serena-Konzept → `ctx_refactor`-Action → HTTP-Endpunkt

| Serena-Konzept             | `ctx_refactor`-Action            | HTTP-Endpunkt                                       |
|----------------------------|----------------------------------|-----------------------------------------------------|
| `find_referencing_symbols` | `references`                     | `POST /references`                                  |
| `find_declaration`         | `declaration`                    | `POST /declaration`                                 |
| (goto definition)          | `definition`                     | `POST /definition`                                  |
| `find_implementations`     | `implementations`                | `POST /implementations`                             |
| `get_symbols_overview`     | `symbols_overview`               | `POST /symbols_overview`                            |
| `type_hierarchy`           | `type_hierarchy`                 | `POST /type_hierarchy`                              |
| `run_inspections` / Liste  | `inspections` (`mode=run\|list`) | `POST /inspections`, `POST /list_inspections`       |
| `replace_symbol_body`      | `replace_symbol_body`            | `POST /replaceSymbolBody`                           |
| `insert_before_symbol`     | `insert_before_symbol`           | `POST /insertBeforeSymbol`                          |
| `insert_after_symbol`      | `insert_after_symbol`            | `POST /insertAfterSymbol`                           |
| `rename`                   | `rename`                         | `POST /renamePreview` → `POST /renameApply`         |
| (reformat_file)            | `reformat`                       | `POST /reformat`                                    |
| `move`                     | `move`                           | `POST /movePreview` → `POST /moveApply`             |
| `safe_delete`              | `safe_delete`                    | `POST /safeDeletePreview` → `POST /safeDeleteApply` |
| `inline`                   | `inline`                         | `POST /inlinePreview` → `POST /inlineApply`         |

> `find_symbol` (reine Symbol-Suche) ist nicht Teil von `ctx_refactor`, sondern von
> `ctx_symbol` / `ctx_outline` (lean-ctx-Symbol-Index). Siehe
> [MCP-Tool-Map](appendix-mcp-tools.md).

---

## 1. Architektur (Plugin ↔ Rust ↔ MCP-Tool)

```text
   Agent
     │  ctx_refactor action=… (MCP)
     ▼
  ┌─────────────────────────────────────────────┐
  │ Rust: ctx_refactor  →  select_backend        │
  └─────────────────────────────────────────────┘
        │ IDE erreichbar?        │ nein
        ▼ ja                     ▼
  Backing B                 Headless / Backing A
  JetBrainsHttpBackend      • local_range_write (Edits, atomar)
  HTTP → Plugin             • overview_from_index (tree-sitter)
        │                   • rust-analyzer (Navigation)
        ▼
  ┌─────────────────────────────────────────────┐
  │ JetBrains-IDE-Plugin (Kotlin HTTP-Server)    │
  │ 127.0.0.1 · Token-guarded · PSI/Read-Action  │
  └─────────────────────────────────────────────┘
```

### 1.1 Backing-Wahl & Degradation (`backend.rs`)

`select_backend` (`rust/src/lsp/router.rs`) entscheidet pro Aufruf, welcher Pfad
greift. Das `LspBackend`-Trait staffelt die Methoden:

| Klasse                                      | Methoden                                                                                                                     | Default ohne IDE                           |
|---------------------------------------------|------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------|
| **Mandatory** (beide Backings)              | `open_file`, `references`, `definition`, `implementations`, `rename`                                                         | von Backing A bedient                      |
| **Default-degrading** (Backing B bevorzugt) | `declaration`, `type_hierarchy`, `inspections`, `list_inspections`                                                           | `Err` — „requires the JetBrains backend"   |
| **Headless-Default** (verlustfrei)          | `symbols_overview` (tree-sitter), `replace_symbol_body`, `insert_before_symbol`, `insert_after_symbol` (`local_range_write`) | funktioniert ohne IDE                      |
| **`BACKEND_REQUIRED`**                      | Refactoring-Engine (`rename`, `move`, `safe_delete`, `inline`)                                                               | `Err` — keine headless-Usage-Suche möglich |

### 1.2 Port-Discovery & Staleness

Das Plugin schreibt beim Projekt-Start eine **Port-Datei** (atomar, idempotent) mit
`port`, `token`, `pid`, `projectRoot`, `ideVersion`, `startedAt`
(`BackendHttpServer.kt` → `LeanCtxPaths.portFile(dataDir, projectRoot)`). Beim
`projectClosing` (Disposable) wird sie gelöscht.

Rust prüft die Erreichbarkeit in **drei Stufen** (`rust/src/lsp/port_discovery.rs`):

1. Port-Datei existiert & ist lesbar → `port`/`token`/`pid`,
2. Prozess mit `pid` lebt,
3. `GET /health` antwortet innerhalb des Timeouts.

Nur wenn alle drei bestehen, gilt Backing B als erreichbar; sonst greift Headless
bzw. `BACKEND_REQUIRED`.

---

## 2. Funktionsreferenz

Konventionen für alle Endpunkte:

- HTTP: `POST` auf `127.0.0.1:<port>`, Header `X-LeanCtx-Token: <token>`,
  Body = JSON. `GET /health` ist die einzige Ausnahme (kein Body).
- **Koordinaten:** Auf der `ctx_refactor`-Ebene ist `line` **1-indexed**, `column`
  **0-indexed**. Auf der **Wire-Ebene** (HTTP-DTO) sind `line`/`character` der
  Navigations-/Edit-Endpunkte **0-based** (LSP-Konvention); die `line`-Felder in
  `type_hierarchy`, `symbols_overview` und `inspections`-Antworten sind **1-based**.
- Fachliche Negativfälle kommen als Envelope `{"error":{"code","message"}}` mit
  HTTP 200 (siehe §5).

### 2.1 Navigation (read-only)

**Actions:** `references`, `definition`, `implementations`, `declaration`
**Endpunkte:** `POST /references` · `/definition` · `/implementations` · `/declaration`

**Was es tut:** Findet semantische Fundstellen eines Symbols (Verwendungen,
Deklaration, Implementierungen). `declaration` ist nur über Backing B verfügbar.

**Agenten-Aufruf:**

```text
ctx_refactor action=references path=src/Main.kt line=42 column=8 scope=project
```

**HTTP (curl):**

```bash
curl -s -X POST http://127.0.0.1:$PORT/references \
  -H "X-LeanCtx-Token: $TOKEN" -H "Content-Type: application/json" \
  -d '{"path":"src/Main.kt","line":41,"character":8,"scope":"project"}'
```

**Response (`LocationsResponse`):**

```text
{"locations":[{"path":"src/Main.kt","range":{"start":{"line":41,"character":8},
 "end":{"line":41,"character":14}}}],"truncated":false,"total":1}
```

**Parameter:** `path`, `line`/`character` (0-based, Wire), `scope ∈ {project, all}`
(Default `project`; `all` bezieht Bibliotheken/SDK ein).
**Backing:** Backing B bevorzugt; Backing A (rust-analyzer) als Fallback für
`references`/`definition`/`implementations`. `declaration` ist Backing-B-only.

### 2.2 Struktur

**Actions:** `type_hierarchy`, `symbols_overview`
**Endpunkte:** `POST /type_hierarchy` · `POST /symbols_overview`

**Was es tut:** `type_hierarchy` liefert den Super-/Subtypen-Baum; `symbols_overview`
listet die Top-Level-Symbole einer Datei.

**Agenten-Aufruf:**

```text
ctx_refactor action=type_hierarchy path=src/Main.kt line=10 column=6 direction=subtypes
ctx_refactor action=symbols_overview path=src/Main.kt
```

**HTTP (curl):**

```bash
curl -s -X POST http://127.0.0.1:$PORT/symbols_overview \
  -H "X-LeanCtx-Token: $TOKEN" -d '{"path":"src/Main.kt"}'
```

**Response (`SymbolsOverviewResponse`, `line` 1-based):**

```text
{"symbols":[{"name":"Main","kind":"class","line":3},
            {"name":"run","kind":"method","line":7}],"truncated":false,"total":2}
```

**Parameter:** `type_hierarchy`: `path`, `line`/`character`, `direction ∈
{supertypes, subtypes}` (Default `supertypes`), `scope`. `symbols_overview`: `path`.
**Backing:** `type_hierarchy` ist Backing-B-only. `symbols_overview` hat einen
**verlustfreien headless-Default** über den tree-sitter-Symbol-Index
(`overview_from_index`, dieselbe Quelle wie `ctx_symbol`/`ctx_outline`).

### 2.3 Qualität — Inspektionen

**Action:** `inspections` (`mode=run|list`)
**Endpunkte:** `POST /inspections` · `POST /list_inspections`

**Was es tut:** `mode=run` führt die aktiven Inspektionen auf einer Datei aus und
liefert Diagnosen; `mode=list` listet die im Projektprofil aktivierten Inspektionen.

**Agenten-Aufruf:**

```text
ctx_refactor action=inspections path=src/Main.kt mode=run
ctx_refactor action=inspections path=src/Main.kt mode=list
```

**Response `run` (`InspectionsResponse`, `line` 1-based):**

```text
{"diagnostics":[{"path":"src/Main.kt","line":12,"severity":"WARNING",
 "message":"Unused symbol"}],"truncated":false,"total":1}
```

**Response `list` (`ListInspectionsResponse`):**

```text
{"inspections":[{"id":"UnusedSymbol","name":"Unused declaration",
 "severity":"WARNING"}],"truncated":false,"total":1}
```

**Backing:** Backing-B-only (kein headless-Äquivalent).

### 2.4 Symbol-Body-Edits (write)

**Actions:** `replace_symbol_body`, `insert_before_symbol`, `insert_after_symbol`
**Endpunkte:** `POST /replaceSymbolBody` · `/insertBeforeSymbol` · `/insertAfterSymbol`

**Was es tut:** Ersetzt die vollständige Deklaration eines benannten Symbols bzw.
fügt ein Geschwister-Element davor/danach ein. Das Ziel wird über `name_path`
adressiert (`'Class/method'` qualifiziert oder bare `'name'`), aufgelöst über den
Symbol-Index. Alternativ als Fallback per `path`+`line`(+`end_line`).

**Agenten-Aufruf:**

```text
ctx_refactor action=replace_symbol_body name_path=Main/run \
  new_body="fun run() { println(\"new\") }" expected_hash=<blake3-hex>

ctx_refactor action=insert_after_symbol name_path=Main/run \
  text="fun helper() = 42"
```

**HTTP (curl) — Wire-Body trägt `path`/`range`/`text` (kein Hash, siehe §3.1):**

```bash
curl -s -X POST http://127.0.0.1:$PORT/replaceSymbolBody \
  -H "X-LeanCtx-Token: $TOKEN" -d '{
    "path":"src/Main.kt",
    "range":{"start":{"line":6,"character":0},"end":{"line":8,"character":1}},
    "text":"fun run() { println(\"new\") }"
  }'
```

**Response (`EditResponse`):**

```text
{"applied":true,
 "newRange":{"start":{"line":6,"character":0},"end":{"line":6,"character":28}},
 "editedText":"fun run() { println(\"new\") }"}
```

**Parameter (Action):** `name_path` **oder** `path`+`line`(+`end_line`);
`new_body` (replace) bzw. `text` (insert); optional `expected_hash`.
**Verhalten:** Backing B führt den Edit als `WriteCommandAction` aus (ein
einzelner Undo-Eintrag, Document-Save). Headless schreibt über `local_range_write`
atomar (temp-Datei + `rename`). **Beide Pfade wenden dieselbe tree-sitter-Range an
→ byte-identisches Ergebnis.** Kein automatisches Reformatieren.

---

## 3. Refactoring-Engine

Alle Refactorings (außer `reformat`) laufen über die **gemeinsame Two-Phase-Engine**:
`*Preview` sammelt Verwendungen + Konflikte und bildet den `plan_hash`; `*Apply`
führt die Multi-File-Änderung als **eine** Transaktion (ein Undo-Eintrag) aus.
Da die semantische Usage-Suche den fertigen IDE-Index braucht, gibt es **keinen**
verlustfreien headless-Pfad — ohne laufende IDE kommt `BACKEND_REQUIRED`.

### 3.1 Rename (Two-Phase)

**Action:** `rename` (`new_name`)
**Endpunkte:** `POST /renamePreview` → `POST /renameApply`

**Was es tut:** Benennt ein Symbol projektweit um — Deklaration **und alle Usages**.
Phase 1 (`/renamePreview`) sammelt `usages` und `conflicts` und bildet daraus den
`plan_hash`; Phase 2 (`/renameApply`) führt die Umbenennung als **eine**
Multi-File-Transaktion aus.

**Agenten-Aufruf:**

```text
ctx_refactor action=rename path=src/Main.kt line=7 column=4 new_name=execute
```

**HTTP (curl) — Phase 1:**

```bash
curl -s -X POST http://127.0.0.1:$PORT/renamePreview \
  -H "X-LeanCtx-Token: $TOKEN" -d '{
    "path":"src/Main.kt",
    "range":{"start":{"line":6,"character":4},"end":{"line":6,"character":7}},
    "new_name":"execute","search_comments":false,"search_text_occurrences":false
  }'
# → {"usages":[{"path":"src/Main.kt","range":{…},"context":"run()"}],"conflicts":[]}
```

**HTTP (curl) — Phase 2:**

```bash
curl -s -X POST http://127.0.0.1:$PORT/renameApply \
  -H "X-LeanCtx-Token: $TOKEN" -d '{
    "path":"src/Main.kt","range":{…},"new_name":"execute","force":false
  }'
# → {"applied":true,"changed_paths":["src/Main.kt","src/Caller.kt"]}
```

**Parameter:** `new_name` (Pflicht); optional `search_comments`,
`search_text_occurrences` (Preview); `force` (Apply — überspringt das Konflikt-Gate).
**Verhalten:** `BACKEND_REQUIRED` ohne laufende IDE. Bestehen Konflikte und ist
`force=false`, blockiert das Gate mit `CONFLICT`. Zwischen Preview und Apply schützt
der `plan_hash` (BLAKE3, Rust-zentral) gegen TOCTOU-Drift.

### 3.2 Reformat

**Action:** `reformat`
**Endpunkt:** `POST /reformat`

**Was es tut:** Formatiert eine Datei in-place nach dem aktiven Code-Style-Profil
der IDE (`CodeStyleManager` — Äquivalent zu `mcp__jetbrains__reformat_file`).
Single-Phase (kein Preview): Formatierung ist idempotent und auf eine Datei begrenzt.

**Agenten-Aufruf:**

```text
ctx_refactor action=reformat path=src/Main.kt
```

**HTTP (curl):**

```bash
curl -s -X POST http://127.0.0.1:$PORT/reformat \
  -H "X-LeanCtx-Token: $TOKEN" -d '{"path":"src/Main.kt"}'
# → {"reformatted":true,"path":"src/Main.kt"}
```

**Verhalten:** Backing-B-only (`WriteCommandAction` → `CodeStyleManager.reformat` →
`saveDocument`). Bewusst **entkoppelt** von den Edit-Ops: Symbol-Body-Edits
reformatieren nicht automatisch; `reformat` wird bei Bedarf nachgezogen.

### 3.3 Move

**Action:** `move`
**Endpunkte:** `POST /movePreview` → `POST /moveApply`

**Was es tut:** Verschiebt ein Symbol (Klasse/Datei/Member) in ein anderes
Package/Ziel und passt alle Referenzen + Imports an. Gleiche Two-Phase-Mechanik wie
`rename`: Preview meldet betroffene Dateien + Konflikte (`plan_hash`), Apply führt
die Multi-File-Transaktion aus. `BACKEND_REQUIRED` ohne IDE.

### 3.4 Safe Delete

**Action:** `safe_delete`
**Endpunkte:** `POST /safeDeletePreview` → `POST /safeDeleteApply`

**Was es tut:** Löscht ein Symbol nur, wenn keine blockierenden Verwendungen
bestehen. Preview meldet die gefundenen Usages als Konflikte; Apply löscht (bzw.
blockiert mit `CONFLICT`, sofern nicht `force`). Gleiche Engine wie `rename`.

### 3.5 Inline

**Action:** `inline`
**Endpunkte:** `POST /inlinePreview` → `POST /inlineApply`

**Was es tut:** Ersetzt ein Symbol durch seinen Rumpf an allen Aufrufstellen und
entfernt die Deklaration. Preview meldet die betroffenen Stellen + Konflikte; Apply
führt die Multi-File-Ersetzung aus. Gleiche Engine wie `rename`.

---

## 4. Verhaltensgarantien & Guards

### 4.1 BLAKE3-Conflict-Guard (Rust-zentral)

Der `expected_hash` (Edits) bzw. `plan_hash` (Refactoring) ist ein **BLAKE3-Hex**
(`crate::core::hasher::hash_hex`) und wird **ausschließlich in Rust** geprüft — das
Plugin hasht nicht und kennt das Feld im Wire-Protokoll nicht (`EditRequest` trägt
nur `path`/`range`/`text`).

- **Headless:** `local_range_write` liest die aktuellen Bytes der Range, vergleicht
  gegen `expected_hash` und bricht bei Abweichung mit `CONFLICT: range hash
  mismatch` ab — die Datei bleibt unverändert.
- **IDE (Backing B):** Rust prüft denselben Hash gegen die Disk-Bytes **vor** dem
  HTTP-POST. So ist der Guard auf beiden Pfaden identisch (gleiche Disk-Bytes,
  gleiche BLAKE3-Prüfung).

Das verhindert das blinde Überschreiben extern geänderter Stellen.

### 4.2 Smart-Mode, Sprache, PathJail

- **Smart-Mode:** Befindet sich die IDE im Dumb-Mode (Index wird gebaut), liefern
  PSI-Operationen `INDEXING` statt eines Teilergebnisses (kein automatisches Warten).
  Für die Refactoring-Engine ist das Pflicht: eine unvollständige Usage-Menge wäre
  ein kaputtes Refactoring.
- **Sprache:** Fehlt eine LSP-Konfiguration (Backing A) oder ein PSI-Processor
  (Backing B), kommt `UNSUPPORTED_LANGUAGE` (defensive, nullbare EP-Auflösung).
- **PathJail:** Jede Datei-Operation wird vor Ausführung gegen den `project_root`
  validiert — sowohl die name_path-/Positions-Auflösung als auch jeder vom Plugin
  zurückgegebene `usage`-/`changed_path`.

### 4.3 Idempotenz & Atomarität

| Operation                                    | Transaktion                                    | Idempotent                    |
|----------------------------------------------|------------------------------------------------|-------------------------------|
| Navigation, Struktur, Inspektionen           | Smart-Mode-ReadAction                          | ja (index-stabil)             |
| Symbol-Body-Edits                            | `WriteCommandAction` (IDE) / atomar (headless) | per `expected_hash` geschützt |
| Refactoring (rename/move/safe_delete/inline) | Multi-File-`WriteCommandAction`                | per `plan_hash` geschützt     |
| Reformat                                     | `WriteCommandAction` (single file)             | ja (formatierungs-stabil)     |

Headless-Writes sind atomar (temp-Datei `.<name>.lean-ctx.v2a.tmp.<pid>` + `rename`).

### 4.4 Cache-Kohärenz

Nach jedem Write evictet lean-ctx die Datei aus dem Cache; das nächste `ctx_read`
re-validiert per mtime (~13 Token). Das `editedText` der `EditResponse` erlaubt ein
sofortiges Rewarm; bei Multi-File-Refactoring wird je `changed_path` mtime-geprüft.

---

## 5. Authentifizierung & Sicherheit

- **Token pro Projekt:** Beim Start erzeugt das Plugin ein zufälliges Token
  (`SecureRandom`, Hex), abgelegt in der Port-Datei. Es wird bei jedem HTTP-Request
  über den Header **`X-LeanCtx-Token`** geprüft.
- **401 bei Fehlen/Abweichung:** `headerToken != token` →
  `HttpResult(401, {"error":{"code":"UNAUTHORIZED",…}})` — keine Verarbeitung.
- **Nur loopback:** Der HTTP-Server lauscht auf `127.0.0.1` (nicht im Netz
  exponiert) und läuft im IDE-Benutzerkontext.
- **Rotation:** Bei IDE-Neustart entsteht eine neue Port-Datei mit neuem Token.

Siehe auch [Journey 13 — Security & Governance](13-security-and-governance.md).

---

## 6. Fehler-Katalog

**HTTP-Status:** `200` = Erfolg **oder** fachlicher Negativfall (Envelope); `401`
= Token fehlt/falsch; `404` = keine Route für `METHOD /path`; `500` = echte,
unerwartete Exception. (Eine `IllegalArgumentException`, z. B. leerer Body, wird als
`200` + `INTERNAL` zurückgegeben.)

**Envelope:** `{"error":{"code":"<CODE>","message":"<text>"}}`

| Code                    | Auslöser                                                        | Quelle                       | Behebung                                                  |
|-------------------------|-----------------------------------------------------------------|------------------------------|-----------------------------------------------------------|
| `UNAUTHORIZED`          | Token fehlt/falsch (401)                                        | Plugin (`RequestRouter`)     | gültigen `X-LeanCtx-Token` senden                         |
| `NOT_FOUND`             | unbekannte Route (404)                                          | Plugin                       | Endpunkt-Pfad prüfen                                      |
| `FILE_NOT_FOUND`        | Datei nicht lesbar                                              | Rust (`edit_apply`) / Plugin | Pfad mit `ctx_tree` verifizieren                          |
| `POSITION_OUT_OF_RANGE` | Zeile/Spalte hinter EOF / `end < start`                         | Rust / Plugin                | Range neu auflösen (`ctx_read`)                           |
| `CONFLICT`              | `expected_hash`/`plan_hash`-Mismatch; oder Konflikte ∧ `!force` | Rust                         | frisch lesen, Hash erneuern; ggf. `force`                 |
| `AMBIGUOUS_SYMBOL`      | `name_path` trifft >1 Symbol                                    | Rust (`ctx_refactor`)        | qualifizieren (`Class/method`) — Kandidatenliste beachten |
| `NO_SYMBOL`             | `name_path` trifft 0 Symbole                                    | Rust / Plugin                | Name/Pfad korrigieren                                     |
| `INDEXING`              | IDE im Dumb-Mode                                                | Plugin (`PsiLocator`)        | warten bis Indexierung fertig, erneut                     |
| `UNSUPPORTED_LANGUAGE`  | keine LSP-Config / kein PSI-Processor                           | Rust / Plugin                | Sprache wird (noch) nicht unterstützt                     |
| `BACKEND_REQUIRED`      | Refactoring ohne laufende IDE                                   | Rust (Trait-Default)         | IDE mit offenem Projekt starten                           |
| `INTERNAL`              | sonstiger Fehler / Parse                                        | beide                        | `message` prüfen; ggf. Bug melden                         |

---

## 7. End-to-End-Beispiele

**Beispiel 1 — Funktionsrumpf konfliktsicher ersetzen.**

```text
# 1. aktuelle Range + Hash holen (ctx_read liefert Bytes; Hash = BLAKE3 der Range)
ctx_refactor action=symbols_overview path=src/Main.kt        # Symbol + Zeile finden
# 2. ersetzen, gegen den erwarteten Hash abgesichert
ctx_refactor action=replace_symbol_body name_path=Main/run \
  new_body="fun run() { println(\"v2\") }" expected_hash=<blake3-hex>
# → applied:true ; bei zwischenzeitlicher Änderung → CONFLICT (Datei unangetastet)
```

**Beispiel 2 — projektweites Rename (Two-Phase).**

```text
# Phase 1: Vorschau — Verwendungen + Konflikte sehen
ctx_refactor action=rename path=src/Main.kt line=7 column=4 new_name=execute
#   intern: POST /renamePreview → {usages:[…], conflicts:[]}
# Phase 2: bei leeren conflicts automatisch Apply (eine Transaktion, ein Undo)
#   intern: POST /renameApply → {applied:true, changed_paths:[…]}
```

**Beispiel 3 — Datei reformatieren (nach einem Edit).**

```text
ctx_refactor action=replace_symbol_body name_path=Main/run new_body="…"
ctx_refactor action=reformat path=src/Main.kt    # Code-Style nachziehen
# → {"reformatted":true,"path":"src/Main.kt"}
```

---

## 8. Querverweise & Quellen

- [Knappe Agent-Referenz](appendix-jetbrains-plugin-de.md) — Tabellen für schnellen Lookup
- [Per-IDE-Quickstarts](appendix-ide-quickstarts.md) — Setup für JetBrains-IDEs
- [MCP-Tool-Map](appendix-mcp-tools.md) — alle MCP-Tools inkl. `ctx_refactor`, `ctx_symbol`
- [Journey 4 — Code Intelligence](04-code-intelligence.md)
- [Journey 13 — Security & Governance](13-security-and-governance.md) — PathJail, Auth
- Quellcode: `rust/src/lsp/{backend,jetbrains_backend,router,edit_apply,port_discovery}.rs`,
  `rust/src/tools/registered/ctx_refactor.rs`,
  `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/{server,endpoint,psi,dto}/…`
