# Design-Spec: lean-ctx JetBrains v2d — Inline & Reformat (Serena-Ablösung, Mold-Breaker, Abschluss)

| Feld             | Wert                                                                                                                  |
| ---------------- | ------------------------------------------------------------------------------------------------------------------- |
| Status           | Draft (Design genehmigt 2026-06-13)                                                                                  |
| Datum            | 2026-06-13                                                                                                            |
| Vorhaben         | Die zwei letzten Serena-Refactoring-Ops (die „Mold-Breaker"): `inline`, `reformat`                                   |
| Scope            | `inline_preview`/`inline_apply` (Two-Phase, v2b-Engine) + `reformat` (Single-Phase) + #576-Schema-diet für `ctx_refactor` |
| Basis-Spec       | `docs/lean-md/specs/2026-06-10-leanctx-jetbrains-v2c-move-safedelete-design.md` (v2c, §10 v2d-Pointer)               |
| Branch           | `feat-jetbrains-plugin` (Fortführung, Muster v1-§12.3 — ein Commit pro Phase, kein worktree)                         |
| Nächster Schritt | `superpowers:writing-plans` (Implementierungsplan)                                                                   |

---

## 1. Context — Warum v2d jetzt, was es abschließt

v2a–v2c haben die **Multi-File-Refactoring-Engine** etabliert und drei Ops darauf gebaut:
`rename` (v2b), `move` + `safe_delete` (v2c) — alle **Two-Phase** (`*_preview`/`*_apply`),
alle über **IntelliJ-Processoren** (Plugin ruft `findUsages()`+`run()`, kein eigener
Transform-Code), alle mit stateless `plan_hash`-Guard, Rust-zentralem Konflikt-Gate,
mehrstufigem `select_backend`/`BACKEND_REQUIRED`, Smart-Mode-`INDEXING` und
Multi-File-Cache-Kohärenz.

v2d zieht die **zwei bewusst aus v2c ausgelagerten** Ops nach — die „Mold-Breaker"
(v2c-§10), weil jede das v2b-Modell anders bricht:

- **`inline`** — Symbol/Methode/lokale Variable an die Aufrufstellen substituieren
  (Body/Initializer einsetzen). **Semantischer** Bruch (Substitution statt Umbenennung).
- **`reformat`** — Code formatieren (Symbol/Region/Datei). **Architektonischer** Bruch
  (Single-Phase, keine Usages, kein `plan_hash`, näher an v2a).

**Schlüssel-Befund der Designphase (2026-06-13):** v2c-§10 vermutete für `inline` einen
„genuin neuen Transform-Kern" (Substitution selbst gebaut). Diese Annahme wird **verworfen**:
IntelliJ hat **eigene Inline-Processoren** (`InlineMethodProcessor`, `InlineLocalHandler`,
sprach-`InlineHandler`-EP). `inline` bleibt damit im **v2c-Delegationsmuster** — Plugin
delegiert, IntelliJ macht Substitution/Parameter-Binding/Präzedenz/Seiteneffekt-Reihenfolge.
Der §10-„semantische Bruch" reduziert sich auf die **Konflikt-Natur** (inline-spezifische
Fälle: rekursiv, mehrere `return`, Override/Polymorphie), nicht auf den Transform.

**Nach v2d** ist Serena als Edit-Engine vollständig entbehrlich (v1-§13.4) — Serena-MCP **und**
der Code-Intelligence-Teil des JetBrains-MCP sind aus der Agent-Konfiguration entfernbar;
lean-ctx ist die alleinige Schnittstelle.

---

## 2. Getroffene Entscheidungen (User, 2026-06-13)

| # | Frage                       | Entscheidung                                                                                                                                                    |
| - | --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1 | v2d-Gesamtscope             | **`inline` + `reformat` zusammen** in einem v2d-Spec (wie v2c-§10). Beide sind die letzten zwei Ops; ein Paket schließt die Serena-Ablösung ab.                  |
| 2 | `inline`-Transform          | **IntelliJ-Inline-Processoren reuse** (v2c-Muster) — kein eigener Transform-Kern. Widerlegt die §10-Pessimismus-Annahme. Konsistent zu rename/move/safe_delete. |
| 3 | `reformat`-Phasenmodell     | **Single-Phase**, eine Action `reformat` (§10). Kein `plan_hash`, kein Preview (keine Usages zu hashen). Ehrliche Abbildung der Op-Natur, näher an v2a.          |
| 4 | `inline`-Konflikt/`force`   | **Kein `force` für `inline`.** Jeder von IntelliJ gemeldete Konflikt blockt hart: überstimmbar → `CONFLICT`, hart verweigert (rekursiv etc.) → `UNSUPPORTED`.     |
| 5 | Schema-diet/Lean-Surface    | **#576-Umstellung als Teil von v2d** (nicht Follow-up): Action-Enum-Array → pipe-delimited Description; Descriptions trimmen; Budget-Gate als Abnahme (§7).      |
| 6 | Action-Form                 | **Drei explizite Actions** in `ctx_refactor` (`inline_preview`/`inline_apply`/`reformat`) — kein neues Tool, wie v2a/v2b/v2c.                                     |
| 7 | Sprach-Scope                | **Generisch** über die IntelliJ-Inline-/Format-Maschinerie. **Akzeptanz-Gate: Kotlin** (Primär); Java optionaler Sekundär-Check (wie v2a–c).                     |

