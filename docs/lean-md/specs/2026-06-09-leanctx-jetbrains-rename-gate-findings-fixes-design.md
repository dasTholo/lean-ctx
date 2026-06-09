# Design: runIde-Rename-Gate — Findings-Fixes + Re-Verifikation

Datum: 2026-06-09
Branch: `feat-jetbrains-plugin`
Bezug:
- Runbook: `docs/lean-md/runbooks/runide-rename-gate.md`
- Harness-Spec: `docs/lean-md/specs/2026-06-09-leanctx-jetbrains-runide-rename-gate-harness-design.md`
- Live-Ergebnisse (durabel): `ctx_knowledge` key `runide-gate-live-results`

## Kontext

Der erste Live-Lauf des runIde-Rename-Gate (Sandbox IU-2026.1.3, Fixture
`tmp/runide-rename-gate`, Aufruf via `lean-ctx call ctx_refactor --project-root
<fix> --json …`) hat den sicherheitskritischen v2b-Pfad bestätigt
(#1 Preview cross-file, #2 Apply + atomarer Undo, #3 TOCTOU-`CONFLICT`,
#7 `BACKEND_REQUIRED`), aber fünf Findings aufgedeckt. Dieses Dokument
spezifiziert deren Fixes und die anschließende Re-Verifikation gegen dasselbe
Fixture.

Kern-Erkenntnis: Die Findings A und B sind **Plugin-Logik** (nicht Fixture),
E ist **kein Code-Bug** (Gate existiert, nur Test-Lücke), C ist Doku, D ist ein
Rust-Anzeige-Zähler.

## Findings, Wurzelanalyse, Fix

### A — Headless-Konflikt öffnet modalen Dialog (Plugin)

**Symptom:** `rename_preview` mit kollidierendem Ziel (`Widget`→`Gadget`, da
`Gadget.kt` existiert) öffnet in der Sandbox einen modalen „Conflicts
detected"-Dialog → EDT blockiert → `timeout: global` + „Cannot execute
background write action in 10 seconds". Alle Unit-Tests waren grün.

**Wurzel:** `SymbolRefactorer.preview()` ruft `CapturingProcessor.collectConflicts()`
→ `preprocessUsages(Ref.create(usages))`. `RenameProcessor.preprocessUsages()`
ruft die `showConflicts`-Override **nicht** auf — es baut die Konflikt-`MultiMap`
selbst (`RenameUtil.addConflictDescriptions` + `findExistingNameConflicts`) und
zeigt im Nicht-Test-Modus **direkt** `ConflictsDialog.showAndGet()`. Im
Unit-Test ist `ApplicationManager.getApplication().isUnitTestMode() == true` →
`ConflictsInTestsException` statt Dialog. In der Sandbox (kein unitTestMode) →
Dialog. Das ist eine Test/Sandbox-Modus-Divergenz.

**Fix (A1 — dry-run conflict collection):**
- `preview()` ruft **nicht mehr** `preprocessUsages()`. Stattdessen werden die
  Konflikte mit denselben Primitiven gesammelt, die `preprocessUsages` intern
  nutzt — ohne UI-Zweig:
  ```kotlin
  val mm = MultiMap<PsiElement, String>()
  RenameUtil.addConflictDescriptions(usages, mm)
  RenamePsiElementProcessor.forElement(element)
      .findExistingNameConflicts(element, newName, mm, processor.allRenames)
  ```
  Läuft in der bestehenden EDT-`invokeAndWait`-Naht (PSI-Lesezugriff +
  Read-Action), aber ohne `showAndGet()`.
- `CapturingProcessor`: `showConflicts`-Override und `collectConflicts()`
  **entfallen**. Ein schmaler Accessor exponiert das geschützte `allRenames`
  (gleiche Subklassen-Technik, mit der bereits `findUsages()` exponiert wird).
- `apply()`: weg von `processor.run()` (ruft intern `preprocessUsages` → Dialog
  bei force + IDE-seitigem Konflikt) hin zu:
  ```kotlin
  CommandProcessor.getInstance().executeCommand(project, {
      WriteCommandAction.runWriteCommandAction(project) {
          processor.performRefactoring(usages)
      }
      FileDocumentManager.getInstance().saveAllDocuments()
  }, "Rename", null)
  ```
  `performRefactoring(usages)` führt die Mutation aus, ohne `preprocessUsages`
  und ohne Dialog; der explizite `executeCommand`-Block stellt den **atomaren
  Single-Undo** sicher.

**Verifikationspflicht:** Case #2 (Multi-File-Rename + **ein** Undo-Eintrag)
muss nach der `apply()`-Umstellung erneut grün sein — das ist die Regression,
die die Umstellung von `run()` auf `performRefactoring()` absichert.

**Prinzip:** Niemals einen UI-zeigenden Platform-Pfad (Conflicts-/Progress-Dialog)
aus dem eingebetteten Server-Thread betreten. Konflikte werden als Daten
gesammelt und über die bestehende `conflictDtos`-Naht zurückgegeben; die
Konflikt-**Entscheidung** (block/force) trifft weiterhin der Rust-Gate.

### B — UNSUPPORTED_LANGUAGE liefert NO_SYMBOL (Plugin + Test-Aufruf)

**Symptom:** `rename_preview` mit `name_path=notes` (Ziel in `notes.txt`)
liefert `NO_SYMBOL` statt `UNSUPPORTED_LANGUAGE`.

**Wurzel (zweiteilig):**
1. Der beobachtete `NO_SYMBOL` kam aus **Rust** `resolve_name_path("notes")`
   (`ctx_refactor.rs`), das kein indexiertes Symbol fand — das Plugin wurde nie
   erreicht.
2. `SymbolRefactorer.resolveTarget` wirft selbst nur `NO_SYMBOL`; es gibt kein
   Sprach-Gate.

**Fix:**
- **Plugin:** Sprach-Gate in `resolveTarget`, **vor** `findElementAt`:
  ```kotlin
  val lang = file.language
  if (lang == PlainTextLanguage.INSTANCE
      || file.fileType == PlainTextFileType.INSTANCE
      || LanguageRefactoringSupport.getInstance().forLanguage(lang) == null) {
      throw BackendException("UNSUPPORTED_LANGUAGE", "rename not supported for ${lang.id}")
  }
  ```
  So kommt der Token zuverlässig vor `NO_SYMBOL`. Das Gate ist **index-frei**
  (reine `LanguageExtension`-Abfragen) und funktioniert daher auch im Dumb Mode.
- **Plugin (Doku-konforme Navigation):** Da `resolveTarget` ohnehin angefasst
  wird, die Leaf→Deklaration-Suche von manuellem
  `generateSequence(at){it.parent}` auf den dokumentierten
  `PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, /*strict=*/false)`
  umstellen — mit beibehaltenem `name != null`-Nachfilter (den `getParentOfType`
  nicht leistet). Kein Verhaltensbruch.
- **Test:** über den `path`+`line`-Fallback statt `name_path` (existiert in
  `resolve_rename_target`, `ctx_refactor.rs:354-367`):
  `{"action":"rename_preview","path":"notes.txt","line":1,"new_name":"x"}`.

### C — Runbook Case #3 ungenau (Doku)

**Wurzel:** `plan_hash` (`ctx_refactor.rs:318-343`) hasht ausschließlich die
**usage-Stellen** (`path | range | text` jeder Referenz, `text` via
`usage_range_text` von Disk gelesen) — **nicht** die Deklarations-Datei. Eine
Mutation an `Widget.kt` (Deklaration) ändert den Hash nicht; nur eine Mutation an
einer usage-Stelle (`Usage.kt`) löst `CONFLICT` aus.

**Fix:** `docs/lean-md/runbooks/runide-rename-gate.md`:
- Case #3 von „`Widget.kt` zwischen #1 und apply ändern" auf „eine
  **usage-Stelle** in `Usage.kt` ändern (z.B. eine Zeile davor einfügen → die
  usage-range verschiebt sich)" korrigieren, mit kurzer Erklärung der
  `plan_hash`-Semantik.
- Case #4 annotieren: nach Fix A erwartet headless `CONFLICT`-Token (kein Dialog).
- Case #6 annotieren: Aufruf über `path:"notes.txt"`-Fallback, erwartet
  `UNSUPPORTED_LANGUAGE`.
- Case #5 annotieren: manuell/best-effort (Invalidate-Caches oder großes
  Projekt); Gate ist unit-getestet.

