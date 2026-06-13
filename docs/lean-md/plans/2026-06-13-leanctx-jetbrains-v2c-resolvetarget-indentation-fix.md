# v2c resolveTarget-Indentation-Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Den in `SymbolInliner.resolveTarget` (v2d) verifizierten Whitespace-Skip auf die vier latenten v2c-Träger (`SymbolDeleter`, `SymbolMover`, `SymbolRefactorer`, `ReferenceFinder`) portieren und je mit einem RED-first-Unit-Test über den headless-sicheren Preview-Pfad absichern.

**Architecture:** Zeilenadressierte Ziele (`character = 0`) landen bei `findElementAt` auf der führenden `PsiWhiteSpace` eingerückter Deklarationen; der anschließende Parent-Aufstieg greift den umschließenden Knoten. Fix: ist das Element `PsiWhiteSpace`, ein `PsiTreeUtil.nextLeaf` weiterspringen, bevor `getParentOfType`/der `generateSequence`-Walk läuft. Bei Realspalten ist `at` kein Whitespace → No-op, rein additiv. Verifikation über die öffentlichen `preview(...)`/`find(...)`-Methoden (resolve + ReferencesSearch, kein Processor, kein Modal).

**Tech Stack:** Kotlin, IntelliJ Platform SDK (PSI: `PsiWhiteSpace`, `PsiTreeUtil.nextLeaf`), `BasePlatformTestCase`, Gradle.

---

## Konventionen für diese Aufgabe

- **Quelldatei-Edits (`.kt`)**: via `mcp__lean-ctx__ctx_edit(path, old_string, new_string)` (für Nicht-Rust sanktioniert). Falls `ctx_edit` nicht greift, `mcp__serena__replace_symbol_body`. **Nie** `sed`/`awk`.
- **Neue Testdateien**: `Write`.
- **Vor jedem `git commit`**: `mcp__jetbrains__reformat_file` auf jede geänderte/neue Datei.
- **Tests laufen**: `mcp__lean-ctx__ctx_shell(command="./gradlew test ...", cwd="packages/jetbrains-lean-ctx")` — bare command, kein `cd … &&`, kein `| tail`/`| grep`.
- **Deferred-Tool-Reflex**: zeigt sich ein MCP-Tool als deferred → zuerst `ToolSearch(query="select:<tool>")`, dann aufrufen. Nie Bash-Workaround davor.

Basis-Pfad Quellen: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/`
Basis-Pfad Tests: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/`

Gemeinsames Test-Fixture (alle vier Tasks, 0-basierte Zeilen nach `trimIndent`):

```
0: package p
1:
2: class Outer {
3:     fun target() {}
4: }
5:
6: val shared = Outer()
7: fun a() { shared.target() }
8: fun b() { shared.target() }
```

`target` steht auf **Zeile 3, eingerückt** → Adressierung `(line=3, character=0)` trifft die Einrückung. Falsche Auflösung → `Outer` (Zeile 2), dessen einzige Referenz `Outer()` auf Zeile 6 ist. Richtige Auflösung → `target`, referenziert 2× als `shared.target()` (Zeilen 7/8). Diskriminator: Anzahl Usages mit Kontext `shared.target()` == 2 (GREEN) vs 0 (RED).

---

## Task 1: SymbolDeleter — resolveTarget Whitespace-Skip

**Files:**
- Test (Create): `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/SymbolDeleterTest.kt`
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolDeleter.kt` (`resolveTarget`, Z.106-108)

Hinweis: `SymbolDeleter.kt` importiert `PsiWhiteSpace` und `PsiTreeUtil` **bereits** (Z.14/16) — kein Import-Edit nötig.

- [ ] **Step 1: Failing test schreiben**

`Write` nach `.../psi/SymbolDeleterTest.kt`:

```kotlin
package com.leanctx.plugin.psi

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.WriteAction
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.VfsUtil
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.leanctx.plugin.dto.PositionDTO
import com.leanctx.plugin.dto.SafeDeletePreviewRequest
import com.leanctx.plugin.dto.TextRangeDTO
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.TimeUnit

class SymbolDeleterTest : BasePlatformTestCase() {

    private val fixture = """
        package p

        class Outer {
            fun target() {}
        }

        val shared = Outer()
        fun a() { shared.target() }
        fun b() { shared.target() }
    """.trimIndent()