---

## 3. Schlüssel-Befund: IntelliJ-Inline-Maschinerie vs. Serenas API

Serena-Referenz (dekompiliert, v2c-§3-Methode — Architektur-Referenz, **nicht** Code-Quelle):

```
de.oraios.serena.service.request.InlineSymbolRequest
  String  namePath              // Ziel-Symbol
  String  relativePath          // Datei
  boolean keepDefinition        // inlinen + Deklaration behalten statt entfernen

de.oraios.serena.service.request.FormatCodeRequest    // Region/Datei (Offsets)
de.oraios.serena.service.request.FormatSymbolRequest  // Symbol (namePath)
```

**Befund 1 (`inline` = Delegation, kein Eigenbau):** IntelliJ stellt fertige Inline-Refactorings
bereit — `InlineMethodProcessor`, `InlineLocalHandler`, der sprach-spezifische `InlineHandler`-EP
(`com.intellij.lang.refactoring.inlineHandler` / `inlineActionHandler`). Diese liefern
`findUsages()`/`preprocessUsages()` (→ `{usages, conflicts}`) **und** `run()` (Substitution),
exakt das v2c-Delegationsmuster. Der §10-vermutete „neue Transform-Kern" entfällt.

**Befund 2 (`keep_definition`):** `InlineSymbolRequest.keepDefinition` mappt direkt auf den
„Inline all and keep declaration"-Modus der IntelliJ-Inline-Processoren (Konstruktor-Flag).
v2d-Flag `keep_definition` reicht es durch.

**Befund 3 (`reformat` = `CodeStyleManager`, Single-File):** `FormatCodeRequest` (Offset-Region)
und `FormatSymbolRequest` (Symbol) + der JetBrains-MCP `reformat_file` werden von **einer**
v2d-Action `reformat` über Adressierungs-Dualität abgedeckt (§5.3). Mechanik:
`CodeStyleManager.reformatText(...)` (+ optional `OptimizeImportsProcessor`) — kein Usage-Scan,
kein Multi-File-Plan.

**Befund 4 (`inline`-Konflikte sind nicht durchweg überstimmbar):** Anders als
Namenskollisionen (rename) oder verbleibende Refs (safe_delete) verweigert IntelliJ inline in
bestimmten Fällen **hart** (rekursive Methode, mehrere `return`-Pfade, Override/Polymorphie).
Daraus folgt Entscheidung 4: **kein `force`** — überstimmbare Konflikte → `CONFLICT`, harte
Verweigerung → `UNSUPPORTED`.

---

## 4. Architektur — Was geerbt wird, was neu ist

Die v2b-Engine bleibt **unverändert**; v2d ist additiv.

| Mechanismus                            | Quelle   | gilt in v2d                                                              |
| -------------------------------------- | -------- | ----------------------------------------------------------------------- |
| Two-Phase `*_preview`/`*_apply`        | v2b-§4   | **nur `inline`** (reformat ist Single-Phase, §5.3)                       |
| `plan_hash` (BLAKE3, Rust-zentral)     | v2b-§5.2 | **nur `inline`** (reformat hat keine Usages → kein Hash)                 |
| Konflikt-Gate                          | v2b-§5.2 | **nur `inline`** — aber **ohne `force`** (§5.2, Entscheidung 4)          |
| mehrstufiges `select_backend`          | v2b-§3.1 | identisch — beide Ops headless → `BACKEND_REQUIRED`, kein A-Fallback     |
| Smart-Mode-Pflicht / `INDEXING`        | v2b-§6   | **`inline`** (semantische Usage-Suche braucht Index); reformat: rein lokal, kein Index-Bedarf (aber Backend-Pflicht) |
| Sprach-Fallback `UNSUPPORTED_LANGUAGE` | v2b-§6   | identisch (nullable EP-Lookup für Inline-/Format-Maschinerie)           |
| Multi-File-Cache-Kohärenz              | v2b-§9   | **`inline`** (Multi-File); reformat = Single-File-Evict (wie v2a)       |
| Rust/Plugin-Rollenteilung              | v2b-§3   | identisch (Rust: Auflösung/Jail/Hash/Gate; Plugin: PSI)                 |

**Neu in v2d** (genau drei Dinge):

1. **`inline`s `force`-loses Konflikt-Modell** (§5.2) — Konflikte sind teils nicht
   überstimmbar; `inline_apply` hat **kein** `force`-Flag. Überstimmbar → `CONFLICT`, hart
   verweigert → `UNSUPPORTED`.
2. **`reformat`s Single-Phase-Pfad** (§5.3) — eine Action, kein Preview, kein `plan_hash`,
   Single-File-Apply mit Adressierungs-Dualität. Bricht bewusst das Two-Phase-Muster.
3. **#576-Schema-diet für `ctx_refactor`** (§7) — Action-Enum-Array → pipe-delimited
   Description, Descriptions trimmen, Budget-Gate. Format-Umstellung, **kein**
   Informationsverlust (Merge-Spec §3b-Invariante).

### 4.1 Fluss `inline` (Two-Phase, geerbt)

