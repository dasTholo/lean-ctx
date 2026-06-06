package com.leanctx.plugin.server

import com.intellij.openapi.Disposable
import com.sun.net.httpserver.HttpServer
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.nio.file.Path
import java.security.SecureRandom
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

/**
 * Per-project localhost HTTP server. lean-ctx (Rust) is the client; this is the server.
 * Disposable → registered against the Project, so projectClosing stops it + deletes the port file.
 */
class BackendHttpServer(
    private val dataDir: Path,
    private val projectRoot: String,
    private val ideVersion: String,
    private val projectName: String,
    private val startedAt: Long,
) : Disposable {
    private val token: String = newToken()
    private var server: HttpServer? = null
    private var executor: ExecutorService? = null
    private var portFile: Path? = null

    val port: Int get() = server?.address?.port ?: -1
    val tokenForTest: String get() = token

    fun start() {
        check(server == null) { "BackendHttpServer already started" }
        val http = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        val router = RequestRouter(token, ideVersion, projectName)
        val exec = Executors.newCachedThreadPool()
        http.executor = exec
        executor = exec
        http.createContext("/") { exchange ->
            try {
                val headerToken = exchange.requestHeaders.getFirst("X-LeanCtx-Token")
                val result = router.route(exchange.requestMethod, exchange.requestURI.path, headerToken)
                val bytes = result.body.toByteArray(StandardCharsets.UTF_8)
                exchange.responseHeaders.add("Content-Type", "application/json")
                exchange.sendResponseHeaders(result.status, bytes.size.toLong())
                exchange.responseBody.use { it.write(bytes) }
            } finally {
                exchange.close()
            }
        }
        http.start()
        server = http

        val pf = LeanCtxPaths.portFile(dataDir, projectRoot)
        PortFileWriter.write(
            pf,
            PortFileData(
                port = http.address.port,
                token = token,
                pid = ProcessHandle.current().pid(),
                projectRoot = projectRoot,
                ideVersion = ideVersion,
                startedAt = startedAt,
            )
        )
        portFile = pf
    }

    override fun dispose() {
        server?.stop(0)
        server = null
        // HttpServer.stop() does not close a user-supplied executor; reclaim its threads now.
        executor?.shutdownNow()
        executor = null
        portFile?.let { PortFileWriter.delete(it) }
        portFile = null
    }

    private fun newToken(): String {
        val bytes = ByteArray(32)
        SecureRandom().nextBytes(bytes)
        return buildString(64) { bytes.forEach { append("%02x".format(it)) } }
    }
}
