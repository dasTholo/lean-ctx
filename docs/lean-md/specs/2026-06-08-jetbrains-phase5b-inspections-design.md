# Design-Spec: JetBrains-Plugin Phase 5b — `inspections` (run + list) + CI-Härtung

| Feld             | Wert                                                                                               |
| ---------------- | -------------------------------------------------------------------------------------------------- |
| Status           | Genehmigt (Design), bereit für `writing-plans`                                                      |
| Datum            | 2026-06-08                                                                                          |
| Branch           | `feat-jetbrains-plugin`                                                                             |
| Vorgänger        | Phase 5a (Commits `4b60d3fa`→`ddd51dcb`) — Härtung H1–H5a, abgeschlossen                            |
| Eltern-Spec      | `docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md` (§9 Phase 5, §13.3)         |
| Schwester-Spec   | Phase 5a — `docs/lean-md/specs/2026-06-08-jetbrains-phase5a-hardening-design.md`                    |
| Nächster Schritt | `superpowers:writing-plans` (Implementierungsplan)                                                  |
| Sprache          | Code/Kommentare Englisch; Spec Deutsch                                                              |

---

## 1. Context — Warum

§9 des Eltern-Specs bündelte „Phase 5" ursprünglich als **format + inspections + Härtung**. Die
Härtung ist als **Phase 5a** gelandet (H1–H5a, automatisierte Gates grün). Übrig bleibt der
**Feature-Rest** — und der wird beim Design weiter geschnitten:

1. **`format` wandert nach v2 (User-Entscheidung 2026-06-08).** `format` ist inhärent eine
   **Edit-Operation** (`reformat` schreibt die Datei via `WriteCommandAction`/EDT). Das Eltern-Spec
   §3 verortet alle Edit-Ops bereits explizit im **v2-Edit-Spec** (EDT, Transaktionalität, Undo,
   Konflikt-Handling, Cache-Kohärenz). Eine „edits liefern, nicht anwenden"-v1-Variante wäre für den
   Agenten halbgar (er bekäme TextEdits, die er nicht sauber anwenden kann). → `format` ist **kein**
   5b-Scope.
2. **`inspections` ist rein read-only Diagnostik** — ein PSI-Feature-Endpoint exakt im Phase-3/4-
   Muster (Handler + DTOs + Wire + Rust-Tool-Action + Degradierung). Das ist Phase 5b.

**Phase 5b = `inspections` (run + list) + CI-Härtung, ein Commit (§12.3 Eltern-Spec).**

### Offener Rest (Feature)

| #  | Item                                                              | Quelle                          |
| -- | ---------------------------------------------------------------- | ------------------------------- |
| F1 | `ctx_refactor action=inspections mode=run`  (run-on-file)        | §9 Phase 5 / §13.3 / Serena-Ref |
| F2 | `ctx_refactor action=inspections mode=list` (verfügbare)         | §9 Phase 5 / §13.3 / Serena-Ref |
| F3 | CI-Härtung: `concurrency` + `timeout-minutes` + Action-SHA-Pin   | Phase-5a §9.5 (out-of-scope)    |
| F4 | `actionlint`-Gate (leichter YAML-Lint)                           | User-Vorschlag 2026-06-08       |

---

## 2. Fixierte Entscheidungen (User, 2026-06-08)

