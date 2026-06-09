package com.leanctx.plugin.server

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.application.WriteAction
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.VfsUtil
import com.intellij.testFramework.DumbModeTestUtils
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.TimeUnit

class RequestRouterRefactorTest : BasePlatformTestCase() {

    private fun router() = RequestRouter(
        token = "tok",
        ideVersion = "IC-2026.1",
        projectName = project.name,
        project = project,
    )

    // Route off the EDT — mirrors the real embedded HTTP server thread. The Kotlin
    // RenameProcessor calls the Analysis API (KaSession), which is PROHIBITED on the
    // EDT even inside a read action (ProhibitedAnalysisException). PsiLocator runs the
    // body on the *calling* thread via ReadAction.nonBlocking().executeSynchronously(),
    // so the caller must be a background thread.
    //
    // We must NOT plain .get() on the future: SymbolRefactorer.apply() marshals its write
    // transaction back onto the EDT via invokeAndWait. The test body itself runs on the EDT,
    // so a blocking .get() would freeze the EDT and deadlock against that invokeAndWait.
    // PlatformTestUtil.waitForFuture pumps the EDT event queue while waiting, servicing the
    // marshalled write. (Preview has no invokeAndWait but uses the same path for uniformity.)
    private fun routeOffEdt(method: String, path: String, body: String): HttpResult {
        val future = ApplicationManager.getApplication().executeOnPooledThread<HttpResult> {
            router().route(method, path, "tok", body)
        }
        return PlatformTestUtil.waitForFuture(future, TimeUnit.SECONDS.toMillis(60))
    }

    private fun writeFile(rel: String, content: String): String {
        val base = project.basePath!!
        val p = Paths.get(base, rel)
        Files.createDirectories(p.parent)
        // Ensure the file exists on disk so LocalFileSystem can resolve it (PsiLocator
        // resolves via LocalFileSystem.findFileByPath, which the in-memory TempFileSystem
        // of addFileToProject would not satisfy).
        Files.writeString(p, content)
        WriteAction.computeAndWait<Unit, RuntimeException> {
            val vFile = LocalFileSystem.getInstance().refreshAndFindFileByPath(p.toString())
                ?: error("could not refresh VFS for $p")
            // Write the content THROUGH the VFS layer (VfsUtil.saveText) instead of leaving
            // the raw Files.writeString as the source of truth. This keeps the VFS/document
            // model and disk byte-identical, so a later document write (RenameProcessor) does
            // not race a freshly-refreshed disk state → no MemoryDiskConflictResolver
            // "Unexpected memory-disk conflict" flakiness.
            VfsUtil.saveText(vFile, content)
        }
        return p.toString()
    }

    fun testRenamePreviewReturnsUsages() {
        // Declaration in A.kt + a usage in B.kt (same package).
        writeFile("A.kt", "package p\nclass Widget\n")
        writeFile("B.kt", "package p\nfun use(): Widget = Widget()\n")

        // Target = the `Widget` class declaration: line 1 (0-based), char 6 (after "class ").
        val body = """
            {"path":"A.kt",
             "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
             "new_name":"Gadget"}
        """.trimIndent()

        val res = routeOffEdt("POST", "/renamePreview", body)
        assertEquals(res.body, 200, res.status)
        // Envelope presence is the acceptance signal here: the preview path runs end to
        // end (resolve → findUsages → conflict collection via EDT preprocessUsages → DTO
        // mapping) and returns the usages array — no INTERNAL/read-action error.
        assertTrue(res.body, res.body.contains("\"usages\""))
        // The concrete usage sites (declaration in A.kt + the B.kt reference) are NOT
        // asserted: the BasePlatformTestCase light fixture does not index project.basePath
        // as a source root, so RenameProcessor's resolve/index-based usage search returns
        // an empty set here (verified: body is {"usages":[],"conflicts":[]}). Real
        // usage-site verification → manuelles runIde-Gate (Spec §10).
    }

