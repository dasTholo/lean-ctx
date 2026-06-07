# JetBrains Phase 3 — PSI-Nav-Endpoints + E2E — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:
> executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fill the Phase-2 HTTP shell with the four PSI navigation endpoints (`/references`, `/definition`,
`/implementations`, `/declaration`) so that `ctx_refactor` against Backing B returns IDE-accurate results, with a
configurable search scope and a result cap.

**Architecture:** The Kotlin plugin gains a `dto/` (gson) + `psi/` (finders under `ReadAction`) + `endpoint/` (one
handler per op) layer; `RequestRouter`/`BackendHttpServer` are extended to carry the `Project` and the POST request
body. The Rust side adds the `declaration` action and an optional `scope` parameter, threaded through the `LspBackend`
trait to `JetBrainsHttpBackend`. Backing A (rust-analyzer) ignores `scope` and keeps returning the default "unsupported"
error for `declaration`.

**Tech Stack:** Rust (`ureq`, `serde_json`, `lsp_types`), Kotlin (IntelliJ Platform PSI, `com.sun.net.httpserver`, gson
`compileOnly`), IC 2026.1.3 / Kotlin 2.3.20, `BasePlatformTestCase` fixtures.

**Spec:** `docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md` §17 (Phase-3 detail design), §6 (wire
protocol), §5.2/§5.3 (PSI APIs / threading).

---

## ⚠ Cross-cutting constraints (read before any task)

- **One commit per phase (§12.3):** Phase 3 lands as **exactly one** commit. **Do NOT commit after individual tasks.**
  Tasks leave the working tree green-but-uncommitted; the single commit is the last task (Task E2).
- **Rust `*.rs` edits → Serena only** (project rule): `mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`,
  `insert_after_symbol`. **Never** native `Edit`/`ctx_edit` on `.rs`.
- **Kotlin/`.kt`, `.gradle.kts` edits → native `Edit`** (Serena Rust rule does not apply); read first with `ctx_read`.
- **Tests:** Rust = `cargo nextest run` (**never** `cargo test`). Kotlin = `./gradlew test`.
- **Wire is 0-based** (line + character); the IDE/PSI side uses 0-based offsets directly. The 1→0 conversion already
  happens once on the Rust input side (`ctx_refactor::handle`, `line.saturating_sub(1)`). Do **not** add another ±1 in
  Kotlin.
- **Deferred-tool reflex:** any MCP tool showing as deferred → `ToolSearch(query="select:<tool>")` first, never a Bash
  workaround.

---

## File Structure

**Rust (modify, Serena edits):**

- `rust/src/lsp/backend.rs` — add `scope: &str` to `references`/`implementations` trait methods.
- `rust/src/lsp/client.rs` — Backing-A trait impl: accept + ignore `scope`.
- `rust/src/lsp/jetbrains_backend.rs` — Backing-B: send `scope` in body; override `declaration` (POST `/declaration`).
- `rust/src/tools/ctx_refactor.rs` — read `scope` arg; dispatch `declaration`; `handle_declaration`.
- `rust/src/tools/registered/ctx_refactor.rs` — schema: `declaration` in enum + `scope` property.

**Kotlin (create), package `com.leanctx.plugin`,
under `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/`:**

- `dto/Wire.kt` — `PositionDTO`, `TextRangeDTO`, `LocationDTO`, `NavRequest`, `LocationsResponse`, `ErrorResponse`,
  `JsonCodec` (gson).
- `server/BackendException.kt` — typed error with a wire `code`.
- `psi/PsiLocator.kt` — path→PsiFile, (line,char)→offset, DumbService guard, element→`LocationDTO`.
- `psi/DefinitionResolver.kt` — definition + declaration (`resolve()?.navigationElement`).
- `psi/ReferenceFinder.kt` — `ReferencesSearch` with scope + cap + truncated.
- `psi/ImplementationFinder.kt` — `DefinitionsScopedSearch`.
- `endpoint/NavHandlers.kt` — `FindReferencesHandler`, `DefinitionHandler`, `ImplementationsHandler`,
  `DeclarationHandler`.

**Kotlin (modify):**

- `server/RequestRouter.kt` — take `Project` + request `body`; POST dispatch.
- `server/BackendHttpServer.kt` — take `Project`; read POST body; pass to router.
- `LeanCtxStartupActivity.kt` — pass `project` into `BackendHttpServer`.
- `build.gradle.kts` — `bundledPlugin("org.jetbrains.kotlin")`, gson `compileOnly` + test.

**Kotlin (create tests), under `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/`:**

- `dto/JsonCodecTest.kt`, `psi/PsiLocatorTest.kt`, `psi/DefinitionResolverTest.kt`, `psi/ReferenceFinderTest.kt`,
  `psi/ImplementationFinderTest.kt`, `server/RequestRouterNavTest.kt`.

---

## Task R1: Rust — `scope` param + `declaration` action (Backing-B wire)

**Files:**

- Modify: `rust/src/lsp/backend.rs` (trait `references`, `implementations`)
- Modify: `rust/src/lsp/client.rs` (trait impl `references`, `implementations`)
- Modify: `rust/src/lsp/jetbrains_backend.rs` (`references`, `implementations`, new `declaration`)
- Modify: `rust/src/tools/ctx_refactor.rs` (dispatch + `handle_references`/`handle_implementations` + new
  `handle_declaration`)

- [ ] **Step 1: Extend the trait signatures (Serena).**

`mcp__serena__jet_brains_find_symbol` for `LspBackend` in `rust/src/lsp/backend.rs`, then `replace_symbol_body` so the
two mandatory method signatures read:

```rust
    fn references(
    &mut self,
    uri: &Uri,
    position: Position,
    scope: &str,
) -> Result<Vec<Location>, String>;
fn implementations(
    &mut self,
    uri: &Uri,
    position: Position,
    scope: &str,
) -> Result<Vec<Location>, String>;
```

Leave `declaration` as the existing default-degrading method (signature unchanged — declaration is single-target, no
scope).

- [ ] **Step 2: Update Backing-A impl to accept + ignore `scope` (Serena).**

