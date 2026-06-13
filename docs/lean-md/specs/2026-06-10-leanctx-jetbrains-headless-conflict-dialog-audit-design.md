# Design: Headless-Konflikt-Audit aller Refactoring-Processor-Pfade

| Feld       | Wert                                                                                          |
|------------|-----------------------------------------------------------------------------------------------|
| Datum      | 2026-06-10                                                                                     |
| Branch     | feat-jetbrains-plugin                                                                          |
| Bezug      | v2c `docs/lean-md/specs/2026-06-10-leanctx-jetbrains-v2c-move-safedelete-design.md` §5.2/§6    |
| Auslöser   | runIde-Gate `runide-move-safedelete-gate.md` #8 — reproduzierter Modal-Dialog                 |
| Scope      | Audit **aller** `BaseRefactoringProcessor.run()`-Pfade (rename/move/safe_delete) auf headless |
| Ansatz     | A — pro-Processor maßgeschneidert (kein universeller Reflection-Guard)                         |

## 1. Motivation & Befund

Das v2c-runIde-Live-Gate (manuelle Verifikation, 2026-06-10) lief #1–#7, #9–#11
grün. **#8** (`safe_delete_apply force=true` auf ein referenziertes Symbol) zeigte
in der Sandbox-IDE einen modalen Dialog **„Conflicts Detected"** (Buttons
„Refactor Anyway" / „Cancel", Text „Usage of class app.Widget that is not safe to
delete") auf dem **HTTP-Server-Thread** → blockiert/Deadlock. Erwartet war (Runbook
#8): headless durchlöschen, Refs bleiben dangling.

> Abgrenzung: Ein zwischenzeitlich vermuteter move-#5-Defekt war ein **Fehltest**
> (Kommentar am Zeilenende verschob keine Usage-Range → `plan_hash` korrekt
> unverändert). Korrekt konstruiert (Zeile **vor** der Referenz eingefügt) liefert
> #5 deterministisch `CONFLICT`. Der move-Pfad ist im Gate **nicht** als fehlerhaft
> belegt — sein Dialog-Risiko ist **latent** und wird in diesem Vorhaben auditiert,
> nicht blind gefixt.

## 2. Root Cause

`SymbolDeleter.apply()` (`SymbolDeleter.kt:61-89`) ruft
`SafeDeleteProcessor.createInstance(project, null, [element], false, false).run()`
**roh**, ohne Konflikt-Dialog-Guard. `run()` → `BaseRefactoringProcessor.preprocessUsages`
→ bei Konflikt `prepareConflictsDialog(...).showAndGet()` modal. Der `catch (Throwable)`
in `apply()` fängt das **nicht**, weil der Dialog nicht wirft, sondern blockiert.

Der bei rename bewährte Guard (`SymbolRefactorer.CapturingProcessor` mit
`prepareConflictsDialog`-Override → UI-loser `ConflictsDialogBase`, `SymbolRefactorer.kt:77-92`,
plus `setPreviewUsages(false)`) ist hier **strukturell unmöglich**:
`SafeDeleteProcessor` ist **`final` mit `private` ctor** (Code-Kommentar
`SymbolDeleter.kt:31-34`) → nicht subklassierbar. Die v2c-Spec-Annahme
(`<…>Processor(…, force=…).run()` sei headless, §-Diagramm Z.136) trifft für
safe_delete nicht zu — `SafeDeleteProcessor.createInstance` hat **keinen**
`force`/`deleteEvenIfUsed`-Parameter.

## 3. Leitprinzip

Einheitlich über **alle** Pfade: **`force` + Konflikt ⇒ headless proceed, niemals
ein Modal auf dem eingebetteten HTTP-Server-Thread.** Der *Mechanismus* wird pro
Processor nach SDK-Realität gewählt:

- **subklassierbar** (rename, evtl. move) → `prepareConflictsDialog`-Override mit
  gemeinsamem UI-losem `ConflictsDialogBase`.
- **final** (safe_delete) → Processor **umgehen**, direkte PSI-Operation.

Kein universeller Reflection-/TestMode-Guard (Ansatz B, verworfen: fragil/versions­abhängig);
kein Vorab-Check-mit-direkter-PSI-Op für rename/move (Ansatz C, verworfen: zerstört
die Multi-File-Processor-Semantik).

## 4. Audit-Phase (zuerst, kein Produktionscode)

Pro Pfad das **tatsächliche** Risiko + SDK-Constraint verifizieren, bevor Code
geändert wird:

| Pfad        | Audit-Schritt                                                                                  | Entscheidet                          |
|-------------|------------------------------------------------------------------------------------------------|--------------------------------------|
| rename      | Guard-Vollständigkeit gegen aktuelle SDK prüfen                                                 | grün / Helper-Extraktion             |
| move        | Move-Konflikt **provozieren** (Unit + runIde): zeigt `MoveFilesOrDirectoriesProcessor` ein Modal? Ist die Klasse subklassierbar? | ob move-Code überhaupt nötig ist     |
| safe_delete | bereits belegt (#8)                                                                             | Fix nötig                            |

Ergibt der move-Audit **kein** Modal-Risiko (file-moves erzeugen selten Konflikte),
wird der move-Pfad nur **dokumentiert**, nicht geändert (YAGNI).

## 5. Komponenten

### 5.1 `SymbolDeleter.apply()` — der #8-Fix (sicher nötig)
Beim Erreichen von `apply` hat das Rust-Gate `force`/Konflikt bereits entschieden
(`render_safe_delete_apply` in `ctx_refactor.rs`: `¬force ∧ conflicts≠∅ → CONFLICT`,
ohne Plugin-Call). Das Plugin muss also nur noch **löschen**, nicht prüfen. Der
`SafeDeleteProcessor` (mit seinem Konflikt-Modal) wird **nicht** mehr aufgerufen;
stattdessen direkte PSI-Löschung in der bestehenden `CommandProcessor.executeCommand`
(ein Undo):

- Ist das Ziel-Element die **einzige** top-level-Deklaration seiner Datei →
  `containingFile` (VirtualFile) löschen — entspricht dem SafeDeleteProcessor-Verhalten
  für „Klasse = Datei".
- sonst → `element.delete()` (Member-Löschung).

Dangling-Refs bleiben bestehen = `force`-Semantik = Runbook-#8-Soll. Die
„sole-top-level-decl"-Bestimmung wird **sprach-robust** im TDD verfeinert
(PSI-basiert, nicht Kotlin-spezifisch).

### 5.2 `SymbolMover.runMove()` — bedingt (nur falls Audit Risiko zeigt)
Falls der move-Audit ein Modal belegt: `MoveFilesOrDirectoriesProcessor`-Subklasse
mit `prepareConflictsDialog`-Override (gemeinsamer Helper); ist die Klasse final,
analoge Umgehung wie safe_delete. Andernfalls: nur Doku-Notiz im Code.

### 5.3 Gemeinsamer Helper
Den UI-losen `ConflictsDialogBase` (heute inline in `SymbolRefactorer`,
`showAndGet()=true`, `isShowConflicts()=false`, `setCommandName`=no-op) in eine kleine
wiederverwendbare Funktion/Objekt ziehen, damit rename und (ggf.) move denselben Guard
teilen. Reines Refactoring ohne Verhaltensänderung — rename bleibt grün.

## 6. Tests & Verifikation

### 6.1 TDD pro Fix
RED-Unit-Test in `RequestRouterRefactorTest` (`BasePlatformTestCase`, `routeOffEdt`)
mit **intra-file** Referenz (Deklaration + Nutzung in derselben Datei). Begründung:
Das Light-Fixture indiziert `project.basePath` nicht als Source-Root → **cross-file**
usages werden nicht gefunden (dokumentiert `RequestRouterRefactorTest.kt:80-84`); eine
**intra-file** Referenz wird PSI-lokal aufgelöst. Im UnitTestMode wird der Modal zur
`ConflictsInTestsException` → der Bug ist headless als fehlende `"applied":true`
reproduzierbar.

- **safe_delete:** `testSafeDeleteApplyForceDeletesReferencedSymbolHeadless` — `force=true`
  auf referenziertes Symbol → `200` + `"applied":true` + Element/Datei von Disk weg.
  Muss **vor** dem Fix fehlschlagen (Exception statt applied), **danach** passen.
- **move:** nur falls Audit es verlangt — analoger provozierter-Konflikt-Test.

### 6.2 Live-Reverify (runIde)
Plugin neu bauen (`./gradlew buildPlugin`) → Sandbox neu → **#8 erneut**:
`safe_delete_apply force=true` auf genutztes `Widget` → headless gelöscht, **kein
Dialog**, Antwort `applied`. Optional #6/#7 erneut (Regression: ungenutzt löschen /
ohne force blockt weiterhin).

### 6.3 Regressions-Gates
`cargo nextest run` grün · `./gradlew test buildPlugin` SUCCESSFUL ·
`cargo clippy --all-targets` + `cargo fmt --check` clean · Drift-Test.

## 7. Akzeptanzkriterien

1. `safe_delete_apply force=true` auf referenziertes Symbol läuft **headless**
   (kein Modal, kein Server-Thread-Block), löscht das Symbol, lässt Refs dangling.
2. RED-Test existierte und schlug **vor** dem Fix fehl (TDD-Nachweis).
3. move-Pfad: Audit-Ergebnis dokumentiert; Code nur bei belegtem Risiko geändert.
4. rename bleibt unverändert grün; ggf. Helper-Extraktion ohne Verhaltensänderung.
5. Alle Regressions-Gates grün; #8 im runIde-Gate live grün.

## 8. Out-of-Scope / Risiken

- **Out-of-scope:** `propagate` (now-unreferenced-deps löschen) bleibt wie v2c;
  `inline`/`reformat` (v2d). Cross-file-Usage-Rewrites im Light-Fixture (bleiben dem
  runIde-Gate vorbehalten, Spec §10).
- **Risiko (sole-decl-Heuristik):** Eine zu aggressive „Datei löschen"-Regel könnte
  bei Dateien mit mehreren top-level-Decls falsch greifen → durch PSI-basierte Prüfung
  (Element ist einziges nicht-triviales top-level-Child) + Unit-Test für den
  Member-Fall abgesichert.
- **Risiko (move-Audit ergebnisoffen):** Falls `MoveFilesOrDirectoriesProcessor`
  final ist UND ein Modal zeigt, ist nur die Umgehung möglich — dann verliert move
  ggf. Multi-File-Ref-Rewrites im Konflikt-Fall; in dem Fall vor Implementierung
  erneut abwägen.