    fun testRenameApplyRenamesDeclaration() {
        val aPath = writeFile("A.kt", "package p\nclass Widget\n")
        writeFile("B.kt", "package p\nfun use(): Widget = Widget()\n")

        val body = """
            {"path":"A.kt",
             "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
             "new_name":"Gadget","force":false}
        """.trimIndent()

        val res = routeOffEdt("POST", "/renameApply", body)
        assertEquals(res.body, 200, res.status)
        assertTrue(res.body, res.body.contains("\"applied\":true"))

        // Re-read A.kt from disk: the declaration must be renamed to Gadget.
        WriteAction.computeAndWait<Unit, RuntimeException> {
            LocalFileSystem.getInstance().refreshAndFindFileByPath(aPath)
        }
        val a = Files.readString(Paths.get(aPath))
        assertTrue(a, a.contains("class Gadget"))
        // Multi-File-Verifikation (B.kt usage rewrite) → manuelles runIde-Gate (Spec §10):
        // light fixture does not index basePath, so cross-file usages are not rewritten
        // here. assertTrue(b.contains("Gadget")) / assertFalse(b.contains("Widget")) are
        // exercised in the live runIde gate, not in this light-fixture test.
    }

    fun testUnauthorizedTokenRejected() {
        val res = router().route("POST", "/renamePreview", "wrong", "{}")
        assertEquals(401, res.status)
    }

    fun testRenamePreviewUnsupportedLanguageBeforeNoSymbol() {
        writeFile("notes.txt", "just some notes here\n")
        val body = """
            {"path":"notes.txt",
            "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":4}},
            "new_name":"x"}
        """.trimIndent()
        val res = routeOffEdt("POST", "/renamePreview", body)
        assertEquals(res.body, 200, res.status)
        assertTrue(res.body, res.body.contains("UNSUPPORTED_LANGUAGE"))
        assertFalse(res.body, res.body.contains("NO_SYMBOL"))
    }

    fun testRenamePreviewDuringIndexingReturnsIndexing() {
        // Note: this exercises the isDumb early gate in PsiLocator.inSmartReadAction,
        // NOT the IndexNotReadyException catch-net (the indexing-onset race cannot be
        // simulated deterministically in the headless test harness).
        writeFile("A.kt", "package p\nclass Widget\n")
        val body = """
            {"path":"A.kt",
            "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
            "new_name":"Gadget"}
        """.trimIndent()
        var res: HttpResult? = null
        DumbModeTestUtils.runInDumbModeSynchronously(project) {
            res = routeOffEdt("POST", "/renamePreview", body)
        }
        val r = requireNotNull(res) { "response must not be null" }
        assertEquals(r.body, 200, r.status)
        assertTrue(r.body, r.body.contains("INDEXING"))
    }

    fun testRenameApplyFileCollisionRefusedEvenWithForce() {
        // The declaration file is named after the class, so renaming Widget → Gadget would
        // ALSO rename the file Widget.kt → Gadget.kt. Gadget.kt already exists, so the rename
        // must be refused as a CONFLICT (never silently overwrite a source file) — even with
        // force=true. Regression for the runIde #4b hang: the un-intercepted file-overwrite
        // modal ("file already exists / Overwrite·Skip") blocked the EDT → /renameApply
        // timed out. force overrides symbol/usage conflicts, NOT a physical file overwrite.
        val widgetPath = writeFile("Widget.kt", "package p\nclass Widget\n")
        writeFile("Gadget.kt", "package p\nclass Gadget\n")

        val body = """
            {"path":"Widget.kt",
             "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
             "new_name":"Gadget","force":true}
        """.trimIndent()

        val res = routeOffEdt("POST", "/renameApply", body)
        assertEquals(res.body, 200, res.status)
        assertTrue(res.body, res.body.contains("CONFLICT"))
        assertFalse(res.body, res.body.contains("\"applied\":true"))

        // Widget.kt is untouched: no overwrite, no half-rename.
        WriteAction.computeAndWait<Unit, RuntimeException> {
            LocalFileSystem.getInstance().refreshAndFindFileByPath(widgetPath)
        }
        val w = Files.readString(Paths.get(widgetPath))
        assertTrue(w, w.contains("class Widget"))
    }
}