In `rust/src/lsp/client.rs`, the `impl LspBackend for LspClient` block (around L427/L441). Use `replace_symbol_body` on
each method so they read:

```rust
    fn references(
    &mut self,
    uri: &lsp_types::Uri,
    position: lsp_types::Position,
    _scope: &str,
) -> Result<Vec<lsp_types::Location>, String> {
    LspClient::references(self, uri, position)
}
```

```rust
    fn implementations(
    &mut self,
    uri: &lsp_types::Uri,
    position: lsp_types::Position,
    _scope: &str,
) -> Result<Vec<lsp_types::Location>, String> {
    LspClient::implementations(self, uri, position)
}
```

(rust-analyzer has no project/library scope toggle here — `_scope` is intentionally ignored.)

- [ ] **Step 3: Backing-B — send `scope`; override `declaration` (Serena).**

In `rust/src/lsp/jetbrains_backend.rs`, replace the `references` and `implementations` methods so they inject `scope`
into the request body, and **add** a `declaration` override. `replace_symbol_body` for the two existing methods:

```rust
    fn references(&mut self, uri: &Uri, position: Position, scope: &str) -> Result<Vec<Location>, String> {
    let mut body = self.position_body(uri, position);
    body["scope"] = serde_json::json!(scope);
    let resp = self.post("/references", &body)?;
    Ok(self.parse_locations(&resp))
}
```

```rust
    fn implementations(&mut self, uri: &Uri, position: Position, scope: &str) -> Result<Vec<Location>, String> {
    let mut body = self.position_body(uri, position);
    body["scope"] = serde_json::json!(scope);
    let resp = self.post("/implementations", &body)?;
    Ok(self.parse_locations(&resp))
}
```

Then `mcp__serena__jet_brains_insert_after_symbol` (after `implementations`) to add:

```rust
    fn declaration(&mut self, uri: &Uri, position: Position) -> Result<Vec<Location>, String> {
    let body = self.position_body(uri, position);
    let resp = self.post("/declaration", &body)?;
    Ok(self.parse_locations(&resp))
}
```

- [ ] **Step 4: Dispatch `declaration` + thread `scope` in the inner handle (Serena).**

In `rust/src/tools/ctx_refactor.rs`, `replace_symbol_body` of `handle` so it reads `scope` and routes `declaration`, and
updates the unknown-action help text:

```rust
pub fn handle(args: &Value, project_root: &str, abs_path: &str) -> String {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("references");
    let line = args.get("line").and_then(Value::as_u64).unwrap_or(1) as u32;
    let column = args.get("column").and_then(Value::as_u64).unwrap_or(0) as u32;
    let scope = args.get("scope").and_then(Value::as_str).unwrap_or("project");
    let uri = match crate::lsp::router::open_file(abs_path, project_root) {
        Ok(u) => u,
        Err(e) => return format!("ERROR: {e}"),
    };
    let position = Position::new(line.saturating_sub(1), column);
    match action {
        "rename" => handle_rename(args, abs_path, project_root, &uri, position),
        "references" => handle_references(abs_path, project_root, &uri, position, scope),
        "definition" => handle_definition(abs_path, project_root, &uri, position),
        "implementations" => handle_implementations(abs_path, project_root, &uri, position, scope),
        "declaration" => handle_declaration(abs_path, project_root, &uri, position),
        _ => format!(
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, implementations, declaration."
        ),
    }
}
```

- [ ] **Step 5: Add `scope` to the two helpers + add `handle_declaration` (Serena).**

`replace_symbol_body` of `handle_references` and `handle_implementations` to take `scope` and pass it through:

```rust
fn handle_references(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
    scope: &str,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.references(uri, position, scope)
    });
    match result {
        Ok(locations) => format_locations(&locations, project_root),
        Err(e) => format!("ERROR: {e}"),
    }
}
```

```rust
fn handle_implementations(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
    scope: &str,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.implementations(uri, position, scope)
    });
    match result {
        Ok(locations) => format_locations(&locations, project_root),
        Err(e) => format!("ERROR: {e}"),
    }
}
```

Then `mcp__serena__jet_brains_insert_after_symbol` (after `handle_implementations`):

```rust
fn handle_declaration(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.declaration(uri, position)
    });
    match result {
        Ok(locations) => format_locations(&locations, project_root),
        Err(e) => format!("ERROR: {e}"),
    }
}
```

- [ ] **Step 6: Add a dispatch test (Serena — append to the `#[cfg(test)] mod tests`).**

`mcp__serena__jet_brains_find_symbol` for the `tests` module in `ctx_refactor.rs`, then `insert_after_symbol` of the
last test fn:

```rust
    /// `declaration` is a known action: the unknown-action arm must not fire for it,
/// and its help text now advertises `declaration`.
#[test]
fn unknown_action_help_lists_declaration() {
    let args = json!({"action": "definitely_bogus", "path": "x.rs", "line": 1});
    let out = super::handle(&args, "/proj", "/proj/x.rs");
    assert!(out.contains("declaration"), "help text missing declaration: {out}");
}
```

> Note: a full `declaration` happy-path needs a live IDE (Backing B) and is covered by the manual `runIde` gate (Task
> E1) + the Kotlin fixtures (Task K4). This unit test only proves the action is wired into the dispatcher.

- [ ] **Step 7: Build + test + lint.**

Run: `cd /home/tholo/Scripts/lean-ctx/rust && cargo nextest run -p lean-ctx lsp:: tools::ctx_refactor`
Expected: PASS (existing `references_parses_wire_locations`, `inner_handle_uses_provided_abs_path_not_raw_args`, plus
the new `unknown_action_help_lists_declaration`).

Run: `cargo clippy -p lean-ctx --all-targets -- -D warnings`
Expected: 0 errors, 0 warnings in the touched files.

> Pre-existing baseline (do not treat as regressions):
`core::pathjail::tests::{rejects_path_outside_root, rejects_symlink_escape_on_unix, error_message_contains_root}` may
> fail under the sandbox env (recorded gate baseline). Everything else must be green.

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task R2: Rust — schema registry (`declaration` + `scope`)

**Files:**

