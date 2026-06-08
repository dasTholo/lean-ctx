# Design-Spec: lean-ctx JetBrains v2a — Symbolische Body-Edits (Serena-Ablösung, Edit-Klasse A)

| Feld             | Wert                                                                                      |
| ---------------- | ----------------------------------------------------------------------------------------- |
| Status           | Draft (Design genehmigt 2026-06-08)                                                        |
| Datum            | 2026-06-08                                                                                 |
| Vorhaben         | Symbol-Body-Edits über das JetBrains-Plugin + lean-ctx (v2 der PSI-Backend-Strategie)      |
| Scope            | `replace_symbol_body`, `insert_before_symbol`, `insert_after_symbol` + `overview`-Fallback |
| Basis-Spec       | `docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md` (v1, §9 v2-Ausblick) |
| Branch           | `feat-jetbrains-plugin` (Fortführung, Muster §12.3 — ein Commit pro Phase)                  |
| Nächster Schritt | `superpowers:writing-plans` (Implementierungsplan)                                          |

---

## 1. Context — Warum v2a jetzt

v1 (read-only) ist abgeschlossen: Navigation, `type_hierarchy`, `overview`, `format`,
`inspections` laufen über Backing B (JetBrains-Plugin) bzw. Backing A (rust-analyzer).
Damit ist das **offizielle JetBrains-MCP** für Code-Intelligence bereits entbehrlich.
**Serena** bleibt nur noch wegen seiner **symbolischen Edit-Ops** in der Agent-Konfiguration.

v2 schließt diese Lücke. Die v1-Spec hat v2 in **zwei technisch unterschiedliche Klassen**
zerlegt (§9 v2-Ausblick, §13.2):

- **Klasse A — Symbol-Body-Edits** (`replace_symbol_body`, `insert_before_symbol`,
  `insert_after_symbol`): symbol-verankerte Text-Ersetzungen, nah an `ctx_edit`.
- **Klasse B — Refactoring-Engine** (`rename_apply`, `move`, `safe_delete`, `inline`):
  IntelliJ-`RefactoringFactory`, Multi-File, Konflikt-Erkennung, Undo.

**Dieser Spec behandelt ausschließlich Klasse A (v2a).** Klasse B (v2b) folgt als
**eigenständiger Folge-Spec nach v2a-Abschluss** — siehe §11. Begründung des Schnitts:
Body-Edits brauchen nur eine Symbol-Range + einen Range-Write; die Refactoring-Engine
braucht ein fundamental anderes Modell (Multi-File-Transaktion, Konflikt-UI-Bypass).
Ein gemeinsamer Spec würde das risikoärmere v2a unnötig an das risikoreichere v2b koppeln.

**Ziel von v2a:** Serenas `replace_symbol_body` / `insert_before_symbol` /
`insert_after_symbol` durch `ctx_refactor`-Actions ablösen — funktionsgleich, aber unter
einem Dach (Token-Kompression, PathJail) und mit dem Zusatznutzen, **auch headless** (CI,
geschlossene IDE) zu editieren, was Serena prinzipiell nicht kann.

---

## 2. Getroffene Entscheidungen (User, 2026-06-08)

| # | Frage                  | Entscheidung                                                                                                                                                                  |
| - | ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1 | v2-Zuschnitt           | **v2a (Klasse A) jetzt**; v2b (Refactoring-Engine) folgt als **eigener Spec** nach v2a-Abschluss. Forward-Pointer in §11.                                                     |
| 2 | Symbol-Adressierung    | **`name_path` primär**, auf der **lean-ctx-Rust-Seite** über den bestehenden Symbol-Index aufgelöst. **Position** `(path, line, character)` als Low-Level-Fallback.          |
| 3 | Apply-Modell           | **Direkt anwenden + Diff/Delta zurück** (kein Preview/Two-Phase). Optionaler `expected_hash`-Guard → `CONFLICT` statt Blind-Überschreiben.                                   |
| 4 | IDE vs. headless       | **IDE-first + Headless-Fallback**, gesteuert über `select_backend`. Kanonische Edit-Grenze = **tree-sitter-Range in beiden Pfaden** → IDE-Pfad ≡ Headless-Pfad byte-identisch. |
| 5 | Headless-Apply-Schicht | **Default-Impl direkt im `LspBackend`-Trait** (lokaler Range-Write); `JetBrainsHttpBackend` **overridet** mit dem HTTP/PSI-Pfad.                                              |
| 6 | Read-Fallback          | **Nur `overview`** bekommt einen tree-sitter-Default-Impl (strukturell, verlustfrei, headless). Semantische Reads (refs/def/impl/type_hierarchy) bleiben unangetastet.       |

