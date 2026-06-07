package com.leanctx.plugin

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.application.ApplicationInfo
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.ProjectActivity
import com.intellij.openapi.util.Disposer
import com.leanctx.plugin.server.BackendHttpServer
import com.leanctx.plugin.server.LeanCtxPaths

class LeanCtxStartupActivity : ProjectActivity {
    private val log = Logger.getInstance(LeanCtxStartupActivity::class.java)

    override suspend fun execute(project: Project) {
        val binary = BinaryResolver.resolve()
        if (binary == null) {
            NotificationGroupManager.getInstance()
                .getNotificationGroup("lean-ctx")
                .createNotification(
                    "lean-ctx binary not found",
                    "Install with: cargo install lean-ctx\nOr: npm install -g lean-ctx-bin",
                    NotificationType.WARNING
                )
                .notify(project)
        }
        startBackend(project)
    }

    /** Boot the per-project HTTP backend; failures must never break the IDE/companion. */
    private fun startBackend(project: Project) {
        val root = project.basePath ?: return
        try {
            val server = BackendHttpServer(
                dataDir = LeanCtxPaths.dataDir(),
                project = project,
                projectRoot = root,
                ideVersion = ApplicationInfo.getInstance().fullVersion,
                projectName = project.name,
                startedAt = System.currentTimeMillis(),
            )
            // Register before start: if register throws nothing is running; if start throws,
            // Disposer still cleans up the (idempotent, null-safe) instance on project close.
            Disposer.register(project, server)
            server.start()
            log.info("lean-ctx backend listening on 127.0.0.1:${server.port} for $root")
        } catch (e: Exception) {
            log.warn("lean-ctx backend failed to start", e)
        }
    }
}