```
PHASE 1 — inline_preview(name_path | path+line[, keep_definition])
   ├─ Rust: resolve_name_path → (src_path, src_range) + PathJail(src)   [reuse v2a/v2b]
   ├─ select_backend: Backing B?  ── nein ─→ Err BACKEND_REQUIRED
   ├─ Plugin POST /inlinePreview:
   │     <InlineMethodProcessor|InlineLocalHandler|InlineHandler-EP>.findUsages()/preprocessUsages()
   │     ← { usages:[…], conflicts:[…] }   (usages = Substitutions-Stellen)
   └─ Rust: plan_hash = BLAKE3(canonical(usages))
         ← Agent: { affected_files, usage_count, conflicts, plan_hash, diff_preview }

PHASE 2 — inline_apply(… , plan_hash[, keep_definition])     // KEIN force
   ├─ Rust: resolve_name_path (erneut) + PathJail(src)
   ├─ select_backend: Backing B?  ── nein ─→ Err BACKEND_REQUIRED
   ├─ Plugin POST /inlinePreview (erneut) → usages + conflicts
   ├─ Rust-Gates: (a) plan_hash neu bilden+vergleichen ≠ → CONFLICT (TOCTOU)
   │              (b) conflicts≠∅ → CONFLICT  (kein force-Bypass, §5.2)
   ├─ Plugin POST /inlineApply:
   │     WriteCommandAction { processor.run() }   // EIN Undo; hart verweigert → UNSUPPORTED
   │     je geänderter Datei: commitDocument + saveDocument
   │     ← { applied:true, changed_paths:[…] }
   └─ Rust: PathJail je changed_path, evict+rewarm, Multi-File-Diff
         ← Agent: { applied:true, changed_paths, diff }
```

### 4.2 Fluss `reformat` (Single-Phase, NEU)

```
reformat(name_path | path | path+line-Range [, optimize_imports])
   ├─ Rust: Adress-Dispatch (§5.3):
   │     name_path → resolve_name_path → (path, src_range)   [Symbol = FormatSymbol]
   │     path                                                 [ganze Datei = reformat_file]
   │     path + line(+end_line)                               [Region = FormatCode]
   │   + PathJail(path)
   ├─ select_backend: Backing B?  ── nein ─→ Err BACKEND_REQUIRED   (braucht CodeStyleManager)
   ├─ Plugin POST /reformat:
   │     WriteCommandAction { CodeStyleManager.reformatText(scope)
   │                          [+ OptimizeImportsProcessor wenn optimize_imports] }   // EIN Undo
   │     commitDocument + saveDocument
   │     ← { applied:true, changed_paths:[path] }
   └─ Rust: PathJail(path), evict+rewarm (Single-File)
         ← Agent: { applied:true, changed_paths, diff }   (diff via ctx_delta, nur geänderte Zeilen)
```

---

## 5. Rust-Seite

### 5.1 `ctx_refactor` — drei neue Actions

- **Neue Actions** (kein neues Tool, wie v2a–c): `inline_preview`, `inline_apply`, `reformat`.
  Match-Block + Hilfetext erweitern; Schema über die **eine** Tool-Registry
  (`registered/ctx_refactor.rs` → `tool_def(...)`), Drift-Regression-Test deckt es ab.
- **Parameter:**
    - **`inline_preview`/`inline_apply`:** Quelle `name_path` (primär) **oder** `path`+`line`
      (+`column`/`end_line` Fallback); `keep_definition` (optional, default `false`); `apply`
      zusätzlich `plan_hash` (required). **Kein `force`** (§5.2).
    - **`reformat`:** Adresse `name_path` **oder** `path` **oder** `path`+`line`(+`end_line`);
      `optimize_imports` (optional, default `false`). **Kein `plan_hash`, kein `force`** (Single-Phase).
- **Auflösungsschritt:** `inline` nutzt `resolve_name_path` (reuse v2a) → `(src_path, src_range)`;
  >1 → `AMBIGUOUS_SYMBOL`; 0 → `NO_SYMBOL`; danach PathJail. `reformat` löst je Adress-Variante
  (§5.3) auf, dann PathJail.

### 5.2 `inline`-Konflikt-Gate — geerbt, aber `force`-los (NEU)

`plan_hash`-Bildung/-Prüfung identisch zu v2b (BLAKE3 über nach `(path,range)` sortierte
Usages + Ist-Inhalt; `*_preview` bildet, `*_apply` re-bildet → Mismatch ⇒ `CONFLICT` (TOCTOU);
`plan_hash` erscheint **nie** auf der Wire). **Abweichung zu v2b/v2c:** das Konflikt-Gate hat
**keinen `force`-Bypass** — `conflicts≠∅ ⇒ CONFLICT` ist final (Entscheidung 4, Befund 4).
Verweigert das Plugin/IntelliJ die Transformation hart (rekursiv, mehrere `return`,
Override/Polymorphie), liefert der Apply-Pfad `UNSUPPORTED` (kein Schein-`force`).

### 5.3 `reformat`-Adressierungs-Dualität & Single-Phase (NEU)

Genau **eine** Adress-Form bestimmt den Scope (kein Two-Phase, kein Plan):

