package com.leanctx.plugin.psi

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Ref
import com.intellij.psi.PsiDocumentManager
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiNamedElement
import com.intellij.refactoring.rename.RenameProcessor
import com.intellij.usageView.UsageInfo
import com.intellij.util.containers.MultiMap
import com.leanctx.plugin.dto.ConflictDTO
import com.leanctx.plugin.dto.RenameApplyRequest
import com.leanctx.plugin.dto.RenameApplyResponse
import com.leanctx.plugin.dto.RenamePreviewRequest
import com.leanctx.plugin.dto.RenamePreviewResponse
import com.leanctx.plugin.dto.UsageSiteDTO
import com.leanctx.plugin.server.BackendException

/**
 * Multi-File rename via IntelliJ's RenameProcessor — the canonical compiler-semantic
 * (resolve-based) usage search the headless lean-ctx stack cannot provide (spec §3).
 *
 * Preview: findUsages + conflict collection, NO write. Apply: one WriteCommandAction
 * → one Undo entry, saved to disk for lean-ctx. The plan_hash CONFLICT guard lives
 * entirely in Rust; this class never hashes.
 */
class SymbolRefactorer(private val project: Project) {
    private val locator = PsiLocator(project)

    /** Subclass exposing protected findUsages + capturing conflicts without a dialog. */
    private class CapturingProcessor(
        project: Project,
        element: PsiElement,
        newName: String,
        searchInComments: Boolean,
        searchTextOccurrences: Boolean,
    ) : RenameProcessor(project, element, newName, searchInComments, searchTextOccurrences) {
        val captured = MultiMap<PsiElement, String>()

        fun usages(): Array<UsageInfo> = findUsages()

        /** Collect conflicts via preprocessUsages → showConflicts hook, then proceed. */
        fun collectConflicts(usages: Array<UsageInfo>) {
            preprocessUsages(Ref.create(usages))
        }

        public override fun showConflicts(
            conflicts: MultiMap<PsiElement, String>,
            usages: Array<out UsageInfo>?,
        ): Boolean {
            captured.putAllValues(conflicts)
            return true // never block here — the Rust gate decides
        }
    }

    fun preview(req: RenamePreviewRequest): RenamePreviewResponse = locator.inSmartReadAction {
        val element = resolveTarget(req)
        val processor = CapturingProcessor(
            project, element, req.new_name, req.search_comments, req.search_text_occurrences,
        )
        val usages = processor.usages()
        processor.collectConflicts(usages)

        val usageDtos = usages.mapNotNull { info ->
            val el = info.element ?: return@mapNotNull null
            locator.toLocation(el)?.let { UsageSiteDTO(it.path, it.range, contextSnippet(el)) }
        }
        val conflictDtos = processor.captured.entrySet().flatMap { entry ->
            val loc = locator.toLocation(entry.key)
            entry.value.map { msg -> ConflictDTO(loc?.path ?: "", loc?.range, msg) }
        }
        RenamePreviewResponse(usageDtos, conflictDtos)
    }

    fun apply(req: RenameApplyRequest): RenameApplyResponse {
        // Resolve + findUsages in a read action; run the transaction on the EDT.
        val element = locator.inSmartReadAction {
            resolveTarget(
                RenamePreviewRequest(req.path, req.range, req.new_name, false, false)
            )
        }
        val processor = locator.inSmartReadAction {
            CapturingProcessor(project, element, req.new_name, false, false)
        }
        val usages = locator.inSmartReadAction { processor.usages() }

        // Distinct changed files = every usage's file (+ the declaration file).
        val changed = LinkedHashSet<String>()
        locator.inSmartReadAction {
            usages.forEach { info -> info.element?.let { el -> locator.toLocation(el)?.let { changed.add(it.path) } } }
            locator.toLocation(element)?.let { changed.add(it.path) }
        }

        // RenameProcessor.run() performs its own WriteCommandAction → one Undo entry.
        var error: Throwable? = null
        ApplicationManager.getApplication().invokeAndWait {
            try {
                processor.setPreviewUsages(false)
                processor.run()
                // Persist every changed document to disk so lean-ctx (reads from disk) sees it.
                WriteCommandAction.runWriteCommandAction(project) {
                    val fdm = FileDocumentManager.getInstance()
                    PsiDocumentManager.getInstance(project).let { /* commits handled by run() */ }
                    fdm.saveAllDocuments()
                }
            } catch (t: Throwable) {
                error = t
            }
        }
        error?.let { throw it }

        return RenameApplyResponse(applied = true, changed_paths = changed.toList())
    }

    /** Resolve the target PsiElement from the declaration range start (walk to a named decl). */
    private fun resolveTarget(req: RenamePreviewRequest): PsiElement {
        val file = locator.psiFile(req.path)
        val offset = locator.offsetOf(file, req.range.start.line, req.range.start.character)
        val at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at ${req.range.start.line}:${req.range.start.character}")
        return generateSequence(at) { it.parent }
            .firstOrNull { it is PsiNamedElement && (it as PsiNamedElement).name != null }
            ?: throw BackendException("NO_SYMBOL", "no named declaration at target range")
    }

    private fun contextSnippet(el: PsiElement): String? {
        val text = el.containingFile?.text ?: return null
        val range = el.textRange ?: return null
        val lineStart = text.lastIndexOf('\n', range.startOffset).let { if (it < 0) 0 else it + 1 }
        val lineEnd = text.indexOf('\n', range.endOffset).let { if (it < 0) text.length else it }
        return text.substring(lineStart, lineEnd).trim().take(200)
    }
}