    // Resolve + ReferencesSearch touch the Kotlin Analysis API (KaSession), prohibited on
    // the EDT. The test body runs on the EDT, so run preview on a pooled thread and pump
    // the EDT while waiting (mirrors RequestRouterRefactorTest.routeOffEdt).
    private fun <T> offEdt(block: () -> T): T {
        val future = ApplicationManager.getApplication().executeOnPooledThread<T> { block() }
        return PlatformTestUtil.waitForFuture(future, TimeUnit.SECONDS.toMillis(60))
    }

    // PsiLocator resolves via LocalFileSystem.findFileByPath, so the file must exist on disk.
    private fun writeFile(rel: String, content: String) {
        val p = Paths.get(project.basePath!!, rel)
        Files.createDirectories(p.parent)
        Files.writeString(p, content)
        WriteAction.computeAndWait<Unit, RuntimeException> {
            val vFile = LocalFileSystem.getInstance().refreshAndFindFileByPath(p.toString())
                ?: error("could not refresh VFS for $p")
            VfsUtil.saveText(vFile, content)
        }
    }

    fun testResolvesIndentedMemberNotEnclosingClass() {
        writeFile("Sample.kt", fixture)
        // target() is on line 3, indented; address char 0 (lands on the indentation).
        val req = SafeDeletePreviewRequest(
            path = "Sample.kt",
            range = TextRangeDTO(PositionDTO(3, 0), PositionDTO(3, 0)),
        )
        val resp = offEdt { SymbolDeleter(project).preview(req) }
        // Correct resolution → the two `shared.target()` call sites are the blocking refs.
        // Wrong resolution (enclosing class Outer) → its only ref is `Outer()`, context
        // "val shared = Outer()", which never contains "shared.target()".
        assertEquals(2, resp.usages.count { it.context?.contains("shared.target()") == true })
    }
}
```

- [ ] **Step 2: Test laufen — RED bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.SymbolDeleterTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL — `expected:<2> but was:<0>` (resolveTarget liefert `Outer`, dessen Ref-Kontext ist `val shared = Outer()`).

- [ ] **Step 3: Fix einbauen**

`ctx_edit` auf `SymbolDeleter.kt` —
old_string:
```kotlin
        val at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at $line:$character")
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
```
new_string:
```kotlin
        var at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at $line:$character")
        // Line-addressed targets (char 0) land on the leading indentation; skip it so
        // getParentOfType resolves the declaration ON the line, not its enclosing
        // class/function. Top-level (col-0) symbols never hit this; surfaced at the v2d
        // inline live-gate (SymbolInliner), ported to the v2c siblings.
        if (at is PsiWhiteSpace) {
            at = PsiTreeUtil.nextLeaf(at) ?: at
        }
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
```

- [ ] **Step 4: Test laufen — GREEN bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.SymbolDeleterTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS.

- [ ] **Step 5: Reformat + Commit**

Reformat: `mcp__jetbrains__reformat_file` auf `SymbolDeleter.kt` und `SymbolDeleterTest.kt`.

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolDeleter.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/SymbolDeleterTest.kt
git commit -m "fix(v2c): SymbolDeleter.resolveTarget skips leading indentation (port v2d inline fix) + RED-first test"
```

---

## Task 2: SymbolMover — resolveSource Whitespace-Skip

**Files:**
- Test (Create): `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/SymbolMoverTest.kt`
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolMover.kt` (Import + `resolveSource`, Z.138-140)

Hinweis: `SymbolMover.kt` importiert `PsiTreeUtil` (Z.17), aber **nicht** `PsiWhiteSpace` → Import ergänzen.

- [ ] **Step 1: Failing test schreiben**

`Write` nach `.../psi/SymbolMoverTest.kt`:

```kotlin
package com.leanctx.plugin.psi

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.WriteAction
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.VfsUtil
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.leanctx.plugin.dto.MovePreviewRequest
import com.leanctx.plugin.dto.MoveTargetDTO
import com.leanctx.plugin.dto.PositionDTO
import com.leanctx.plugin.dto.TextRangeDTO
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.TimeUnit

class SymbolMoverTest : BasePlatformTestCase() {

    private val fixture = """
        package p

        class Outer {
            fun target() {}
        }

        val shared = Outer()
        fun a() { shared.target() }
        fun b() { shared.target() }
    """.trimIndent()

    private fun <T> offEdt(block: () -> T): T {
        val future = ApplicationManager.getApplication().executeOnPooledThread<T> { block() }
        return PlatformTestUtil.waitForFuture(future, TimeUnit.SECONDS.toMillis(60))
    }