```
name_path        → resolve_name_path → (path, src_range)   → Symbol-Reformat (FormatSymbol)
path (allein)    → ganze Datei                              → File-Reformat (reformat_file)
path + line[+end_line] → Region (Offset-Range)              → Region-Reformat (FormatCode)
keine Adresse / widersprüchlich → INVALID_TARGET (vor Backend-Call)
```

- PathJail auf den aufgelösten `path` **vor** dem Backend-Call (Quelle stammt aus dem Index
  bzw. ist aufrufer-geliefert → jailen wie v2c).
- **Kein `plan_hash`/Preview:** reformat ändert nur Whitespace/Formatierung, hat keine Usages
  und keinen semantischen Effekt → ein Hash über „die zu formatierende Region" wäre künstliche
  Zeremonie ohne Schutzwert (Entscheidung 3). Apply schreibt direkt.
- **Single-File-Cache:** nur der eine `changed_path` wird evicted+rewarmed (kein Multi-File-Plan).
- Diff zurück via **`ctx_delta`** (nur geänderte Zeilen — **kein** `ctx_diff`, existiert nicht;
  Projekt-Rule).

### 5.4 `LspBackend`-Trait — drei neue Methoden (Default = `Err`)

Additiv zu v1/v2a/v2b/v2c. Default = `Err(String)` (BackendRequired-Idiom, **realer** Code —
v2c-Spec hatte fälschlich `BackendError` als Draft angenommen; v2d folgt der Ist-Signatur
`Result<RenamePlan, String>`):

```rust
// rust/src/lsp/backend.rs (Erweiterung)
fn inline_preview(&mut self, _req: &InlineQuery) -> Result<RenamePlan, String> { Err("BACKEND_REQUIRED".into()) }
fn inline_apply(&mut self, _req: &InlineApply)   -> Result<RenameResult, String> { Err("BACKEND_REQUIRED".into()) }
fn reformat(&mut self, _req: &ReformatQuery)     -> Result<ReformatResult, String> { Err("BACKEND_REQUIRED".into()) }
```

- **`RenamePlan`/`RenameResult` wiederverwendet** für `inline` (`{usages, conflicts}` bzw.
  `{applied, changed_paths}` — op-unabhängig, wie v2c).
- **`ReformatResult`** — neuer, schlanker Typ `{ applied: bool, changed_paths: Vec<String> }`
  (keine Usages, kein Plan). Alternativ Wiederverwendung von `RenameResult` möglich; eigener
  Typ macht „reformat hat keinen Usage-Begriff" im Typsystem explizit (Empfehlung: eigener Typ).
- **`InlineQuery`**: `{ abs_path, src_range, keep_definition: bool }`.
  **`InlineApply`**: `InlineQuery` (+ `plan_hash` wird Rust-seitig gegen den re-berechneten
  geprüft, nicht an den Trait gereicht — wie v2b). **Kein `force`-Feld.**
- **`ReformatQuery`**: `{ abs_path, scope: ReformatScope, optimize_imports: bool }` mit
  `enum ReformatScope { File, Region{ range }, Symbol{ range } }` (Adresse bereits aufgelöst —
  der Trait sieht nie einen `name_path`, exakt wie v2a–c).
- `LspClient` (Backing A) **erbt den `Err`-Default** → headless → sauberes `BACKEND_REQUIRED`.

### 5.5 Änderungsstellen (Rust)

| Datei                                       | Änderung                                                                                                                                                              |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `rust/src/tools/ctx_refactor.rs`            | +3 Actions; `inline` = v2b-`plan_hash`/Konflikt-Gate reuse **ohne `force`** (§5.2); `reformat` = Single-Phase-Pfad mit Adress-Dispatch (§5.3), kein `plan_hash`; `changed`-Set um `inline_apply`/`reformat` ergänzen |
| `rust/src/tools/registered/ctx_refactor.rs` | **#576-Umstellung** (§7): Action-Enum-Array → pipe-delimited Description (+`inline_preview`/`inline_apply`/`reformat`); Tool-Description kürzen+aktualisieren; neue Props `keep_definition`/`optimize_imports` knapp; `schema_tests`-needles ergänzen |
| `rust/src/lsp/backend.rs`                   | +3 Trait-Methoden (Default `Err`) + Typen `InlineQuery`/`InlineApply`/`ReformatQuery`/`ReformatScope`/`ReformatResult`                                                |
| `rust/src/lsp/jetbrains_backend.rs`         | HTTP-Override der 3 Methoden (`/inlinePreview`, `/inlineApply`, `/reformat`)                                                                                          |
| `rust/src/lsp/client.rs`                    | erbt `Err`-Default (keine Änderung außer ggf. Trait-Re-Export)                                                                                                        |

---

## 6. Plugin-Seite (Kotlin) — additiv

Integriert additiv in `packages/jetbrains-lean-ctx/.../com/leanctx/plugin` (koexistiert mit
v1/v2a/v2b/v2c, ersetzt nichts).