---

## 3. Schlüssel-Befund (verifiziert 2026-06-08): name_path ist Rust-seitig auflösbar

Der gesamte v2a-Architekturhebel beruht auf einem geprüften Befund: die
`name_path`→Position-Auflösung braucht **keinen** neuen PSI-Resolver im Plugin, weil
lean-ctx den Symbol-Index bereits besitzt.

- `ctx_symbol action=find name=…` löst **Klassen und Methoden** (auch per Bare-Name) auf
  **exakte Zeilen-Ranges** auf — aus dem tree-sitter-Index (18 Sprachen, ~17.780 Symbole):
  - `InspectionRunner → (class, L25-89)`
  - `runOnFile → (method, L33-63)`
- **Mehrdeutigkeit ist schon abgedeckt:** Bare `runOnFile` liefert mehrere Treffer
  (`InspectionRunner` + `InspectionHandlers`) → ein qualifizierter `name_path`
  (`InspectionRunner/runOnFile`) disambiguiert.
- `ctx_read mode=map|signatures` exponiert die Symbol-Oberfläche zusätzlich; `ctx_symbol`
  ist die **präzise Range-Quelle**.

**Konsequenz (vorhandener Code hat Priorität vor Serena):** Das Plugin bleibt
**positions-/range-basiert** (`PsiLocator`, wie ganz v1). `name_path` ist eine
**Rust-Komfortschicht** über dem schon vorhandenen Index — kein neuer Kotlin-Name-Resolver,
keine Duplikat-Symbolik. Die `name_path`-Auflösung ist zudem **backend-unabhängig** (läuft
ohne IDE) und ermöglicht damit erst den Headless-Pfad (§5).

---

## 4. Architektur — Schichtung

```
ctx_refactor action=replace_symbol_body name_path="Klasse/Methode" new_body="…"
   │
   ├─ (1) name_path → (path, range)        Rust, backend-UNABHÄNGIG (Symbol-Index)
   │        + PathJail(jail_path) auf den aufgelösten Pfad  ── VOR jedem Apply
   │
   └─ (2) Apply — select_backend entscheidet:
            ├─ Backing B erreichbar  → JetBrainsHttpBackend.replace_symbol_body(path, range, text)
            │                            → HTTP → Plugin: WriteCommandAction
            │                            (PSI-Commit + saveDocument + VFS-Kohärenz + Undo)
            └─ sonst (headless)      → Trait-Default-Impl: lokaler Range-Write
                                         (ctx_edit-Schreibpfad, dieselbe tree-sitter-Range)
```

**Kernprinzip:** `select_backend` ist der **einzige** Entscheidungspunkt IDE-vs-headless —
keine neue Mechanik, sondern direkte Fortführung der v1-Phase-0-Trait-Architektur. Die
MCP-Tool-Oberfläche (`ctx_refactor`) und PathJail bleiben unverändert.

---

## 5. Rust-Seite

### 5.1 `LspBackend`-Trait — drei neue Edit-Methoden

Additiv zu den v1-Methoden (kein Breaking Change — entspricht dem v1-Versprechen
„additiv als Default-`Err`-Trait-Methoden", §9/§13.2; hier mit **Default-Apply** statt
Default-`Err`, weil Edits *immer* laufen sollen):

```rust
// rust/src/lsp/backend.rs (Erweiterung)
fn replace_symbol_body(&mut self, edit: RangeEdit) -> Result<EditResult, BackendError> {
    // DEFAULT-IMPL = lokaler Range-Write (headless). JetBrainsHttpBackend overridet.
    local_range_write(edit)
}
fn insert_before_symbol(&mut self, edit: RangeEdit) -> Result<EditResult, BackendError> { … }
fn insert_after_symbol(&mut self,  edit: RangeEdit) -> Result<EditResult, BackendError> { … }
```

- **`RangeEdit`** (Begleittyp): `{ abs_path, range: TextRange0Based, text, expected_hash: Option<String> }`.
  Die Range ist **bereits aufgelöst** (name_path-Auflösung passiert davor in `ctx_refactor`)
  → das Trait sieht nie einen `name_path`.