- Modify: `rust/src/tools/registered/ctx_refactor.rs` (the `tool_def(...)` schema in `fn tool_def`)

- [ ] **Step 1: Extend the schema (Serena `replace_symbol_body` of `tool_def`).**

`mcp__serena__jet_brains_find_symbol` for `CtxRefactorTool::tool_def`, then `replace_symbol_body` so the schema enum
includes `declaration` and a `scope` property is added (single schema source — §4.4; do **not** create a second
hand-maintained copy):

```rust
    fn tool_def(&self) -> Tool {
    tool_def(
        "ctx_refactor",
        "LSP-powered refactoring. Actions: rename, references, definition, implementations, declaration. \
             Requires a running language server (rust-analyzer, typescript-language-server, pylsp, gopls) \
             or the JetBrains backend (declaration is JetBrains-only).",
        json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["rename", "references", "definition", "implementations", "declaration"],
                        "description": "Refactoring action"
                    },
                    "path": { "type": "string", "description": "File path" },
                    "line": { "type": "integer", "description": "1-indexed line number" },
                    "column": { "type": "integer", "description": "0-indexed character offset" },
                    "new_name": { "type": "string", "description": "New name (only for rename action)" },
                    "scope": {
                        "type": "string",
                        "enum": ["project", "all"],
                        "description": "Search scope for references/implementations (JetBrains backend). 'project' = project sources only (default); 'all' = include libraries/SDK."
                    }
                },
                "required": ["action", "path", "line"]
            }),
    )
}
```

- [ ] **Step 2: Add a schema test (Serena — `insert_after_symbol` in the test module, or create one if absent).**

If `registered/ctx_refactor.rs` has no `#[cfg(test)]` module, `insert_after_symbol` of the
`impl McpTool for CtxRefactorTool` block:

```rust
#[cfg(test)]
mod schema_tests {
    use super::*;
    use crate::server::tool_trait::McpTool;

    #[test]
    fn schema_advertises_declaration_and_scope() {
        let tool = CtxRefactorTool;
        let def = tool.tool_def();
        let schema = serde_json::to_string(&def.input_schema).unwrap();
        assert!(schema.contains("declaration"), "enum missing declaration: {schema}");
        assert!(schema.contains("\"scope\""), "missing scope property: {schema}");
    }
}
```

> If `def.input_schema` is not directly serializable as shown, serialize the whole `Tool` (
`serde_json::to_string(&def)`) and assert on that string instead — the goal is only to detect schema drift.

- [ ] **Step 3: Run the drift/schema test.**

Run: `cd /home/tholo/Scripts/lean-ctx/rust && cargo nextest run -p lean-ctx ctx_refactor`
Expected: PASS, including `schema_advertises_declaration_and_scope`.

- [ ] **Step 4: Regenerate the MCP-tools reference doc if the repo drift-tests it.**

Run: `cd /home/tholo/Scripts/lean-ctx/rust && cargo nextest run -p lean-ctx generated 2>&1 | head -40`
If a generated-doc drift test fails, regenerate per the repo's documented command (check `docs/reference/generated/`),
then re-run. If no such test exists, skip.

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task K1: Kotlin — build deps (Kotlin PSI fixtures + gson)

**Files:**

- Modify: `packages/jetbrains-lean-ctx/build.gradle.kts`

- [ ] **Step 1: Read the current build file.**

`ctx_read("/home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx/build.gradle.kts")`

- [ ] **Step 2: Add gson + Kotlin bundled plugin to `dependencies`.**

In the `dependencies { intellijPlatform { ... } }` area, add (native `Edit`):

```kotlin
dependencies {
    compileOnly("com.google.code.gson:gson:2.11.0")
    testImplementation("com.google.code.gson:gson:2.11.0")
    testImplementation("junit:junit:4.13.2")
    intellijPlatform {
        intellijIdea("2026.1.3")
        bundledPlugin("org.jetbrains.kotlin")
        testFramework(TestFrameworkType.Platform)
    }
}
```

(gson is `compileOnly` because the IDE bundles gson at runtime — §5.4; the Kotlin bundled plugin is needed for Kotlin
PSI both at runtime and in `BasePlatformTestCase` fixtures.)

- [ ] **Step 3: Verify the project still configures + compiles.**

Run:
`cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew compileKotlin compileTestKotlin --console=plain`
Expected: BUILD SUCCESSFUL (no source changes yet; this only proves the dependency graph resolves and the Kotlin plugin
artifact downloads).

> Risk (spec §17.7): if the Kotlin bundled plugin fails to resolve for IC 2026.1 (K2/Analysis-API coupling), record it
> and fall back to Java fixtures for automated tests + Kotlin only in the manual `runIde` smoke. Do not silently drop
> coverage — log the deviation in the spec.

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task K2: Kotlin — wire DTOs + JsonCodec

**Files:**

- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt`

- [ ] **Step 1: Write the failing round-trip test.**

```kotlin
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
```

- [ ] **Step 2: Run it, verify it fails to compile (types missing).**

Run:
`cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew test --tests "com.leanctx.plugin.dto.JsonCodecTest" --console=plain`
Expected: FAIL — unresolved references `JsonCodec`, `NavRequest`, `LocationsResponse`, etc.

- [ ] **Step 3: Implement the DTOs + codec.**

```kotlin
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
```

> Note: gson can set a `val` to `null` via reflection even though Kotlin declares it non-null; `parseNavRequest`
> normalizes `scope`. The `isNullOrBlank()` call is deliberate (platform-null from gson).

- [ ] **Step 4: Run the test, verify it passes.**

Run: `./gradlew test --tests "com.leanctx.plugin.dto.JsonCodecTest" --console=plain`
Expected: PASS (3 tests).

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task K3: Kotlin — `PsiLocator` (path/offset/location mapping)

**Files:**

- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt`
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/BackendException.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/PsiLocatorTest.kt`

- [ ] **Step 1: Write `BackendException` (typed wire error).**

```kotlin
package com.leanctx.plugin.server