- **`psi/SymbolRefactorer.kt`** (Erweiterung der v2c-Naht): zwei neue Operationen.
  (Architektur-Referenz — **nicht** Code-Quelle: Serenas `InlineSymbolRequest`/
  `FormatCodeRequest`/`FormatSymbolRequest`, dekompiliert §3.)
    - **`inline` Preview:** passenden IntelliJ-Inline-Mechanismus je Symbol-Art —
      `InlineMethodProcessor` (Methode/Funktion), `InlineLocalHandler` (lokale Variable),
      sprach-`InlineHandler`-EP (`com.intellij.lang.refactoring.inlineHandler`) generisch.
      `findUsages()`+`preprocessUsages()` → `{usages, conflicts}`. **Kein** Write.
    - **`inline` Apply:** `WriteCommandAction.runWriteCommandAction(project) { processor.run() }`
      → Multi-File-Transaktion als **ein** Undo. `keep_definition` steuert das
      „keep declaration"-Flag des Processors. Harte Verweigerung (IntelliJ wirft/lehnt ab) →
      `UNSUPPORTED` (kein Teil-Edit).
    - **`reformat`:** je Scope (`SymbolRefactorer` ermittelt aus dem Wire-Scope) —
      `CodeStyleManager.getInstance(project).reformatText(file, ranges)` für Datei/Region,
      Symbol-Range für Symbol; `optimize_imports` → zusätzlich
      `OptimizeImportsProcessor(project, file).run()`. Single-File, **ein** Undo. **Kein** Preview-Pfad.
    - Nach dem Lauf je betroffener Datei: `PsiDocumentManager.commitDocument` +
      `FileDocumentManager.saveDocument` (auf Platte, damit lean-ctx es sieht).
- **`endpoint/RefactorHandlers.kt`** (Erweiterung): `inlinePreview`/`inlineApply`/`reformat`,
  registriert im `RequestRouter` (Token-Check wie v1).
- **Threading + Index-Schutz** (geerbt v2b-§6): `inline`-`findUsages`/`preprocessUsages` unter
  `DumbService.runReadActionInSmartMode` (off-EDT), Smart-Mode Pflicht → sonst `INDEXING`.
  `reformat` braucht **keinen** Index (rein lexikalisch/CodeStyle) — kein `INDEXING`-Gate, aber
  `WriteCommandAction` (EDT) wie alle Applies. Beide: `BACKEND_REQUIRED` ohne IDE.
- **Sprach-Fallback** (geerbt v2b-§6): Sprache ohne Inline-Handler bzw. ohne `CodeStyleManager`-
  Support → `UNSUPPORTED_LANGUAGE` (nullable EP-Lookup, keine harten Imports, kein Crash).
- **gson `compileOnly`** (v1-§5.4): neue DTOs/Handler nutzen die IDE-gebündelte gson.
- **Kanonische Refactoring-Grenze** (geerbt v2b-§6): Plugin = alleinige Usage-Quelle für
  `inline`; Rust spiegelt keine zweite Range-Berechnung — `plan_hash` hasht nur den Ist-Inhalt
  der gemeldeten Stellen. `reformat` meldet nur `changed_paths` (kein Hash).

---

## 7. Schema-diet (#576) + Lean-Surface (#575) — verbindlich (NEU)

Bezug: Merge-Spec `2026-06-13-merge-main-into-feat-jetbrains-plugin-design.md` §3a/§3b. Das
aktuelle `ctx_refactor`-Schema ist noch der **alte Bloat-Stil** (17-Action-`enum`-Array,
veraltete Tool-Description, die v2c gar nicht nennt, 14+ teils lange Property-Descriptions).
v2d fügt 3 Actions + 2 Properties hinzu und macht die #576-Angleichung **gleich mit** (nicht
als Follow-up) — sonst verstärkt v2d den Bloat.

**#576 — Schema-Umstellung (Pflicht-Teil von v2d):**

- **Action-Enum-Array → pipe-delimited Description**, Vorbild `registered/ctx_graph.rs`:
  ```diff
  - "action": { "type": "string",
  -   "enum": ["rename","references",…,"safe_delete_apply"],   // 17er-Array
  -   "description": "Refactoring action" },
  + "action": { "type": "string",
  +   "description": "rename|references|definition|implementations|declaration|\
  +     type_hierarchy|symbols_overview|inspections|replace_symbol_body|\
  +     insert_before_symbol|insert_after_symbol|rename_preview|rename_apply|\
  +     move_preview|move_apply|safe_delete_preview|safe_delete_apply|\
  +     inline_preview|inline_apply|reformat" },               // 20, pipe-delimited
  ```
- **Invariante (§3b): Format-Umstellung, KEIN Informationsverlust.** Alle 20 Actions + alle
  Params bleiben erreichbar — nur das Format wandert von `enum`-Array in die Description.
- **Tool-Description kürzen + aktualisieren** (−36 %-Idiom): die veraltete Prosa (kennt v2c
  nicht) durch eine knappe, vollständige ersetzen.
- **Property-Descriptions trimmen** am #576-Stil; neue Props `keep_definition` (inline) und
  `optimize_imports` (reformat) knapp halten.
- **`schema_tests`** bleibt grün (`contains`-needles sind Substrings der pipe-Description) und
  wird um `inline_preview`/`inline_apply`/`reformat`/`keep_definition`/`optimize_imports` ergänzt.

**Budget-Beweis statt Annahme (Abnahme-Gate):** Nach der Umstellung müssen
`bench_total_input_overhead` (<12000) und `bench_tool_descriptions` (<3000) in
`intensive_benchmarks.rs` grün sein. Das erledigt zugleich den Code-TODO des Branch-Autors
*„v2c FOLLOW-UP: analyze the real overhead drivers instead of raising this ceiling further"* —
die Decke wird **gesenkt**, nicht erhöht.