- **`EditResult`**: `{ applied: bool, new_range: TextRange0Based, edited_text: String, diff: String }`.
- **`local_range_write`** (gemeinsame Hilfsfn, `lsp/edit_apply.rs`): liest die Datei, prüft
  optional `expected_hash` gegen den aktuellen Range-Inhalt (→ `CONFLICT`), ersetzt
  `range`→`text`, schreibt über den **`ctx_edit`-Schreibpfad** (Cache-Kohärenz, §8), gibt
  `EditResult` zurück. Backing A (`LspClient`) erbt diesen Default → headless-Edits ohne
  rust-analyzer-Beteiligung.

### 5.2 `overview`-Default-Impl (Read-Fallback, §2-Entscheidung 6)

- `symbols_overview` bekommt im Trait einen **tree-sitter-Default-Impl** (Quelle = derselbe
  Symbol-Index wie `ctx_symbol`/`ctx_outline`), statt wie in v1 auf Backing A zu `Err` zu
  degradieren. Strukturell, verlustfrei, läuft headless.
- `JetBrainsHttpBackend` behält seinen PSI-`overview` (Override, IDE-genau via Structure-View).
- **Unverändert:** `references`/`definition`/`declaration`/`implementations`/`type_hierarchy`
  bleiben Backing-A-semantisch (rust-analyzer) bzw. `Err`/`UNSUPPORTED_LANGUAGE`. **Keine**
  tree-sitter-Heuristik für semantische Reads (würde Fidelity unterlaufen — bewusst nicht).

### 5.3 `ctx_refactor` — neue Actions + name_path-Auflösung

- **Neue Actions** (kein neues Tool — vermeidet Nachziehen in `tool_profiles.rs`/
  `dynamic_tools.rs`/`workflow/types.rs`): `replace_symbol_body`, `insert_before_symbol`,
  `insert_after_symbol`. Match-Block + Hilfetext erweitern; Schema über die **eine**
  Tool-Registry `tool_defs::tool_def(...)` (Changelog 3.7.4 #141 — kein zweites
  handgepflegtes Schema; Drift-Regression-Test deckt es ab).
- **Eingabe-Parameter:** `name_path` (primär) **oder** `path`+`line`+`character` (Fallback);
  `new_body`/`text`; optional `expected_hash`.
- **Auflösungsschritt (vor dem Backend-Dispatch):**
  1. `name_path` → Kandidaten über den Symbol-Index (graph_provider, dieselbe Quelle wie
     `ctx_symbol`). Genau 1 Treffer → `(path, range)`. >1 → `AMBIGUOUS_SYMBOL` mit
     Kandidatenliste (qualifizierte name_paths). 0 → `NO_SYMBOL`.
  2. **PathJail:** den aufgelösten Pfad durch `core::path_resolve::resolve_tool_path` →
     `jail_path` schicken (v1-§4.5-Naht — bei Writes doppelt kritisch). Außerhalb
     `project_root` → Fehler **vor** jedem Apply.
  3. **Einrück-/Anker-Berechnung** für `insert_before/after` in Rust (führende Einrückung
     des Anker-Symbols) → beide Apply-Pfade byte-identisch.

### 5.4 Änderungsstellen (Rust)

| Datei                                       | Änderung                                                              |
| ------------------------------------------- | -------------------------------------------------------------------- |
| `rust/src/lsp/backend.rs`                   | +3 Edit-Methoden (Default-Apply) + `RangeEdit`/`EditResult` + `overview`-Default-Impl |
| `rust/src/lsp/edit_apply.rs`                | NEU: `local_range_write` (gemeinsamer headless-Range-Write)          |
| `rust/src/lsp/jetbrains_backend.rs`         | Override der 3 Edit-Methoden (HTTP), `overview`-Override bleibt       |
| `rust/src/lsp/client.rs`                    | erbt Default-Apply (keine Änderung außer ggf. Trait-Re-Export)       |
| `rust/src/tools/ctx_refactor.rs`            | +3 Actions, name_path-Auflösung, PathJail-Naht, Einrück-Berechnung   |
| `rust/src/tools/registered/ctx_refactor.rs` | Schema-Erweiterung über `tool_def(...)`                              |

---

## 6. Plugin-Seite (Kotlin) — additiv

Integriert additiv in `com.leanctx.plugin` (koexistiert mit v1, ersetzt nichts).

- **`psi/SymbolEditor.kt`** (neu): kapselt den Write.
  - `WriteCommandAction.runWriteCommandAction(project) { … }` auf EDT.
  - `Document.replaceString(startOffset, endOffset, text)` (Offsets aus der Wire-Range via
    `PsiLocator`/`Document` umgerechnet — dieselbe 0-basiert↔offset-Logik wie v1).
  - `PsiDocumentManager.getInstance(project).commitDocument(doc)`.
  - `FileDocumentManager.getInstance().saveDocument(doc)` → **schreibt auf Platte**, damit
    lean-ctx (liest von Platte) das Ergebnis sieht.
  - **Kein Auto-Reformat** im Edit (entkoppelt) — siehe §6.1.
