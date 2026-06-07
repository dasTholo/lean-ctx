package com.leanctx.plugin.psi

import com.intellij.testFramework.fixtures.BasePlatformTestCase

class ReferenceFinderTest : BasePlatformTestCase() {

    fun testFindsAllUsagesInProjectScope() {
        val file = myFixture.configureByText(
            "A.kt",
            """
            fun target() {}
            fun a() { target() }
            fun b() { target() }
            """.trimIndent(),
        )
        val locator = PsiLocator(project)
        val finder = ReferenceFinder(locator)
        val declCol = file.text.lines()[0].indexOf("target")
        val result = locator.inSmartReadAction {
            finder.find(file, line = 0, character = declCol, scope = "project")
        }
        // two call sites
        assertEquals(2, result.locations.size)
        assertFalse(result.truncated)
        assertEquals(2, result.total)
    }
}