**#575 — Lean default surface (Doku-Punkt, kein Code):** `ctx_refactor` ist **nicht** Core →
standardmäßig nur via `ctx_call` sichtbar. Das Projekt pinnt `tool_profile = power`
(`explicit_profile=true` → `ProfileAuthoritative`) → voll sichtbar (kein Problem für
Projekt/Subagent). v2d-Spec hält fest: Plugin-Endnutzer **ohne** gepinntes Profil erreichen
`inline`/`reformat` über `ctx_call` bzw. `lean-ctx tools reset` — analog zu v2c.

---

## 8. Wire-Protokoll (DTO) — additiv zu v1/v2a/v2b/v2c

- **0-basiert** (Zeile + Spalte), Pfade relativ zu `project_root`. Token-Header
  `X-LeanCtx-Token` wie v1.
- **Neue Endpoints (POST):**
    - `POST /inlinePreview` — Request `{ path, range:{start,end}, keep_definition }`
      → Response `{ usages:[{path,range,context?}], conflicts:[{path,range,message}] }`.
    - `POST /inlineApply` — Request `{ path, range, keep_definition }`
      → `{ applied:true, changed_paths:[…] }`. **Kein `force`-Feld.**
    - `POST /reformat` — Request `{ path, scope:{ kind:"file"|"region"|"symbol", range? }, optimize_imports }`
      → `{ applied:true, changed_paths:[path] }`.
- **`plan_hash` erscheint NICHT auf der Wire** (v2a-§7-Regel; nur `inline`, reine Rust-Logik;
  `reformat` hat gar keinen).
- **Fehler** (additiv):
    - **Reuse:** `BACKEND_REQUIRED`, `CONFLICT` (nur `inline`: plan_hash-Mismatch **oder**
      geblocktes Gate), `AMBIGUOUS_SYMBOL`, `NO_SYMBOL`, `INDEXING` (nur `inline`),
      `UNSUPPORTED_LANGUAGE`.
    - **`UNSUPPORTED`** (NEU, nur `inline`-Apply) — IntelliJ verweigert die Transformation hart
      (rekursiv, mehrere `return`, Override/Polymorphie). Kein `force`-Bypass.
    - **`INVALID_TARGET`** (reuse aus v2c, nun auch `reformat`) — keine/widersprüchliche Adresse,
      oder aufgelöster Pfad außerhalb `project_root` (Jail-Verletzung). Rust-seitig vor Backend-Call.
    - HTTP 200 für fachliche Negativfälle, 401 nur Token, 500 nur echte Exceptions.

---

## 9. Op-Semantik (Serena-Parität)

| Action           | Eingabe                                                       | Wirkung                                                                                       |
| ---------------- | ------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| `inline_preview` | Quelle [+ `keep_definition`]                                 | **kein** Write; Substitutions-Stellen, Usage-Count, Konflikte, `plan_hash`, Diff             |
| `inline_apply`   | Quelle + `plan_hash` [+ `keep_definition`]                   | substituiert an allen Aufrufstellen (Multi-File, ein Undo); Konflikt → `CONFLICT`/`UNSUPPORTED` (kein `force`) |
| `reformat`       | Adresse (`name_path` \| `path` \| `path`+Range) [+ `optimize_imports`] | formatiert Symbol/Region/Datei (Single-File, ein Undo); kein Preview, kein `plan_hash` |

- Adressierung des Quell-Symbols identisch zu v2a–c (name_path primär, Position-Fallback).
- Drop-in-kompatibel zu Serenas `jet_brains_inline_symbol` + den Format-Requests, plus
  expliziter Preview-Schritt für `inline`, den Serena nicht hat.

---

## 10. Verifikation (End-to-End)

- **Rust-Einheit (`cargo nextest run`, nie `cargo test`):**
    - `inline` `plan_hash`: deterministisch über sortierte Usages+Inhalt; Match → Apply;
      Mismatch → `CONFLICT`.
    - `inline`-Gate **ohne `force`:** `conflicts≠∅ → CONFLICT` immer (kein Bypass-Pfad existiert).
    - `reformat` Adress-Dispatch: `name_path`→Symbol, `path`→Datei, `path`+`line`→Region;
      keine/widersprüchliche Adresse → `INVALID_TARGET` **vor** Backend-Call; Pfad außerhalb
      `project_root` → `INVALID_TARGET`.
    - `select_backend` (mehrstufig, v1-§8): stale Port / toter pid / Health-Timeout →
      `BACKEND_REQUIRED`, kein Apply, **kein** A-Fallback — für **beide** Ops.
    - 0/1-Basierungs-Naht (Tool-Eingabe 1-basiert ↔ Wire 0-basiert ↔ Offset).
    - **#576-Budget (§7):** `bench_total_input_overhead` (<12000) + `bench_tool_descriptions`
      (<3000) grün nach der Schema-Umstellung. `schema_tests`-needles für die 3 Actions + 2 Props.
