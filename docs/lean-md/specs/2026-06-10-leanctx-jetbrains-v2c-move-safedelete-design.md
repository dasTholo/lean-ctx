# Design-Spec: lean-ctx JetBrains v2c — Move & Safe-Delete (Serena-Ablösung, Edit-Klasse B, Forts.)

| Feld             | Wert                                                                                                            |
|------------------|-----------------------------------------------------------------------------------------------------------------|
| Status           | Draft (Design genehmigt 2026-06-10)                                                                             |
| Datum            | 2026-06-10                                                                                                      |
| Vorhaben         | Zwei weitere Refactoring-Ops auf der v2b-Engine: `move`, `safe_delete`                                          |
| Scope            | `move_preview`/`move_apply` + `safe_delete_preview`/`safe_delete_apply` (Two-Phase, v2b-Engine wiederverwendet) |
| Basis-Spec       | `docs/lean-md/specs/2026-06-09-leanctx-jetbrains-v2b-refactoring-rename-design.md` (v2b, §11 v2c-Pointer)       |
| Branch           | `feat-jetbrains-plugin` (Fortführung, Muster v1-§12.3 — ein Commit pro Phase)                                   |
| Nächster Schritt | `superpowers:writing-plans` (Implementierungsplan)                                                              |

---

## 1. Context — Warum v2c jetzt, was bewusst NICHT

v2b hat die **Multi-File-Refactoring-Engine etabliert** und exemplarisch `rename` implementiert
(Two-Phase `rename_preview`/`rename_apply`, `plan_hash`-Guard, Rust-zentrales Konflikt-Gate,
mehrstufiges `select_backend`/`BACKEND_REQUIRED`, Smart-Mode-`INDEXING`, `UNSUPPORTED_LANGUAGE`,
Multi-File-Cache-Kohärenz). Damit trägt die **erste** Op das gesamte Architektur-Risiko; die
übrigen Refactorings sind inkrementell (v2b-§1, §11).

v2c zieht die **zwei Engine-nativen** Folge-Ops nach:

- **`move`** — Symbol/Datei an einen neuen Ort verschieben (alle Referenzen mitgezogen).
- **`safe_delete`** — Symbol löschen, **sofern** keine blockierenden Referenzen verbleiben.

**Bewusst NICHT in v2c** (eigener Folge-Spec **v2d**, §10): `inline` und `reformat`. Begründung
(Split-Entscheidung 2026-06-10, Option A): `move`/`safe_delete` sind **direkte** v2b-Inkremente
— beide „Symbol → Multi-File-Edit", die `plan_hash`, Konflikt-Gate, `BACKEND_REQUIRED` und
Multi-File-Cache **1:1** teilen. `inline` bricht das Modell **semantisch** (Substitution statt
Umbenennung — Parameter-Binding, Präzedenz, Seiteneffekte, sprach-spezifischer
`InlineHandler`-EP), `reformat` **architektonisch** (Single-Phase, kein `plan_hash`, näher an
v2a). Die zwei „Mold-Breaker" gehören nicht ins selbe Paket wie die zwei homogenen
Engine-Inkremente — getrennte Specs halten v2c klein, homogen und reviewbar.

---

## 2. Getroffene Entscheidungen (User, 2026-06-10)

| # | Frage                | Entscheidung                                                                                                                                                            |
|---|----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1 | v2c-Gesamtscope      | **`move` + `safe_delete`.** `inline` + `reformat` als eigenes **v2d** (§10, Split-Option A). Risiko nach Op-Natur getrennt: Engine-Inkremente ≠ Mold-Breaker.           |
| 2 | Apply-Modell         | **Two-Phase** (wie v2b): `*_preview` liefert Plan, `*_apply` schreibt. Geerbt, nicht neu entworfen.                                                                     |
| 3 | `move`-Ziel-Form     | **Zwei getrennte Ziel-Felder** `target_path` XOR `target_parent` (Serena-Spiegel `targetRelativePath`/`targetParentNamePath`). Aufrufer wählt über das gesetzte Feld.   |
| 4 | `move`-Jail          | **3-stufig** (NEU): Quelle, **aufgelöstes Ziel vor Backend-Call**, zurückgemeldete `changed_paths`. Ziel ist aufrufer-kontrolliert → muss vor dem Apply gejailt werden. |
| 5 | `safe_delete`-Policy | **Preview meldet blockierende Usages, Apply blockt** (außer `force`=`deleteEvenIfUsed`). Optional `propagate`. Konflikt-Gate = v2b-`CONFLICT`.                          |
| 6 | Action-Form          | **Vier explizite Actions** in `ctx_refactor` (kein neues Tool — wie v2a/v2b).                                                                                           |
| 7 | Plan→Apply-Übergabe  | **Stateless `plan_hash`** (BLAKE3, Rust-zentral) — unverändert aus v2b geerbt.                                                                                          |
| 8 | Sprach-Scope         | **Generisch** über die jeweiligen IntelliJ-Processoren. **Akzeptanz-Gate: Kotlin** (Primär); Java optionaler Sekundär-Check (wie v2a/v2b).                              |