/** Carries a wire error `code` (spec §6) for a fachlicher Negativfall (HTTP 200). */
class BackendException(val code: String, message: String) : RuntimeException(message)
```

- [ ] **Step 2: Write the failing offset/location test (0/1 seam).**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.testFramework.fixtures.BasePlatformTestCase

class PsiLocatorTest : BasePlatformTestCase() {

    fun testOffsetFromZeroBasedLineChar() {
        // line 0 = "class A", line 1 = "fun f() {}"; char 4 on line 1 = the "f" of fun? -> pick 'f' of f()
        val file = myFixture.configureByText("A.kt", "class A\nfun f() {}\n")
        val locator = PsiLocator(project)
        // 0-based: line 1, character 4 -> offset of 'f' in "fun f"
        val offset = locator.offsetOf(file, line = 1, character = 4)
        assertEquals("class A\n".length + 4, offset)
    }

    fun testOutOfRangeLineThrowsPositionError() {
        val file = myFixture.configureByText("A.kt", "class A\n")
        val locator = PsiLocator(project)
        val e = assertThrows(com.leanctx.plugin.server.BackendException::class.java) {
            locator.offsetOf(file, line = 99, character = 0)
        }
        assertEquals("POSITION_OUT_OF_RANGE", e.code)
    }

    fun testToLocationRelativePathAndZeroBasedRange() {
        val file = myFixture.configureByText("A.kt", "class Foo\n")
        val locator = PsiLocator(project)
        val psiClass = file.firstChild // KtClass-ish; use the file's first named element's range
        val loc = locator.toLocation(psiClass)
        assertNotNull(loc)
        assertEquals(0, loc!!.range.start.line) // first line, 0-based
    }
}
```

> `assertThrows` comes from `org.junit.Assert.assertThrows` (junit 4.13). `BasePlatformTestCase` methods (`myFixture`,
`project`) are inherited.

- [ ] **Step 3: Run it, verify it fails.**

Run:
`cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew test --tests "com.leanctx.plugin.psi.PsiLocatorTest" --console=plain`
Expected: FAIL — unresolved reference `PsiLocator`.

- [ ] **Step 4: Implement `PsiLocator`.**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.openapi.application.ReadAction
import com.intellij.openapi.project.DumbService
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.psi.PsiDocumentManager
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.PsiManager
import com.leanctx.plugin.dto.LocationDTO
import com.leanctx.plugin.dto.PositionDTO
import com.leanctx.plugin.dto.TextRangeDTO
import com.leanctx.plugin.server.BackendException
import java.nio.file.Paths

/**
 * Maps wire coordinates (project-relative path + 0-based line/character) to PSI and back.
 * All PSI access must run inside ReadAction (callers use [inSmartReadAction]).
 */
class PsiLocator(private val project: Project) {

    private val projectRoot: String = project.basePath ?: ""

    /** Resolve a project-relative path to a PsiFile, or throw FILE_NOT_FOUND. */
    fun psiFile(relPath: String): PsiFile {
        val abs = Paths.get(projectRoot, relPath).toString()
        val vFile = LocalFileSystem.getInstance().findFileByPath(abs)
            ?: throw BackendException("FILE_NOT_FOUND", "no file at $relPath")
        return PsiManager.getInstance(project).findFile(vFile)
            ?: throw BackendException("FILE_NOT_FOUND", "not a PSI file: $relPath")
    }

    /** 0-based (line, character) → document offset, or throw POSITION_OUT_OF_RANGE. */
    fun offsetOf(file: PsiFile, line: Int, character: Int): Int {
        val doc = PsiDocumentManager.getInstance(project).getDocument(file)
            ?: throw BackendException("INTERNAL", "no document for ${file.name}")
        if (line < 0 || line >= doc.lineCount) {
            throw BackendException("POSITION_OUT_OF_RANGE", "line $line outside 0..${doc.lineCount - 1}")
        }
        val lineStart = doc.getLineStartOffset(line)
        val lineEnd = doc.getLineEndOffset(line)
        val offset = lineStart + character
        if (offset < lineStart || offset > lineEnd) {
            throw BackendException("POSITION_OUT_OF_RANGE", "character $character outside line $line")
        }
        return offset
    }

    /** PSI element → wire location (project-relative path, 0-based range). Null if no physical file. */
    fun toLocation(element: PsiElement): LocationDTO? {
        val containing = element.containingFile ?: return null
        val vFile = containing.virtualFile ?: return null
        val doc = PsiDocumentManager.getInstance(project).getDocument(containing) ?: return null
        val range = element.textRange ?: return null
        val startLine = doc.getLineNumber(range.startOffset)
        val endLine = doc.getLineNumber(range.endOffset)
        val start = PositionDTO(startLine, range.startOffset - doc.getLineStartOffset(startLine))
        val end = PositionDTO(endLine, range.endOffset - doc.getLineStartOffset(endLine))
        val rel = relativize(vFile.path)
        return LocationDTO(rel, TextRangeDTO(start, end))
    }

    private fun relativize(absPath: String): String {
        if (projectRoot.isNotEmpty() && absPath.startsWith(projectRoot)) {
            return absPath.removePrefix(projectRoot).removePrefix("/")
        }
        return absPath
    }

    /**
     * Run [body] in a smart-mode ReadAction. If the IDE is indexing, fail fast with INDEXING
     * instead of blocking the HTTP handler (spec §5.3).
     */
    fun <T> inSmartReadAction(body: () -> T): T {
        if (DumbService.getInstance(project).isDumb) {
            throw BackendException("INDEXING", "IDE is indexing; retry shortly")
        }
        return ReadAction.compute<T, RuntimeException> { body() }
    }
}
```

- [ ] **Step 5: Run the test, verify it passes.**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.PsiLocatorTest" --console=plain`
Expected: PASS.

> If `toLocation` on `file.firstChild` returns a whitespace/non-element range in the Kotlin PSI tree, adjust the test to
> target `file.children.first { it.text.contains("Foo") }`. The production code is unaffected.

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task K4: Kotlin — `DefinitionResolver` (definition + declaration)

**Files:**

- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/DefinitionResolver.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/DefinitionResolverTest.kt`

- [ ] **Step 1: Write the failing test.**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.testFramework.fixtures.BasePlatformTestCase

class DefinitionResolverTest : BasePlatformTestCase() {

    fun testResolvesUsageToDeclaration() {
        val file = myFixture.configureByText(
            "A.kt",
            """
            fun target() {}
            fun caller() { target() }
            """.trimIndent(),
        )
        val locator = PsiLocator(project)
        val resolver = DefinitionResolver(locator)
        // 0-based caret on the "target" call inside caller(): line 1.
        val callLine = 1
        val callCol = file.text.lines()[1].indexOf("target")
        val locs = locator.inSmartReadAction {
            resolver.resolve(file, callLine, callCol)
        }
        assertEquals(1, locs.size)
        // The declaration `fun target()` is on line 0.
        assertEquals(0, locs[0].range.start.line)
    }

    fun testNoSymbolThrows() {
        val file = myFixture.configureByText("A.kt", "fun f() { }\n")
        val locator = PsiLocator(project)
        val resolver = DefinitionResolver(locator)
        val e = assertThrows(com.leanctx.plugin.server.BackendException::class.java) {
            locator.inSmartReadAction {
                resolver.resolve(file, line = 0, character = 8) // inside the empty braces
            }
        }
        assertEquals("NO_SYMBOL_AT_POSITION", e.code)
    }
}
```

- [ ] **Step 2: Run it, verify it fails.**

Run:
`cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew test --tests "com.leanctx.plugin.psi.DefinitionResolverTest" --console=plain`
Expected: FAIL — unresolved reference `DefinitionResolver`.

- [ ] **Step 3: Implement `DefinitionResolver`.**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.psi.PsiFile
import com.leanctx.plugin.dto.LocationDTO
import com.leanctx.plugin.server.BackendException

/**
 * definition + declaration. Both go through the same resolver and normalize via
 * navigationElement (spec §17.1 #7: declaration ≡ definition in Kotlin/Java, by design).
 * Must be called inside a ReadAction (use PsiLocator.inSmartReadAction).
 */
class DefinitionResolver(private val locator: PsiLocator) {

    fun resolve(file: PsiFile, line: Int, character: Int): List<LocationDTO> {
        val offset = locator.offsetOf(file, line, character)
        val reference = file.findReferenceAt(offset)
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no reference at $line:$character")
        val target = reference.resolve()
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "reference did not resolve")
        val nav = target.navigationElement ?: target
        val loc = locator.toLocation(nav)
            ?: throw BackendException("INTERNAL", "resolved element has no physical location")
        return listOf(loc)
    }
}
```

- [ ] **Step 4: Run the test, verify it passes.**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.DefinitionResolverTest" --console=plain`
Expected: PASS (both tests).

> If `findReferenceAt` returns null on the call site for K2 Kotlin (reference range differs), adjust `callCol` to point
> at the middle of `target` in the call. The resolver logic is correct; only the test caret needs to land on the reference
> range.

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task K5: Kotlin — `ReferenceFinder` (scope + cap + truncated)

**Files:**

- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/ReferenceFinder.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/ReferenceFinderTest.kt`

- [ ] **Step 1: Write the failing test.**

```kotlin
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
```

- [ ] **Step 2: Run it, verify it fails.**

Run:
`cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew test --tests "com.leanctx.plugin.psi.ReferenceFinderTest" --console=plain`
Expected: FAIL — unresolved reference `ReferenceFinder`.

- [ ] **Step 3: Implement `ReferenceFinder`.**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.search.GlobalSearchScope
import com.intellij.psi.search.searches.ReferencesSearch
import com.intellij.util.Processor
import com.leanctx.plugin.dto.LocationDTO
import com.leanctx.plugin.dto.LocationsResponse
import com.leanctx.plugin.server.BackendException

/**
 * references via ReferencesSearch. Resolves the target declaration first, then searches.
 * Caps at MAX_LOCATIONS and reports `truncated` when more exist (spec §17.1 #5, §17.3).
 * Must run inside a ReadAction.
 */
class ReferenceFinder(private val locator: PsiLocator) {

    companion object {
        const val MAX_LOCATIONS = 500
    }

    fun find(file: PsiFile, line: Int, character: Int, scope: String): LocationsResponse {
        val target = resolveTarget(file, line, character)
        val searchScope = when (scope) {
            "all" -> GlobalSearchScope.allScope(file.project)
            else -> GlobalSearchScope.projectScope(file.project)
        }
        val locations = ArrayList<LocationDTO>(MAX_LOCATIONS)
        var truncated = false
        ReferencesSearch.search(target, searchScope).forEach(Processor { ref ->
            val element = ref.element
            val loc = locator.toLocation(usageElement(element, ref.rangeInElement.startOffset))
            if (loc != null) locations.add(loc)
            if (locations.size >= MAX_LOCATIONS) {
                truncated = true
                false // stop the search: a cap hit means "more may exist"
            } else {
                true
            }
        })
        return LocationsResponse(
            locations = locations,
            truncated = truncated,
            total = locations.size,
        )
    }

    /** The named declaration to search usages of. */
    private fun resolveTarget(file: PsiFile, line: Int, character: Int): PsiElement {
        val offset = locator.offsetOf(file, line, character)
        // Caret on a usage → resolve to declaration; caret on the declaration name → use it directly.
        val reference = file.findReferenceAt(offset)
        if (reference != null) {
            val resolved = reference.resolve()
            if (resolved != null) return resolved
        }
        val element = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no element at $line:$character")
        return generateSequence(element) { it.parent }
            .firstOrNull { it is PsiNamedElement }
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no named symbol at $line:$character")
    }

    /** Map a usage to the element whose textRange we report (the reference's host element). */
    private fun usageElement(element: PsiElement, @Suppress("UNUSED_PARAMETER") offsetInElement: Int): PsiElement =
        element
}
```

> The cap uses an early-exit Processor: on the (MAX+1)th hit we stop and mark `truncated=true`. `total` reports the
> number returned (spec §17.3 accepts `total == locations.size`; exact-total is a §17.6 follow-up). `usageElement` keeps
> the reference's host element range; refine to `rangeInElement` precision in a later phase if needed.

