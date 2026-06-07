package com.leanctx.plugin.dto

import com.google.gson.Gson
import com.google.gson.GsonBuilder

/** Wire position: 0-based line + character (LSP convention, spec §6). */
data class PositionDTO(val line: Int, val character: Int)

data class TextRangeDTO(val start: PositionDTO, val end: PositionDTO)

/** A single result location. `path` is project-relative (spec §6). */
data class LocationDTO(val path: String, val range: TextRangeDTO)

/** Request body for /references|/definition|/implementations|/declaration. */
data class NavRequest(
    val path: String,
    val line: Int,
    val character: Int,
    val scope: String = "project",
)

/** Response body for the nav endpoints. */
data class LocationsResponse(
    val locations: List<LocationDTO>,
    val truncated: Boolean,
    val total: Int,
)

/** Error envelope: {"error":{"code":..,"message":..}} (spec §6). */
data class ErrorBody(val code: String, val message: String)
data class ErrorResponse(val error: ErrorBody)

object JsonCodec {
    private val gson: Gson = GsonBuilder().disableHtmlEscaping().create()

    fun parseNavRequest(body: String): NavRequest {
        val parsed = gson.fromJson(body, NavRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")
        // gson leaves scope null when the key is absent → apply the default.
        return if (parsed.scope.isNullOrBlank()) parsed.copy(scope = "project") else parsed
    }

    fun toJson(value: Any): String = gson.toJson(value)

    fun error(code: String, message: String): String =
        gson.toJson(ErrorResponse(ErrorBody(code, message)))
}