- **Plugin (Kotlin-Unit + manuelles `runIde`-Gate, wie v1/v2b/v2c):**
    - **Akzeptanz-Gate Kotlin** (Primär):
        - `inline`: (a) lokale Variable inlinen → alle Vorkommen ersetzt, Deklaration entfernt;
          (b) Methode/Funktion inlinen → Body an Aufrufstellen, korrektes Parameter-Binding;
          (c) `keep_definition=true` → inlinen, Deklaration bleibt; (d) rekursive Methode →
          `UNSUPPORTED`, kein Teil-Edit.
        - `reformat`: (a) Datei formatieren → Whitespace normalisiert; (b) Region (`line`-Range)
          → nur Region; (c) Symbol (`name_path`) → nur Symbol-Body; (d) `optimize_imports=true`
          → ungenutzte Imports entfernt.
    - `*.run()` erzeugt **einen** Undo-Eintrag; `saveDocument` persistiert je Datei.
    - **Index-Schutz:** `inline` während Indizierung (Dumb-Mode) → `INDEXING`, **kein** Teil-Edit.
    - **Sprach-Fallback:** Op in Sprache ohne Handler → `UNSUPPORTED_LANGUAGE`, kein Crash.
    - Java optionaler Sekundär-Check (nicht akzeptanzkritisch).
- **Fallback:** ohne laufende IDE → `BACKEND_REQUIRED` (inline: beide Phasen; reformat: die eine
  Action), kein Apply.

### 10.1 Live-Gate — eigenes runIde-Runbook

v2d bekommt ein **manuelles Live-Verifikations-Gate** nach dem Muster des v2c-Gates
(`docs/lean-md/runbooks/runide-move-safedelete-gate.md`). Es verifiziert den vollen v2d-Stack
**live** — Rust-Gate (`inline` `plan_hash`/TOCTOU, `force`-loses Konflikt-Gate, PathJail,
Cache-Evict; `reformat` Adress-Dispatch + Single-File-Evict) **und** Plugin (Inline-Processoren,
`CodeStyleManager`, Multi-File- bzw. Single-File-Transaktion, ein Undo) — gegen ein
Kotlin-Gradle-Fixture.

**Liefergegenstand (in v2d zu erstellen):** `docs/lean-md/runbooks/runide-inline-reformat-gate.md`
mit identischer Struktur (Voraussetzungen → Fixture-Setup → `./gradlew runIde --args="$FIX"` →
Gate-Checks via `lean-ctx call ctx_refactor --project-root "$FIX" --json '<args>'` → Teardown).
Das Fixture stellt bereit: eine inline-bare lokale Variable, eine inline-bare Methode mit
Aufrufstellen, eine rekursive Methode (für `UNSUPPORTED`), eine schlecht formatierte Datei/Region
und eine Datei mit ungenutzten Imports.

**Voraussetzung — frisches Binary (Daemon-Stopp ist Pflicht):** Die neuen Actions
(`inline_*`/`reformat`) existieren erst nach Neubau. Ein **laufender** lean-ctx-Daemon hält den
**alten** Action-Satz im Speicher → `Unknown action`. Reihenfolge **vor** dem Gate:
1. `lean-ctx serve --stop` — Daemon stoppen (gibt Binary frei + entlädt alten Action-Satz).
2. `cargo build` (cwd=`rust`) [+ ggf. Binary neu installieren].
3. `lean-ctx serve --daemon` (neu) **oder** den ersten `lean-ctx call` auto-starten lassen.

> **Achtung MCP-Session:** In einer aktiven Agent-/MCP-Session ist dieser Daemon zugleich der
> `ctx_*`-Server — `serve --stop` unterbricht die eigenen `ctx_*`-Tools bis zum Neustart. Gate
> als **separaten** Schritt fahren (nicht mitten in einer ctx_*-getriebenen Aufgabe).

**Gate-Checks (Soll-Ergebnisse):**

| #  | Fall                              | Aufruf (`--json`, Auszug)                                                                  | Soll-Ergebnis                                                        |
| -- | --------------------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| 1  | inline Preview (lokal)            | `{"action":"inline_preview","name_path":"Foo/calc/tmp"}`                                   | usages = Vorkommen, `files≥1`, `plan_hash` gesetzt                  |
| 2  | inline Apply + Undo               | `{"action":"inline_apply","name_path":"Foo/calc/tmp","plan_hash":"<#1>"}`                  | Vorkommen ersetzt, Deklaration weg; **ein** Undo (Strg+Z komplett) |
| 3  | inline Methode                    | `{"action":"inline_preview"/"inline_apply","name_path":"Helper/calc",…}`                   | Body an Aufrufstellen, Parameter-Binding korrekt                   |
| 4  | inline `keep_definition`          | `{"action":"inline_apply","name_path":"Helper/calc","keep_definition":true,"plan_hash":…}` | inlined, Deklaration bleibt                                        |
| 5  | inline rekursiv → UNSUPPORTED     | `{"action":"inline_apply","name_path":"Recurse/loop","plan_hash":…}`                       | `UNSUPPORTED`, kein Teil-Edit                                      |
| 6  | inline TOCTOU                     | eine usage zwischen #1 und Apply ändern, dann Apply mit altem `plan_hash`                  | `CONFLICT`                                                          |
| 7  | reformat Datei                    | `{"action":"reformat","path":"app/Messy.kt"}`                                              | Whitespace normalisiert, Diff via ctx_delta                       |
| 8  | reformat Region                   | `{"action":"reformat","path":"app/Messy.kt","line":10,"end_line":20}`                      | nur Region formatiert                                              |
| 9  | reformat Symbol                   | `{"action":"reformat","name_path":"Messy/render"}`                                         | nur Symbol-Body formatiert                                         |
| 10 | reformat optimize_imports         | `{"action":"reformat","path":"app/Imports.kt","optimize_imports":true}`                    | ungenutzte Imports entfernt                                       |
| 11 | reformat INVALID_TARGET           | `{"action":"reformat"}` (keine Adresse) **und** `path:"../escape"`                          | je `INVALID_TARGET`, **vor** Backend-Call                         |
| 12 | INDEXING (inline)                 | Projekt neu öffnen, sofort `inline_preview` während Indizierung                            | `INDEXING`, kein Teil-Edit                                         |
| 13 | UNSUPPORTED_LANGUAGE              | `{"action":"inline_preview","path":"notes.txt","line":1}` (Position-Fallback)             | `UNSUPPORTED_LANGUAGE`, kein Crash                                |
| 14 | BACKEND_REQUIRED                  | IDE schließen, dann inline (beide Phasen) **und** reformat                                 | `BACKEND_REQUIRED`                                                 |