- [ ] **Step 4: Run the test, verify it passes.**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.ReferenceFinderTest" --console=plain`
Expected: PASS.

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task K6: Kotlin — `ImplementationFinder`

**Files:**

- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/ImplementationFinder.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/ImplementationFinderTest.kt`

- [ ] **Step 1: Write the failing test.**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.testFramework.fixtures.BasePlatformTestCase

class ImplementationFinderTest : BasePlatformTestCase() {

    fun testFindsInterfaceImplementations() {
        val file = myFixture.configureByText(
            "A.kt",
            """
            interface Animal
            class Dog : Animal
            class Cat : Animal
            """.trimIndent(),
        )
        val locator = PsiLocator(project)
        val finder = ImplementationFinder(locator)
        val ifaceCol = file.text.lines()[0].indexOf("Animal")
        val result = locator.inSmartReadAction {
            finder.find(file, line = 0, character = ifaceCol, scope = "project")
        }
        // Dog + Cat
        assertEquals(2, result.locations.size)
        assertFalse(result.truncated)
    }
}
```

- [ ] **Step 2: Run it, verify it fails.**

Run:
`cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew test --tests "com.leanctx.plugin.psi.ImplementationFinderTest" --console=plain`
Expected: FAIL — unresolved reference `ImplementationFinder`.

- [ ] **Step 3: Implement `ImplementationFinder`.**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.search.GlobalSearchScope
import com.intellij.psi.search.PsiElementProcessor
import com.intellij.psi.search.searches.DefinitionsScopedSearch
import com.leanctx.plugin.dto.LocationDTO
import com.leanctx.plugin.dto.LocationsResponse
import com.leanctx.plugin.server.BackendException

/**
 * implementations via DefinitionsScopedSearch (language-neutral: covers Kotlin/Java
 * subclasses and overriding members). Caps like ReferenceFinder. Runs inside a ReadAction.
 */
class ImplementationFinder(private val locator: PsiLocator) {

    fun find(file: PsiFile, line: Int, character: Int, scope: String): LocationsResponse {
        val target = resolveNamed(file, line, character)
        val searchScope = when (scope) {
            "all" -> GlobalSearchScope.allScope(file.project)
            else -> GlobalSearchScope.projectScope(file.project)
        }
        val locations = ArrayList<LocationDTO>(ReferenceFinder.MAX_LOCATIONS)
        var truncated = false
        DefinitionsScopedSearch.search(target, searchScope).forEach(PsiElementProcessor { impl ->
            val named = if (impl is PsiNamedElement) (impl.navigationElement ?: impl) else impl
            locator.toLocation(named)?.let { locations.add(it) }
            if (locations.size >= ReferenceFinder.MAX_LOCATIONS) {
                truncated = true
                false
            } else {
                true
            }
        })
        return LocationsResponse(locations, truncated, locations.size)
    }

    private fun resolveNamed(file: PsiFile, line: Int, character: Int): PsiElement {
        val offset = locator.offsetOf(file, line, character)
        file.findReferenceAt(offset)?.resolve()?.let { return it }
        val element = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no element at $line:$character")
        return generateSequence(element) { it.parent }
            .firstOrNull { it is PsiNamedElement }
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no named symbol at $line:$character")
    }
}
```

> `DefinitionsScopedSearch.search(element, scope)` returns a `Query<PsiElement>`; `forEach(PsiElementProcessor)` is the
> standard early-exit form. If the K2 signature requires `DefinitionsScopedSearch.SearchParameters`, switch to
`DefinitionsScopedSearch.search(target)` (no scope) and filter results by
`searchScope.contains(it.containingFile.virtualFile)` inside the processor — record the deviation.

- [ ] **Step 4: Run the test, verify it passes.**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.ImplementationFinderTest" --console=plain`
Expected: PASS.

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task K7: Kotlin — endpoint handlers

**Files:**

- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/NavHandlers.kt`

- [ ] **Step 1: Implement the four handlers.**

```kotlin
package com.leanctx.plugin.endpoint

import com.intellij.openapi.project.Project
import com.leanctx.plugin.dto.LocationsResponse
import com.leanctx.plugin.dto.NavRequest
import com.leanctx.plugin.psi.DefinitionResolver
import com.leanctx.plugin.psi.ImplementationFinder
import com.leanctx.plugin.psi.PsiLocator
import com.leanctx.plugin.psi.ReferenceFinder

/**
 * One callable per nav op. Each parses an already-deserialized NavRequest, runs PSI in a
 * smart-mode ReadAction, and returns a LocationsResponse. BackendException (typed code) is
 * thrown for fachliche Negativfälle and translated to a wire error by the RequestRouter.
 */
class NavHandlers(project: Project) {
    private val locator = PsiLocator(project)
    private val definitionResolver = DefinitionResolver(locator)
    private val referenceFinder = ReferenceFinder(locator)
    private val implementationFinder = ImplementationFinder(locator)

    fun references(req: NavRequest): LocationsResponse = locator.inSmartReadAction {
        referenceFinder.find(file(req), req.line, req.character, req.scope)
    }

    fun implementations(req: NavRequest): LocationsResponse = locator.inSmartReadAction {
        implementationFinder.find(file(req), req.line, req.character, req.scope)
    }

    fun definition(req: NavRequest): LocationsResponse = locator.inSmartReadAction {
        LocationsResponse(definitionResolver.resolve(file(req), req.line, req.character), truncated = false, total = 1)
            .let { it.copy(total = it.locations.size) }
    }

    fun declaration(req: NavRequest): LocationsResponse = definition(req) // §17.1 #7: ≡ definition

    private fun file(req: NavRequest) = locator.psiFile(req.path)
}
```

> `psiFile()` is called inside `inSmartReadAction` (PSI resolution needs the read lock). `declaration` delegates to
`definition` by design (spec §17.1 #7).

- [ ] **Step 2: Compile check.**

Run: `cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew compileKotlin --console=plain`
Expected: BUILD SUCCESSFUL. (Behavioral coverage of the handlers comes via Task K8's router test + the finder tests
above.)

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task K8: Kotlin — wire `RequestRouter` + `BackendHttpServer` + startup

**Files:**

- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt`
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/BackendHttpServer.kt`
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStartupActivity.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterNavTest.kt`