### D — Preview-`files`-Zähler (Rust)

**Wurzel:** `render_rename_preview` (`ctx_refactor.rs:411-413`) bildet die
`files`-Menge nur aus `plan.usages` (= Referenz-Stellen, also `Usage.kt`). Die
Deklarations-Datei `Widget.kt` fehlt → `files: 1`. Der `apply`-Pfad zählt korrekt
`2`, weil das Plugin die Deklarations-Datei explizit zu `changed_paths`
hinzufügt.

**Fix:** Die `files`-Menge um die Deklarations-Datei ergänzen:
`distinct(usages.paths ∪ {query.rel_path})` → `files: 2`, deckungsgleich mit
`apply`. `plan_hash` bleibt unberührt (rein kosmetischer Anzeige-Zähler).
Bestehenden Unit-Test `preview_renders_plan_hash_and_files` anpassen.

### E — INDEXING headless nicht getriggert (Test-Absicherung + Robustheit)

**Wurzel:** Das Gate existiert (`PsiLocator.kt:79-81`,
`DumbService.getInstance(project).isDumb` → `BackendException("INDEXING")`). Beim
Mini-Fixture ist das Re-Index-Fenster zu kurz, um headless ein `preview` während
Dumb Mode abzufeuern. Zusätzlich besteht eine Race: `resolveTarget`
(`findElementAt` + Parent-Walk) funktioniert laut Doku im Dumb Mode, aber das
nachfolgende `findUsages()` (index-gestützt) kann `IndexNotReadyException`
werfen, falls die Indizierung **nach** dem `isDumb`-Check startet.

