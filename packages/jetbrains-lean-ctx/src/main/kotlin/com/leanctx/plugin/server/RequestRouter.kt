package com.leanctx.plugin.server

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.leanctx.plugin.dto.JsonCodec
import com.leanctx.plugin.dto.LocationsResponse
import com.leanctx.plugin.dto.NavRequest
import com.leanctx.plugin.endpoint.NavHandlers

data class HttpResult(val status: Int, val body: String)

/**
 * Token-guarded request routing. Phase 3 adds the four POST nav endpoints alongside
 * GET /health. PSI work is delegated to NavHandlers (read-action guarded).
 */
class RequestRouter(
    private val token: String,
    private val ideVersion: String,
    private val projectName: String,
    project: Project,
) {
    private val log = Logger.getInstance(RequestRouter::class.java)
    private val handlers = NavHandlers(project)

    fun route(method: String, path: String, headerToken: String?, body: String): HttpResult {
        if (headerToken != token) {
            return HttpResult(401, JsonCodec.error("UNAUTHORIZED", "missing or invalid token"))
        }
        if (method == "GET" && path == "/health") {
            return HttpResult(200, "{\"status\":\"ok\",\"ideVersion\":${q(ideVersion)},\"project\":${q(projectName)}}")
        }
        if (method == "POST") {
            val handler: ((NavRequest) -> LocationsResponse)? = when (path) {
                "/references" -> handlers::references
                "/definition" -> handlers::definition
                "/implementations" -> handlers::implementations
                "/declaration" -> handlers::declaration
                else -> null
            }
            if (handler != null) {
                return dispatch(body, handler)
            }
        }
        return HttpResult(404, JsonCodec.error("NOT_FOUND", "no route for $method $path"))
    }

    private fun dispatch(
        body: String,
        handler: (NavRequest) -> LocationsResponse,
    ): HttpResult = try {
        val req = JsonCodec.parseNavRequest(body)
        HttpResult(200, JsonCodec.toJson(handler(req)))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code)) // fachlicher Negativfall = 200
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("nav endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error")) // 500 = echte Exception
    }

    private fun q(s: String) = "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"") + "\""
}