- [ ] **Step 1: Write the failing router dispatch test.**

```kotlin
package com.leanctx.plugin.server

import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.leanctx.plugin.dto.JsonCodec

class RequestRouterNavTest : BasePlatformTestCase() {

    private fun router() = RequestRouter(
        token = "tok",
        ideVersion = "IC-2026.1",
        projectName = project.name,
        project = project,
    )

    fun testReferencesRouteReturnsLocations() {
        myFixture.configureByText(
            "A.kt",
            """
            fun target() {}
            fun a() { target() }
            """.trimIndent(),
        )
        val declCol = 4 // 0-based char of "target" in "fun target() {}"
        val body = """{"path":"A.kt","line":0,"character":$declCol,"scope":"project"}"""
        val res = router().route("POST", "/references", "tok", body)
        assertEquals(200, res.status)
        assertTrue(res.body.contains("\"locations\""))
        assertTrue(res.body.contains("\"truncated\""))
    }

    fun testWrongTokenIs401() {
        val res = router().route("POST", "/references", "WRONG", "{}")
        assertEquals(401, res.status)
        assertTrue(res.body.contains("UNAUTHORIZED"))
    }

    fun testFileNotFoundIsErrorBodyHttp200() {
        val body = """{"path":"DoesNotExist.kt","line":0,"character":0}"""
        val res = router().route("POST", "/references", "tok", body)
        assertEquals(200, res.status) // fachlicher Negativfall = 200 + error envelope (spec §6)
        assertTrue(res.body.contains("FILE_NOT_FOUND"))
    }

    fun testHealthStillWorks() {
        val res = router().route("GET", "/health", "tok", "")
        assertEquals(200, res.status)
        assertTrue(res.body.contains("\"status\":\"ok\""))
    }
}
```

> Note: A `configureByText` file is materialized in the fixture's in-memory VFS under a temp root. If
`PsiLocator.psiFile("A.kt")` cannot resolve the fixture-relative path against `project.basePath`, change the test to
> resolve via `myFixture.file.virtualFile.path` and pass that as `path` (the production path-join logic is exercised by
> the manual `runIde` gate). Keep `testFileNotFoundIsErrorBodyHttp200` and the 401/health assertions unconditionally.

- [ ] **Step 2: Run it, verify it fails.**

Run:
`cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew test --tests "com.leanctx.plugin.server.RequestRouterNavTest" --console=plain`
Expected: FAIL — `RequestRouter` has no `project` param / no `body` arg on `route`.

- [ ] **Step 3: Rewrite `RequestRouter` (native `Edit`).**

Replace the whole file content:

```kotlin
package com.leanctx.plugin.server

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.leanctx.plugin.dto.JsonCodec
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
            val handler: ((com.leanctx.plugin.dto.NavRequest) -> com.leanctx.plugin.dto.LocationsResponse)? = when (path) {
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
        handler: (com.leanctx.plugin.dto.NavRequest) -> com.leanctx.plugin.dto.LocationsResponse,
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
```

- [ ] **Step 4: Extend `BackendHttpServer` to take `project` + read the POST body (native `Edit`).**

Read it first (`ctx_read`). Add a `project: Project` constructor param, pass it to `RequestRouter`, and read the request
body in the context handler. The relevant edits:

Constructor — add the param:

```kotlin
class BackendHttpServer(
    private val dataDir: Path,
    private val project: com.intellij.openapi.project.Project,
    private val projectRoot: String,
    private val ideVersion: String,
    private val projectName: String,
    private val startedAt: Long,
) : Disposable {
```

Router construction inside `start()`:

```kotlin
        val router = RequestRouter(token, ideVersion, projectName, project)
```

Context handler — read the body and pass it:

```kotlin
        http.createContext("/") { exchange ->
            try {
                val headerToken = exchange.requestHeaders.getFirst("X-LeanCtx-Token")
                val body = exchange.requestBody.readBytes().toString(StandardCharsets.UTF_8)
                val result = router.route(exchange.requestMethod, exchange.requestURI.path, headerToken, body)
                val bytes = result.body.toByteArray(StandardCharsets.UTF_8)
                exchange.responseHeaders.add("Content-Type", "application/json")
                exchange.sendResponseHeaders(result.status, bytes.size.toLong())
                exchange.responseBody.use { it.write(bytes) }
            } finally {
                exchange.close()
            }
        }
```

> `InputStream.readBytes()` is the Kotlin stdlib extension (reads the full stream). For GET /health the body is empty (
`""`) and ignored by the router.

- [ ] **Step 5: Pass `project` from `LeanCtxStartupActivity` (native `Edit`).**

In `startBackend(project)`, update the `BackendHttpServer(...)` constructor call to include `project = project` and
confirm `projectRoot = root` is passed:

```kotlin
            val server = BackendHttpServer(
                dataDir = LeanCtxPaths.dataDir(),
                project = project,
                projectRoot = root,
                ideVersion = ApplicationInfo.getInstance().fullVersion,
                projectName = project.name,
                startedAt = System.currentTimeMillis(),
            )
```

- [ ] **Step 6: Run the router test + full plugin test suite.**

Run: `cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew test --console=plain`
Expected: PASS — `RequestRouterNavTest` (401/health/file-not-found unconditionally; references-route per the Step-1
note) plus all Phase-2 tests (`LeanCtxPathsTest`, `PortFileWriterTest`, `BackendHttpServerTest`) still green (no
regression).

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task E1: Manual `runIde` E2E gate

**Files:** none (verification only).

- [ ] **Step 1: Launch a sandbox IDE.**

Run: `cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew runIde --console=plain` (runs in
background; opens a sandbox IDE).

- [ ] **Step 2: Open a Kotlin test project + locate the port file.**

In the sandbox IDE, open (or create) a small Kotlin project with an interface + two implementors and a function called
from two sites. Then read the port file:

`ctx_read("<data_dir>/jetbrains-<hash>.port")` — confirm `port` + `token` present, perms `0600`.
(`<data_dir>` per `LeanCtxPaths`/`lean_ctx_data_dir`; `<hash>` = `sha256(realpath(projectRoot))[..16]`.)

