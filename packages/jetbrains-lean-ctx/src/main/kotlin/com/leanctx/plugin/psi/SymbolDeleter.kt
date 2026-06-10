package com.leanctx.plugin.psi

import com.intellij.lang.LanguageRefactoringSupport
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.CommandProcessor
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileTypes.PlainTextFileType
import com.intellij.openapi.fileTypes.PlainTextLanguage
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.search.searches.ReferencesSearch
import com.intellij.psi.util.PsiTreeUtil
import com.intellij.refactoring.safeDelete.SafeDeleteProcessor
import com.leanctx.plugin.dto.ConflictDTO
import com.leanctx.plugin.dto.RenameApplyResponse
import com.leanctx.plugin.dto.RenamePreviewResponse
import com.leanctx.plugin.dto.SafeDeleteApplyRequest
import com.leanctx.plugin.dto.SafeDeletePreviewRequest
import com.leanctx.plugin.dto.UsageSiteDTO
import com.leanctx.plugin.server.BackendException

/**
 * Safe-delete via IntelliJ's SafeDeleteProcessor (spec §6). Preview reports the
 * remaining (blocking) references as usages+conflicts (NO write). Apply runs the
 * delete as one CommandProcessor.executeCommand → one Undo entry, saved to disk
 * for lean-ctx. The plan_hash + conflict gate live entirely in Rust; this class
 * never hashes.
 *
 * API note (IC-2026.1.3): SafeDeleteProcessor is final with a private constructor.
 * The only public entry point is SafeDeleteProcessor.createInstance(project, runnable?,
 * elements[], searchInComments, searchInNonJavaFiles). There is no deleteEvenIfUsed/
 * force parameter — the Rust gate owns that decision; apply() calls run() unconditionally.
 * For preview, ReferencesSearch is used (same approach as SymbolMover) since findUsages()
 * is protected and the class cannot be subclassed.
 */
class SymbolDeleter(private val project: Project) {
    private val locator = PsiLocator(project)

    fun preview(req: SafeDeletePreviewRequest): RenamePreviewResponse {
        val (element, refDtos) = locator.inSmartReadAction {
            val el = resolveTarget(req.path, req.range.start.line, req.range.start.character)
            // Collect all references to the symbol — these are the "blocking" usages that
            // prevent a safe delete. ReferencesSearch is used because SafeDeleteProcessor
            // is final (cannot subclass to expose protected findUsages()).
            val refs = ReferencesSearch.search(el).findAll()
            val dtos = refs.mapNotNull { ref ->
                val refEl = ref.element
                // Skip the declaration itself.
                if (PsiTreeUtil.isAncestor(el, refEl, false)) return@mapNotNull null
                locator.toLocation(refEl)?.let { UsageSiteDTO(it.path, it.range, contextSnippet(refEl)) }
            }
            Pair(el, dtos)
        }
        // Every remaining reference is a blocking conflict (spec §5.4).
        val conflictDtos = refDtos.map { ConflictDTO(it.path, it.range, "symbol is still referenced here") }
        return RenamePreviewResponse(refDtos, conflictDtos)
    }

    fun apply(req: SafeDeleteApplyRequest): RenameApplyResponse {
        val element = locator.inSmartReadAction {
            resolveTarget(req.path, req.range.start.line, req.range.start.character)
        }
        val changed = LinkedHashSet<String>()
        locator.inSmartReadAction { locator.toLocation(element)?.let { changed.add(it.path) } }
        var error: Throwable? = null
        ApplicationManager.getApplication().invokeAndWait {
            try {
                CommandProcessor.getInstance().executeCommand(project, {
                    // createInstance(project, prepareSuccessfulCallback, elements,
                    //                searchInComments, searchInNonJavaFiles)
                    // No force/deleteEvenIfUsed param exists in IC-2026.1.3 — the Rust gate
                    // already blocked the non-force path; we proceed unconditionally here.
                    val processor = SafeDeleteProcessor.createInstance(
                        project, null, arrayOf(element), false, false,
                    )
                    processor.run()
                    WriteCommandAction.runWriteCommandAction(project) {
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

    /** Resolve the target named declaration from a 0-based (line, character), or throw. */
    private fun resolveTarget(relPath: String, line: Int, character: Int): PsiElement {
        val file = locator.psiFile(relPath)
        val lang = file.language
        if (lang == PlainTextLanguage.INSTANCE ||
            file.fileType == PlainTextFileType.INSTANCE ||
            LanguageRefactoringSupport.getInstance().forLanguage(lang) == null
        ) {
            throw BackendException("UNSUPPORTED_LANGUAGE", "safe_delete not supported for ${lang.id}")
        }
        val offset = locator.offsetOf(file, line, character)
        val at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at $line:$character")
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
        if (named != null && named.name != null) return named
        throw BackendException("NO_SYMBOL", "no named declaration at target range")
    }

    private fun contextSnippet(el: PsiElement): String? {
        val text = el.containingFile?.text ?: return null
        val range = el.textRange ?: return null
        val lineStart = text.lastIndexOf('\n', range.startOffset).let { if (it < 0) 0 else it + 1 }
        val lineEnd = text.indexOf('\n', range.endOffset).let { if (it < 0) text.length else it }
        return text.substring(lineStart, lineEnd).trim().take(200)
    }
}