> Wie bei v2b/v2c: für TOCTOU-Fälle zuerst ein eigenes `inline_preview` ausführen, um den
> aktuellen `plan_hash` zu erhalten. Deterministische Teile (INDEXING-Dumb-Mode, PathJail,
> Adress-Dispatch, #576-Budget) zusätzlich als Rust-Unit-Test absichern (§10); das Live-Gate
> deckt die nicht-headless-reproduzierbare Plugin-Naht ab.

---

## 11. Branch- & Commit-Strategie

- Fortführung auf `feat-jetbrains-plugin` (Muster v1-§12.3): **ein Commit pro Phase** nach
  erfülltem Phasen-Gate, kein Squash während der Entwicklung. Direkt auf dem Branch, **kein
  worktree** (Projekt-Rule).
- Finaler Merge nach `main` via Squash-Merge-PR (am Schluss).
- **Schema-Drift-Gate:** `ctx_refactor`-Schema-Änderung (inkl. #576-Umstellung) →
  `docs/reference/generated/mcp-tools.md` via `cargo run --example gen_docs --features dev-tools`
  (cwd=rust) regenerieren (Drift-Test `generated_reference…`). Zusätzlich
  `docs/reference/appendix-mcp-tools.md` (human tool map) um die drei Actions ergänzen.

---

## 12. Bewusst NICHT in v2d (YAGNI)

- **Kein `force` für `inline`** (Konflikte teils nicht überstimmbar — Entscheidung 4; harte
  Verweigerung → `UNSUPPORTED`, kein Schein-Bypass).
- **Kein `plan_hash`/Preview für `reformat`** (Single-Phase, keine Usages — Entscheidung 3).
- **Kein eigener Inline-Transform-Kern** (IntelliJ-Processoren reuse — Entscheidung 2; §10-Annahme
  verworfen).
- **Kein Headless-inline/-reformat** (kein verlustfreier Default, geerbt v2b-§3 →
  `BACKEND_REQUIRED`).
- **Kein plugin-seitiges Hashing** (Integritäts-Guards in Rust, v2a-§7).
- **Kein Auto-Reformat im `inline`-Apply** (entkoppelt; `reformat` ist die separate Op, v2a-§6.1).
- **Keine Anhebung der Budget-Decke** — die #576-Umstellung senkt sie (§7); der v2c-Code-TODO
  wird eingelöst, nicht umgangen.

**Out of v2d (bleibt out of scope, v2c-§10):**

- `serena.jet_brains_debug` (kein Code-Intelligence-Äquivalent).
- `jetbrains.generate_psi_tree` — tree-sitter (`ctx_symbol`/`ctx_outline`/`symbols_overview`)
  deckt den Bedarf headless ab → kein Ablöse-Treiber (v1-§13.1 „optional").
- DB-/Run-/SQL-/Terminal-/Editor-UI-Tools des JetBrains-MCP (v1-§1, v2a-§11).

**Nach v2d** ist Serena auch als Edit-Engine vollständig entbehrlich (v1-§13.4) — danach sind
Serena-MCP **und** JetBrains-MCP (Code-Intelligence-Teil) aus der Agent-Konfiguration
entfernbar; lean-ctx ist die alleinige Schnittstelle. Damit ist die Refactoring-Ablöse-Reihe
(v2a–v2d) abgeschlossen.

---

## 13. Korrektur: v1-§13.3-Diskrepanz (`format`-Action)

v1-§13.3 listet `reformat_file → ctx_refactor action=format` als „v1 (erledigt)". Im Code
existiert **keine** `format`-Action (`ctx_refactor.rs` Match-Block; bestätigt 2026-06-13). Die
v1-Tabelle ist hier **veraltet** — `reformat` ist real offen und wird **erst in v2d** umgesetzt
(als Action `reformat`, nicht `format`). v2d korrigiert die v1-§13.3-Zeile entsprechend
(Status: v2d statt „v1 erledigt"; Action-Name `reformat`).
