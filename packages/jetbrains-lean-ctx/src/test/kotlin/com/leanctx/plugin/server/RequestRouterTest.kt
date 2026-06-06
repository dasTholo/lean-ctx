package com.leanctx.plugin.server

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class RequestRouterTest {
    private val router = RequestRouter(token = "secret", ideVersion = "IC-2026.1.3", projectName = "demo")

    @Test
    fun healthWithValidTokenReturns200() {
        val r = router.route("GET", "/health", "secret")
        assertEquals(200, r.status)
        assertTrue(r.body.contains("\"status\":\"ok\""))
        assertTrue(r.body.contains("\"ideVersion\":\"IC-2026.1.3\""))
        assertTrue(r.body.contains("\"project\":\"demo\""))
    }

    @Test
    fun missingTokenReturns401() {
        val r = router.route("GET", "/health", null)
        assertEquals(401, r.status)
        assertTrue(r.body.contains("UNAUTHORIZED"))
    }

    @Test
    fun wrongTokenReturns401() {
        assertEquals(401, router.route("GET", "/health", "nope").status)
    }

    @Test
    fun unknownPathWithValidTokenReturns404() {
        val r = router.route("GET", "/nope", "secret")
        assertEquals(404, r.status)
    }
}