---

## 3. Schlüssel-Befund: Serenas tatsächliche API-Oberfläche (dekompiliert 2026-06-10)

Quelle: `tmp/serena-jetbrains-plugin/lib/serena-jetbrains-plugin-2023.2.16.jar` (kompiliert,
`javap`-Signaturen — **Architektur-Referenz, nicht Code-Quelle**, wie v2b-§6). Das resolved
zwei Design-Forks definitiv:

```
de.oraios.serena.service.request.MoveRequest
  String  namePath              // Quell-Symbol
  String  relativePath          // Quell-Datei
  String  targetRelativePath    // Ziel: Datei/Verzeichnis   → FileMoveProcessor
  String  targetParentNamePath  // Ziel: Eltern-Symbol (FQN) → SymbolMoveProcessor

de.oraios.serena.symbol.move.MoveProcessor.fromRequest(req, ctx)
  // dispatcht NACH gesetztem Ziel-Feld (kein Symbol-Inspect):
  //   targetParentNamePath ≠ null → SymbolMoveProcessor  (Member in Eltern-Symbol)
  //   targetRelativePath   ≠ null → FileMoveProcessor    (Datei/Klasse in Verzeichnis)

de.oraios.serena.service.request.SafeDeleteRequest
  String  namePath
  String  relativePath
  boolean deleteEvenIfUsed      // = unser `force`
  boolean propagate             // mitlöschende Abhängigkeiten
```

**Befund 1 (`move`-Ziel):** Serena macht **kein** Symbol-Inspect-Auto-Detect, sondern lässt den
**Aufrufer** über das gesetzte Ziel-Feld dispatchen (`targetParentNamePath` → Member-Move,
`targetRelativePath` → Datei/Verzeichnis-Move). v2c spiegelt das: `target_parent` XOR
`target_path`, genau eines gesetzt.

**Befund 2 (`safe_delete`):** `deleteEvenIfUsed` ist exakt unser `force`-Konzept; `propagate`
ist ein eigenständiges Flag (mitlöschen unreferenzierter Abhängigkeiten). Beide übernommen.

---

## 4. Architektur — Was geerbt wird, was neu ist

Die v2b-Engine bleibt **unverändert**; v2c ist additiv. Geerbt (NICHT neu entworfen, nur
referenziert):

| Mechanismus                            | Quelle   | gilt in v2c                                                |
|----------------------------------------|----------|------------------------------------------------------------|
| Two-Phase `*_preview`/`*_apply`        | v2b-§4   | identisch für `move`/`safe_delete`                         |
| `plan_hash` (BLAKE3, Rust-zentral)     | v2b-§5.2 | identisch (TOCTOU-Guard über betroffene Stellen)           |
| Konflikt-Gate (`conflicts≠∅∧¬force`)   | v2b-§5.2 | identisch                                                  |
| mehrstufiges `select_backend`          | v2b-§3.1 | identisch — kein A-Fallback, headless → `BACKEND_REQUIRED` |
| Smart-Mode-Pflicht / `INDEXING`        | v2b-§6   | identisch (semantische Usage-Suche braucht Index)          |
| Sprach-Fallback `UNSUPPORTED_LANGUAGE` | v2b-§6   | identisch (Processor-EP nullable-Lookup)                   |
| Multi-File-Cache-Kohärenz              | v2b-§9   | identisch (evict+rewarm je `changed_path`)                 |
| Rust/Plugin-Rollenteilung              | v2b-§3   | identisch (Rust: Auflösung/Jail/Hash/Gate; Plugin: PSI)    |

**Neu in v2c** (genau zwei Dinge):

1. **`move`s 3. Jail-Stufe** — das **Ziel** ist aufrufer-kontrolliert (§5.3). `rename`/`safe_delete`
   schreiben nur an Pfade, die aus dem **bestehenden** Symbol abgeleitet sind (vertrauenswürdig,
   bereits im Baum). `move` schreibt an einen **neuen, gelieferten** Ort → dieser muss
   `resolve_tool_path`/PathJail durchlaufen, **bevor** das Plugin erfährt, wohin verschoben wird.
2. **`safe_delete`s Blocking-Usages-Semantik** — das „safe" = Preview liefert die
   verbleibenden Referenzen; Apply blockt bei `¬force` (§5.4). Mechanisch dasselbe Konflikt-Gate
   wie v2b, nur ist die Konflikt-Quelle „Referenz existiert noch" statt „Namenskollision".

### 4.1 Two-Phase-Fluss (geerbt, hier für `move`/`safe_delete` instanziiert)