- **`endpoint/EditHandlers.kt`** (neu): `replaceSymbolBody` / `insertBeforeSymbol` /
  `insertAfterSymbol`, registriert im `RequestRouter` (Token-Check wie v1).
- **Threading:** Reads in v1 laufen off-EDT unter `ReadAction`; Writes brauchen EDT
  (`WriteCommandAction` dispatcht selbst auf EDT). Handler bleiben off-EDT (HttpServer-Pool)
  und übergeben den Write an `WriteCommandAction`.
- **Kanonische Edit-Grenze:** Das Plugin wendet `WriteCommandAction` auf **exakt die
  übergebene Wire-Range** an (= tree-sitter-Range aus Rust) — **kein** erneutes PSI-Resolving
  der Symbol-Grenze. So ist der IDE-Pfad byte-identisch zum Headless-Pfad; der IDE-Vorteil
  beschränkt sich bewusst auf VFS-Kohärenz + Undo-Eintrag.

### 6.1 Reformat-Entkopplung

Edit und Format sind getrennt: v2a schreibt `text` verbatim in die Range. Will der Agent
formatieren, ruft er das **bestehende v1-`action=format`** nach (IDE-backed, wenn erreichbar).
Das hält IDE- und Headless-Pfad konsistent (kein Format-Divergenz-Risiko) und vermeidet
überraschende Umformatierungen außerhalb der Edit-Range.

---

## 7. Wire-Protokoll (DTO)

- **0-basiert** auf der Wire (Zeile + Spalte), wie v1 (`ctx_refactor` rechnet die
  1-basierte Tool-Eingabe genau einmal um). Pfade relativ zu `project_root`.
- **Neue Endpoints** (POST, Token-Header `X-LeanCtx-Token` wie v1):
  - `POST /replaceSymbolBody`
  - `POST /insertBeforeSymbol`
  - `POST /insertAfterSymbol`
- **Request:** `{ path, range: { start:{line,character}, end:{line,character} }, text, expected_hash? }`
  (name_path erscheint **nicht** auf der Wire — bereits in Rust zu `range` aufgelöst).
- **Response:** `{ applied: true, new_range: {start,end}, edited_text }` → Rust baut den Diff
  und wärmt den Cache aus `edited_text`.
- **Fehler** (additiv zum v1-Code-Set `{UNSUPPORTED_LANGUAGE, INDEXING, FILE_NOT_FOUND,
  POSITION_OUT_OF_RANGE, NO_SYMBOL_AT_POSITION, UNAUTHORIZED, INTERNAL}`):
  - `+CONFLICT` (expected_hash ≠ aktueller Range-Inhalt)
  - `+AMBIGUOUS_SYMBOL` (Rust-seitig vor dem HTTP-Call; trägt Kandidatenliste)
  - `+NO_SYMBOL` (Rust-seitig: name_path löst auf 0 Treffer auf — distinkt vom
    positions-basierten v1-`NO_SYMBOL_AT_POSITION`)
  - HTTP 200 für fachliche Negativfälle, 401 nur Token, 500 nur echte Exceptions. Rust mappt
    `code` → `ERROR: …`-String.

---

## 8. Body-Semantik (Serena-Parität)

| Action                 | Range / Insert-Punkt                                  | `text`-Semantik                                |
| ---------------------- | ----------------------------------------------------- | ---------------------------------------------- |
| `replace_symbol_body`  | **volle Declaration-Range** L_start–L_end (inkl. Signaturzeile) | kompletter Ersatztext der Declaration |
| `insert_before_symbol` | Insert-Punkt = Zeilenanfang L_start                   | neuer Sibling, Einrückung = Anker-Einrückung   |
| `insert_after_symbol`  | Insert-Punkt = Zeilenende L_end (Folgezeile)          | neuer Sibling, Einrückung = Anker-Einrückung   |

- Die Declaration-Range stammt aus `ctx_symbol` (tree-sitter). Einrückung wird in Rust aus
  der führenden Whitespace-Sequenz der Anker-Start-Zeile abgeleitet (§5.3 Schritt 3).
