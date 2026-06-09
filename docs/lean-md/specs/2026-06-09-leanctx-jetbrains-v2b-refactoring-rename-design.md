# Design-Spec: lean-ctx JetBrains v2b — Refactoring-Engine (Serena-Ablösung, Edit-Klasse B), exemplarisch `rename`

| Feld             | Wert                                                                                              |
| ---------------- | ------------------------------------------------------------------------------------------------- |
| Status           | Draft (Design genehmigt 2026-06-09)                                                               |
| Datum            | 2026-06-09                                                                                         |
| Vorhaben         | Multi-File-Refactoring-Engine über das JetBrains-Plugin + lean-ctx; erste Op `rename`             |
| Scope            | `rename_preview` + `rename_apply` (Two-Phase) — Engine etablieren; `move`/`safe_delete`/`inline` → v2c |
| Basis-Spec       | `docs/lean-md/specs/2026-06-08-leanctx-jetbrains-v2a-symbol-edits-design.md` (v2a, §11 v2b-Pointer) |
| Branch           | `feat-jetbrains-plugin` (Fortführung, Muster v1-§12.3 — ein Commit pro Phase)                     |
| Nächster Schritt | `superpowers:writing-plans` (Implementierungsplan)                                                |

---

## 1. Context — Warum v2b jetzt

v2a (Klasse A — Symbol-Body-Edits: `replace_symbol_body`, `insert_before_symbol`,
`insert_after_symbol` + `overview`-Headless-Fallback) ist abgeschlossen und gemerged
(Commit `cf523517`). Damit ist Serenas **Body-Edit**-Klasse abgelöst. Was bleibt: Serenas
**Refactoring-Engine** (`rename`, `move`, `safe_delete`, `inline`) — die letzte Lücke, bevor
Serena auch als Edit-Engine vollständig entbehrlich ist (v1-§13.4).

Die v2a-Spec hat diese Klasse B explizit als **eigenständigen Folge-Spec** ausgewiesen
(v2a-§11): andere Op-Natur als der Single-Range-Write von v2a — **Multi-File**, semantische
Usage-Suche, Konflikt-Erkennung, Transaktionalität über mehrere Dateien, und (anders als v2a)
**kein** sinnvoller verlustfreier Headless-Default.

**Zuschnitt-Entscheidung (User, 2026-06-09):** v2b **etabliert die Engine** und implementiert
exemplarisch genau **eine** Op — `rename`, das mit Abstand häufigste und wertvollste
Refactoring. `move`/`safe_delete`/`inline` folgen als schlankes **v2c** auf derselben Engine
(§11). Begründung: Das gesamte Architektur-Risiko (Multi-File-Transaktion, Konflikt-Gate,
Two-Phase-Protokoll, Multi-File-Cache-Kohärenz) trägt die **erste** Op; danach sind die
übrigen inkrementell. v2b isoliert dieses Risiko.

---

## 2. Getroffene Entscheidungen (User, 2026-06-09)

| #  | Frage                  | Entscheidung                                                                                                                                                  |
| -- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1  | v2b-Zuschnitt          | **Engine + `rename` zuerst.** `move`/`safe_delete`/`inline` als eigenes **v2c** auf derselben Engine (§11). Risiko-Isolation in der ersten Op.                 |
| 2  | Apply-Modell           | **Two-Phase** (immer): `rename_preview` liefert Plan, `rename_apply` schreibt. Bewusster Bruch mit v2a-Direkt-Apply — Multi-File-Blast-Radius ist vorab unsichtbar. |
| 3  | Konflikt-Policy        | **Plan meldet, Apply blockt.** `preprocessUsages`-Konflikte erscheinen im Plan; `rename_apply` blockt per Default (`CONFLICT`), `force=true` als bewusster Opt-out. |
| 4  | Plan→Apply-Übergabe    | **Stateless `plan_hash` (BLAKE3, Rust-zentral).** Kein Server-State. Apply wiederholt die Usage-Suche, bildet den Hash neu, vergleicht → Mismatch = `CONFLICT` (TOCTOU). |
| 5  | Action-Form            | **Zwei explizite Actions** `rename_preview` + `rename_apply` in `ctx_refactor` (kein neues Tool — wie v2a).                                                    |
| 6  | Action-Namen           | **`rename_preview` / `rename_apply`** — `preview`/`apply` als selbsterklärender lesen-vs-schreiben-Gegensatz (Terraform-/Refactoring-Konvention).             |
| 7  | Sprach-Scope           | **Generisch** über `RenameProcessor` (jede PSI-Sprache, Plugin gibt durch). **Akzeptanz-Gate: Kotlin** (Primär); Java optionaler Sekundär-Check.             |