```
PHASE 1 — *_preview(name_path | path+line, <op-spezifische Ziel-/Flag-Parameter>)
   ├─ Rust: resolve_name_path → (src_path, src_range)   [reuse v2a, backend-unabh.]
   │        + PathJail(src)                               ── vor jedem Backend-Call
   │        + [move only] Ziel auflösen + PathJail(Ziel)  ── §5.3, NEU
   ├─ select_backend: Backing B?  ── nein ─→ Err BACKEND_REQUIRED
   ├─ Plugin POST /<op>Preview:
   │     <MoveProcessor|SafeDeleteProcessor>.findUsages()/preprocessUsages()
   │     ← { usages:[…], conflicts:[…] }   (safe_delete: conflicts = blockierende Refs)
   └─ Rust: plan_hash = BLAKE3(canonical(usages))
         ← Agent: { target/op-info, affected_files, usage_count, conflicts, plan_hash, diff_preview }

PHASE 2 — *_apply(… , plan_hash[, force=false])
   ├─ Rust: resolve_name_path (erneut) + PathJail(src) [+ PathJail(Ziel) für move]
   ├─ select_backend: Backing B?  ── nein ─→ Err BACKEND_REQUIRED
   ├─ Plugin POST /<op>Preview (erneut) → usages + conflicts
   ├─ Rust-Gates (Reihenfolge): (a) plan_hash neu bilden+vergleichen ≠ → CONFLICT (TOCTOU)
   │                            (b) conflicts≠∅ ∧ ¬force → CONFLICT
   ├─ Plugin POST /<op>Apply:
   │     WriteCommandAction { <Move|SafeDelete>Processor(…, force=…).run() }   // EIN Undo
   │     je geänderter Datei: commitDocument + saveDocument
   │     ← { applied:true, changed_paths:[…] }
   └─ Rust: PathJail je changed_path, evict+rewarm (mtime-Auto-Validierung), Multi-File-Diff
         ← Agent: { applied:true, changed_paths, diff }
```

---

## 5. Rust-Seite

### 5.1 `ctx_refactor` — vier neue Actions

- **Neue Actions** (kein neues Tool — vermeidet Nachziehen in `tool_profiles.rs`/
  `dynamic_tools.rs`/`workflow/types.rs`, exakt wie v2a/v2b): `move_preview`, `move_apply`,
  `safe_delete_preview`, `safe_delete_apply`. Match-Block + Hilfetext erweitern; Schema über die
  **eine** Tool-Registry `tool_defs::tool_def(...)` (Drift-Regression-Test deckt es ab).
- **Parameter:**
    - **`move_preview`/`move_apply`:** Quelle `name_path` (primär) **oder** `path`+`line`(+`character`);
      Ziel **genau eines** von `target_path` **oder** `target_parent` (§5.3); `apply` zusätzlich
      `plan_hash` (required) + `force` (optional, default `false`).
    - **`safe_delete_preview`/`safe_delete_apply`:** Quelle wie oben; `apply` zusätzlich `plan_hash`
      (required) + `force` (optional, default `false`, = `deleteEvenIfUsed`) + `propagate`
      (optional, default `false`).
- **Auflösungsschritt (alle vier, vor dem Backend-Dispatch):** `resolve_name_path` (reuse v2a)
  → `(src_path, src_range)`; >1 → `AMBIGUOUS_SYMBOL`; 0 → `NO_SYMBOL`. Danach PathJail (§5.3).

### 5.2 `plan_hash` + Konflikt-Gate — unverändert aus v2b geerbt

Bildung (`*_preview`): `plan_hash = hash_hex(canonical(usages))` über nach `(path,range)`
sortierte Usage-Stellen + deren Ist-Inhalt (`core::hasher::hash_hex`, BLAKE3). Prüfung
(`*_apply`): Phase 2 holt Usages erneut, bildet Hash neu, Mismatch ⇒ `CONFLICT` (TOCTOU).
Zusätzlich `conflicts≠∅ ∧ ¬force ⇒ CONFLICT`. **`plan_hash` erscheint NICHT auf der Wire**
(v2a-§7-Regel verbindlich; das Plugin hasht nicht).

### 5.3 `move`-Ziel-Auflösung & 3-stufiges PathJail (NEU)

Genau **eines** von `target_path`/`target_parent` muss gesetzt sein (sonst `INVALID_TARGET`
**vor** jedem Backend-Call). Auflösung + Jail:

```
(1) Quell-Symbol:  resolve_name_path → src_path → resolve_tool_path → ∈ project_root   [wie v2b]
(2) ZIEL (genau eines):                                                                [NEU]
    target_path   → resolve_tool_path(target_path)  → Ziel-Dir/Datei ∈ project_root
    target_parent → resolve_name_path(target_parent) → dessen Datei  ∈ project_root
    keines / beides gesetzt → INVALID_TARGET (vor Backend-Call)
    Ziel unauflösbar (target_parent → 0/Mehrdeutig) → NO_SYMBOL / AMBIGUOUS_SYMBOL
(3) POST-APPLY: jeder zurückgemeldete changed_path (inkl. NEU erstellter Ziel-Datei)
    → resolve_tool_path → ∈ project_root, BEVOR evict/rewarm läuft                     [wie v2b]
```

**Begründung (Kernsatz):** Bei `rename`/`safe_delete` kommen alle Schreib-Pfade aus dem Index
(vertrauenswürdig); bei `move` kommt das Ziel vom **Aufrufer** (nicht vertrauenswürdig) → es
muss `resolve_tool_path`/PathJail durchlaufen, **bevor** das Plugin erfährt, wohin verschoben
werden soll. Ein `target_path` wie `../../etc/skel/` würde sonst einen Jail-Escape erlauben.

