# Headless-Konflikt-Audit & Safe-Delete-Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Alle `BaseRefactoringProcessor.run()`-Pfade (rename/move/safe_delete) headless-sicher machen — der reproduzierte „Conflicts Detected"-Modaldialog (#8) blockiert nie mehr den eingebetteten HTTP-Server-Thread.

**Architecture:** Pro-Processor maßgeschneidert (Spec-Ansatz A). `safe_delete` (final, `private` ctor → nicht subklassierbar) umgeht den `SafeDeleteProcessor` komplett und löscht direkt per PSI. `rename` (subklassierbar) behält seinen bewährten `prepareConflictsDialog`-Override. `move` wird zuerst **auditiert** und nur bei belegtem Modal-Risiko angefasst (YAGNI). Reine Plugin-Änderung — **kein** Rust-Code (der Rust-Gate `render_safe_delete_apply` entscheidet `force`/Konflikt bereits).

**Tech Stack:** Kotlin, IntelliJ Platform SDK (IC-2026.1.3), JUnit `BasePlatformTestCase`, Gradle (`buildPlugin`/`test`), Serena-Tools für `*.kt`-Edits.

---

## Hintergrund (kompakt — vor Task-Start lesen)

**Root Cause (#8):** `SymbolDeleter.apply()` ruft `SafeDeleteProcessor.createInstance(...).run()` roh. Bei einer verbleibenden Referenz öffnet `BaseRefactoringProcessor.preprocessUsages` einen modalen `prepareConflictsDialog(...).showAndGet()`. Der `catch (Throwable)` fängt das nicht (der Dialog **wirft nicht, er blockiert**). `SafeDeleteProcessor` ist `final` mit `private` ctor → der rename-Guard (`prepareConflictsDialog`-Override) ist hier **strukturell unmöglich**. `createInstance` hat **keinen** `force`/`deleteEvenIfUsed`-Parameter.

**Fix-Prinzip (Spec §3/§5.1):** `force`+Konflikt ⇒ headless durchlöschen, nie ein Modal auf dem Server-Thread. Beim Erreichen von `apply()` hat der Rust-Gate `force`/Konflikt schon entschieden — das Plugin muss nur noch **löschen**, nicht prüfen. Statt `SafeDeleteProcessor`:
- Ziel ist **einzige** nicht-triviale top-level-Deklaration seiner Datei → `containingFile` (VirtualFile) löschen (= „Klasse = Datei").
- sonst → `element.delete()` (Member-Löschung).
Dangling-Refs bleiben = `force`-Semantik = Runbook-#8-Soll.

**TDD-Reproduktion (Spec §6.1):** Light-Fixture (`BasePlatformTestCase`) indiziert `project.basePath` **nicht** als Source-Root → cross-file-Usages werden nicht gefunden; **intra-file**-Referenzen schon (PSI-lokal). Im UnitTestMode wird der Modal zur `ConflictsInTestsException` → der Bug ist headless als fehlende `"applied":true` (Error-Envelope) reproduzierbar.

**Schlüsseldateien:**
- `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolDeleter.kt` (der Fix)
- `.../psi/SymbolRefactorer.kt` (rename-Guard, Vorlage für Helper-Extraktion)
- `.../psi/SymbolMover.kt` (move — bedingt)
- `.../server/RequestRouterRefactorTest.kt` (RED-Tests, `routeOffEdt`-Harness)
- `docs/lean-md/runbooks/runide-move-safedelete-gate.md` (#8 Live-Reverify)

**Hard Rules für jede Task:** `*.kt` editieren **nur** mit Serena-Tools (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_before_symbol`/`insert_after_symbol`, `replace_content`) — **nie** native `Edit`/`ctx_edit`. Lesen → `ctx_read`, Suchen → `ctx_search`, Shell → `ctx_shell` (bare command + `cwd=`, **nie** `cd … &&`, **kein** `2>&1`). Tests **immer** `cargo nextest run` / `./gradlew test` (bare, kein `| tail`). Vor jedem `git add`: `mcp__jetbrains__reformat_file` auf jede geänderte Datei. Auf `feat-jetbrains-plugin` arbeiten — **keine Worktrees**.

---

## Task 1: Audit (kein Produktionscode)

Verifiziert das **tatsächliche** Risiko + SDK-Constraint pro Pfad, bevor Code geändert wird (Spec §4). Output: eine schriftliche Audit-Entscheidung, die Task 3 (move) freischaltet oder überspringt.

**Files:**
- Modify: `docs/lean-md/runbooks/runide-move-safedelete-gate.md` (Audit-Notiz anhängen)
- Test (Charakterisierung, optional): `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterRefactorTest.kt`

- [ ] **Step 1: rename-Guard-Status bestätigen**

Bestehende rename-Tests müssen grün sein (Guard intakt gegen aktuelle SDK).

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterRefactorTest"`
(cwd=`packages/jetbrains-lean-ctx`)
Expected: BUILD SUCCESSFUL, alle `testRename*`/`testUnsupported*`-Tests grün.
Notiz: rename-Audit = **grün** (Guard `SymbolRefactorer.CapturingProcessor.prepareConflictsDialog` deckt den Modal ab). Keine rename-Code-Änderung nötig.

- [ ] **Step 2: `MoveFilesOrDirectoriesProcessor`-Subklassierbarkeit prüfen**

Klären, ob die Klasse subklassierbar ist (→ `prepareConflictsDialog`-Override möglich) oder `final` (→ Umgehung wie safe_delete).

Run: `mcp__jetbrains__get_symbol_info` für Symbol `MoveFilesOrDirectoriesProcessor` (oder `ctx_search "class MoveFilesOrDirectoriesProcessor"` über die SDK-Sources).
Expected: Feststellung „subklassierbar" (Java-Klasse, public ctor — bereits in `SymbolMover.kt:104` direkt konstruiert) **oder** „final". Bewährter Befund: direkt konstruiert → subklassierbar.
Notiere das Ergebnis.

- [ ] **Step 3 (optional): Charakterisierungs-Test für Move-Konflikt im Test-Modus**

Dokumentiert das Test-Modus-Verhalten (Modal → Exception → `CONFLICT`). **Wichtig:** Das beweist NUR den UnitTestMode; der echte runIde-Modal blockiert (wirft nicht). Decisive ist Step 4.

Native lesen, dann via Serena `insert_after_symbol` nach `testRenameApplyFileCollisionRefusedEvenWithForce` einfügen:

```kotlin
fun testMoveCollisionReturnsConflictHeadless_characterization() {
    // CHARACTERIZATION (test-mode only): move Widget.kt into a dir that already holds a
    // Widget.kt. In UnitTestMode a would-be modal becomes an exception → SymbolMover.apply
    // catches it → CONFLICT. Proves the call RETURNS (no test-mode hang); the real runIde
    // modal risk is decided manually in Step 4, not here.
    writeFile("app/Widget.kt", "package app\nclass Widget\n")
    writeFile("app/moved/Widget.kt", "package app\nclass Widget\n")

    val body = """
        {"path":"app/Widget.kt",
         "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
         "target":{"kind":"path","path":"app/moved"},"force":false}
    """.trimIndent()

    val res = routeOffEdt("POST", "/moveApply", body)
    // Acceptance: the call RETURNS with 200 (no deadlock in test mode). Body is CONFLICT or
    // applied depending on SDK collision handling — both are non-hang outcomes.
    assertEquals(res.body, 200, res.status)
}
```

- [ ] **Step 4: Test ausführen (Charakterisierung)**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterRefactorTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS (Call kehrt zurück, kein Test-Modus-Hang). Notiere das Body-Resultat (`CONFLICT` vs `applied`).

- [ ] **Step 5: Manuelle runIde-Move-Konflikt-Provokation (decisive)**

> **Separater Schritt — nicht in der MCP-Session** (runIde startet eine Sandbox-IDE; der Daemon-Stopp würde die eigenen `ctx_*`-Tools unterbrechen, vgl. Runbook „Achtung MCP-Session").

Sandbox-IDE auf dem Fixture starten (Runbook `runide-move-safedelete-gate.md` §2), dann einen **Konflikt** provozieren: `move_apply` von `Widget` in ein Ziel, das eine gleichnamige Datei enthält **oder** das einen ungelösten-Referenz-Konflikt erzeugt.
Beobachte: erscheint ein **modaler Dialog** auf dem Server-Thread (Hang/Timeout des `lean-ctx call`) — oder läuft der Move headless durch / liefert sauber `CONFLICT`?

- [ ] **Step 6: Audit-Entscheidung dokumentieren**

`ctx_read` den Runbook, dann via native `Edit` (Markdown, **keine** `*.kt`) eine Sektion nach der Tabelle (vor `## 4. Teardown`) einfügen:

```markdown
## Audit-Ergebnis (Headless-Konflikt, 2026-06-10)

- **rename:** grün — Guard `CapturingProcessor.prepareConflictsDialog` deckt den Modal; keine Änderung.
- **safe_delete:** Fix umgesetzt — direkte PSI-Löschung statt `SafeDeleteProcessor` (kein Modal mehr).
- **move:** <MODAL belegt → Task 3 ausgeführt | KEIN Modal → nur dokumentiert, kein Code-Change>.
  Befund Step 5: <hier das beobachtete runIde-Verhalten eintragen>.
  `MoveFilesOrDirectoriesProcessor` subklassierbar: <ja/nein, aus Step 2>.
```

Die spitzen Klammern mit dem **tatsächlichen** Audit-Befund füllen. Diese Entscheidung steuert, ob **Task 3** ausgeführt oder übersprungen wird.

- [ ] **Step 7: Commit**

Reformat zuerst (falls Step 3 den Test geändert hat):
```bash
# (über mcp__jetbrains__reformat_file auf RequestRouterRefactorTest.kt, falls geändert)
git add docs/lean-md/runbooks/runide-move-safedelete-gate.md packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterRefactorTest.kt
git commit -m "docs(jetbrains): headless-conflict audit — rename green, move decision recorded"
```

---

## Task 2: safe_delete headless Fix (TDD — sicher nötig)

Ersetzt den `SafeDeleteProcessor`-Aufruf in `SymbolDeleter.apply()` durch direkte PSI-Löschung. RED-Tests zuerst (sole-decl + member), dann GREEN.

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolDeleter.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterRefactorTest.kt`

- [ ] **Step 1: RED-Test #1 schreiben (sole-decl → Datei löschen)**

`ctx_read` den Test, dann via Serena `insert_after_symbol` nach dem letzten Test einfügen:

```kotlin
fun testSafeDeleteApplyForceDeletesSoleDeclarationFileHeadless() {
    // Widget is the ONLY top-level declaration in its file AND referenced intra-file (the
    // self() return type). A raw SafeDeleteProcessor would raise the "Conflicts Detected"
    // modal on the server thread (runIde gate #8). Headless + force must delete the WHOLE
    // file (class == file) and leave the dangling ref. Intra-file refs ARE resolved in the
    // light fixture (Spec §6.1), so this reproduces #8 as a missing "applied":true.
    val widgetPath = writeFile(
        "app/Widget.kt",
        "package app\nclass Widget {\n    fun self(): Widget = this\n}\n",
    )

    val body = """
        {"path":"app/Widget.kt",
         "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
         "force":true}
    """.trimIndent()

    val res = routeOffEdt("POST", "/safeDeleteApply", body)
    assertEquals(res.body, 200, res.status)
    assertTrue(res.body, res.body.contains("\"applied\":true"))

    WriteAction.computeAndWait<Unit, RuntimeException> {
        LocalFileSystem.getInstance().refreshAndFindFileByPath(widgetPath)
    }
    assertFalse("Widget.kt must be deleted from disk", Files.exists(Paths.get(widgetPath)))
}
```

- [ ] **Step 2: RED-Test #2 schreiben (Member → element.delete())**

Via Serena `insert_after_symbol` nach Test #1 einfügen:

```kotlin
fun testSafeDeleteApplyForceDeletesReferencedMemberHeadless() {
    // `target` is referenced intra-file by `caller`. force + headless must delete JUST the
    // member (element.delete()), leaving the file, the class and the now-dangling call —
    // never delete the whole file (Spec §8 sole-decl-heuristic risk guard).
    val holderPath = writeFile(
        "app/Holder.kt",
        "package app\nclass Holder {\n    fun target() {}\n    fun caller() { target() }\n}\n",
    )

    val body = """
        {"path":"app/Holder.kt",
         "range":{"start":{"line":2,"character":8},"end":{"line":2,"character":14}},
         "force":true}
    """.trimIndent()

    val res = routeOffEdt("POST", "/safeDeleteApply", body)
    assertEquals(res.body, 200, res.status)
    assertTrue(res.body, res.body.contains("\"applied\":true"))

    WriteAction.computeAndWait<Unit, RuntimeException> {
        LocalFileSystem.getInstance().refreshAndFindFileByPath(holderPath)
    }
    val text = Files.readString(Paths.get(holderPath))
    assertTrue(text, text.contains("class Holder"))   // file + class survive
    assertTrue(text, text.contains("fun caller"))     // sibling member survives
    assertFalse(text, text.contains("fun target"))    // deleted member is gone
}
```

- [ ] **Step 3: Beide Tests ausführen — müssen FEHLSCHLAGEN (RED)**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterRefactorTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL bei beiden neuen Tests — der rohe `SafeDeleteProcessor.run()` löst im UnitTestMode `ConflictsInTestsException` aus → Endpoint liefert Error-Envelope (kein `"applied":true`), bzw. Datei/Member bleiben bestehen. Bestehende rename-Tests bleiben grün.
> Falls ein Test unerwartet GRÜN ist: Befund prüfen (intra-file-Ref wurde evtl. nicht gefunden) — Test-Fixture anpassen, bis er den Bug rot reproduziert, **bevor** der Fix kommt.

- [ ] **Step 4: `isSoleTopLevelDeclaration`-Helper einfügen**

Via Serena `insert_before_symbol` vor `contextSnippet` in `SymbolDeleter` einfügen:

```kotlin
/**
 * True if [element] is the ONLY non-trivial top-level declaration of its file — i.e.
 * deleting it means deleting the whole file (SafeDeleteProcessor's "class IS the file"
 * behavior). Language-robust: [element] must be a DIRECT top-level child (a member, whose
 * parent is a class body, is never the file), and it must be the sole significant top-level
 * child (whitespace, comments and package/import housekeeping ignored). MUST run in a read
 * action (PSI access).
 */
private fun isSoleTopLevelDeclaration(element: PsiElement): Boolean {
    val file = element.containingFile ?: return false
    if (element.parent != file) return false // a member → never the whole file
    val significant = file.children.filter { isSignificantTopLevel(it) }
    return significant.size == 1 && significant.first() === element
}

/** A top-level child that is a real declaration (not whitespace/comment/package/import). */
private fun isSignificantTopLevel(child: PsiElement): Boolean {
    if (child is PsiWhiteSpace || child is PsiComment) return false
    val text = child.text.trim()
    if (text.isEmpty()) return false
    // Language-neutral housekeeping filter (avoids depending on Kotlin PSI classes).
    return !(text.startsWith("package ") || text.startsWith("import "))
}
```

- [ ] **Step 5: `apply()` auf direkte PSI-Löschung umstellen**

Via Serena `replace_symbol_body` den Body von `SymbolDeleter.apply` ersetzen:

```kotlin
fun apply(req: SafeDeleteApplyRequest): RenameApplyResponse {
    val element = locator.inSmartReadAction {
        resolveTarget(req.path, req.range.start.line, req.range.start.character)
    }
    val changed = LinkedHashSet<String>()
    val deleteWholeFile = locator.inSmartReadAction {
        locator.toLocation(element)?.let { changed.add(it.path) }
        isSoleTopLevelDeclaration(element)
    }
    var error: Throwable? = null
    ApplicationManager.getApplication().invokeAndWait {
        try {
            CommandProcessor.getInstance().executeCommand(project, {
                WriteCommandAction.runWriteCommandAction(project) {
                    // The Rust gate (render_safe_delete_apply) already decided force/conflict;
                    // by the time we reach apply() we only DELETE — never re-check, never call
                    // SafeDeleteProcessor (its conflict modal would block the embedded HTTP
                    // server thread, runIde gate #8). Dangling refs stay = force = Runbook #8.
                    if (deleteWholeFile) {
                        val vFile = element.containingFile?.virtualFile
                            ?: throw BackendException("NO_SYMBOL", "element has no virtual file to delete")
                        vFile.delete(this@SymbolDeleter)
                    } else {
                        element.delete() // member deletion; file and siblings stay
                    }
                    FileDocumentManager.getInstance().saveAllDocuments()
                }
            }, "Safe Delete", null)
        } catch (t: Throwable) {
            error = t
        }
    }
    error?.let { throw it }
    return RenameApplyResponse(applied = true, changed_paths = changed.toList())
}
```

- [ ] **Step 6: Ungenutzten `SafeDeleteProcessor`-Import + ctor-Doc-Notiz bereinigen**

`SafeDeleteProcessor` wird nicht mehr aufgerufen. Via Serena `replace_content` die Import-Zeile entfernen:
- Remove: `import com.intellij.refactoring.safeDelete.SafeDeleteProcessor`
- Imports ergänzen (via `replace_content`/`insert`): `import com.intellij.psi.PsiComment` und `import com.intellij.psi.PsiWhiteSpace`.

Den Klassen-Doc-Block (`API note (IC-2026.1.3): SafeDeleteProcessor is final …`) so anpassen, dass er die **neue** Direkt-PSI-Strategie beschreibt (statt `createInstance().run()`): ein Satz, dass `apply()` bewusst keinen Processor mehr nutzt, weil dessen Konflikt-Modal den Server-Thread blockiert; Preview nutzt weiterhin `ReferencesSearch`.

- [ ] **Step 7: Beide Tests ausführen — müssen PASSEN (GREEN)**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterRefactorTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: BUILD SUCCESSFUL — beide neuen safe_delete-Tests grün, alle rename-Tests weiterhin grün.

- [ ] **Step 8: Reformat + Commit**

Reformat via `mcp__jetbrains__reformat_file` auf `SymbolDeleter.kt` **und** `RequestRouterRefactorTest.kt`.
```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolDeleter.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterRefactorTest.kt
git commit -m "fix(jetbrains): safe_delete force deletes headless via direct PSI — no conflict modal (#8)"
```

---

## Task 3: move headless Guard (CONDITIONAL — nur bei rotem Audit)

> **GATE:** Diese Task **nur** ausführen, wenn **Task 1 / Step 5** ein Modal/Hang im runIde-Move-Konflikt belegt hat. Ist der move-Pfad grün (kein Modal — file-moves erzeugen selten Konflikte, Spec §4): diese Task **überspringen**, das Audit-Ergebnis ist bereits in Task 1 dokumentiert. Dann ist auch die Helper-Extraktion (Step 1) überflüssig (rename bleibt mit Inline-Guard, einziger Nutzer — YAGNI).

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/HeadlessConflictsDialog.kt`
- Modify: `.../psi/SymbolRefactorer.kt` (Inline-Guard → Helper)
- Modify: `.../psi/SymbolMover.kt` (`runMove` „path"-Branch → Subklasse)
- Test: `.../server/RequestRouterRefactorTest.kt`

- [ ] **Step 1: Gemeinsamen Helper extrahieren (Spec §5.3, reines Refactoring)**

Native `Write` (neue Datei, kein bestehendes `*.kt`-Symbol → Write zulässig):

```kotlin
package com.leanctx.plugin.psi

import com.intellij.refactoring.ConflictsDialogBase

/**
 * A UI-less [ConflictsDialogBase] that auto-approves conflicts so a
 * BaseRefactoringProcessor proceeds headless instead of blocking the embedded HTTP server
 * thread with a modal. Shared by [SymbolRefactorer] (rename) and [SymbolMover] (move):
 * force+conflict → proceed; the Rust layer owns the force/conflict gate.
 *
 * ConflictsDialogBase is a 3-method interface (NOT a DialogWrapper) — implementing it
 * directly creates no Swing peer and starts no modal event pump.
 */
internal fun headlessConflictsDialog(): ConflictsDialogBase = object : ConflictsDialogBase {
    override fun setCommandName(name: String?) {}   // no-op; headless
    override fun showAndGet(): Boolean = true        // auto-approve → "conflicts accepted" branch
    override fun isShowConflicts(): Boolean = false   // moot: showAndGet always true
}
```

- [ ] **Step 2: rename auf den Helper umstellen (Verhalten unverändert)**

Via Serena `replace_symbol_body` den Body von `SymbolRefactorer.CapturingProcessor.prepareConflictsDialog` ersetzen — den großen Doc-Kommentar (warum die Factory überschrieben wird, nicht `preprocessUsages`) **behalten**, nur das inline `object : ConflictsDialogBase {...}` durch den Helper-Aufruf ersetzen:

```kotlin
override fun prepareConflictsDialog(
    conflicts: MultiMap<PsiElement, String>,
    usages: Array<out UsageInfo>?,
): ConflictsDialogBase = headlessConflictsDialog()
```

- [ ] **Step 3: rename-Regression bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterRefactorTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: alle rename- **und** safe_delete-Tests weiterhin grün (Helper-Extraktion ist verhaltensneutral).

- [ ] **Step 4: RED-Test für provozierten Move-Konflikt schreiben**

Via Serena `insert_after_symbol` nach dem letzten Test einfügen — ein Move, der einen Konflikt erzeugt, muss headless mit `force` durchlaufen (`applied:true`), statt zu blockieren/zu fehlern:

```kotlin
fun testMoveApplyForceProceedsThroughConflictHeadless() {
    // Provoke a move conflict (e.g. a name collision in the destination) and require it to
    // proceed headless with force — mirroring rename's CapturingProcessor guard. Without the
    // HeadlessMoveProcessor override the SDK would route through prepareConflictsDialog and
    // (in UnitTestMode) throw → no "applied":true. RED before the override, GREEN after.
    writeFile("app/Widget.kt", "package app\nclass Widget {\n    fun self(): Widget = this\n}\n")
    // destination dir exists; the intra-file self-reference is the conflict source the
    // move processor reports.
    writeFile("app/moved/.keep", "")

    val body = """
        {"path":"app/Widget.kt",
         "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
         "target":{"kind":"path","path":"app/moved"},"force":true}
    """.trimIndent()

    val res = routeOffEdt("POST", "/moveApply", body)
    assertEquals(res.body, 200, res.status)
    assertTrue(res.body, res.body.contains("\"applied\":true"))
}
```

> **Anpassen an den realen Audit-Befund:** Wenn der konkrete Konflikt aus Step-1-Audit ein anderer ist (z. B. Ziel-Datei-Kollision), das Fixture entsprechend konstruieren, sodass der Test **vor** dem Override rot ist.

- [ ] **Step 5: Test ausführen — FEHLSCHLAGEN (RED)**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterRefactorTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL bei `testMoveApplyForceProceedsThroughConflictHeadless` (kein `"applied":true`).

- [ ] **Step 6: `HeadlessMoveProcessor`-Subklasse + `runMove`-Naht**

Via Serena `insert_before_symbol` vor `runMove` in `SymbolMover` die Subklasse einfügen:

```kotlin
/**
 * MoveFilesOrDirectoriesProcessor with a headless conflict gate: the [prepareConflictsDialog]
 * override returns a UI-less [ConflictsDialogBase] so a force-move proceeds instead of
 * blocking the embedded HTTP server thread with a modal (mirrors SymbolRefactorer's rename
 * guard; audit-confirmed move modal risk).
 */
private class HeadlessMoveProcessor(
    project: Project,
    elements: Array<PsiElement>,
    targetDir: PsiDirectory,
    searchInComments: Boolean,
    searchInNonJavaFiles: Boolean,
) : MoveFilesOrDirectoriesProcessor(
    project, elements, targetDir, searchInComments, searchInNonJavaFiles, null, null,
) {
    override fun prepareConflictsDialog(
        conflicts: MultiMap<PsiElement, String>,
        usages: Array<out UsageInfo>?,
    ): ConflictsDialogBase = headlessConflictsDialog()
}
```

Dann via Serena `replace_content` im `runMove`-„path"-Branch den rohen `MoveFilesOrDirectoriesProcessor(...).run()` durch die Subklasse ersetzen:

```kotlin
val processor = HeadlessMoveProcessor(
    project, arrayOf(file), destDir, /* searchInComments = */ true, /* searchInNonJavaFiles = */ true,
)
processor.setPreviewUsages(false)
processor.run()
```

Imports in `SymbolMover.kt` ergänzen (via `replace_content`): `com.intellij.psi.PsiDirectory` (schon vorhanden), `com.intellij.refactoring.ConflictsDialogBase`, `com.intellij.usageView.UsageInfo`, `com.intellij.util.containers.MultiMap`.

> **Falls Audit `MoveFilesOrDirectoriesProcessor` als `final` belegt** (Step 2): statt Subklasse den Move per direkter PSI-Operation umgehen (analog safe_delete) — `MoveFilesOrDirectoriesUtil.doMoveFile(file, destDir)` in einem WriteCommandAction. Dann entfällt die Subklasse; der Helper wird nur von rename genutzt.

- [ ] **Step 7: Test ausführen — PASSEN (GREEN)**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterRefactorTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: BUILD SUCCESSFUL — der Move-Konflikt-Test grün, alle übrigen Tests grün.

- [ ] **Step 8: Reformat + Commit**

Reformat via `mcp__jetbrains__reformat_file` auf `HeadlessConflictsDialog.kt`, `SymbolRefactorer.kt`, `SymbolMover.kt`, `RequestRouterRefactorTest.kt`.
```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/HeadlessConflictsDialog.kt packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolRefactorer.kt packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolMover.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterRefactorTest.kt
git commit -m "fix(jetbrains): move force proceeds headless via shared ConflictsDialog guard"
```

---

## Task 4: Regressions-Gates & Live-Reverify-Doku

Alle Gates grün ziehen und den Runbook für das #8-Live-Reverify aktualisieren (Spec §6.2/§6.3/§7).

**Files:**
- Modify: `docs/lean-md/runbooks/runide-move-safedelete-gate.md` (#8-Mechanismus-Notiz)

- [ ] **Step 1: Kotlin-Gate — voller Plugin-Build + Tests**

Run: `./gradlew test buildPlugin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: BUILD SUCCESSFUL — alle Plugin-Tests grün, `buildPlugin` erzeugt das Artefakt.

- [ ] **Step 2: Rust-Regression (kein Rust geändert — reiner Grün-Nachweis)**

Run: `cargo nextest run` (cwd=`rust`)
Expected: alle Tests grün (`Summary […] N tests run`). Der Rust-Gate `render_safe_delete_apply` ist unverändert.

- [ ] **Step 3: Lint/Format-Gate (Rust)**

Run: `cargo clippy --all-targets` und `cargo fmt --check` (cwd=`rust`)
Expected: clippy clean (keine Warnings), `fmt --check` ohne Diff.

- [ ] **Step 4: Drift-Test (MCP-Tool-Doku)**

Run: `cargo nextest run drift` (cwd=`rust`) — bzw. der projektübliche Drift-Test.
Expected: grün. (Kein Schema-Change → kein Drift erwartet.)

- [ ] **Step 5: Runbook #8-Mechanismus-Notiz aktualisieren**

`ctx_read` den Runbook, dann native `Edit` (Markdown) die #8-Zeile der Tabelle ergänzen — das Soll-Ergebnis bleibt („gelöscht; Refs dangling"), aber der Mechanismus ist jetzt direkte PSI-Löschung (kein `SafeDeleteProcessor`, kein Modal):

- Old (#8-Soll-Text): `… IntelliJ-SafeDeleteProcessor in IC-2026.1.3 kennt kein deleteEvenIfUsed — run() löscht immer durch)`
- New: `… Plugin umgeht SafeDeleteProcessor komplett (final/private ctor → nicht subklassierbar) und löscht direkt per PSI: sole-top-level-decl → ganze Datei, sonst element.delete(). Kein Konflikt-Modal mehr auf dem Server-Thread.)`

- [ ] **Step 6: Commit**

```bash
git add docs/lean-md/runbooks/runide-move-safedelete-gate.md
git commit -m "docs(runbook): #8 reverify — direct-PSI safe_delete, no conflict modal"
```

- [ ] **Step 7 (manuell, separater Schritt — nicht in MCP-Session): Live-Reverify runIde**

> Daemon-Stopp ist Pflicht (Runbook §Voraussetzungen) und unterbricht die `ctx_*`-Tools — als eigenständigen Schritt fahren.

1. `lean-ctx serve --stop` → `cargo build` (cwd=`rust`) → `./gradlew buildPlugin` (cwd=`packages/jetbrains-lean-ctx`).
2. Sandbox neu: `./gradlew runIde --args="$FIX"` (Fixture via `./scripts/runide-move-safedelete-gate-setup.sh`).
3. **#8 erneut:** `safe_delete_apply force=true` auf genutztes `Widget` → headless gelöscht, **kein** Dialog, Antwort `applied`.
4. Regression #6 (ungenutzt löschen) + #7 (ohne force → `CONFLICT`) erneut.
5. Falls Task 3 ausgeführt: den auditierten Move-Konflikt-Fall live verifizieren (force → headless durch).
Akzeptanz: kein Modal/Server-Thread-Block; alle Soll-Ergebnisse erfüllt (Spec §7).

---

## Self-Review (Plan-Abgleich gegen Spec)

- **§2 Root Cause / §5.1 Fix:** Task 2 ersetzt `SafeDeleteProcessor` durch direkte PSI-Löschung (sole-decl → Datei, sonst `element.delete()`). ✓
- **§4 Audit-Phase:** Task 1 (rename grün, move-Charakterisierung + runIde-Provokation + dokumentierte Entscheidung). ✓
- **§5.2 move bedingt:** Task 3 ist hinter dem Audit-Gate; übersprungen bei grünem move. ✓
- **§5.3 Helper:** Task 3 Step 1 extrahiert `headlessConflictsDialog()` (nur wenn move ihn braucht — sonst YAGNI). ✓
- **§6.1 TDD:** RED-Tests intra-file, `routeOffEdt`-Harness, `applied:true`-Reproduktion. ✓ (sole-decl + member decken die §8-Heuristik-Risiken ab).
- **§6.2 Live-Reverify / §6.3 Gates:** Task 4. ✓
- **§7 Akzeptanz 1–5:** headless löschen (T2), RED-vor-Fix (T2 Step 3), move dokumentiert/bedingt (T1/T3), rename unverändert grün (T1 Step 1 / T3 Step 3), Gates grün (T4). ✓
- **§8 Risiken:** sole-decl-Heuristik über `isSoleTopLevelDeclaration` (direktes top-level-Child + sole-significant) + Member-Test abgesichert; move-ergebnisoffen über das Audit-Gate. ✓
- **Typ-Konsistenz:** `isSoleTopLevelDeclaration`/`isSignificantTopLevel`/`headlessConflictsDialog`/`HeadlessMoveProcessor` durchgängig identisch benannt. ✓