    private fun writeFile(rel: String, content: String) {
        val p = Paths.get(project.basePath!!, rel)
        Files.createDirectories(p.parent)
        Files.writeString(p, content)
        WriteAction.computeAndWait<Unit, RuntimeException> {
            val vFile = LocalFileSystem.getInstance().refreshAndFindFileByPath(p.toString())
                ?: error("could not refresh VFS for $p")
            VfsUtil.saveText(vFile, content)
        }
    }

    fun testResolvesIndentedMemberNotEnclosingClass() {
        writeFile("Sample.kt", fixture)
        // preview() uses only range.start to resolveSource; target is required but unused.
        val req = MovePreviewRequest(
            path = "Sample.kt",
            range = TextRangeDTO(PositionDTO(3, 0), PositionDTO(3, 0)),
            target = MoveTargetDTO(kind = "file", path = "Dest.kt"),
        )
        val resp = offEdt { SymbolMover(project).preview(req) }
        assertEquals(2, resp.usages.count { it.context?.contains("shared.target()") == true })
    }
}
```

- [ ] **Step 2: Test laufen — RED bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.SymbolMoverTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL — `expected:<2> but was:<0>`.

- [ ] **Step 3a: Import ergänzen**

`ctx_edit` auf `SymbolMover.kt` —
old_string:
```kotlin
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.search.searches.ReferencesSearch
```
new_string:
```kotlin
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.PsiWhiteSpace
import com.intellij.psi.search.searches.ReferencesSearch
```

- [ ] **Step 3b: Fix einbauen**

`ctx_edit` auf `SymbolMover.kt` —
old_string:
```kotlin
        val at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at $line:$character")
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
```
new_string:
```kotlin
        var at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at $line:$character")
        // Line-addressed targets (char 0) land on the leading indentation; skip it so
        // getParentOfType resolves the declaration ON the line, not its enclosing
        // class/function. Top-level (col-0) symbols never hit this; surfaced at the v2d
        // inline live-gate (SymbolInliner), ported to the v2c siblings.
        if (at is PsiWhiteSpace) {
            at = PsiTreeUtil.nextLeaf(at) ?: at
        }
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
```

- [ ] **Step 4: Test laufen — GREEN bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.SymbolMoverTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS.

- [ ] **Step 5: Reformat + Commit**

Reformat: `mcp__jetbrains__reformat_file` auf `SymbolMover.kt` und `SymbolMoverTest.kt`.

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolMover.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/SymbolMoverTest.kt
git commit -m "fix(v2c): SymbolMover.resolveSource skips leading indentation (port v2d inline fix) + RED-first test"
```

---

## Task 3: SymbolRefactorer — resolveTarget Whitespace-Skip

**Files:**
- Test (Create): `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/SymbolRefactorerTest.kt`
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolRefactorer.kt` (Import + `resolveTarget`, Z.206-208)

Hinweis: `SymbolRefactorer.kt` importiert `PsiTreeUtil` (Z.13), aber **nicht** `PsiWhiteSpace` → Import ergänzen.

- [ ] **Step 1: Failing test schreiben**

`Write` nach `.../psi/SymbolRefactorerTest.kt`:

```kotlin
package com.leanctx.plugin.psi

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.WriteAction
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.VfsUtil
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.leanctx.plugin.dto.PositionDTO
import com.leanctx.plugin.dto.RenamePreviewRequest
import com.leanctx.plugin.dto.TextRangeDTO
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.TimeUnit

class SymbolRefactorerTest : BasePlatformTestCase() {

    private val fixture = """
        package p

        class Outer {
            fun target() {}
        }

        val shared = Outer()
        fun a() { shared.target() }
        fun b() { shared.target() }
    """.trimIndent()

    private fun <T> offEdt(block: () -> T): T {
        val future = ApplicationManager.getApplication().executeOnPooledThread<T> { block() }
        return PlatformTestUtil.waitForFuture(future, TimeUnit.SECONDS.toMillis(60))
    }

    private fun writeFile(rel: String, content: String) {
        val p = Paths.get(project.basePath!!, rel)
        Files.createDirectories(p.parent)
        Files.writeString(p, content)
        WriteAction.computeAndWait<Unit, RuntimeException> {
            val vFile = LocalFileSystem.getInstance().refreshAndFindFileByPath(p.toString())
                ?: error("could not refresh VFS for $p")
            VfsUtil.saveText(vFile, content)
        }
    }

