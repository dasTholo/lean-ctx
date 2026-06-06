package com.leanctx.plugin.server

data class HttpResult(val status: Int, val body: String)

/** Token-guarded request routing. Phase 2 serves only GET /health. */
class RequestRouter(
    private val token: String,
    private val ideVersion: String,
    private val projectName: String,
) {
    fun route(method: String, path: String, headerToken: String?): HttpResult {
        if (headerToken != token) {
            return HttpResult(401, errorJson("UNAUTHORIZED", "missing or invalid token"))
        }
        return when {
            method == "GET" && path == "/health" -> HttpResult(
                200,
                "{\"status\":\"ok\",\"ideVersion\":${q(ideVersion)},\"project\":${q(projectName)}}"
            )
            else -> HttpResult(404, errorJson("NOT_FOUND", "no route for $method $path"))
        }
    }

    private fun errorJson(code: String, msg: String) =
        "{\"error\":{\"code\":${q(code)},\"message\":${q(msg)}}}"

    private fun q(s: String) = "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"") + "\""
}
