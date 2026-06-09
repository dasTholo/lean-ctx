package com.leanctx.plugin.endpoint

import com.intellij.openapi.project.Project
import com.leanctx.plugin.dto.RenameApplyRequest
import com.leanctx.plugin.dto.RenameApplyResponse
import com.leanctx.plugin.dto.RenamePreviewRequest
import com.leanctx.plugin.dto.RenamePreviewResponse
import com.leanctx.plugin.psi.SymbolRefactorer

/**
 * Endpoint layer for the Two-Phase rename. Preview runs PSI off-EDT in a smart-mode
 * read action (SymbolRefactorer.preview). Apply runs the Multi-File transaction on
 * the EDT (SymbolRefactorer.apply handles invokeAndWait + WriteCommandAction).
 */
class RefactorHandlers(project: Project) {
    private val refactorer = SymbolRefactorer(project)

    fun renamePreview(req: RenamePreviewRequest): RenamePreviewResponse = refactorer.preview(req)

    fun renameApply(req: RenameApplyRequest): RenameApplyResponse = refactorer.apply(req)
}
