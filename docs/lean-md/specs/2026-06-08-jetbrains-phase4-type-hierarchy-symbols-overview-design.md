# Design-Spec: JetBrains-Plugin Phase 4 — `type_hierarchy` + `symbols_overview` (B-only)

| Feld             | Wert                                                                                                     |
|------------------|----------------------------------------------------------------------------------------------------------|
| Status           | Genehmigt (Design), bereit für `writing-plans`                                                           |
| Datum            | 2026-06-08                                                                                               |
| Branch           | `feat-jetbrains-plugin`                                                                                  |
| Vorgänger        | Phase 3 (Commit `4d139ce0`) — Nav-Endpoints + gson-DTOs + E2E, abgeschlossen                             |
| Eltern-Spec      | `docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md` (§9 Phase 4, §17 Phase-3-Detail) |
| Nächster Schritt | `superpowers:writing-plans` (Implementierungsplan)                                                       |
| Sprache          | Code/Kommentare Englisch; Spec Deutsch                                                                   |

---

## 1. Context — Warum

Phase 4 ist die **zweite Phase mit echter PSI-Logik** im Plugin. Sie füllt zwei
B-only-Trait-Methoden, deren **Rust-Shapes bereits stehen** (`rust/src/lsp/backend.rs`,
seit Phase 0 als Default-`Err`-Methoden angelegt):

- `type_hierarchy(uri, position, direction) -> TypeHierarchyNode` — Super-/Subtyp-Baum.
- `symbols_overview(uri) -> Vec<SymbolOverviewItem>` — flache Top-Level-Struktur einer Datei.

Beide degradieren auf Backing A (rust-analyzer) **sauber** (Trait-Default-`Err`), sind also
**B-only** (nur im laufenden JetBrains-Plugin verfügbar). Phase 4 implementiert die
PSI-Seite (Plugin) + das Wire-/Tool-Wiring (Rust: `jetbrains_backend.rs` + `ctx_refactor.rs`).

**Schnitt:** `type_hierarchy` + `symbols_overview` zusammen, **ein Commit** (§12.3 des
Eltern-Specs: ein Commit pro Phase). `format`/`inspections` bleiben Phase 5;
symbolische Edits bleiben v2-Edit-Spec.

---

## 2. Fixierte Entscheidungen (User, 2026-06-08)

