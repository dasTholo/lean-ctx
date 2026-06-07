package com.leanctx.plugin.dto

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class JsonCodecTest {
    @Test
    fun parsesNavRequestWithDefaultScope() {
        val req = JsonCodec.parseNavRequest("""{"path":"src/Foo.kt","line":3,"character":7}""")
        assertEquals("src/Foo.kt", req.path)
        assertEquals(3, req.line)
        assertEquals(7, req.character)
        assertEquals("project", req.scope) // default applied
    }

    @Test
    fun parsesExplicitScope() {
        val req = JsonCodec.parseNavRequest("""{"path":"a","line":0,"character":0,"scope":"all"}""")
        assertEquals("all", req.scope)
    }

    @Test
    fun serializesLocationsResponse() {
        val resp = LocationsResponse(
            locations = listOf(
                LocationDTO("src/Foo.kt", TextRangeDTO(PositionDTO(2, 4), PositionDTO(2, 7)))
            ),
            truncated = false,
            total = 1,
        )
        val json = JsonCodec.toJson(resp)
        assertTrue(json.contains("\"locations\""))
        assertTrue(json.contains("\"path\":\"src/Foo.kt\""))
        assertTrue(json.contains("\"truncated\":false"))
        assertTrue(json.contains("\"total\":1"))
    }
}