### 5.4 `safe_delete`-Semantik

- `safe_delete_preview` liefert `usages` = die **verbleibenden Referenzen** (blockierende
  Usages); `conflicts` enthält sie als Block-Gründe.
- `safe_delete_apply`: `conflicts≠∅ ∧ ¬force ⇒ CONFLICT` (das v2b-Gate; Konflikt-Quelle =
  „Referenz existiert noch"). `force=true` reicht `deleteEvenIfUsed=true` an den
  `SafeDeleteProcessor` durch. `propagate` reicht das gleichnamige Serena-Flag durch
  (mitlöschende, dann unreferenzierte Abhängigkeiten).
- Kein neues Ziel → **Zwei**-Stufen-Jail aus v2b unverändert (Quelle + `changed_paths`).

### 5.5 `LspBackend`-Trait — vier neue Methoden (Default = `Err`)

Additiv zu v1/v2a/v2b. Default = `Err BackendRequired` (kein verlustfreier Headless-Pfad, v2b-§3):

```rust
// rust/src/lsp/backend.rs (Erweiterung)
fn move_preview(&mut self, req: MoveQuery) -> Result<RenamePlan, BackendError> { Err(BackendError::BackendRequired) }
fn move_apply(&mut self, req: MoveApply) -> Result<RenameResult, BackendError> { Err(BackendError::BackendRequired) }
fn safe_delete_preview(&mut self, req: SafeDeleteQuery) -> Result<RenamePlan, BackendError> { Err(BackendError::BackendRequired) }
fn safe_delete_apply(&mut self, req: SafeDeleteApply) -> Result<RenameResult, BackendError> { Err(BackendError::BackendRequired) }
```

- **`RenamePlan`/`RenameResult` werden wiederverwendet** (`{usages, conflicts}` bzw.
  `{applied, changed_paths}` aus v2b) — die Plan-/Ergebnisform ist op-unabhängig.
- **`MoveQuery`**: `{ abs_path, src_range, target: MoveTarget }` mit
  `enum MoveTarget { Path(String), Parent{ abs_path, range } }` (Ziel bereits aufgelöst — der
  Trait sieht nie einen `name_path`, exakt wie v2a/v2b).
- **`MoveApply`**: `MoveQuery` + `force`.
- **`SafeDeleteQuery`**: `{ abs_path, src_range }`. **`SafeDeleteApply`**: `+ force, propagate`.
- `LspClient` (Backing A) **erbt den `Err`-Default** → headless ohne IDE liefert sauberes
  `BACKEND_REQUIRED`.

### 5.6 Änderungsstellen (Rust)

| Datei                                       | Änderung                                                                                                                           |
|---------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------|
| `rust/src/tools/ctx_refactor.rs`            | +4 Actions, `move`-Ziel-Auflösung + 3-Stufen-Jail (§5.3), `safe_delete`-Gate (§5.4); reuse `plan_hash`/Konflikt-Gate/Cache aus v2b |
| `rust/src/tools/registered/ctx_refactor.rs` | Schema-Erweiterung über `tool_def(...)` (`target_path`, `target_parent`, `propagate`; `plan_hash`/`force` schon aus v2b)           |
| `rust/src/lsp/backend.rs`                   | +4 Trait-Methoden (Default `Err`) + `MoveQuery`/`MoveTarget`/`MoveApply`/`SafeDeleteQuery`/`SafeDeleteApply`-Typen                 |
| `rust/src/lsp/jetbrains_backend.rs`         | HTTP-Override der 4 Methoden (`/movePreview`, `/moveApply`, `/safeDeletePreview`, `/safeDeleteApply`)                              |
| `rust/src/lsp/client.rs`                    | erbt `Err`-Default (keine Änderung außer ggf. Trait-Re-Export)                                                                     |

---

## 6. Plugin-Seite (Kotlin) — additiv

Integriert additiv in `com.leanctx.plugin` (koexistiert mit v1/v2a/v2b, ersetzt nichts).

- **`psi/SymbolRefactorer.kt`** (Erweiterung der v2b-Naht): zwei neue Operationen.
  (Architektur-Referenz — **nicht** Code-Quelle: Serenas `MoveProcessor`/`FileMoveProcessor`/
  `SymbolMoveProcessor`/`SafeDeleteHandler`, dekompiliert §3.)
    - **`move` Preview:** je Ziel-Feld den passenden IntelliJ-Processor —
      `target_path` → `MoveFilesOrDirectoriesProcessor`/`MoveClassesOrPackagesProcessor`;
      `target_parent` → Member-Move-Processor (`MoveMembersProcessor` o. Sprach-Äquiv.).
      `findUsages()` + `preprocessUsages()` → `{usages, conflicts}`. **Kein** Write.
    - **`move` Apply:** `WriteCommandAction.runWriteCommandAction(project) { processor.run() }`
      → Multi-File-Transaktion als **ein** Undo-Eintrag; `force` steuert Konflikt-Durchgriff (bei
      `force=false`+Konflikt bricht bereits das Rust-Gate ab, §5.2).
    - **`safe_delete` Preview:** `SafeDeleteProcessor.createDelete(...)` → `findUsages()` liefert
      die **verbleibenden** Referenzen als `usages`/`conflicts`. **Kein** Write.
    - **`safe_delete` Apply:** `WriteCommandAction { SafeDeleteProcessor(…, deleteEvenIfUsed=force,
      propagate=propagate).run() }` → ein Undo.
    - Nach dem Lauf je betroffener Datei: `PsiDocumentManager.commitDocument` +
      `FileDocumentManager.saveDocument` (schreibt auf Platte, damit lean-ctx es sieht).
    - **Kein Auto-Reformat** (entkoppelt, wie v2a-§6.1 / v2b — Formatierung kommt in v2d).
- **`endpoint/RefactorHandlers.kt`** (Erweiterung): `movePreview`/`moveApply`/`safeDeletePreview`/
  `safeDeleteApply`, registriert im `RequestRouter` (Token-Check wie v1).
- **Threading + Index-Schutz** (geerbt v2b-§6): `findUsages`/`preprocessUsages` unter
  `DumbService.runReadActionInSmartMode` (off-EDT). Smart-Mode Pflicht → sonst `INDEXING`
  (unvollständige Usages = kaputter move/delete). Apply über `WriteCommandAction` (EDT).
- **Sprach-Fallback** (geerbt v2b-§6): Sprache ohne passenden Move-/SafeDelete-Processor →
  `UNSUPPORTED_LANGUAGE` (nullable EP-Lookup, keine harten Imports, kein Crash).
- **gson `compileOnly`** (v1-§5.4): neue DTOs/Handler nutzen die IDE-gebündelte gson.
- **Kanonische Refactoring-Grenze** (geerbt v2b-§6): Plugin = alleinige Usage-Quelle; Rust
  spiegelt keine zweite, unabhängige Range-Berechnung — `plan_hash` hasht nur den Ist-Inhalt der
  gemeldeten Stellen.

---

## 7. Wire-Protokoll (DTO) — additiv zu v1/v2a/v2b

- **0-basiert** (Zeile + Spalte), Pfade relativ zu `project_root`. Token-Header
  `X-LeanCtx-Token` wie v1.
- **Neue Endpoints (POST):**
    - `POST /movePreview` — Request
      `{ path, range:{start,end}, target:{ kind:"path"|"parent", path?, range? } }`
      → Response `{ usages:[{path,range,context?}], conflicts:[{path,range,message}] }`.
    - `POST /moveApply` — Request `{ path, range, target:{…}, force }`
      → `{ applied:true, changed_paths:[…] }`.
    - `POST /safeDeletePreview` — Request `{ path, range }`
      → `{ usages:[…], conflicts:[…] }` (usages = verbleibende Refs).
    - `POST /safeDeleteApply` — Request `{ path, range, force, propagate }`
      → `{ applied:true, changed_paths:[…] }`.
- **`plan_hash` erscheint NICHT auf der Wire** (v2a-§7-Regel, reine Rust-Logik).
- **Fehler** (additiv zum v1/v2a/v2b-Set):
    - **Reuse v2b:** `BACKEND_REQUIRED`, `CONFLICT` (plan_hash-Mismatch **oder** geblocktes Gate),
      `AMBIGUOUS_SYMBOL`, `NO_SYMBOL`, `INDEXING`, `UNSUPPORTED_LANGUAGE`.
    - **`+INVALID_TARGET`** (NEU, nur `move`) — keines/beides von `target_path`/`target_parent`
      gesetzt, **oder** aufgelöstes Ziel außerhalb `project_root` (Jail-Verletzung, §5.3).
      Rust-seitig vor Backend-Call.
    - HTTP 200 für fachliche Negativfälle, 401 nur Token, 500 nur echte Exceptions.

---

## 8. Op-Semantik (Serena-Parität)

| Action                | Eingabe                                          | Wirkung                                                                          |
|-----------------------|--------------------------------------------------|----------------------------------------------------------------------------------|
| `move_preview`        | Quelle + (`target_path` XOR `target_parent`)     | **kein** Write; betroffene Dateien, Usage-Count, Konflikte, `plan_hash`, Diff    |
| `move_apply`          | Quelle + Ziel + `plan_hash` [+ `force`]          | verschiebt Symbol/Datei + zieht **alle** Referenzen mit (Multi-File, ein Undo)   |
| `safe_delete_preview` | Quelle                                           | **kein** Write; liefert **blockierende** (verbleibende) Referenzen + `plan_hash` |
| `safe_delete_apply`   | Quelle + `plan_hash` [+ `force`] [+ `propagate`] | löscht Symbol (blockt bei Refs außer `force`; `propagate` löscht Abhängige mit)  |

- Adressierung des **Quell**-Symbols identisch zu v2a/v2b (name_path primär, Position-Fallback).
- Drop-in-kompatibel zu Serenas `jet_brains_move` / `jet_brains_safe_delete` (gleiche Semantik),
  plus expliziter Preview-Schritt, den Serena nicht hat.

---

## 9. Verifikation (End-to-End)

- **Rust-Einheit (`cargo nextest run`, nie `cargo test`):**
    - `plan_hash`: deterministisch über sortierte Usages+Inhalt; Match → Apply; Mismatch → `CONFLICT`.
    - `move`-Ziel-Validierung: keines/beides Ziel-Feld → `INVALID_TARGET`; Ziel außerhalb
      `project_root` → `INVALID_TARGET` **vor** Backend-Call; `target_parent` 0/mehrdeutig →
      `NO_SYMBOL`/`AMBIGUOUS_SYMBOL`.
    - **3-Stufen-Jail (`move`):** Quelle **und** Ziel **und** zurückgemeldete `changed_paths`
      außerhalb `project_root` → Fehler vor Apply/Cache-Mutation.
    - `safe_delete`-Gate: `conflicts≠∅ ∧ ¬force → CONFLICT`; `force=true`/`propagate` durchgereicht.
    - `select_backend` (mehrstufig, v1-§8): stale Port / toter pid / Health-Timeout →
      `BACKEND_REQUIRED`, kein Apply, **kein** A-Fallback.
    - 0/1-Basierungs-Naht (Tool-Eingabe 1-basiert ↔ Wire 0-basiert ↔ Offset).
- **Plugin (Kotlin-Unit + manuelles `runIde`-Gate, wie v1/v2b):**
    - **Akzeptanz-Gate Kotlin** (Primär):
        - `move`: (a) Top-Level-Klasse via `target_path` in anderes Package verschieben → Datei
          umgezogen, alle Imports/Refs angepasst; (b) Member via `target_parent` in andere Klasse
          verschieben → Deklaration + Aufrufstellen korrekt. Ergebnisdateien prüfen.
        - `safe_delete`: ungenutztes Symbol löschen → weg; genutztes Symbol ohne `force` →
          `CONFLICT` mit blockierenden Refs; mit `force` → gelöscht.
    - `preprocessUsages`/`findUsages` meldet konstruierte Konflikte/Refs korrekt im Preview.
    - `*.run()` erzeugt **einen** Undo-Eintrag; `saveDocument` persistiert je Datei.
    - **Index-Schutz:** Op während Indizierung (Dumb-Mode) → `INDEXING`, **kein** Teil-Edit.
    - **Sprach-Fallback:** Op in Sprache ohne Processor → `UNSUPPORTED_LANGUAGE`, kein Crash.
    - Java optionaler Sekundär-Check (nicht akzeptanzkritisch).
- **Fallback:** ohne laufende IDE → `BACKEND_REQUIRED` in beiden Phasen, kein Apply.

### 9.1 Live-Gate — eigenes runIde-Runbook (analog `runide-rename-gate.md`)

v2c bekommt ein **manuelles Live-Verifikations-Gate** nach dem Muster des v2b-Rename-Gates
(`docs/lean-md/runbooks/runide-rename-gate.md` + Harness-Spec
`2026-06-09-leanctx-jetbrains-runide-rename-gate-harness-design.md`). Es verifiziert den vollen
v2c-Two-Phase-Stack **live** — Rust-Gate (`plan_hash`/TOCTOU, Konflikt-Gate, 3-Stufen-PathJail,
Cache-Evict) **und** Plugin (`MoveProcessor`/`SafeDeleteProcessor`-Naht, Multi-File-Transaktion,
ein Undo) — gegen ein sauberes **Kotlin-Gradle-Fixture** mit korrektem Find-Usages-Scope.

**Liefergegenstand (in v2c zu erstellen):** `docs/lean-md/runbooks/runide-move-safedelete-gate.md`
mit identischer Struktur (Voraussetzungen → Fixture-Setup-Script → `./gradlew runIde --args="$FIX"`
→ Gate-Checks via `lean-ctx call ctx_refactor --project-root "$FIX" --json '<args>'` → Teardown).
Das Fixture erweitert das Rename-Fixture um Move-/Delete-taugliche Symbole (z.B. eine
verschiebbare Top-Level-Klasse + ein Ziel-Package, ein Member + eine Ziel-Klasse, ein
ungenutztes und ein genutztes Symbol).

**Voraussetzung — frisches Binary (Daemon-Stopp ist Pflicht, NICHT optional):** Die neuen
Actions (`move_*`/`safe_delete_*`) existieren erst nach Neubau. Ein **laufender** lean-ctx-Daemon
hält den **alten** Action-Satz im Speicher — ein `lean-ctx call ctx_refactor '{"action":"move_preview",…}'`
gegen den laufenden Daemon liefert sonst `Unknown action` (kein Build-Effekt, weil der Prozess den
alten Code weiterbedient). Reihenfolge **vor** dem Gate:
1. `lean-ctx serve --stop` — Daemon stoppen (gibt das Binary frei + entlädt den alten Action-Satz;
   `cli/dispatch/network.rs:555,703`).
2. `cargo build` (cwd=`rust`) [+ ggf. Binary neu installieren] — baut die `move_*`/`safe_delete_*`-Actions ein.
3. `lean-ctx serve --daemon` (neu starten) **oder** den ersten `lean-ctx call` den Daemon auto-starten lassen.

> **Achtung MCP-Session:** In einer aktiven Agent-/MCP-Session ist genau dieser Daemon zugleich der
> `ctx_*`-Server — `serve --stop` unterbricht die eigenen `ctx_*`-Tools bis zum Neustart. Das Gate
> daher als **separater** Schritt fahren (nicht mitten in einer ctx_*-getriebenen Aufgabe).

**Gate-Checks (Soll-Ergebnisse):**

| # | Fall | Aufruf (`--json`, Auszug) | Soll-Ergebnis |
| - | ---- | ------------------------- | ------------- |
| 1 | move Preview (`target_path`) | `{"action":"move_preview","name_path":"Widget","target_path":"app/moved"}` | usages cross-file, `files≥1`, `plan_hash` gesetzt |
| 2 | move Apply + Undo | `{"action":"move_apply","name_path":"Widget","target_path":"app/moved","plan_hash":"<#1>"}` | Datei umgezogen, Refs/Imports angepasst; **ein** Undo-Eintrag (Strg+Z revertet komplett) |
| 3 | move Member (`target_parent`) | `{"action":"move_preview","name_path":"Helper/calc","target_parent":"OtherClass"}` | Member-Move-Plan; danach Apply → Deklaration + Aufrufstellen korrekt |
| 4 | INVALID_TARGET | `{"action":"move_preview","name_path":"Widget"}` (kein Ziel) **und** beide Ziele gesetzt **und** `target_path:"../escape"` | je `INVALID_TARGET`, **vor** Backend-Call, kein Apply |
| 5 | move TOCTOU | eine usage-Stelle zwischen #1 und Apply ändern, dann Apply mit altem `plan_hash` | `CONFLICT` |
| 6 | safe_delete Preview (ungenutzt) | `{"action":"safe_delete_preview","name_path":"Unused"}` | leere/keine blockierenden usages, `plan_hash` gesetzt |
| 7 | safe_delete Apply ohne force (genutzt) | `{"action":"safe_delete_apply","name_path":"Widget","plan_hash":"<preview>"}` | `CONFLICT` mit blockierenden Refs, **kein** Löschen |
| 8 | safe_delete Apply mit force | wie #7 + `"force":true` | gelöscht; Refs bleiben dangling (bewusst, `deleteEvenIfUsed`) |
| 9 | INDEXING | Projekt neu öffnen, sofort `move_preview`/`safe_delete_preview` während Indizierung | `INDEXING`, kein Teil-Edit |
| 10 | UNSUPPORTED_LANGUAGE | `{"action":"move_preview","path":"notes.txt","line":1,"target_path":"x"}` (`path`+`line`-Fallback nutzen, nicht `name_path`) | `UNSUPPORTED_LANGUAGE`, kein Crash |
| 11 | BACKEND_REQUIRED | IDE schließen, dann preview **und** apply | `BACKEND_REQUIRED` in beiden Phasen |

> Wie beim Rename-Gate: für force-/TOCTOU-Fälle ggf. zuerst ein eigenes `*_preview` ausführen,
> um den aktuellen `plan_hash` zu erhalten. Deterministische Teile (INDEXING-Dumb-Mode,
> PathJail) zusätzlich als Rust-Unit-Test absichern (§9); das Live-Gate deckt die
> nicht-headless-reproduzierbare Plugin-Naht ab.

---

## 10. v2d-Forward-Pointer (eigener Folge-Spec, NICHT hier)

Nach v2c folgt **v2d** mit den zwei „Mold-Breaker"-Ops — sie brechen das v2b-Modell jeweils
anders und bekommen deshalb einen fokussierten eigenen Spec:

- **`inline`** (`inline_preview`/`inline_apply`, Serena `jet_brains_inline_symbol`/
  `InlineSymbolRequest`): **semantischer** Bruch — **Substitution** statt Umbenennung (Body/
  Initializer an die Aufrufstellen, mit Parameter-Binding, Präzedenz-Klammerung,
  Seiteneffekt-Reihenfolge). Flag `keep_definition` (= Serenas `keepDefinition`: inlinen +
  Deklaration behalten statt entfernen). Fehlerklassen, die `rename`/`move` nie haben (rekursiv,
  mehrere `return`, Override/Polymorphie) → Preview meldet sie als Konflikte; oft kein „force"
  (IntelliJ verweigert → `CONFLICT`/`UNSUPPORTED`). Sprach-spezifischer `InlineHandler`-EP.
  Erbt sonst die v2b-Engine (Two-Phase, `plan_hash`, Multi-File-Cache).
  **v2d-Review-Punkt (festgehalten 2026-06-10):** vor Implementierung **explizit prüfen, welche
  v2b-Funktionen `inline` wiederverwenden kann** — Erst-Einschätzung: das **Scaffolding** ist
  wiederverwendbar (✅ `findUsages`/Usage-Sammlung = die Substitutions-Stellen, ✅ `plan_hash`/
  TOCTOU-Guard über die betroffenen Stellen, ✅ Two-Phase `*_preview`/`*_apply`, ✅
  Multi-File-Cache/`BACKEND_REQUIRED`/`INDEXING`); **neu** sind nur (❌) der **Apply-Transform**
  (Substitution statt Text-Swap, kein `processor.run()`-Reuse) und (❌) die **Konflikt-Erkennung**
  (inline-spezifisch: rekursiv/Override/Mehrfach-`return` statt Namenskollision). Fazit:
  Wiederverwendung bringt **etwas** (das gesamte Gerüst), nur der Transform-Kern ist genuin neu —
  in v2d zu verifizieren, nicht anzunehmen.
- **`reformat`** (Serena `FormatCodeRequest` + `FormatSymbolRequest`, JetBrains-MCP
  `reformat_file`): **architektonischer** Bruch — **Single-Phase** (kein `plan_hash`, keine
  Usage-Suche, keine Konflikte), näher an v2a (lokaler Single-File-Apply). **Eine** Action
  `reformat` deckt beide Serena-Requests + `reformat_file` ab via Adressierungs-Dualität:
  `name_path` → Symbol (= `FormatSymbol`), `path` → ganze Datei, `path`+`line`-Range → Region
  (= `FormatCode`). Flag `optimize_imports` (datei-weit). Diff via **`ctx_delta`** (nur geänderte
  Zeilen — **kein** `ctx_diff`, das existiert nicht; Projekt-Rule). Headless trotzdem
  `BACKEND_REQUIRED` (braucht IntelliJ-`CodeStyleManager`). **Diskrepanz-Klärung mitnehmen:**
  v1-§13.3 listet `reformat_file → ctx_refactor action=format` als „v1 (erledigt)", im Code
  existiert **keine** `format`-Action (`ctx_refactor.rs` Match-Block) — `reformat` ist real
  offen; die v1-Tabelle ist hier veraltet und in v2d zu korrigieren.

**Geerbte Gates** (in v2d nur referenzieren, nicht neu entwerfen, soweit zutreffend):
Smart-Mode/`INDEXING`, Sprach-Fallback `UNSUPPORTED_LANGUAGE`, mehrstufiges `select_backend`/
`BACKEND_REQUIRED`.

**Out of v2c/v2d (bleibt out of scope):**

- `serena.jet_brains_debug` (kein Code-Intelligence-Äquivalent).
- `jetbrains.generate_psi_tree` — PSI-Dump/Debug; tree-sitter (`ctx_symbol`/`ctx_outline`/
  `symbols_overview`) deckt den Bedarf **headless** ab → kein Ablöse-Treiber (v1-§13.1
  „optional"). Bei echtem PSI-Bedarf später als reiner Debug-Read nachziehbar.
- DB-/Run-/SQL-/Terminal-/Editor-UI-Tools des JetBrains-MCP (v1-§1, v2a-§11).

**Erst nach v2d** ist Serena auch als Edit-Engine vollständig entbehrlich (v1-§13.4) — danach
sind Serena-MCP **und** JetBrains-MCP (Code-Intelligence-Teil) aus der Agent-Konfiguration
entfernbar; lean-ctx ist die alleinige Schnittstelle.

---

## 11. Branch- & Commit-Strategie

- Fortführung auf `feat-jetbrains-plugin` (Muster v1-§12.3): **ein Commit pro Phase** nach
  erfülltem Phasen-Gate, kein Squash während der Entwicklung. Direkt auf dem Branch, **kein
  worktree** (Projekt-Rule).
- Finaler Merge nach `main` via Squash-Merge-PR (am Schluss).
- **Schema-Drift-Gate:** `ctx_refactor`-Schema-Änderung → `docs/reference/generated/mcp-tools.md`
  via `cargo run --example gen_docs --features dev-tools` (cwd=rust) regenerieren
  (Drift-Test `generated_reference…`). Zusätzlich `docs/reference/appendix-mcp-tools.md`
  (human tool map) um die vier Actions ergänzen.

---

## 12. Bewusst NICHT in v2c (YAGNI)

- **Kein** `inline`/`reformat` (eigenes v2d, §10).
- **Kein** Blast-Radius-Limit (Preview macht den Radius vorab sichtbar — redundant, wie v2b-§13).
- **Kein** Server-State / `plan_id` (stateless `plan_hash`, geerbt v2b).
- **Kein** Headless-move/-delete (kein verlustfreier Default, geerbt v2b-§3).
- **Kein** plugin-seitiges Hashing (Integritäts-Guards in Rust, v2a-§7).
- **Kein** Auto-Reformat im Apply (entkoppelt, v2a-§6.1).
- **Kein** Symbol-Inspect-Auto-Detect für `move` (Aufrufer wählt über gesetztes Ziel-Feld,
  Serena-Spiegel §3) — vermeidet fragiles Heuristik-Dispatch.