| # | Entscheidung                                                                                       | Begründung                                                                                                                                                                                                                          |
| - | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1 | **`format` → v2-Edit-Spec, nicht 5b.**                                                             | Inhärente Edit-Op (EDT/`WriteCommandAction`/Undo). Eltern-Spec §3 verortet Edits in v2. „Liefern-nicht-anwenden" wäre halbgar. Der bestehende `format`-Trait-Stub (`backend.rs:103`, default-`Err`) **bleibt** für v2 liegen.        |
| 2 | **`inspections` deckt `run` + `list` ab, Dispatch über `mode`-Param (Variante A).**               | Konsistent mit dem Phase-4-`type_hierarchy`-`direction`-Param-Idiom. Kompakte Action-Oberfläche (ein Action-Eintrag, ein Hilfetext). Rust dispatcht intern auf zwei Trait-Methoden + zwei Wire-Endpoints.                            |
| 3 | **Test-Sprachabdeckung: Kotlin-only.**                                                             | Bleibt beim Phase-3/4-Muster. Java-Fixture (Eltern-Spec §17.6 #3 / 5a-Follow-up #4) wird **eigener** Follow-up — bläht 5b nicht auf.                                                                                                 |
| 4 | **CI-Härtung (5a §9.5) in 5b mitnehmen.**                                                          | 5b fasst den Plugin-CI-Test-Job ohnehin an (neuer `inspections`-Test). `concurrency` + `timeout-minutes` + SHA-Pinning als kleiner Block schließt den bewusst ausgelagerten 5a-Rest sauber ab.                                       |
| 5 | **`actionlint` als Gate, `act` nur dokumentiert.**                                                 | `github/local-action` passt nicht (wir haben keine Custom-JS-Action). `actionlint` ist leichtgewichtig (kein Docker), validiert YAML/Expressions/SHA-Pins. `nektos/act` (voller Workflow lokal, ~1 GB IC-Image) bleibt manuell.      |
| 6 | **`list_inspections` listet nur das *enabled* Projekt-Profil, nicht alle registrierten Tools.**   | Alle registrierten IntelliJ-Inspektionen sind Hunderte → Token-Explosion. Das aktuelle `InspectionProfile` des Projekts ist der relevante, gebundene Use-Case. Zusätzlich Cap + `truncated`/`total`.                                 |

---

## 3. Architektur — Neue/erweiterte Komponenten

Muster identisch zu Phase 3 (Nav) / Phase 4 (Hierarchie/Overview): keine neuen Sicherheits-Nähte.
Rust-PathJail (`jail_path`) bleibt der **alleinige** Validierungspunkt, läuft VOR jedem HTTP-Request
(Eltern-Spec §2 Entscheidung 4); das Plugin re-validiert Pfade nicht.

### 3.1 Rust-Seite

| Datei (~erweitert)                      | Aufgabe                                                                                                                                                                                                                                                                                  |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `rust/src/lsp/backend.rs` (~)           | Neuer Typ `InspectionInfo { id: String, name: String, severity: String }`. Neue **Default-degrading** Trait-Methode `fn list_inspections(&mut self) -> Result<Vec<InspectionInfo>, String>` (default-`Err`, im Default-degrading-Block). `inspections(uri) -> Vec<InspectionDiag>` existiert bereits (`backend.rs:106`), `InspectionDiag` bereits definiert (`backend.rs:42`). |
| `rust/src/lsp/jetbrains_backend.rs` (~) | Beide Methoden implementieren: `inspections` → `path_body(uri)` → `post("/inspections")` → neuer Parser `parse_inspections` (Muster `parse_symbols` `:149`) + `self.last_meta = Self::parse_truncation(...)`. `list_inspections` → `path_body`-artiger `{path}`-Body → `post("/list_inspections")` → neuer Parser `parse_inspection_list` + `last_meta`. Error-Envelope-Mapping wie `:263-269`. |
| `rust/src/tools/ctx_refactor.rs` (~)    | Action `"inspections"` in den Match-Block (`:26-38`) + Hilfetext (`:35-36`). Neuer `handle_inspections`: `mode = args.mode` (default `run`); `run` → `with_backend(file, root, |b,_| b.inspections(uri))`; `list` → `with_backend(file, root, |b,_| b.list_inspections())`. Output kompakte Zeilenliste; `truncated`-Suffix via bestehendem H3-Pfad (`backend.last_truncation()`). |

**`mode`-Dispatch (Variante A):** `run` (default) braucht eine Datei (Position irrelevant —
file-level); `list` braucht den `path` nur zur **Backend-/Projekt-Wahl** (`with_backend` selektiert
über die Datei-Extension), die Inspektions-Liste selbst ist projektweit. `list` benötigt **keinen**
`open_file`-Inhalt (Detail: ob `handle_inspections` für `list` `open_file` überspringt, entscheidet
der Plan; minimal-invasiv ist die Wiederverwendung des bestehenden Pfads, da der Agent ohnehin einen
realen Projekt-Pfad übergibt).

**Output-Format (Vorschlag, Detail im Plan):**
- `run`: je Zeile `path:line  SEVERITY  message`; bei Cap Suffix `… (truncated — N von M)`.
- `list`: je Zeile `id  name  severity`; bei Cap analoges Suffix.

### 3.2 Kotlin-Seite

| Datei (neu/~)                                                  | Aufgabe                                                                                                                                                                                                                                                              |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `dto/Wire.kt` (~)                                             | Neue DTOs: `InspectionDiagDTO { path, line, severity, message }`, `InspectionsResponse { diagnostics, truncated, total }`, `InspectionInfoDTO { id, name, severity }`, `ListInspectionsResponse { inspections, truncated, total }`. `line` **1-basiert** (wie `SymbolOverviewItemDTO`). `JsonCodec.parseFileRequest` wird für beide Endpoints wiederverwendet (Body = `{path}`). |
| `endpoint/InspectionHandlers.kt` (neu)                        | `class InspectionHandlers(project)` analog `StructureHandlers`. `runOnFile(req: FileRequest): InspectionsResponse` — `inSmartReadAction`; `InspectionManager` + enabled Tools des Projekt-`InspectionProfile` auf das `PsiFile`; `ProblemDescriptor` → DTO (`Document` für 1-basierte Zeile; `HighlightSeverity` → String). `listAvailable(req: FileRequest): ListInspectionsResponse` — enabled Tools des Projekt-Profils → `{ id = shortName, name = displayName, severity }`. Beide mit Cap + `truncated`/`total`. |
| `server/RequestRouter.kt` (~)                                 | Zwei POST-Routen (`:39-41`-Muster): `/inspections` → `dispatchInspections`, `/list_inspections` → `dispatchListInspections`. Je ein `try/catch`-Dispatcher wie `dispatchOverview` (`:79`): `BackendException` → 200 mit Error-Envelope, `IllegalArgumentException` → 200 `INTERNAL`, `Exception` → 500. `InspectionHandlers` als Feld instanziieren. |

### 3.3 CI-Härtung (`.github/workflows/jetbrains-plugin.yml`)

Stand vor 5b (geprüft 2026-06-08): bereits gehärtet (5a) — `persist-credentials: false`,
`permissions: contents: read` (Release-Job eskaliert gezielt auf `contents: write`),
`gradle/actions/wrapper-validation`. **Offen (5a §9.5):**

| Item                  | Aufgabe                                                                                                                                                              |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `concurrency`-Group   | Workflow-Ebene: `group: ${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: true` (parallele Läufe pro Ref canceln).                                    |
| `timeout-minutes`     | An allen 3 Jobs (build/test/release). Richtwert: build/release 20, test 30 (IC-Download ~1 GB).                                                                     |
| Action-SHA-Pinning    | Alle `uses:` auf Commit-SHA + `# vX.Y.Z`-Kommentar pinnen (checkout, setup-java, gradle/actions/setup-gradle, gradle/actions/wrapper-validation, upload-artifact). |
| `actionlint`-Gate     | Leichter Lint-Step (kein Docker) validiert YAML/Expressions/SHA-Pins. Als eigener kurzer Job oder lokaler Pre-Commit-Lauf (Detail im Plan).                          |

---

## 4. Wire-Protokoll (zwei neue snake_case-Endpoints)

Konvention wie Phase 3/4 (`/type_hierarchy`, `/symbols_overview`): fachliche Negativfälle = HTTP 200
mit `{error:{code,message}}`, 401 nur Token, 500 nur echte Exception. Pfade **snake_case** (nicht das
camelCase aus Eltern-Spec §6 — der reale Router nutzt snake_case).

**`POST /inspections`** (run-on-file)
- Req: `{ path }` (projekt-relativ, wie `/symbols_overview`).
- Resp: `{ diagnostics: [{ path, line, severity, message }], truncated, total }`.
  `line` 1-basiert; `severity ∈ {ERROR, WARNING, WEAK_WARNING, INFO}` (IntelliJ-`HighlightSeverity`
  → fester String).
- Cap analog Phase 3/4 (z. B. 500 Diagnostics) → `truncated`/`total`.

**`POST /list_inspections`** (verfügbare Inspektionen)
- Req: `{ path }` (nur Backend-/Projekt-Wahl; Liste ist projektweit).
- Resp: `{ inspections: [{ id, name, severity }], truncated, total }`.
  **enabled** Tools des aktuellen Projekt-`InspectionProfile` (nicht alle registrierten), gecappt.

**Fehler-Codes (wiederverwendet, Eltern-Spec §6):** `FILE_NOT_FOUND`, `INDEXING`,
`UNSUPPORTED_LANGUAGE` (nur `run`), `INTERNAL`. Rust mappt `code` → `ERROR: …`-String.

**Degradierung (unverändert):** auto → Fallback A → `inspections`/`list_inspections` sind dort
default-`Err` → `ERROR: inspections requires the JetBrains backend` (konsistent mit `type_hierarchy`).
`b_only` ohne IDE → `select_backend`-`Err`.

---

## 5. Verifikation (End-to-End) — Gate

1. **`cargo nextest run`** grün (niemals `cargo test`):
    - **F1/F2 Parser:** `parse_inspections` / `parse_inspection_list` gegen Wire-Mock (Muster
      `symbols_overview_parses_wire_items` `:400`); Error-Envelope → `Err`.
    - **Dispatch:** `ctx_refactor action=inspections` mit `mode=run` und `mode=list` rufen die
      korrekte Trait-Methode; unbekannter `mode` → definiertes Verhalten (Default `run` oder `ERROR`,
      Detail im Plan).
    - **Truncated-Surfacing:** Output trägt das Suffix bei `truncated=true`, keins bei `false`.
    - **Degradierung:** Backing A → `Err` (kein `inspections`/`list_inspections`).
    - `cargo clippy --all-targets` ohne neue Lints.
2. **Kotlin `./gradlew check`** grün (headless), inkl. der neuen Tests (§6). H5a-Hygiene gilt
   weiter: Suite hinterlässt **keine** Port-Dateien.
3. **Drift-Gate:** Neue Action `inspections` + `mode`-Param ändern das `ctx_refactor`-Schema →
   `docs/reference/generated/mcp-tools.md` **regenerieren** + `docs/reference/appendix-mcp-tools.md`
   pflegen, sonst rote `reference_docs_drift` / `docs_tool_counts_up_to_date` /
   `mcp_manifest_up_to_date`.
4. **CI:** `actionlint` grün gegen `jetbrains-plugin.yml`; `concurrency`/`timeout-minutes`/SHA-Pins
   im YAML vorhanden.
5. **Manuelles `runIde`** (user-gated, IC/IU-2026.1.x): IDE auf → `action=inspections mode=run` auf
   eine Datei mit bekanntem Problem → erwartete Diagnostics; `mode=list` → nicht-leere Profil-Liste;
   IDE schließen → nächster Call fällt **sauber** auf `ERROR` (kein Hänger gegen toten Endpoint).
6. **Fallback ohne IDE** → Backing A unverändert (Regressionsschutz, kein `inspections`).

---

## 6. Tests (Phase-3/4-Parität)

Bestehendes Muster: Handler werden **router-getrieben** getestet (`RequestRouterNavTest`,
`RequestRouterStructureTest` = BasePlatformTestCase mit echtem PSI), **nicht** über dedizierte
`*HandlersTest`. DTOs separat via `JsonCodecTest`. Generisches Auth/404 in `RequestRouterTest`.

| Datei (neu/~)                                            | Abdeckung                                                                                                                                                                                                                |
| -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `server/RequestRouterInspectionTest.kt` (neu)           | Spiegelt `RequestRouterStructureTest`. Treibt den Router mit Kotlin-Fixture über `POST /inspections` (run → erwartete Diagnostics auf Datei mit bekanntem Problem) + `POST /list_inspections` (Liste nicht-leer, enthält erwartete Inspektion). Deckt **`InspectionHandlers` + Router-Routing** zusammen ab. |
| `dto/JsonCodecTest.kt` (~)                               | Round-Trip-Fälle für `InspectionsResponse`, `ListInspectionsResponse`, `InspectionInfoDTO`/`InspectionDiagDTO`; `parseFileRequest`-Wiederverwendung bestätigt.                                                            |
| `server/RequestRouterTest.kt` (~, optional)             | 401/404 ist generisch bereits abgedeckt; ein 404-Negativfall für einen unbekannten Inspektions-Pfad nur falls sinnvoll.                                                                                                  |

**Coverage-Matrix:** `InspectionHandlers.kt` → `RequestRouterInspectionTest`; `Wire.kt`-DTOs →
`JsonCodecTest`; `RequestRouter.kt`-Routen → `RequestRouterInspectionTest`. Rust-Seite über
`cargo nextest` (§5.1).

---

## 7. Offene Follow-ups (in 5b angelegt / übernommen, später)

1. **`format`** — v2-Edit-Spec (EDT/`WriteCommandAction`/Undo). Trait-Stub bleibt liegen
   (`backend.rs:103`).
2. **Java-Fixture-Abdeckung** — Plugin-Regression bleibt Kotlin-only; Java relevant spätestens beim
   K2-Fallback (Eltern-Spec §17.6 #3 / 5a-Follow-up #4). Eigener Follow-up.
3. **Same-Root-Port-Datei-Kollision (H5b, aus 5a)** — `runIde`-Sandbox + Produktiv-IDE auf demselben
   Root. Bekannte Limitierung; pid-suffigierter Dateiname als Fix-Skizze.
4. **`scope`/Token-Volumen** — `list_inspections` ist auf das enabled Profil + Cap begrenzt; bei
   großen Profilen beobachten, nicht vorab optimieren.
5. **`nektos/act`** — voller Workflow lokal (Docker, ~1 GB IC-Image). Nur dokumentiert, kein
   Pflicht-Gate.

---

## 8. Risiken

- **`list_inspections`-Volumen.** Alle registrierten Tools wären Hunderte → bewusst auf das
  **enabled Projekt-Profil** begrenzt + Cap/`truncated` (Entscheidung #6). Test bestätigt nicht-leere,
  aber gebundene Liste.
- **`InspectionManager`/ReadAction.** Die Inspektions-API verlangt Smart-Mode/ReadAction → während
  Indizierung `INDEXING`-Code zurückgeben (wie Phase 3/4), nicht blockieren.
- **`HighlightSeverity` → String-Mapping.** Muss stabil sein (Rust erwartet feste Tokens
  `ERROR|WARNING|WEAK_WARNING|INFO`); unbekannte Severities auf `INFO` o. nächstliegend mappen.
- **SHA-Pinning-Wartung.** Gepinnte Action-SHAs veralten; `# vX.Y.Z`-Kommentar hält sie nachführbar
  (Dependabot kann SHA-Pins aktualisieren — optionaler Follow-up).
- **CI-Headless-Stabilität.** Unverändert zu 5a: BasePlatformTestCase lädt die IC-Plattform; Gradle-/
  IC-Cache + Pin `2026.1.3` halten den Lauf deterministisch. Kein `runIde` in CI.

---

## 9. Referenz-Artefakte

- Trait: `rust/src/lsp/backend.rs:42` (`InspectionDiag`), `:106` (`inspections`-Stub), `:88-108`
  (Default-degrading-Block), `:117-121` (`last_truncation`). `list_inspections` + `InspectionInfo`
  neu.
- HTTP-Backend: `rust/src/lsp/jetbrains_backend.rs:63` (`post`), `:149` (`parse_symbols`-Muster),
  `:166` (`parse_truncation`), `:175` (`path_body`), `:251-287` (`type_hierarchy`/`symbols_overview`
  als Methoden-Muster), `:263-269` (Error-Envelope-Mapping).
- Tool: `rust/src/tools/ctx_refactor.rs:6-39` (Dispatch + Hilfetext), H3-Truncated-Pfad.
- Router/Selektion: `rust/src/lsp/router.rs:58-141` (`select_backend`/`with_backend`/Degradierung).
- Kotlin: `server/RequestRouter.kt:39-41` (POST-Routen), `:79` (`dispatchOverview`-Muster);
  `endpoint/StructureHandlers.kt` (Handler-Muster); `dto/Wire.kt` (DTOs + `JsonCodec`).
- Kotlin-Tests: `server/RequestRouterStructureTest.kt`, `server/RequestRouterNavTest.kt`,
  `dto/JsonCodecTest.kt` (Muster).
- CI: `.github/workflows/jetbrains-plugin.yml` (Build/Test/Release-Jobs).
- Serena-Ref (Architektur/Namen, **keine** Code-Quelle): `endpoint/InspectionRunner` +
  `RunInspectionsOnFileHandler`/`ListInspectionsHandler`; DTOs `InspectionInfoDTO`/
  `InspectionProblemDTO`/`InspectionsResponse`.
