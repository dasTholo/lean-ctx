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

/** Request body for /type_hierarchy. direction ∈ {supertypes, subtypes}. */
data class HierarchyRequest(
    val path: String,
    val line: Int,
    val character: Int,
    val direction: String = "supertypes",
    val scope: String = "project",
)

/** Request body for /symbols_overview (file-level). */
data class FileRequest(val path: String)

/**
 * A node in a super/subtype tree. `line` is 1-BASED (matches Rust TypeHierarchyNode.line),
 * unlike the 0-based PositionDTO used by nav endpoints.
 */
data class TypeHierarchyNodeDTO(
    val name: String,
    val path: String,
    val line: Int,
    val children: List<TypeHierarchyNodeDTO>,
)

data class TypeHierarchyResponse(val tree: TypeHierarchyNodeDTO, val truncated: Boolean)

/** A single top-level symbol. `line` is 1-BASED (matches Rust SymbolOverviewItem.line). */
data class SymbolOverviewItemDTO(val name: String, val kind: String, val line: Int)

data class SymbolsOverviewResponse(
    val symbols: List<SymbolOverviewItemDTO>,
    val truncated: Boolean,
    val total: Int,
)

/** A single inspection diagnostic. `line` is 1-BASED (matches Rust InspectionDiag.line). */
data class InspectionDiagDTO(
    val path: String,
    val line: Int,
    val severity: String,
    val message: String,
)

data class InspectionsResponse(
    val diagnostics: List<InspectionDiagDTO>,
    val truncated: Boolean,
    val total: Int,
)

/** A single available inspection (the `list` mode). */
data class InspectionInfoDTO(val id: String, val name: String, val severity: String)

data class ListInspectionsResponse(
    val inspections: List<InspectionInfoDTO>,
    val truncated: Boolean,
    val total: Int,
)

object JsonCodec {
    private val gson: Gson = GsonBuilder().disableHtmlEscaping().create()

    fun parseNavRequest(body: String): NavRequest {
        val parsed = gson.fromJson(body, NavRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")
        // gson leaves scope null when the key is absent → apply the default.
        return if (parsed.scope.isNullOrBlank()) parsed.copy(scope = "project") else parsed
    }

    fun parseHierarchyRequest(body: String): HierarchyRequest {
        val parsed = gson.fromJson(body, HierarchyRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")
        val direction = if (parsed.direction.isNullOrBlank()) "supertypes" else parsed.direction
        val scope = if (parsed.scope.isNullOrBlank()) "project" else parsed.scope
        return parsed.copy(direction = direction, scope = scope)
    }

    fun parseFileRequest(body: String): FileRequest =
        gson.fromJson(body, FileRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")

    fun toJson(value: Any): String = gson.toJson(value)

    fun error(code: String, message: String): String =
        gson.toJson(ErrorResponse(ErrorBody(code, message)))
}