    fun testResolvesIndentedMemberNotEnclosingClass() {
        writeFile("Sample.kt", fixture)
        val req = RenamePreviewRequest(
            path = "Sample.kt",
            range = TextRangeDTO(PositionDTO(3, 0), PositionDTO(3, 0)),
            new_name = "renamed",
        )
        val resp = offEdt { SymbolRefactorer(project).preview(req) }
        // Renaming target() finds its two call sites; renaming the enclosing Outer would
        // instead surface the `Outer()` usage (context "val shared = Outer()").
        assertEquals(2, resp.usages.count { it.context?.contains("shared.target()") == true })
    }
}
```

- [ ] **Step 2: Test laufen — RED bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.SymbolRefactorerTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL — `expected:<2> but was:<0>`.

- [ ] **Step 3a: Import ergänzen**

`ctx_edit` auf `SymbolRefactorer.kt` —
old_string:
```kotlin
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.util.PsiTreeUtil
```
new_string:
```kotlin
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.PsiWhiteSpace
import com.intellij.psi.util.PsiTreeUtil
```

- [ ] **Step 3b: Fix einbauen**

`ctx_edit` auf `SymbolRefactorer.kt` —
old_string:
```kotlin
        val at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at ${req.range.start.line}:${req.range.start.character}")
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
```
new_string:
```kotlin
        var at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at ${req.range.start.line}:${req.range.start.character}")
        // Line-addressed targets (char 0) land on the leading indentation; skip it so
        // getParentOfType resolves the declaration ON the line, not its enclosing
        // class/function. Top-level (col-0) symbols never hit this; surfaced at the v2d
        // inline live-gate (SymbolInliner), ported to the v2c siblings.
        if (at is PsiWhiteSpace) {
            at = PsiTreeUtil.nextLeaf(at) ?: at
        }
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
```

- [ ] **Step 4: Test laufen — GREEN bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.SymbolRefactorerTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS.

- [ ] **Step 5: Reformat + Commit**

Reformat: `mcp__jetbrains__reformat_file` auf `SymbolRefactorer.kt` und `SymbolRefactorerTest.kt`.

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolRefactorer.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/SymbolRefactorerTest.kt
git commit -m "fix(v2c): SymbolRefactorer.resolveTarget skips leading indentation (port v2d inline fix) + RED-first test"
```

---

## Task 4: ReferenceFinder — resolveTarget Whitespace-Skip

**Files:**
- Test (Modify): `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/ReferenceFinderTest.kt` (neue Testmethode anfügen)
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/ReferenceFinder.kt` (Imports + `resolveTarget`, Z.59-61)

Hinweis: `ReferenceFinder.kt` importiert **weder** `PsiWhiteSpace` **noch** `PsiTreeUtil` → beide ergänzen. `ReferenceFinder.find(file, ...)` nimmt die `PsiFile` direkt → `configureByText` reicht (kein Disk-File), Aufruf auf der EDT in `inSmartReadAction` wie der bestehende Test.

- [ ] **Step 1: Failing test schreiben**

`ctx_edit` auf `ReferenceFinderTest.kt` — die neue Methode vor die schließende Klammer der Klasse einfügen.
old_string:
```kotlin
        assertEquals(2, result.total)
    }
}
```
new_string:
```kotlin
        assertEquals(2, result.total)
    }

    fun testResolvesIndentedMemberNotEnclosingClass() {
        val file = myFixture.configureByText(
            "Sample.kt",
            """
            class Outer {
                fun target() {}
            }
            val shared = Outer()
            fun a() { shared.target() }
            fun b() { shared.target() }
            """.trimIndent(),
        )
        val locator = PsiLocator(project)
        val finder = ReferenceFinder(locator)
        // target() is on line 1 (0-based), indented; address char 0 (lands on indentation).
        val result = locator.inSmartReadAction {
            finder.find(file, line = 1, character = 0, scope = "project")
        }
        // Correct resolution → the two `shared.target()` call sites (lines 4 and 5).
        // Wrong resolution (enclosing class Outer) → its single `Outer()` usage (line 3).
        assertEquals(2, result.locations.size)
        assertEquals(setOf(4, 5), result.locations.map { it.range.start.line }.toSet())
    }
}
```

Hinweis Zeilen (nach `trimIndent`, 0-basiert): 0 `class Outer {`, 1 `    fun target() {}`, 2 `}`, 3 `val shared = Outer()`, 4 `fun a() { shared.target() }`, 5 `fun b() { shared.target() }`.

- [ ] **Step 2: Test laufen — RED bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.ReferenceFinderTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL — `expected:<2> but was:<1>` (resolveTarget liefert `Outer`, dessen einzige Referenz `Outer()` auf Zeile 3 ist).

- [ ] **Step 3a: Imports ergänzen**

`ctx_edit` auf `ReferenceFinder.kt` —
old_string:
```kotlin
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.search.GlobalSearchScope
import com.intellij.psi.search.searches.ReferencesSearch
```
new_string:
```kotlin
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.PsiWhiteSpace
import com.intellij.psi.search.GlobalSearchScope
import com.intellij.psi.search.searches.ReferencesSearch
import com.intellij.psi.util.PsiTreeUtil
```

- [ ] **Step 3b: Fix einbauen**

`ctx_edit` auf `ReferenceFinder.kt` —
old_string:
```kotlin
        val element = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no element at $line:$character")
        return generateSequence(element) { it.parent }