---

## 3. Schlüssel-Befund: der Architektur-Bruch ggü. v2a

Der gesamte v2b-Schnitt beruht auf **einem** Unterschied zu v2a, der das Backend-Modell
umkehrt:

- **v2a:** Die Edit-Range ist aus dem **tree-sitter-Symbol-Index** (Rust) auflösbar. Der
  Apply ist ein lokaler Range-Write → **headless möglich**; das IDE-Plugin ist nur Bonus
  (VFS-Kohärenz + Undo).
- **v2b:** Ein `rename` braucht **alle Usages** des Ziel-Symbols. Das ist eine **semantische**
  Suche (Scope-/Typ-/Override-Auflösung), die tree-sitter prinzipiell nicht leisten kann —
  **nur das IDE liefert sie** (`RenameProcessor.findUsages`). ∴ Schon **`rename_preview`
  braucht zwingend Backing B** (laufende IDE). Es gibt **keinen** verlustfreien
  Headless-Default — headless → `Err BACKEND_REQUIRED` (v1-Muster „Backing B erforderlich,
  sonst Err", §11-konform).

**Rollenverteilung daraus:**

| Seite      | Aufgabe in v2b                                                                                                            |
| ---------- | ------------------------------------------------------------------------------------------------------------------------ |
| **Rust**   | name_path → Ziel-Symbol (`resolve_name_path`, **reuse v2a**), PathJail, `plan_hash` bilden/prüfen, Konflikt-Gate, Multi-File-Cache-Kohärenz, Diff-Bau |
| **Plugin** | `RenameProcessor.findUsages` (semantische Usage-Suche), `preprocessUsages` (Konflikt-Sammlung), transaktionaler Apply (`WriteCommandAction { RenameProcessor.run() }`) |

`resolve_name_path` aus v2a (`ctx_refactor.rs:230`) wird wiederverwendet, um das **Ziel**-Symbol
zu adressieren — name_path primär, Position-Fallback. Die **Usages** kommen ausschließlich vom
Plugin; Rust indexiert sie nicht.

---

## 4. Architektur — Schichtung & Two-Phase-Fluss

```
PHASE 1 — rename_preview(name_path | path+line[+character], new_name)
   │
   ├─ Rust: resolve_name_path → (target_path, target_range)   [reuse v2a, backend-unabh.]
   │        + PathJail(jail_path) auf den aufgelösten Pfad     ── VOR jedem Backend-Call
   │
   ├─ select_backend:  Backing B erreichbar?  ── nein ─→  Err BACKEND_REQUIRED
   │                          │ ja
   │                          ▼
   ├─ Plugin POST /renamePreview:
   │     RenameProcessor(project, element, new_name)
   │       .findUsages()            → alle Usage-Stellen (Multi-File)
   │       .preprocessUsages()      → Konflikte (Kollision/Sichtbarkeit/Override)
   │     ← { usages:[{path,range,context}…], conflicts:[{path,range,message}…] }
   │
   └─ Rust: plan_hash = BLAKE3( norm(usages) ⊕ Ist-Inhalt je usage-Stelle )
         ← Agent: { target, new_name, affected_files, usage_count,
                    conflicts, plan_hash, diff_preview }

PHASE 2 — rename_apply(name_path | …, new_name, plan_hash[, force=false])
   │
   ├─ Rust: resolve_name_path (erneut) + PathJail
   ├─ select_backend:  Backing B?  ── nein ─→  Err BACKEND_REQUIRED
   ├─ Plugin POST /renamePreview (erneut)  → usages + conflicts
   ├─ Rust-Gates (in dieser Reihenfolge):
   │     (a) plan_hash neu bilden, vergleichen   ≠ → CONFLICT (TOCTOU: Quelle änderte sich)
   │     (b) conflicts ≠ ∅  ∧  ¬force            → CONFLICT (Konflikt-Gate)
   │
   ├─ Plugin POST /renameApply:
   │     WriteCommandAction.runWriteCommandAction(project) {
   │         RenameProcessor(…, force=…).run()      // EIN Undo-Eintrag
   │     }
   │     je geänderter Datei: PsiDocumentManager.commitDocument + FileDocumentManager.saveDocument
   │     ← { applied:true, changed_paths:[…] }
   │
   └─ Rust: evict + rewarm je changed_path (mtime-Auto-Validierung),
         Multi-File-Diff/Delta bauen
         ← Agent: { applied:true, changed_paths, diff }
```

**Kernprinzip (unverändert ggü. v1/v2a):** `select_backend` ist der einzige
IDE-vs-headless-Entscheidungspunkt. Neu in v2b: der headless-Zweig führt **nicht** zu einem
Default-Apply (wie v2a), sondern zu `BACKEND_REQUIRED` — weil es keine verlustfreie
Headless-Semantik für Multi-File-rename gibt.

**Warum die Usage-Suche in Phase 2 wiederholt wird:** Der `plan_hash`-Vergleich ist nur
aussagekräftig, wenn Phase 2 den **aktuellen** Zustand gegen den Phase-1-Hash hält. Das ist
der TOCTOU-Schutz und ersetzt jeden Server-State (Entscheidung 4). Die Wiederholung ist der
bewusste Preis für Statelessness; ein RenameProcessor-`findUsages` ist günstig ggü. der
Sicherheit, die es kauft.

---

## 5. Rust-Seite

### 5.1 `ctx_refactor` — zwei neue Actions

- **Neue Actions** (kein neues Tool — vermeidet Nachziehen in `tool_profiles.rs`/
  `dynamic_tools.rs`/`workflow/types.rs`, exakt wie v2a): `rename_preview`, `rename_apply`.
  Match-Block + Hilfetext erweitern; Schema über die **eine** Tool-Registry
  `tool_defs::tool_def(...)` (kein zweites handgepflegtes Schema; Drift-Regression-Test deckt es ab).
- **Parameter:**
    - `rename_preview`: `name_path` (primär) **oder** `path`+`line`(+`character`) (Fallback); `new_name`.
    - `rename_apply`: dieselben Adressierungs-Parameter + `new_name` + `plan_hash` (required) + `force` (optional, default `false`).
- **Auflösungsschritt (beide Actions, vor dem Backend-Dispatch):**
    1. `resolve_name_path` (reuse v2a) → `(target_path, target_range)`. >1 → `AMBIGUOUS_SYMBOL`
       mit Kandidatenliste; 0 → `NO_SYMBOL` (beide reuse v2a).
    2. **PathJail:** aufgelösten Pfad durch `core::path_resolve::resolve_tool_path` → `jail_path`.
       Außerhalb `project_root` → Fehler **vor** jedem Backend-Call (bei Multi-File-Writes
       doppelt kritisch — der Apply kann *weitere* Dateien berühren; siehe §5.4 Jail-Hinweis).

### 5.2 `plan_hash` — Multi-File-Integritätsguard (BLAKE3, Rust-zentral)

Direkte Verallgemeinerung des v2a-`expected_hash` auf Multi-File. **Verbindlich aus v2a-§7:**
Integritäts-Guards leben in Rust; das Plugin hasht nicht und sieht `plan_hash` nie auf der Wire.

- **Bildung (`rename_preview`):** `plan_hash = hash_hex( canonical(usages) )`, wobei
  `canonical(usages)` deterministisch über die nach `(path, range)` **sortierten** Usage-Stellen
  plus deren **Ist-Inhalt** (der jeweils ersetzte Range-Text) serialisiert. `hash_hex` ist
  BLAKE3 (`core::hasher::hash_hex`).
- **Prüfung (`rename_apply`):** Phase 2 holt die Usages erneut, bildet `plan_hash` neu,
  vergleicht gegen den übergebenen → Mismatch ⇒ `CONFLICT` (Quelle/Usages änderten sich
  zwischen Preview und Apply). Kein Server-State, kein Lifetime/Eviction-Problem.
- **Konflikt-Gate:** zusätzlich `conflicts ≠ ∅ ∧ ¬force ⇒ CONFLICT`. `force=true` reicht
  das `force`-Flag an den `RenameProcessor` durch (IntelliJ „proceed anyway").

### 5.3 `LspBackend`-Trait — zwei neue Methoden (Default = `Err`)

Additiv zu v1/v2a. **Anders als v2a** (Default = lokaler Apply) ist der Default hier ein
**Fehler**, weil es keinen verlustfreien Headless-Pfad gibt (§3):

```rust
// rust/src/lsp/backend.rs (Erweiterung)
fn rename_preview(&mut self, req: RenameQuery) -> Result<RenamePlan, BackendError> {
    Err(BackendError::BackendRequired)   // headless: keine semantische Usage-Suche
}
fn rename_apply(&mut self, req: RenameApply) -> Result<RenameResult, BackendError> {
    Err(BackendError::BackendRequired)
}
```

- **`RenameQuery`**: `{ abs_path, target_range, new_name }` (Ziel bereits aufgelöst — der Trait
  sieht nie einen `name_path`, exakt wie v2a).
- **`RenamePlan`**: `{ usages: Vec<UsageSite>, conflicts: Vec<Conflict> }`.
- **`RenameApply`**: `{ abs_path, target_range, new_name, force }`.
- **`RenameResult`**: `{ applied: bool, changed_paths: Vec<String> }`.
- `LspClient` (Backing A / rust-analyzer) **erbt den `Err`-Default** → headless ohne IDE liefert
  sauberes `BACKEND_REQUIRED`, keinen Hänger, keinen Apply.

### 5.4 Änderungsstellen (Rust)

| Datei                                       | Änderung                                                                                          |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `rust/src/tools/ctx_refactor.rs`            | +2 Actions (`rename_preview`/`rename_apply`), `plan_hash` bilden/prüfen, Konflikt-Gate, Multi-File-Cache-Kohärenz, Multi-File-Diff; reuse `resolve_name_path`/PathJail aus v2a |
| `rust/src/tools/registered/ctx_refactor.rs` | Schema-Erweiterung über `tool_def(...)` (`new_name`, `plan_hash`, `force`)                         |
| `rust/src/lsp/backend.rs`                   | +`rename_preview`/`rename_apply`-Trait-Methoden (Default `Err BackendRequired`) + `RenameQuery`/`RenamePlan`/`RenameApply`/`RenameResult`/`UsageSite`/`Conflict`-Typen |
| `rust/src/lsp/jetbrains_backend.rs`         | HTTP-Override der 2 Methoden (`/renamePreview`, `/renameApply`)                                    |
| `rust/src/lsp/client.rs`                    | erbt `Err`-Default (keine Änderung außer ggf. Trait-Re-Export)                                     |

**Multi-File-Jail-Hinweis:** Anders als v2a (genau eine Datei) kann ein rename *mehrere*
Dateien berühren, deren Pfade Rust **vor** dem Apply nicht kennt (sie stehen erst im
Preview-Ergebnis). Konsequenz: PathJail wird **zweistufig** angewandt — (a) auf das
aufgelöste Ziel-Symbol vor Phase 1, und (b) Rust prüft jeden `path` der zurückgemeldeten
`usages`/`changed_paths` gegen `project_root`, bevor Cache-Eviction/Rewarm läuft. Ein
Usage-Pfad außerhalb `project_root` → Fehler (das Plugin darf nichts außerhalb der Jail
zurückmelden).

---

## 6. Plugin-Seite (Kotlin) — additiv

Integriert additiv in `com.leanctx.plugin` (koexistiert mit v1/v2a, ersetzt nichts).

- **`psi/SymbolRefactorer.kt`** (neu): kapselt die `RefactoringFactory`/`RenameProcessor`-Naht.
    - Ziel-`PsiElement` aus der übergebenen Wire-Range via `PsiLocator` (dieselbe
      0-basiert↔offset-Logik wie v1/v2a).
    - **Preview:** `RenameProcessor(project, element, newName, /*searchInComments*/…, /*searchTextOccurrences*/…)`
      → `findUsages()` (Array<UsageInfo>) + `preprocessUsages(refUsages)` zum Sammeln der
      `conflicts` (IntelliJ liefert sie als `MultiMap<PsiElement, String>` — wird zu
      `[{path,range,message}]` serialisiert). **Kein** Write in der Preview-Phase.
    - **Apply:** `WriteCommandAction.runWriteCommandAction(project) { renameProcessor.run() }`
      → IntelliJ führt die Multi-File-Transaktion als **einen** Undo-Eintrag aus. `force` steuert,
      ob Konflikte den Lauf abbrechen (im headless/Plugin-Pfad gibt es keinen Dialog — bei
      `force=false` und vorhandenen Konflikten bricht bereits das **Rust**-Gate ab, §5.2, sodass
      der Apply-Call gar nicht erst abgesetzt wird).
    - Nach dem Lauf je betroffener Datei: `PsiDocumentManager.commitDocument` +
      `FileDocumentManager.saveDocument` → **schreibt auf Platte**, damit lean-ctx (liest von
      Platte) das Ergebnis sieht.
    - **Kein Auto-Reformat** (entkoppelt, wie v2a-§6.1) — Formatierung via bestehendem
      v1-`action=format` nachziehbar.
- **`endpoint/RefactorHandlers.kt`** (neu): `renamePreview` / `renameApply`, registriert im
  `RequestRouter` (Token-Check wie v1).
- **Threading:** `findUsages`/`preprocessUsages` unter `ReadAction` (off-EDT, wie v1-Reads);
  der Apply über `WriteCommandAction` (dispatcht selbst auf EDT). Handler bleiben off-EDT
  (HttpServer-Pool).
- **Kanonische Refactoring-Grenze:** Das Plugin ist die **alleinige** Quelle der Usage-Menge
  (semantisch, IDE-genau). Anders als v2a gibt es hier **keine** tree-sitter-Spiegelung der
  Edit-Stellen in Rust — der `plan_hash` hasht nur den *Ist-Inhalt* der vom Plugin gemeldeten
  Stellen, nicht eine zweite, unabhängige Range-Berechnung.

---

## 7. Wire-Protokoll (DTO) — additiv zu v1/v2a

- **0-basiert** auf der Wire (Zeile + Spalte), wie v1/v2a. Pfade relativ zu `project_root`.
- **Neue Endpoints** (POST, Token-Header `X-LeanCtx-Token` wie v1):
    - `POST /renamePreview` — Request `{ path, range:{start,end}, new_name, search_comments?, search_text_occurrences? }`
      → Response `{ usages:[{path, range:{start,end}, context?}], conflicts:[{path, range:{start,end}, message}] }`.
    - `POST /renameApply` — Request `{ path, range:{start,end}, new_name, force }`
      → Response `{ applied:true, changed_paths:[…] }`.
- **`plan_hash` erscheint NICHT auf der Wire** — reine Rust-Logik (v2a-§7-Regel verbindlich:
  „neue mutierende Endpoints bekommen keinen plugin-seitigen Hash — Integritäts-Guards leben in
  Rust"). Das Plugin hasht nicht.
- **Fehler** (additiv zum v1/v2a-Set):
    - `+BACKEND_REQUIRED` — headless / Backing B nicht erreichbar (Rust-seitig, vor jedem Call).
    - `+CONFLICT` — Doppelbelegung: `plan_hash`-Mismatch (TOCTOU) **oder** geblockte
      Refactoring-Konflikte (`conflicts≠∅ ∧ ¬force`). Beide Rust-seitig erzwungen.
    - **Reuse v2a:** `AMBIGUOUS_SYMBOL` (Kandidatenliste), `NO_SYMBOL` (name_path → 0 Treffer).
    - HTTP 200 für fachliche Negativfälle, 401 nur Token, 500 nur echte Exceptions. Rust mappt
      `code` → `ERROR: …`-String.

---

## 8. rename-Semantik (Serena-Parität)

| Action           | Eingabe                                            | Wirkung                                                                 |
| ---------------- | ------------------------------------------------- | ----------------------------------------------------------------------- |
| `rename_preview` | Ziel-Symbol (`name_path`/Position) + `new_name`   | **kein** Write; liefert betroffene Dateien, Usage-Count, Konflikte, `plan_hash`, Diff-Vorschau |
| `rename_apply`   | Ziel + `new_name` + `plan_hash` [+ `force`]       | benennt Deklaration **und alle Usages** um (Multi-File, ein Undo)        |

- Adressierung des **Ziel**-Symbols identisch zu v2a (name_path primär, Position-Fallback).
- Damit drop-in-kompatibel zu Serenas `jet_brains_rename` (gleiche Semantik „Symbol +
  alle Referenzen"), plus expliziter Preview-Schritt, den Serena nicht hat.

---

## 9. Cache-Kohärenz (Multi-File)

Direkte Verallgemeinerung von v2a-§9 auf mehrere Dateien:

- **Plugin = autoritativer Writer** (`RenameProcessor.run()` im `WriteCommandAction` →
  `saveDocument` je Datei auf Platte).
- **Rust** evictet **jeden** `changed_path` aus File-/Session-Cache. Anders als v2a (eine
  Datei, `edited_text` in der Response) liefert das Plugin bei Multi-File nur die
  **Pfadliste** (`changed_paths`) — Rust re-readt sie über die **mtime-Auto-Validierung**
  (Re-Read ~13 tok je unveränderter, voller Read je geänderter Datei). Kein `edited_text`-Rewarm
  pro Datei (bei vielen Dateien zu groß) — der Multi-File-Diff wird aus den frischen Reads
  gebaut.
- **VFS-Kohärenz (IDE offen):** Da das Plugin durch die IDE schreibt, kein „Datei auf Platte
  geändert"-Konflikt-Dialog. (Der reine Headless-Schreibpfad existiert in v2b nicht —
  `BACKEND_REQUIRED`.)

---

## 10. Verifikation (End-to-End)

- **Rust-Einheit (`cargo nextest run`, nie `cargo test`):**
    - `plan_hash`: deterministische Bildung über sortierte Usages+Inhalt; Match → Apply läuft;
      Mismatch → `CONFLICT`.
    - Konflikt-Gate: `conflicts≠∅ ∧ ¬force → CONFLICT`; `force=true` → durchgereicht.
    - Ziel-Auflösung: name_path eindeutig/mehrdeutig/0 → `(path,range)`/`AMBIGUOUS_SYMBOL`/`NO_SYMBOL` (reuse v2a-Tests).
    - headless / Backing B nicht erreichbar → `BACKEND_REQUIRED` (kein Apply, kein Hänger).
    - PathJail: Ziel **und** zurückgemeldete Usage-Pfade außerhalb `project_root` → Fehler vor Apply/Cache-Mutation.
    - 0/1-Basierungs-Naht (Tool-Eingabe 1-basiert ↔ Wire 0-basiert ↔ Offset).
- **Plugin (Kotlin-Unit + manuelles `runIde`-Gate, wie v1):**
    - **Akzeptanz-Gate Kotlin** (Primär): rename eines Symbols mit Usages über **mehrere**
      Dateien im Kotlin-Testprojekt → alle Deklarationen+Referenzen korrekt umbenannt;
      Ergebnisdateien prüfen.
    - `preprocessUsages` meldet einen konstruierten Konflikt (z.B. Namenskollision) korrekt im Preview.
    - `RenameProcessor.run()` erzeugt **einen** Undo-Eintrag; `saveDocument` persistiert je Datei.
    - Java optionaler Sekundär-Check (nicht akzeptanzkritisch).
- **Fallback:** ohne laufende IDE → `BACKEND_REQUIRED` in beiden Phasen, kein Apply.

---

## 11. v2c-Forward-Pointer (eigener Folge-Spec, NICHT hier)

Nach v2b-Abschluss folgt **v2c** auf **derselben Engine**: `move`, `safe_delete`, `inline`
(Serena-Äquivalente `jet_brains_move`/`safe_delete`/`inline_symbol`). Sie nutzen dieselbe
v2b-Infrastruktur — Two-Phase (`*_preview`/`*_apply`), `plan_hash`-Guard, Rust-zentrales
Konflikt-Gate, Multi-File-Cache-Kohärenz, `BACKEND_REQUIRED`-Headless-Verhalten — und sind
deshalb inkrementell. Eigenheiten je Op (in v2c zu spezifizieren):

- **`move`**: braucht ein **Ziel** (Ziel-Package/-Datei) als Zusatzparameter; Ziel-Auflösung
  + Ziel-Jail.
- **`safe_delete`**: Preview liefert die **blockierenden Usages** (das „safe" = Abbruch bei
  verbleibenden Referenzen, sofern nicht `force`).
- **`inline`**: Usage-**Substitution** (Body an die Aufrufstellen), nicht nur Umbenennung —
  die semantisch komplexeste Op.

Erst nach v2c ist Serena auch als Edit-Engine vollständig entbehrlich (v1-§13.4).

**Out of scope (bleibt, wie v2a-§11):** `serena.jet_brains_debug` (kein
Code-Intelligence-Äquivalent), DB-/Run-/SQL-/Terminal-Tools des JetBrains-MCP. Serena-Memory-Ops
sind durch `ctx_knowledge` abgedeckt.

---

## 12. Branch- & Commit-Strategie

- Fortführung auf `feat-jetbrains-plugin` (Muster v1-§12.3): **ein Commit pro Phase** nach
  erfülltem Phasen-Gate, kein Squash während der Entwicklung. Direkt auf dem Branch, **kein
  worktree** (Projekt-Rule).
- Finaler Merge nach `main` via Squash-Merge-PR (am Schluss).
- **Schema-Drift-Gate:** `ctx_refactor`-Schema-Änderung → `docs/reference/generated/mcp-tools.md`
  via `cargo run --example gen_docs --features dev-tools` (cwd=rust) regenerieren
  (Drift-Test `generated_reference…`). Zusätzlich `docs/reference/appendix-mcp-tools.md`
  (human tool map) um die zwei Actions ergänzen.

---

## 13. Bewusst NICHT in v2b (YAGNI)

- **Kein** Blast-Radius-Limit (`rename_apply` bei >N Dateien): der Preview-Schritt macht den
  Blast-Radius explizit sichtbar, bevor geschrieben wird — eine zusätzliche Schwelle wäre
  redundant.
- **Kein** Server-State / `plan_id` (Entscheidung 4: stateless `plan_hash`).
- **Kein** Headless-rename (kein verlustfreier Default möglich, §3).
- **Kein** plugin-seitiges Hashing (v2a-§7 verbindlich: Integritäts-Guards in Rust).
- **Kein** Auto-Reformat im Apply (entkoppelt, v2a-§6.1).