**Fix:**
- **Catch-Net:** den index-berührenden Block (Resolve/`findUsages`) zusätzlich in
  `catch (IndexNotReadyException) → BackendException("INDEXING")` wickeln. Das
  `isDumb`-Früh-Gate bleibt (Normalfall); das Net deckt die Race.
- **Unit-Test:** `DumbService` dumb erzwingen → erwarte `BackendException` mit
  Code `INDEXING` aus `inSmartReadAction` bzw. dem Catch-Net.
- **Runbook:** Case #5 als manuell/best-effort annotieren (siehe C).

## Re-Verifikation gegen `tmp/runide-rename-gate/`

Nach den Fixes: `cargo build` (cwd `rust`) + Plugin neu bauen, Fixture via
`bash ./scripts/runide-gate-setup.sh` zurücksetzen, `runIde
--args="<abs>/tmp/runide-rename-gate"`, dann die 7 Cases mit den **korrigierten**
Aufrufen:

| # | Aufruf | Erwartung |
|---|--------|-----------|
| 1 | `rename_preview Widget→Renamed` | `files: 2` (Fix D), `plan_hash` gesetzt |
| 2 | `rename_apply` + Strg+Z | Multi-File-Apply + **atomarer Undo** (Regression Fix A) |
| 3 | usage in `Usage.kt` mutieren, dann apply mit altem `plan_hash` | `CONFLICT` (Fix C) |
| 4 | `rename_preview Widget→Gadget` | headless `CONFLICT`-Token, **kein Dialog** (Fix A) |
| 5 | best-effort live + Unit-Test | `INDEXING` (Fix E) |
| 6 | `rename_preview path:"notes.txt" line:1` | `UNSUPPORTED_LANGUAGE` (Fix B) |
| 7 | preview + apply ohne IDE | `BACKEND_REQUIRED` (unverändert) |

## Testing

- **Rust:** `cargo nextest run` — `files`-Zähler (Fix D, angepasster
  `preview_renders_plan_hash_and_files`) + bestehende Gate-Tests bleiben grün.
- **Plugin:** bestehende `RequestRouterRefactorTest` + neue Unit-Tests:
  - A: dry-run Konflikt-Sammlung liefert `conflictDtos` ohne Dialog; `apply` via
    `performRefactoring` erzeugt genau einen Undo-Eintrag.
  - B: `.txt`-Ziel → `UNSUPPORTED_LANGUAGE` vor `NO_SYMBOL`.
  - E: Dumb Mode → `INDEXING` (Gate + Catch-Net).

## Umsetzungs-Reihenfolge (Vorschlag)

D + C (risikolos) → B → A (riskantester; mit Case-#2-Regression) → E →
Live-Re-Test.

## Betroffene Dateien

- `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolRefactorer.kt` (A, B)
- `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt` (E Catch-Net)
- `rust/src/tools/ctx_refactor.rs` (D)
- `docs/lean-md/runbooks/runide-rename-gate.md` (C, Annotationen #4/#5/#6)
- Plugin-Tests + Rust-Test `preview_renders_plan_hash_and_files`

## Referenzen (JetBrains Platform SDK)

- Rename Refactoring: https://plugins.jetbrains.com/docs/intellij/rename-refactoring.html
- Threading Model (EDT/WriteAction/Modality): https://plugins.jetbrains.com/docs/intellij/threading-model.html
- PSI Files (`PsiManager.findFile`, `getLanguage`/`getFileType`, FileViewProvider): https://plugins.jetbrains.com/docs/intellij/psi-files.html
- PSI Elements (Leaf→Deklaration): https://plugins.jetbrains.com/docs/intellij/psi-elements.html
- Navigating the PSI (`PsiTreeUtil.getParentOfType`, `PsiReference.resolve`): https://plugins.jetbrains.com/docs/intellij/navigating-psi.html
- Indexing & PSI Stubs (Dumb Mode, `IndexNotReadyException`; Tree-Ops gehen im Dumb Mode, Resolve/Suche nicht): https://plugins.jetbrains.com/docs/intellij/indexing-and-psi-stubs.html
- File-Based Indexes (Index-Queries nur im Smart Mode): https://plugins.jetbrains.com/docs/intellij/file-based-indexes.html
- `BaseRefactoringProcessor` (Quelle): https://github.com/JetBrains/intellij-community/blob/master/platform/refactoring/src/com/intellij/refactoring/BaseRefactoringProcessor.java
- `RenameProcessor` (Quelle): https://github.com/JetBrains/intellij-community/blob/master/platform/lang-impl/src/com/intellij/refactoring/rename/RenameProcessor.java
- `RenameUtil` (`addConflictDescriptions`): https://github.com/JetBrains/intellij-community/blob/master/platform/lang-impl/src/com/intellij/refactoring/rename/RenameUtil.java