```
new_string:
```kotlin
        var element = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no element at $line:$character")
        // Line-addressed targets (char 0) land on the leading indentation; skip it so the
        // parent walk resolves the declaration ON the line, not its enclosing class/function.
        // (findReferenceAt above returns null on whitespace.) Ported from the v2d SymbolInliner fix.
        if (element is PsiWhiteSpace) {
            element = PsiTreeUtil.nextLeaf(element) ?: element
        }
        return generateSequence(element) { it.parent }
```

- [ ] **Step 4: Test laufen — GREEN bestätigen**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.ReferenceFinderTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS (beide Methoden: `testFindsAllUsagesInProjectScope` + `testResolvesIndentedMemberNotEnclosingClass`).

- [ ] **Step 5: Reformat + Commit**

Reformat: `mcp__jetbrains__reformat_file` auf `ReferenceFinder.kt` und `ReferenceFinderTest.kt`.

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/ReferenceFinder.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/ReferenceFinderTest.kt
git commit -m "fix(v2c): ReferenceFinder.resolveTarget skips leading indentation (port v2d inline fix) + RED-first test"
```

---

## Task 5: Voller Testlauf + Runbook-Update

**Files:**
- Modify: `docs/lean-md/runbooks/runide-inline-reformat-gate.md` (Abschnitt „Begleitender Fix")

- [ ] **Step 1: Gesamte Plugin-Testsuite grün**

Run: `./gradlew test` (cwd=`packages/jetbrains-lean-ctx`) — bare command, kein `| tail`/`| grep`.
Expected: BUILD SUCCESSFUL; insbesondere `SymbolDeleterTest`, `SymbolMoverTest`, `SymbolRefactorerTest`, `ReferenceFinderTest` grün, keine Regression in `RequestRouter*Test`.

- [ ] **Step 2: Runbook-Notiz aktualisieren**

`ctx_edit` auf `docs/lean-md/runbooks/runide-inline-reformat-gate.md` —
old_string:
```
  `PsiWhiteSpace` via `PsiTreeUtil.nextLeaf` überspringen. Die v2c-Geschwister
  (`SymbolMover`/`SymbolDeleter`) tragen dasselbe latente Muster, exponieren es aber
  nicht (ihre Testsymbole stehen auf Spalte 0); dort bewusst **nicht** angefasst.
```
new_string:
```
  `PsiWhiteSpace` via `PsiTreeUtil.nextLeaf` überspringen. Die v2c-Geschwister
  (`SymbolMover`/`SymbolDeleter`/`SymbolRefactorer`/`ReferenceFinder`) trugen dasselbe
  latente Muster (Testsymbole auf Spalte 0 → nie exponiert); seit dem v2c-Port
  (Spec `2026-06-13-leanctx-jetbrains-v2c-resolvetarget-indentation-fix.md`) ist der
  Fix dort gespiegelt und je mit einem RED-first-Unit-Test über den Preview-Pfad
  (`SymbolDeleterTest`/`SymbolMoverTest`/`SymbolRefactorerTest`/`ReferenceFinderTest`)
  abgesichert — nicht mehr latent.
```

- [ ] **Step 3: Commit**

```bash
git add docs/lean-md/runbooks/runide-inline-reformat-gate.md
git commit -m "docs(v2c): runbook — resolveTarget indentation fix ported to v2c siblings, no longer latent"
```

---

## Verifikations-Checkliste (Akzeptanzkriterien der Spec)

- [ ] Alle vier `resolveTarget`/`resolveSource` überspringen führende `PsiWhiteSpace` vor dem Parent-Aufstieg (Tasks 1-4 Step 3).
- [ ] Vier RED-first-Tests existieren, je RED ohne Fix (Step 2) und GREEN mit Fix (Step 4).
- [ ] `./gradlew test` vollständig grün, keine Regression (Task 5 Step 1).
- [ ] Geänderte `.kt`-Dateien reformatiert (jeweils Step 5).
- [ ] Runbook-Notiz aktualisiert (Task 5 Step 2).