| # | Entscheidung                                                                                                                       | Begründung                                                                                                                                                                                                                     |
|---|------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1 | **Schnitt: beide Ops, ein Commit.** `type_hierarchy` + `symbols_overview` + DTOs + Rust-Wiring + E2E in einem reviewbaren Commit.  | Teilen Degradierungs-/Wire-/Test-Infrastruktur; konsistent mit Phase 2/3 (§12.3).                                                                                                                                              |
| 2 | **`type_hierarchy`-Tiefe: transitiv mit depth-Cap + Knoten-Cap + `truncated`.** Vorschlagswerte: `maxDepth = 5`, `maxNodes = 200`. | Nutzt die rekursive `children`-Struktur des Traits voll; token-sicher bei zentralen Symbolen (Object-Kette, viele Inheritors); Agent erkennt Unvollständigkeit. Spiegelt die Phase-3-Cap-Logik (500/`truncated`).              |
| 3 | **`symbols_overview`-Granularität: nur Top-Level.**                                                                                | Trait-Shape ist flach (`{name, kind, line}`, kein Container-Feld) — Member ohne Container wären mehrdeutig. Wie Serena `get_symbols_overview`-Default; token-effizient. Cap (Vorschlag 500) + `truncated`/`total` wie Phase 3. |
| 4 | **Sprachabdeckung: nur Kotlin** (Fixtures + `runIde`-Smoke).                                                                       | Konsistent mit Phase 3 (Entscheidung #3 dort). Java bleibt Follow-up (§6.3 hier / §17.6 #3 Eltern-Spec). ⚠ K2-Analysis-API-Risiko, siehe §7.                                                                                   |
| 5 | **`direction`-Param, Default `supertypes`.** `ctx_refactor action=type_hierarchy direction ∈ {supertypes, subtypes}`.              | Spiegelt `HierarchyDirection { Subtypes, Supertypes }` (ein Direction-Wert pro Call). `supertypes` = häufigster Use-Case („was erbt dieses Symbol").                                                                           |
| 6 | **Degradierung wie §9:** Backing A → sauberer ERROR; unsupported Sprache → `UNSUPPORTED_LANGUAGE`.                                 | Kein Crash, kein stiller A-Fallback. Nicht-JVM-Datei am Backing B → expliziter Fehlercode.                                                                                                                                     |

---

## 3. Architektur — Neue/erweiterte Komponenten

### 3.1 Kotlin (`com.leanctx.plugin`, `packages/jetbrains-lean-ctx`)

| Datei (neu / ~erweitert)                   | Aufgabe                                                                                                                                                   |
|--------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|
| `dto/HierarchyRequest.kt` (neu)            | `{path, line, character, direction, scope?}`; `direction ∈ {supertypes, subtypes}`; `scope` optional, Default `project`.                                  |
| `dto/FileRequest.kt` (neu)                 | `{path}` für `symbols_overview`.                                                                                                                          |
| `dto/TypeHierarchyNode.kt` (neu)           | rekursiv `{name, path, line, children:[TypeHierarchyNode]}` — exakt Rust-`TypeHierarchyNode` (§5).                                                        |
| `dto/TypeHierarchyResponse.kt` (neu)       | `{tree: TypeHierarchyNode, truncated: Boolean}`.                                                                                                          |
| `dto/SymbolOverviewItem.kt` (neu)          | `{name, kind, line}` — exakt Rust-`SymbolOverviewItem`.                                                                                                   |
| `dto/SymbolsOverviewResponse.kt` (neu)     | `{symbols:[SymbolOverviewItem], truncated: Boolean, total: Int}` (Phase-3-`LocationsResponse`-Muster).                                                    |
| `psi/TypeHierarchyResolver.kt` (neu)       | Offset → benanntes Element; JVM: `ClassInheritorsSearch`/`OverridingMethodsSearch`; Kotlin: K2-Analysis-API; **transitiv mit `maxDepth`/`maxNodes`-Cap**. |
| `psi/FileStructureScanner.kt` (neu)        | `PsiFile` → **Top-Level**-Deklarationen → `kind`-Mapping.                                                                                                 |
| `endpoint/TypeHierarchyHandler.kt` (neu)   | Body-Parse → PSI unter `ReadAction` → `TypeHierarchyResponse`.                                                                                            |
| `endpoint/SymbolsOverviewHandler.kt` (neu) | Body-Parse → PSI unter `ReadAction` → `SymbolsOverviewResponse`.                                                                                          |
| `server/RequestRouter.kt` (~erweitern)     | +2 POST-Routen `/type_hierarchy`, `/symbols_overview`; gson-Parse; Token-Check (401) unverändert; `/health` unverändert.                                  |
| `psi/PsiLocator.kt` (wiederverwenden)      | Phase-3-Komponente (`path → VirtualFile → PsiFile`, `(line,char) → offset`, `DumbService`-Smart-Mode-Guard). **Kein Umbau.**                              |

### 3.2 PSI-Auflösung (editor-los — Handler laufen off-EDT)

- **`type_hierarchy`:** Offset → `findReferenceAt`/`findElementAt` → Hochlaufen zum benannten
  Element (Klasse/Interface/Methode). Dann nach `direction`:
    - `supertypes`: Superklassen/Interfaces (JVM: `PsiClass.supers`/Typhierarchie; Methoden: überschriebene); transitiv
      hochlaufen.
    - `subtypes`: `ClassInheritorsSearch` (Klassen/Interfaces) bzw. `OverridingMethodsSearch` (Methoden), eingeschränkt
      durch `scope`.
    - **Rekursion** bis `maxDepth` (Vorschlag 5); globaler Knoten-Zähler bis `maxNodes` (Vorschlag 200) → bei
      Überschreitung Abbruch + `truncated=true`.
    - Kein Symbol an Offset → `NO_SYMBOL_AT_POSITION` (HTTP 200).
    - Nicht-JVM-/nicht-hierarchiefähige Datei → `UNSUPPORTED_LANGUAGE` (HTTP 200).
- **`symbols_overview`:** Top-Level-Kinder des `PsiFile` (Klassen, Top-Level-Funktionen/
  Properties, Objects). `kind` aus PSI-Typ (`class`/`interface`/`object`/`function`/`property`).
  Cap (Vorschlag 500) + `truncated`/`total` wie Phase 3.
- **Scope** (relevant v.a. `subtypes`): `project` → `GlobalSearchScope.projectScope(project)`;
  `all` → `allScope(project)`. Default `project`.
- **Threading:** alle Reads `ReadAction.nonBlocking{ … }.executeSynchronously()` im Smart-Mode
  (`runReadActionInSmartMode` → sonst `INDEXING`) — **off-EDT** (§7-Risiko).

---

## 4. Wire-Protokoll (Deltas gegen §6 Eltern-Spec)

### `POST /type_hierarchy`

```
Request : { path, line, character, direction: "supertypes"|"subtypes", scope?: "project"|"all" }
Response: { tree: { name, path, line, children: [ … ] }, truncated: bool }
Error   : { error: { code, message } }   // NO_SYMBOL_AT_POSITION | UNSUPPORTED_LANGUAGE | INDEXING | FILE_NOT_FOUND
```

### `POST /symbols_overview`

```
Request : { path }
Response: { symbols: [ { name, kind, line } ], truncated: bool, total: int }
Error   : { error: { code, message } }   // FILE_NOT_FOUND | INDEXING | UNSUPPORTED_LANGUAGE
```

- 0/1-Naht: Wire-`line`/`character` **0-basiert** ↔ PSI-Offset; Response-`line` **1-basiert**
  (konsistent mit `TypeHierarchyNode.line` / `SymbolOverviewItem.line`-Doc in `backend.rs`).
- `truncated` wird Rust-seitig in Phase 4 **toleriert, noch nicht ausgewertet** (Follow-up §6.1,
  wie Phase-3-Follow-up §17.6 #1).

---

## 5. Rust-Deltas (Phase-4-Bestandteil)

- **`rust/src/lsp/jetbrains_backend.rs`:** `type_hierarchy` + `symbols_overview` **überschreiben**
  (Trait-Default-`Err` → echter HTTP-POST gegen `/type_hierarchy` bzw. `/symbols_overview`).
  Response-Parser: `tree` (rekursiv → `TypeHierarchyNode`), `symbols` (→ `Vec<SymbolOverviewItem>`);
  `truncated`/`total` toleriert. `direction` + `scope` im Request-Body durchreichen.
- **`rust/src/tools/ctx_refactor.rs`:** zwei neue Actions —
    - `action=type_hierarchy` mit `direction`-Param (Default `supertypes`) + optionalem `scope`.
    - `action=symbols_overview` (nur `path`).
    - **Single schema source** (`tool_def`-Registry, §4.4 / #141) — **keine** zweite Schema-Kopie;
      Drift-Regression-Test grün halten.
- **Degradierung:** Backing A (kein IDE / Fallback) → Trait-Default-`Err`
  („… requires the JetBrains backend") surfacet als **sauberer `ctx_refactor`-ERROR** (kein Crash,
  kein stiller A-Fallback). Plugin-seitiges `UNSUPPORTED_LANGUAGE` wird als Fehler durchgereicht.
- *Gate:* `cargo nextest run` grün (Wire-/`direction`-/`scope`-Durchreichung; Backing-A-Regressionsschutz).

---

## 6. Offene Follow-ups (in Phase 4 angelegt, später)

1. **`truncated`/`total` Rust-seitig auswerten** — `ctx_refactor`-Ausgabe sollte
   „… (truncated, N nodes)" surfacen. Ziel: Phase 5 oder Begleit-Commit (= §17.6 #1).
2. **`scope=all` Token-Volumen** bei `subtypes` zentraler Interfaces — beobachten, ggf.
   niedrigeres Cap. Nicht vorab optimieren (= §17.6 #2).
3. **Java-Fixtures nachziehen** — Phase 4 ist Kotlin-only (Entscheidung #4). Java-Abdeckung
   (stabilere Hierarchie-Regression) als Follow-up; relevant spätestens wenn der K2-Fallback
   (§7) gezogen werden muss (= §17.6 #3).
4. **Direction-`both`** — Phase 4 erlaubt pro Call **eine** Direction (`supertypes` XOR
   `subtypes`, gespiegelt vom `HierarchyDirection`-Enum). Eine kombinierte „beide
   Richtungen"-Antwort wäre additiv möglich, ist aber nicht Teil von v1.

---

## 7. Risiken

- **K2-/Analysis-API-Kopplung (Hauptrisiko, Entscheidung #4).** `type_hierarchy` auf Kotlin
  (Subtyp-/Inheritor-Suche) triggert auf IC 2026.1/K2 die Analysis-API
  (`KotlinFirDefinitionsSearcher`) und wirft `ProhibitedAnalysisException`
  („Analysis is not allowed: Called in the EDT thread"), wenn nicht **off-EDT** ausgeführt.
  **Pflicht:** alle Hierarchie-Reads in `ReadAction.nonBlocking{}.executeSynchronously()`
  off-EDT. Falls das In-Process-Kotlin-Test-Setup blockiert: **Fallback** = Java-Fixtures für
  die automatisierte Regression (§6.3), Kotlin nur im manuellen `runIde`-Smoke — analog
  Phase-3-Fallback §17.7. (Memory-Befund `jetbrains-k2-inheritor-search-edt`.)
- **`assertThrows`-Falle (JUnit3-Hierarchie).** In `BasePlatformTestCase` immer
  `org.junit.Assert.assertThrows(...)` **voll qualifiziert** — der geerbte unqualifizierte
  `assertThrows` liefert `Throwable`, `.code`/Feldzugriff bleibt unauflösbar.
  (Memory-Befund `K4-assertThrows-pattern`.)
- **PsiLocator-Fixture-Auflösung.** `PsiLocator.psiFile` löst über
  `LocalFileSystem.findFileByPath(...)` auf; `BasePlatformTestCase`-Light-Fixtures
  (`myFixture.configureByText`) liegen im `TempFileSystem` — Fixture-Pfade entsprechend setzen.
  (Memory-Befund `jetbrains-psilocator-light-fixture-resolution`.)
- **`runIde`-Index-Abhängigkeit.** Wie Phase 3: ein nur als *Ordner* geöffnetes Repo liefert
  ohne Kotlin-Modul-Import `total:0`. Für nicht-leere Live-Treffer ein konfiguriertes
  Kotlin-Projekt öffnen (= Phase-3-Gate-Protokoll, offener Punkt 1).

---

## 8. Verifikation (End-to-End) — Gate

1. **Kotlin-Fixtures** (`BasePlatformTestCase`):
    - `type_hierarchy supertypes`: Superklassen-/Interface-Kette korrekt; Methoden-Override.
    - `type_hierarchy subtypes`: `ClassInheritorsSearch`/`OverridingMethodsSearch`-Treffer.
    - depth-/node-Cap → `truncated=true`; `NO_SYMBOL_AT_POSITION`; `INDEXING`;
      `scope=project` vs `all`; `UNSUPPORTED_LANGUAGE` (Nicht-JVM-Fixture).
    - `symbols_overview`: Top-Level-Soll-Liste + `kind`-Mapping; Cap/`truncated`.
    - explizite 0/1-Naht (Wire 0-basiert ↔ PSI-Offset ↔ Response 1-basiert).
    - `assertThrows` voll qualifiziert (§7).
2. **Manuelles `runIde`** (IC/IU-2026.1.x): curl `/type_hierarchy|/symbols_overview` gegen
   Kotlin-Testprojekt; Abgleich mit IDE „Type Hierarchy" / „Structure"-View.
3. **`cargo nextest run`** grün (Rust-Wire + `direction`/`scope`-Durchreichung + Backing-A-Regression);
   `cargo clippy` ohne neue Lints.
4. **Fallback ohne IDE** → Backing A → sauberer ERROR (kein Crash); Regressionsschutz.
5. **Companion-Plugin** (Statusbar/Actions) weiterhin funktional (keine Regression).

---

## 9. Referenz-Artefakte

- Rust-Shapes (bereits vorhanden): `rust/src/lsp/backend.rs:13-94`
  (`HierarchyDirection`, `TypeHierarchyNode`, `SymbolOverviewItem`, Trait-Default-`Err`).
- Wire-Backend: `rust/src/lsp/jetbrains_backend.rs` (Phase-3-Parser als Muster).
- Tool-Schema: `rust/src/tools/ctx_refactor.rs` + `tool_def`-Registry (§4.4).
- Plugin-Phase-3-Muster: `packages/jetbrains-lean-ctx` (`PsiLocator`, `dto/*`, `endpoint/*`,
  `server/RequestRouter`).
- Serena-Architektur-Referenz (**nicht** Code-Quelle, §9 Eltern-Spec Phase 4):
  `endpoint/TypeHierarchyHandler` + `GetSubtypesHandler`/`GetSupertypesHandler`,
  `symbol/TypeHierarchy(.Node)`/`SubtypeHierarchy`/`SupertypeHierarchy`,
  `exception/TypeHierarchyNotSupportedException`; `endpoint/GetSymbolsOverviewHandler`,
  `symbol/FileStructure`; DTOs `TypeHierarchyNodeDTO`/`TypeHierarchyResponse`/
  `GetSymbolsOverviewResponse`/`FileStructureDTO`.

---

## 10. Gate-Protokoll Phase 4 (2026-06-08)

**Implementierung** (subagent-driven-development, je Task spec+quality-reviewed) — 9 Task-Commits
`6228d88e`→`4a6fde2d` (R1 Schema, R2 Dispatch/Format, R3 `JetBrainsHttpBackend`-Overrides,
K1 Wire-DTOs, K2 `TypeHierarchyResolver` +Fix `b35d073c`, K3 `FileStructureScanner`,
K4 `StructureHandlers`, K5 `RequestRouter`-Routen) + Doc-Regen `1a9687df` + E1-Fix `d1d73c14`.

**Automatisierte Gates:** Kotlin `./gradlew test` **54/54 grün**; Rust `cargo nextest run` grün
(2 vorbestehende, unabhängige Fails: `hn_hardening_scenarios` Shell-Compression-Cluster);
`cargo clippy --all-targets` ohne neue Lints; Drift-Gate `generated/mcp-tools.md` grün.
Finaler Rust↔Kotlin-Paritäts-Review: alle 7 Punkte ✅ (Wire-Keys, **1-based-line-Seam ohne
Doppel-+1**, `direction`-Default, Error-Envelope, Degradation).

**E1 `runIde`-Gate (manuell, IC2026.1.3, Projekt `packages/jetbrains-lean-ctx`):**
Direkte HTTP-Verifikation aller Endpoints **bestanden**:
- `symbols_overview` → `{symbols[],truncated,total}`, **1-basierte Zeilen** bestätigt.
- `type_hierarchy` `supertypes` (`scope=all`) → volle transitive Kette inkl. Plattform-/JDK-Typen
  (`BackendException`→`RuntimeException`→`Exception`→`Throwable`→`Object`/`Serializable`;
  `LeanCtxStatusBarFactory`→`StatusBarWidgetFactory`).
- `type_hierarchy` `subtypes` → Root mit leeren `children` (final), `truncated:false`.
- Falscher Token → **401**; unbekannter Pfad → `FILE_NOT_FOUND`-Envelope (HTTP 200).

**Zwei E1-Funde (nur via runIde sichtbar — Tests grün) → gefixt in `d1d73c14`:**
1. **KRITISCH:** Plugin nutzt `org.jetbrains.kotlin.psi.*` (`KtFile`/`KtClassOrObject`/…) zur
   Laufzeit → `NoClassDefFoundError` (Handler-Crash, curl `exit 52`). `bundledPlugin(...)` in
   `build.gradle.kts` deckt nur Compile/Sandbox; die **Laufzeit-Classloader-Bindung** fehlte.
   Fix in `plugin.xml`: `<depends>org.jetbrains.kotlin>` + `<supportsKotlinPluginMode supportsK2="true"/>`
   (K2-Pflicht-Deklaration ab IC2026.1). Latent seit Phase 2/3 — Tests grün, weil
   `BasePlatformTestCase` das Kotlin-Plugin im Test-Classpath führt, der Laufzeit-Plugin-Classloader
   aber nicht.
2. `rust/src/lsp/config.rs::language_for_extension` mappte `.kt`/`.kts` nicht → `ctx_refactor`
   erreichte den JetBrains-Backend für Kotlin-Dateien nie (Gate in `router.rs` vor `select_backend`).
   Fix: `"kt" | "kts" => Some("kotlin")`.

**Env-gebundene Restpunkte (deferred):** Live-`ctx_refactor`-für-`.kt`-E2E (§8 Step 4) setzt
voraus, dass die MCP-Server-Projektwurzel mit dem in der Sandbox geöffneten Projekt übereinstimmt
(Port-Discovery über `project_hash`); separat vom hier bestätigten direkten HTTP-Pfad. Backing-A-
Degradation (§8 Step 5, `.rs`/kein IDE → `ERROR: … requires the JetBrains backend`) strukturell
über Trait-Default-`Err` abgedeckt.