- [ ] **Step 3: Exercise each endpoint with curl (token from the port file).**

```bash
PORT=<from port file>; TOK=<from port file>
curl -s -H "X-LeanCtx-Token: $TOK" -H 'Content-Type: application/json' \
  -d '{"path":"src/Foo.kt","line":<decl-line-0based>,"character":<col>,"scope":"project"}' \
  http://127.0.0.1:$PORT/references
```

Repeat for `/definition`, `/implementations`, `/declaration`.
Expected: JSON `{locations:[...], truncated:false, total:N}` matching the IDE's **Find Usages** / **Go to Declaration
** / **Go to Implementations**. Verify `scope=all` returns ≥ the `project` count.

- [ ] **Step 4: Verify the end-to-end path through `ctx_refactor` (Backing B).**

With the sandbox IDE still open on that project, from a lean-ctx session pointed at the same project root:
`ctx_refactor(action="references", path="src/Foo.kt", line=<1-based>, column=<0-based>)`
Expected: the same locations (proves Rust select_backend → Backing B → plugin). Then `action="declaration"` returns a
location (Backing-B-only op).

- [ ] **Step 5: Verify the fallback + jail.**

Close the IDE (or test a path outside the project root): `ctx_refactor(action="references", path="../escape.kt", ...)` →
must fail at the PathJail with no HTTP call (spec §10 security). With no IDE running, `ctx_refactor` references against
a rust-analyzer-supported file still works (Backing A regression).

- [ ] **Step 6: Record the gate result** in the spec (a one-line note under §17.8: pass/fail + IDE version).

- [ ] **Do NOT commit** (one commit per phase — Task E2).

---

## Task E2: Single Phase-3 commit

**Files:** all of the above.

- [ ] **Step 1: Reformat every changed file (project rule, before `git add`).**

For each changed `.kt` / `.rs` / `.gradle.kts` file run `mcp__jetbrains__reformat_file` (deferred →
`ToolSearch(query="select:mcp__jetbrains__reformat_file")` first).

- [ ] **Step 2: Final full gates.**

Run:
`cd /home/tholo/Scripts/lean-ctx/rust && cargo nextest run -p lean-ctx && cargo clippy -p lean-ctx --all-targets -- -D warnings`
Expected: green (modulo the recorded pathjail-env baseline).

Run: `cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx && ./gradlew test --console=plain`
Expected: BUILD SUCCESSFUL.

- [ ] **Step 3: Stage + commit (one commit for the whole phase, §12.3).**

```bash
cd /home/tholo/Scripts/lean-ctx
git add rust/src/lsp/backend.rs rust/src/lsp/client.rs rust/src/lsp/jetbrains_backend.rs \
        rust/src/tools/ctx_refactor.rs rust/src/tools/registered/ctx_refactor.rs \
        packages/jetbrains-lean-ctx/build.gradle.kts \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin \
        docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md
git commit -m "feat(jetbrains): Phase 3 — PSI nav endpoints (references/definition/implementations/declaration) + scope + cap

Kotlin: dto/ (gson) + psi/ finders (ReadAction, DumbService guard) + endpoint/ handlers;
RequestRouter/BackendHttpServer carry Project + POST body. Rust: declaration action +
configurable scope (project|all) threaded through LspBackend to JetBrainsHttpBackend;
schema registry updated. Verified via Kotlin fixtures + manual runIde gate (spec §17)."
```

- [ ] **Step 4: Confirm the single commit.**

Run: `git log --oneline -1`
Expected: the Phase-3 commit on `feat-jetbrains-plugin`.

---

## Self-Review (against spec §17)

- **§17.1 #1 nav-only:** Tasks R1–K8 cover references/definition/implementations/declaration only; no
  type_hierarchy/overview/format/inspections. ✓
- **§17.1 #2 test strategy:** Kotlin fixtures (K2–K8) + manual runIde (E1). ✓
- **§17.1 #3 Kotlin-only:** all fixtures use `.kt` + `bundledPlugin("org.jetbrains.kotlin")` (K1); risk + fallback
  documented (K1 Step 3). ✓
- **§17.1 #4 scope (project|all, default project):** wire DTO (K2), finders (K5/K6), Rust threading (R1), schema (R2). ✓
- **§17.1 #5 cap 500 + truncated/total:** `ReferenceFinder.MAX_LOCATIONS` + early-exit (K5), reused by K6. ✓
- **§17.1 #6 gson:** `JsonCodec` + DTOs (K2), `compileOnly` (K1). ✓
- **§17.1 #7 declaration ≡ definition:** `DefinitionResolver` shared; `NavHandlers.declaration → definition` (K4/K7);
  Rust override returns plain locations (R1). ✓
- **§17.4 wire response {locations,truncated,total}; Rust ignores extra fields:** `parse_locations` reads only
  `locations` (unchanged) — forward-compatible. ✓
- **§17.5 Rust deltas (scope through ctx_refactor; tool_def single source):** R1 + R2. ✓
- **§17.6 follow-ups:** total-exactness + scope=all volume noted in code comments (K5); §14.1 carry-overs (project_root
  canonicalization, stale-cache) are out of Phase-3 scope and remain tracked in the spec.
- **§17.8 gate:** E1 (runIde + curl + ctx_refactor + fallback + jail) and final gates in E2. ✓
- **§12.3 one commit per phase:** every implementation task says "do NOT commit"; single commit in E2. ✓

**Type consistency check:** `LocationsResponse(locations, truncated, total)`,
`NavRequest(path, line, character, scope)`, `BackendException(code, message)`,
`PsiLocator.{psiFile, offsetOf, toLocation, inSmartReadAction}`, `ReferenceFinder.MAX_LOCATIONS` — names used
identically across K2–K8. ✓

**Open risk (carried, not blocking):** exact IC 2026.1 / K2 signatures for `ReferencesSearch.search`,
`DefinitionsScopedSearch.search`, and `findReferenceAt` on Kotlin call sites. Each affected task carries an inline
fallback note; deviations must be recorded in the spec, not silently coded around.