- Damit drop-in-kompatibel zu Serenas `replace_symbol_body`/`insert_before_symbol`/
  `insert_after_symbol` (gleiche Adressierungssemantik „ganzes Symbol").

---

## 9. Cache-Kohärenz (v2-Kernanforderung, v1-§9)

- **Plugin = autoritativer Writer** (WriteCommandAction → `saveDocument` auf Platte).
- **Rust** evictet den editierten Pfad aus File-/Session-Cache und **wärmt** ihn aus
  `edited_text` der Response (Roundtrip-Ersparnis).
- **mtime-Auto-Validierung** deckt jeden weiteren `ctx_read` (Re-Read ~13 tok).
- Headless-Pfad: `local_range_write` nutzt denselben `ctx_edit`-Schreibpfad → identische
  Cache-Semantik, kein Sonderfall.
- **VFS-Kohärenz (IDE offen):** Da das Plugin durch die IDE schreibt (Document→save), gibt es
  **keinen** „Datei auf Platte geändert"-Konflikt-Dialog. Der Headless-Pfad schreibt direkt
  auf Platte — relevant nur, wenn dieselbe Datei zeitgleich in einer offenen IDE ungespeichert
  geändert ist (durch `select_backend` = B in genau diesem Fall ausgeschlossen).

---

## 10. Verifikation (End-to-End)

- **Rust-Einheit (`cargo nextest run`, nie `cargo test`):**
  - name_path-Auflösung: eindeutig → `(path,range)`; mehrdeutig → `AMBIGUOUS_SYMBOL`+Kandidaten; 0 → `NO_SYMBOL`.
  - Range-/Einrück-Berechnung für insert_before/after.
  - `local_range_write`: replace + insert, `expected_hash`-Match/Mismatch (`CONFLICT`).
  - `overview`-Default-Impl liefert ohne IDE Struktur.
  - 0/1-Basierungs-Naht (Tool-Eingabe 1-basiert ↔ Wire 0-basiert ↔ Offset).
  - PathJail: name_path/Position außerhalb `project_root` → Fehler **vor** Apply.
- **Plugin (Kotlin-Unit + manuelles `runIde`-Gate, wie v1):**
  - Edit gegen Java/Kotlin-Testprojekt; Ergebnisdatei prüfen.
  - **IDE-Pfad ≡ Headless-Pfad byte-identisch** für dieselbe Eingabe (Kern-Gate der
    Konsistenz-Garantie aus §2-Entscheidung 4).
  - WriteCommandAction erzeugt **einen** Undo-Eintrag; `saveDocument` persistiert.
- **Fallback:** ohne laufende IDE → Edits über Default-Impl, kein Hänger, kein Backend-Call.

---

## 11. v2b-Forward-Pointer (eigener Folge-Spec, NICHT hier)

Nach v2a-Abschluss folgt **v2b — Refactoring-Engine** als eigenständiger Spec:
`rename_apply`, `move`, `safe_delete`, `inline` (Serena-Äquivalente `jet_brains_rename`/
`move`/`safe_delete`/`inline_symbol`). Begründung der Trennung: diese Ops nutzen IntelliJ-
`RefactoringFactory`/Refactoring-Engine, sind **Multi-File**, brauchen Konflikt-Erkennung,
Usage-Suche und Transaktionalität über mehrere Dateien — ein anderes Modell als der
Single-Range-Write von v2a. Sie kommen ebenfalls additiv als `LspBackend`-Methoden, haben
aber (anders als v2a) **keinen** sinnvollen verlustfreien Headless-Default → dort gilt wieder
das v1-Muster „Backing B erforderlich, sonst `Err`". Erst nach v2b ist Serena auch als
Edit-Engine vollständig entbehrlich (§13.4 der v1-Spec).

**Out of scope (bleibt):** `serena.jet_brains_debug` (kein Code-Intelligence-Äquivalent),
DB-/Run-/SQL-/Terminal-Tools des JetBrains-MCP (deckt lean-ctx anders ab). Serena-Memory-Ops
(`write/read/list/delete/edit/rename_memory`) sind bereits durch `ctx_knowledge` abgedeckt.

---

## 12. Branch- & Commit-Strategie

- Fortführung auf `feat-jetbrains-plugin` (Muster v1-§12.3): **ein Commit pro Phase** nach
  erfülltem Phasen-Gate, kein Squash während der Entwicklung. Direkt auf dem Branch, **kein
  worktree** (Projekt-Rule).
- Finaler Merge nach `main` via Squash-Merge-PR (am Schluss).
- **Schema-Drift-Gate:** `ctx_refactor`-Schema-Änderung → `docs/reference/generated/mcp-tools.md`
  via `cargo run --example gen_docs --features dev-tools` (cwd=rust) regenerieren
  (Drift-Test `generated_reference…`).
