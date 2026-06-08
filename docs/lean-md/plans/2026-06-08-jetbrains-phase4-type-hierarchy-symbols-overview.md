# JetBrains Phase 4 — `type_hierarchy` + `symbols_overview` (B-only) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add two B-only (JetBrains-PSI-only) code-intelligence ops — `type_hierarchy` (super/subtype tree, depth+node capped) and `symbols_overview` (flat top-level file structure) — wired end-to-end from `ctx_refactor` through `JetBrainsHttpBackend` to two new PSI endpoints in the plugin, with clean degradation on Backing A.

**Architecture:** The Rust trait shapes already exist (`rust/src/lsp/backend.rs`: `TypeHierarchyNode`, `SymbolOverviewItem`, `HierarchyDirection`, both default-`Err`). Phase 4 (a) extends the `ctx_refactor` schema + dispatch, (b) overrides the two methods in `JetBrainsHttpBackend` with HTTP POST + parsers, and (c) adds the plugin-side PSI resolution (`TypeHierarchyResolver`, `FileStructureScanner`), two endpoint handlers, and two router routes. Mirrors the Phase-3 nav pattern 1:1 (gson DTOs, `inSmartReadAction`, cap+`truncated`, off-EDT execution for K2 search).

**Tech Stack:** Rust (`lsp_types`, `serde_json`, `ureq`), Kotlin (IntelliJ Platform PSI, Kotlin PSI/light-classes, gson), `cargo nextest`, Gradle + `BasePlatformTestCase` (JUnit4 runner / JUnit3 hierarchy).

---

## ⚠ Cross-cutting constraints (read before any task)

1. **Rust `*.rs` edits → Serena tools only** (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_after_symbol`, …). Never native `Edit`/`ctx_edit` on `.rs`.
2. **Kotlin/markdown edits → `ctx_edit`** (non-Rust). New files → `Write`.
3. **Tests:** Rust `cargo nextest run` (never `cargo test`). Kotlin `./gradlew test`. Run bare via `ctx_shell` with `cwd=`, never `cd … &&`, never `| tail`/`2>&1`.
4. **0/1 seam (CRITICAL):**
   - Nav DTOs (`PositionDTO`) stay **0-based** (unchanged).
   - **`TypeHierarchyNodeDTO.line` and `SymbolOverviewItemDTO.line` are 1-based** — matches the Rust struct docs (`backend.rs:26` "1-indexed line", `backend.rs:36` "1-indexed line"). The plugin converts `Document.getLineNumber(...)` (0-based) → `+1`. Rust parses verbatim; `ctx_refactor` prints it verbatim.
5. **K2 / EDT:** Kotlin subtype search (`ClassInheritorsSearch`/`OverridingMethodsSearch`) + Kotlin supertype resolution use the K2 Analysis API, **forbidden on the EDT** (`ProhibitedAnalysisException`). Production HTTP handlers run off-EDT, so `inSmartReadAction` (current-thread `ReadAction.compute`) is correct there. **In `BasePlatformTestCase` the test thread IS the EDT** → any test that triggers Kotlin hierarchy search MUST wrap the call in `ApplicationManager.getApplication().executeOnPooledThread<T> { … }.get()` (see `ImplementationFinderTest` for the exact pattern). The K2 analysis phase is fast (up to ~376% quicker than K1, per the K2 migration benchmarks), so synchronous off-EDT resolution is latency-acceptable.
6. **Single schema source:** the `ctx_refactor` JSON schema lives **only** in `rust/src/tools/registered/ctx_refactor.rs::tool_def`. Do not add a second schema copy. The drift test there must stay green.
7. **DTO location:** the existing codebase keeps **all** wire DTOs + the `JsonCodec` in one file `dto/Wire.kt` (spec §3.1 listed separate `dto/*.kt` files; we follow the codebase convention and add to `Wire.kt` instead — documented deviation).
8. **Degradation:** Backing A keeps the trait-default `Err` → surfaces as `ERROR: type_hierarchy requires the JetBrains backend` (no crash, no silent A-fallback). Backing B, non-class/non-method target → `UNSUPPORTED_LANGUAGE` (HTTP-200 envelope).

---

## File Structure

**Rust (modify):**
- `rust/src/tools/registered/ctx_refactor.rs` — schema: `+type_hierarchy`,`+symbols_overview` actions, `+direction` enum param; extend drift test.
- `rust/src/tools/ctx_refactor.rs` — dispatch arms + `handle_type_hierarchy`, `handle_symbols_overview`, `format_type_hierarchy`, `format_symbols_overview`, `parse_direction`; StubBackend tests.
- `rust/src/lsp/jetbrains_backend.rs` — override `type_hierarchy` + `symbols_overview` (HTTP POST + parsers `parse_type_hierarchy`, `parse_symbols`); mock-server tests.
- `rust/src/lsp/backend.rs` — **no change** (shapes already present).

**Kotlin (create):**
- `…/psi/TypeHierarchyResolver.kt` — class/method super/subtype tree, depth+node cap.
- `…/psi/FileStructureScanner.kt` — top-level symbols of a `KtFile`.
- `…/endpoint/StructureHandlers.kt` — `typeHierarchy` + `symbolsOverview`, read-action guarded.
- `…/test/.../psi/TypeHierarchyResolverTest.kt`, `…/psi/FileStructureScannerTest.kt`, `…/server/RequestRouterStructureTest.kt`.

**Kotlin (modify):**
- `…/dto/Wire.kt` — new DTOs + `JsonCodec.parseHierarchyRequest`/`parseFileRequest`.
- `…/server/RequestRouter.kt` — `+/type_hierarchy`, `+/symbols_overview` routes + 2 dispatch helpers.
- `…/test/.../dto/JsonCodecTest.kt` — parse tests for the two new requests.

---

## Task R1: Rust — schema (`type_hierarchy`/`symbols_overview` actions + `direction`)

**Files:**
- Modify: `rust/src/tools/registered/ctx_refactor.rs:15-42` (`tool_def`), `:75-87` (`schema_tests`)

- [ ] **Step 1: Extend the drift test to require the new surface**

In `rust/src/tools/registered/ctx_refactor.rs`, replace the body of `schema_advertises_declaration_and_scope` (use Serena `replace_symbol_body` on `schema_tests::schema_advertises_declaration_and_scope`) with:

```rust
let tool = CtxRefactorTool;
let def = tool.tool_def();
let schema = serde_json::to_string(&def).unwrap();
for needle in [
    "declaration",
    "\"scope\"",
    "type_hierarchy",
    "symbols_overview",
    "\"direction\"",
    "supertypes",
    "subtypes",
] {
    assert!(schema.contains(needle), "schema missing {needle}: {schema}");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo nextest run -p lean-ctx schema_advertises_declaration_and_scope` (cwd=`rust`)
Expected: FAIL — `schema missing type_hierarchy`.

- [ ] **Step 3: Add the actions + `direction` param to the schema**

In `tool_def`, edit the `json!({...})` schema (Serena `replace_symbol_body` on `CtxRefactorTool::tool_def`). Change the `action` enum and add a `direction` property:

```rust
fn tool_def(&self) -> Tool {
    tool_def(
        "ctx_refactor",
        "LSP-powered refactoring. Actions: rename, references, definition, implementations, \
         declaration, type_hierarchy, symbols_overview. Requires a running language server \
         (rust-analyzer, typescript-language-server, pylsp, gopls) or the JetBrains backend \
         (declaration, type_hierarchy, symbols_overview are JetBrains-only).",
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["rename", "references", "definition", "implementations",
                             "declaration", "type_hierarchy", "symbols_overview"],
                    "description": "Refactoring action"
                },
                "path": { "type": "string", "description": "File path" },
                "line": { "type": "integer", "description": "1-indexed line number" },
                "column": { "type": "integer", "description": "0-indexed character offset" },
                "new_name": { "type": "string", "description": "New name (only for rename action)" },
                "scope": {
                    "type": "string",
                    "enum": ["project", "all"],
                    "description": "Search scope for references/implementations/type_hierarchy (JetBrains backend). 'project' = project sources only (default); 'all' = include libraries/SDK."
                },
                "direction": {
                    "type": "string",
                    "enum": ["supertypes", "subtypes"],
                    "description": "type_hierarchy direction (JetBrains backend). 'supertypes' (default) = parents; 'subtypes' = children/implementors."
                }
            },
            "required": ["action", "path"]
        }),
    )
}
```

Note: `line` is dropped from `required` because `symbols_overview` is a file-level op (no line). `type_hierarchy`/nav still default `line` to 1 in the dispatcher (R2).

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo nextest run -p lean-ctx schema_advertises_declaration_and_scope` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add rust/src/tools/registered/ctx_refactor.rs
git commit -m "feat(refactor): schema — type_hierarchy/symbols_overview actions + direction param"
```

---

## Task R2: Rust — dispatch + handlers + formatters (`ctx_refactor.rs`)

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs` (`handle` `:6-38`, add handlers + formatters + tests)

- [ ] **Step 1: Write the failing tests**

In `rust/src/tools/ctx_refactor.rs` test module, add a StubBackend-driven test. The existing `StubBackend` (in the `unknown_action_help_lists_declaration` test) implements only the 5 mandatory methods; add `type_hierarchy` + `symbols_overview` overrides there and a new test. Using Serena `insert_after_symbol` after the last test fn, add:

```rust
#[test]
fn type_hierarchy_formats_indented_tree() {
    use crate::lsp::backend::{HierarchyDirection, SymbolOverviewItem, TypeHierarchyNode};

    struct HierBackend;
    impl crate::lsp::backend::LspBackend for HierBackend {
        fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> { Ok(()) }
        fn references(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
        fn definition(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position) -> Result<lsp_types::GotoDefinitionResponse, String> { Ok(lsp_types::GotoDefinitionResponse::Array(vec![])) }
        fn implementations(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
        fn rename(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _n: &str) -> Result<Option<lsp_types::WorkspaceEdit>, String> { Ok(None) }
        fn type_hierarchy(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, dir: HierarchyDirection) -> Result<TypeHierarchyNode, String> {
            assert_eq!(dir, HierarchyDirection::Subtypes);
            Ok(TypeHierarchyNode {
                name: "Animal".into(), path: "A.kt".into(), line: 1,
                children: vec![TypeHierarchyNode { name: "Dog".into(), path: "A.kt".into(), line: 2, children: vec![] }],
            })
        }
        fn symbols_overview(&mut self, _u: &lsp_types::Uri) -> Result<Vec<SymbolOverviewItem>, String> {
            Ok(vec![SymbolOverviewItem { name: "Animal".into(), kind: "interface".into(), line: 1 }])
        }
    }

    let tree = HierBackend.type_hierarchy(
        &crate::lsp::client::file_path_to_uri("/p/A.kt").unwrap(),
        lsp_types::Position::new(0, 0),
        HierarchyDirection::Subtypes,
    ).unwrap();
    let out = format_type_hierarchy(&tree);
    assert!(out.contains("Animal (A.kt:1)"), "{out}");
    assert!(out.contains("  Dog (A.kt:2)"), "{out}"); // child indented

    let items = HierBackend.symbols_overview(
        &crate::lsp::client::file_path_to_uri("/p/A.kt").unwrap(),
    ).unwrap();
    let out2 = format_symbols_overview(&items);
    assert!(out2.contains("interface Animal (line 1)"), "{out2}");
}

#[test]
fn parse_direction_defaults_to_supertypes() {
    use crate::lsp::backend::HierarchyDirection;
    assert_eq!(parse_direction(&json!({})), HierarchyDirection::Supertypes);
    assert_eq!(parse_direction(&json!({"direction": "subtypes"})), HierarchyDirection::Subtypes);
    assert_eq!(parse_direction(&json!({"direction": "supertypes"})), HierarchyDirection::Supertypes);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run -p lean-ctx ctx_refactor` (cwd=`rust`)
Expected: FAIL — `format_type_hierarchy`/`format_symbols_overview`/`parse_direction` not found.

- [ ] **Step 3: Add dispatch, handlers, formatters, parser**

(a) Add the two match arms in `handle` (Serena `replace_symbol_body` on `handle`). Insert after the `"declaration"` arm, before `_ =>`:

```rust
        "type_hierarchy" => handle_type_hierarchy(args, abs_path, project_root, &uri, position),
        "symbols_overview" => handle_symbols_overview(abs_path, project_root, &uri),
```

and update the unknown-action help string to list both:

```rust
        _ => format!(
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, \
             implementations, declaration, type_hierarchy, symbols_overview."
        ),
```

(b) Add `use crate::lsp::backend::{HierarchyDirection, SymbolOverviewItem, TypeHierarchyNode};` to the imports (Serena `insert_after_symbol` on the existing `use` of `uri_to_file_path`, or extend the top `use` block).

(c) Add the new fns (Serena `insert_after_symbol` after `handle_declaration`):

```rust
fn parse_direction(args: &Value) -> HierarchyDirection {
    match args.get("direction").and_then(Value::as_str) {
        Some("subtypes") => HierarchyDirection::Subtypes,
        _ => HierarchyDirection::Supertypes, // default + any unknown
    }
}

fn handle_type_hierarchy(
    args: &Value,
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
) -> String {
    let direction = parse_direction(args);
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.type_hierarchy(uri, position, direction)
    });
    match result {
        Ok(tree) => format_type_hierarchy(&tree),
        Err(e) => format!("ERROR: {e}"),
    }
}

fn handle_symbols_overview(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.symbols_overview(uri)
    });
    match result {
        Ok(items) => format_symbols_overview(&items),
        Err(e) => format!("ERROR: {e}"),
    }
}

fn format_type_hierarchy(root: &TypeHierarchyNode) -> String {
    fn walk(node: &TypeHierarchyNode, depth: usize, out: &mut String) {
        let indent = "  ".repeat(depth);
        out.push_str(&format!("{indent}{} ({}:{})\n", node.name, node.path, node.line));
        for child in &node.children {
            walk(child, depth + 1, out);
        }
    }
    let mut out = String::new();
    walk(root, 0, &mut out);
    out
}

fn format_symbols_overview(items: &[SymbolOverviewItem]) -> String {
    if items.is_empty() {
        return "No symbols found.".to_string();
    }
    let mut out = format!("{} symbol(s):\n", items.len());
    for item in items {
        out.push_str(&format!("  {} {} (line {})\n", item.kind, item.name, item.line));
    }
    out
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run -p lean-ctx ctx_refactor` (cwd=`rust`)
Expected: PASS (both new tests + existing ones).

- [ ] **Step 5: Commit**

```bash
git add rust/src/tools/ctx_refactor.rs
git commit -m "feat(refactor): dispatch + format type_hierarchy/symbols_overview (Backing-A degrades)"
```

---

## Task R3: Rust — `JetBrainsHttpBackend` overrides + parsers

**Files:**
- Modify: `rust/src/lsp/jetbrains_backend.rs` (impl block + parsers + test module)

- [ ] **Step 1: Write the failing mock tests**

In the `tests` module of `rust/src/lsp/jetbrains_backend.rs` (Serena `insert_after_symbol` after `references_parses_wire_locations`), add:

```rust
#[test]
fn type_hierarchy_parses_wire_tree() {
    use crate::lsp::backend::HierarchyDirection;
    let body = r#"{"tree":{"name":"Animal","path":"A.kt","line":1,"children":[{"name":"Dog","path":"A.kt","line":2,"children":[]}]},"truncated":false}"#;
    let port = mock_once(body);
    let mut backend = JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string());
    let uri = file_path_to_uri("/proj/A.kt").unwrap();
    let tree = backend
        .type_hierarchy(&uri, Position { line: 0, character: 0 }, HierarchyDirection::Subtypes)
        .expect("should parse");
    assert_eq!(tree.name, "Animal");
    assert_eq!(tree.line, 1);
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].name, "Dog");
    assert_eq!(tree.children[0].path, "A.kt");
}

#[test]
fn symbols_overview_parses_wire_items() {
    let body = r#"{"symbols":[{"name":"Animal","kind":"interface","line":1},{"name":"main","kind":"function","line":9}],"truncated":false,"total":2}"#;
    let port = mock_once(body);
    let mut backend = JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string());
    let uri = file_path_to_uri("/proj/A.kt").unwrap();
    let items = backend.symbols_overview(&uri).expect("should parse");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].kind, "interface");
    assert_eq!(items[1].name, "main");
    assert_eq!(items[1].line, 9);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run -p lean-ctx jetbrains_backend` (cwd=`rust`)
Expected: FAIL — `type_hierarchy`/`symbols_overview` resolve to the trait default `Err` (assertion on `.expect` panics) or method not overridden.

- [ ] **Step 3: Add the imports, parsers, and trait overrides**

(a) Extend the top `use super::...`/backend import to bring in the shapes. Serena `replace_symbol_body` is not applicable to a `use`; instead edit the import line via Serena `insert_after_symbol` on the module's first `use`. Target end-state imports include:

```rust
use crate::lsp::backend::{HierarchyDirection, LspBackend, SymbolOverviewItem, TypeHierarchyNode};
```

(b) Add private parsers as methods on `JetBrainsHttpBackend` (Serena `insert_after_symbol` after `parse_locations`):

```rust
fn parse_type_hierarchy(v: &Value) -> TypeHierarchyNode {
    fn node(v: &Value) -> TypeHierarchyNode {
        TypeHierarchyNode {
            name: v.get("name").and_then(Value::as_str).unwrap_or("?").to_string(),
            path: v.get("path").and_then(Value::as_str).unwrap_or_default().to_string(),
            line: v.get("line").and_then(Value::as_u64).unwrap_or(0) as u32,
            children: v
                .get("children")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().map(node).collect())
                .unwrap_or_default(),
        }
    }
    v.get("tree").map(node).unwrap_or_else(|| TypeHierarchyNode {
        name: String::new(),
        path: String::new(),
        line: 0,
        children: vec![],
    })
}

fn parse_symbols(v: &Value) -> Vec<SymbolOverviewItem> {
    v.get("symbols")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    Some(SymbolOverviewItem {
                        name: s.get("name")?.as_str()?.to_string(),
                        kind: s.get("kind")?.as_str()?.to_string(),
                        line: s.get("line")?.as_u64()? as u32,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `{path}` request body (file-level ops, no position).
fn path_body(&self, uri: &Uri) -> Value {
    let abs = crate::lsp::client::uri_to_file_path(uri).unwrap_or_default();
    let rel = abs
        .strip_prefix(&self.project_root)
        .map(|s| s.strip_prefix('/').unwrap_or(s).to_string())
        .unwrap_or(abs);
    serde_json::json!({ "path": rel })
}
```

(c) Add the trait overrides inside `impl LspBackend for JetBrainsHttpBackend` (Serena `insert_after_symbol` after the `declaration` method):

```rust
fn type_hierarchy(
    &mut self,
    uri: &Uri,
    position: Position,
    direction: HierarchyDirection,
) -> Result<TypeHierarchyNode, String> {
    let mut body = self.position_body(uri, position);
    body["direction"] = serde_json::json!(match direction {
        HierarchyDirection::Supertypes => "supertypes",
        HierarchyDirection::Subtypes => "subtypes",
    });
    let resp = self.post("/type_hierarchy", &body)?;
    if let Some(err) = resp.get("error") {
        return Err(err.get("code").and_then(Value::as_str).unwrap_or("INTERNAL").to_string());
    }
    Ok(Self::parse_type_hierarchy(&resp))
}

fn symbols_overview(&mut self, uri: &Uri) -> Result<Vec<SymbolOverviewItem>, String> {
    let body = self.path_body(uri);
    let resp = self.post("/symbols_overview", &body)?;
    if let Some(err) = resp.get("error") {
        return Err(err.get("code").and_then(Value::as_str).unwrap_or("INTERNAL").to_string());
    }
    Ok(Self::parse_symbols(&resp))
}
```

Note: `position_body` already serializes `scope` only for references/implementations; `type_hierarchy` reuses `position_body` and adds `direction`. `scope` for `type_hierarchy` is forwarded only if present — keep it simple: the plugin defaults `scope=project` when absent, so we do not need to add `scope` here (subtype scope defaults project-side). (Follow-up if `scope=all` for subtypes is needed: add `body["scope"] = scope` like `references`.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run -p lean-ctx jetbrains_backend` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 5: Full Rust gate + commit**

Run: `cargo nextest run` (cwd=`rust`) → all green. Run: `cargo clippy --all-targets` (cwd=`rust`) → no new lints.

```bash
git add rust/src/lsp/jetbrains_backend.rs
git commit -m "feat(jetbrains): wire type_hierarchy/symbols_overview HTTP overrides + parsers"
```

---

## Task K1: Kotlin — wire DTOs + `JsonCodec` parsers

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt`
- Modify: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt`

- [ ] **Step 1: Write the failing tests**

Append to `JsonCodecTest.kt` (inside the test class, via `ctx_edit`):

```kotlin
    fun testParseHierarchyRequestDefaultsDirectionAndScope() {
        val req = JsonCodec.parseHierarchyRequest("""{"path":"A.kt","line":0,"character":4}""")
        assertEquals("A.kt", req.path)
        assertEquals(0, req.line)
        assertEquals(4, req.character)
        assertEquals("supertypes", req.direction)
        assertEquals("project", req.scope)
    }

    fun testParseHierarchyRequestHonorsExplicitValues() {
        val req = JsonCodec.parseHierarchyRequest("""{"path":"A.kt","line":1,"character":0,"direction":"subtypes","scope":"all"}""")
        assertEquals("subtypes", req.direction)
        assertEquals("all", req.scope)
    }

    fun testParseFileRequest() {
        val req = JsonCodec.parseFileRequest("""{"path":"A.kt"}""")
        assertEquals("A.kt", req.path)
    }

    fun testTypeHierarchyResponseRoundTrips() {
        val node = TypeHierarchyNodeDTO("Animal", "A.kt", 1, listOf(TypeHierarchyNodeDTO("Dog", "A.kt", 2, emptyList())))
        val json = JsonCodec.toJson(TypeHierarchyResponse(node, truncated = false))
        assertTrue(json.contains("\"tree\""))
        assertTrue(json.contains("\"children\""))
        assertTrue(json.contains("Dog"))
    }
```

(If `JsonCodecTest.kt` lacks the relevant imports, ensure `import com.leanctx.plugin.dto.*` or explicit imports for `TypeHierarchyNodeDTO`/`TypeHierarchyResponse` are present.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `./gradlew test --tests "com.leanctx.plugin.dto.JsonCodecTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL — `parseHierarchyRequest`/`parseFileRequest`/`TypeHierarchyNodeDTO` unresolved (compile error).

- [ ] **Step 3: Add the DTOs + codec methods to `Wire.kt`**

Append to `Wire.kt` (via `ctx_edit`), before the `object JsonCodec` block add the data classes, and add the two parse methods + reuse `toJson`/`error` inside `JsonCodec`:

```kotlin
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
```

Inside `object JsonCodec`, add (after `parseNavRequest`):

```kotlin
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `./gradlew test --tests "com.leanctx.plugin.dto.JsonCodecTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt
git commit -m "feat(plugin): wire DTOs + codec for type_hierarchy/symbols_overview"
```

---

## Task K2: Kotlin — `TypeHierarchyResolver` (class/method tree, depth+node cap)

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/TypeHierarchyResolver.kt`
- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/TypeHierarchyResolverTest.kt`

- [ ] **Step 1: Write the failing tests**

Create `TypeHierarchyResolverTest.kt` (mirrors `ImplementationFinderTest`'s off-EDT pooled-thread pattern — MANDATORY for Kotlin hierarchy search):

```kotlin
package com.leanctx.plugin.psi

import com.intellij.openapi.application.ApplicationManager
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.leanctx.plugin.dto.TypeHierarchyResponse

class TypeHierarchyResolverTest : BasePlatformTestCase() {

    private fun resolve(
        file: com.intellij.psi.PsiFile, line: Int, character: Int, direction: String, scope: String = "project",
    ): TypeHierarchyResponse {
        val locator = PsiLocator(project)
        val resolver = TypeHierarchyResolver(locator)
        // K2 inheritor/supertype resolution uses the Analysis API → forbidden on EDT.
        return ApplicationManager.getApplication().executeOnPooledThread<TypeHierarchyResponse> {
            locator.inSmartReadAction { resolver.resolve(file, line, character, direction, scope) }
        }.get()
    }

    fun testSubtypesOfInterface() {
        val file = myFixture.configureByText(
            "A.kt",
            """
            interface Animal
            class Dog : Animal
            class Cat : Animal
            """.trimIndent(),
        )
        val col = file.text.lines()[0].indexOf("Animal")
        val res = resolve(file, line = 0, character = col, direction = "subtypes")
        assertEquals("Animal", res.tree.name)
        val childNames = res.tree.children.map { it.name }.toSet()
        assertEquals(setOf("Dog", "Cat"), childNames)
        assertFalse(res.truncated)
    }

    fun testSupertypesOfClass() {
        val file = myFixture.configureByText(
            "B.kt",
            """
            interface Animal
            open class Pet : Animal
            class Dog : Pet()
            """.trimIndent(),
        )
        val dogLine = 2
        val col = file.text.lines()[dogLine].indexOf("Dog")
        val res = resolve(file, line = dogLine, character = col, direction = "supertypes")
        assertEquals("Dog", res.tree.name)
        // Pet is a direct super; Animal appears transitively beneath Pet.
        val superNames = res.tree.children.map { it.name }.toSet()
        assertTrue("supers=$superNames", superNames.contains("Pet"))
    }

    fun testNoSymbolAtPosition() {
        val file = myFixture.configureByText("C.kt", "class X\n")
        try {
            resolve(file, line = 0, character = 0, direction = "supertypes") // 'c' of "class", no named symbol
            fail("expected NO_SYMBOL_AT_POSITION")
        } catch (e: java.util.concurrent.ExecutionException) {
            val cause = e.cause
            assertTrue(cause is com.leanctx.plugin.server.BackendException)
            assertEquals("NO_SYMBOL_AT_POSITION", (cause as com.leanctx.plugin.server.BackendException).code)
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.TypeHierarchyResolverTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL — `TypeHierarchyResolver` does not exist (compile error).

- [ ] **Step 3: Implement `TypeHierarchyResolver`**

Create `TypeHierarchyResolver.kt`:

```kotlin
package com.leanctx.plugin.psi

import com.intellij.psi.PsiClass
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.PsiMethod
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.search.GlobalSearchScope
import com.intellij.psi.search.searches.ClassInheritorsSearch
import com.intellij.psi.search.searches.OverridingMethodsSearch
import com.intellij.util.Processor
import com.leanctx.plugin.dto.TypeHierarchyNodeDTO
import com.leanctx.plugin.dto.TypeHierarchyResponse
import com.leanctx.plugin.server.BackendException
import org.jetbrains.kotlin.asJava.toLightClass
import org.jetbrains.kotlin.asJava.toLightMethods
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtNamedFunction

/**
 * Super/subtype tree for a class/interface or method. Language-neutral via Kotlin light
 * classes (KtClassOrObject.toLightClass()) → PsiClass/PsiMethod APIs work for Kotlin + Java.
 * Transitive with hard depth + node caps; must run inside a smart-mode ReadAction off the EDT.
 */
class TypeHierarchyResolver(private val locator: PsiLocator) {

    companion object {
        const val MAX_DEPTH = 5
        const val MAX_NODES = 200
    }

    private class Budget(var nodes: Int = 0, var truncated: Boolean = false)

    fun resolve(file: PsiFile, line: Int, character: Int, direction: String, scope: String): TypeHierarchyResponse {
        val target = resolveNamed(file, line, character)
        val searchScope = if (scope == "all") GlobalSearchScope.allScope(file.project)
        else GlobalSearchScope.projectScope(file.project)
        val wantSub = direction == "subtypes"
        val budget = Budget()

        val psiClass = asPsiClass(target)
        val root: TypeHierarchyNodeDTO = if (psiClass != null) {
            buildClassNode(psiClass, wantSub, searchScope, 0, budget)
        } else {
            val psiMethod = asPsiMethod(target)
                ?: throw BackendException("UNSUPPORTED_LANGUAGE", "type_hierarchy needs a class or method")
            buildMethodNode(psiMethod, wantSub, searchScope, 0, budget)
        }
        return TypeHierarchyResponse(root, budget.truncated)
    }

    private fun buildClassNode(cls: PsiClass, sub: Boolean, scope: GlobalSearchScope, depth: Int, b: Budget): TypeHierarchyNodeDTO {
        val children = ArrayList<TypeHierarchyNodeDTO>()
        if (depth < MAX_DEPTH) {
            val next: List<PsiClass> = if (sub) directSubclasses(cls, scope) else cls.supers.toList()
            for (n in next) {
                if (b.nodes >= MAX_NODES) { b.truncated = true; break }
                b.nodes++
                children.add(buildClassNode(n, sub, scope, depth + 1, b))
            }
        } else if ((if (sub) directSubclasses(cls, scope) else cls.supers.toList()).isNotEmpty()) {
            b.truncated = true
        }
        return nodeOf(cls, children)
    }

    private fun buildMethodNode(m: PsiMethod, sub: Boolean, scope: GlobalSearchScope, depth: Int, b: Budget): TypeHierarchyNodeDTO {
        val children = ArrayList<TypeHierarchyNodeDTO>()
        if (depth < MAX_DEPTH) {
            val next: List<PsiMethod> = if (sub) directOverriders(m, scope) else m.findSuperMethods().toList()
            for (n in next) {
                if (b.nodes >= MAX_NODES) { b.truncated = true; break }
                b.nodes++
                children.add(buildMethodNode(n, sub, scope, depth + 1, b))
            }
        }
        return nodeOf(m, children)
    }

    private fun directSubclasses(cls: PsiClass, scope: GlobalSearchScope): List<PsiClass> {
        val out = ArrayList<PsiClass>()
        // checkDeep=false → direct inheritors only; recursion builds the tree.
        ClassInheritorsSearch.search(cls, scope, false).forEach(Processor { c: PsiClass -> out.add(c); true })
        return out
    }

    private fun directOverriders(m: PsiMethod, scope: GlobalSearchScope): List<PsiMethod> {
        val out = ArrayList<PsiMethod>()
        OverridingMethodsSearch.search(m, scope, false).forEach(Processor { mm: PsiMethod -> out.add(mm); true })
        return out
    }

    private fun nodeOf(element: PsiElement, children: List<TypeHierarchyNodeDTO>): TypeHierarchyNodeDTO {
        val nav = element.navigationElement ?: element
        val name = (element as? PsiNamedElement)?.name ?: "?"
        val loc = locator.toLocation(nav)
        val path = loc?.path ?: ""
        val line = (loc?.range?.start?.line ?: 0) + 1 // 0-based PSI → 1-based wire (see constraint 4)
        return TypeHierarchyNodeDTO(name, path, line, children)
    }

    private fun asPsiClass(element: PsiElement): PsiClass? = when (element) {
        is PsiClass -> element
        is KtClassOrObject -> element.toLightClass()
        else -> null
    }

    private fun asPsiMethod(element: PsiElement): PsiMethod? = when (element) {
        is PsiMethod -> element
        is KtNamedFunction -> element.toLightMethods().firstOrNull()
        else -> null
    }

    private fun resolveNamed(file: PsiFile, line: Int, character: Int): PsiElement {
        val offset = locator.offsetOf(file, line, character)
        file.findReferenceAt(offset)?.resolve()?.let { return it }
        val element = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no element at $line:$character")
        return generateSequence(element) { it.parent }
            .firstOrNull { it is KtClassOrObject || it is KtNamedFunction || it is PsiClass || it is PsiMethod }
            ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no class/method at $line:$character")
    }
}
```

Note: `LocationDTO.path` is the field name on the nav `LocationDTO` (Phase 3) — reused here via `locator.toLocation(...).path`. Confirm `LocationDTO` exposes `path` (it does: `data class LocationDTO(val path: String, val range: TextRangeDTO)`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.TypeHierarchyResolverTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS (3 tests). If a Kotlin-supertypes assertion is flaky on light classes, prefer asserting `contains("Pet")` (already used) rather than exact set.

- [ ] **Step 5: Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/TypeHierarchyResolver.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/TypeHierarchyResolverTest.kt
git commit -m "feat(plugin): TypeHierarchyResolver — capped super/subtype tree (Kotlin light classes)"
```

---

## Task K3: Kotlin — `FileStructureScanner` (top-level symbols)

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/FileStructureScanner.kt`
- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/FileStructureScannerTest.kt`

- [ ] **Step 1: Write the failing tests**

Create `FileStructureScannerTest.kt` (top-level scan is resolve-free → EDT-safe, no pooled thread needed):

```kotlin
package com.leanctx.plugin.psi

import com.intellij.testFramework.fixtures.BasePlatformTestCase

class FileStructureScannerTest : BasePlatformTestCase() {

    fun testTopLevelSymbols() {
        val file = myFixture.configureByText(
            "A.kt",
            """
            interface Animal
            class Dog : Animal
            object Registry
            fun freeFun() {}
            val topProp = 1
            """.trimIndent(),
        )
        val scanner = FileStructureScanner(PsiLocator(project))
        val res = locator_scan(scanner, file)
        val byName = res.symbols.associateBy { it.name }
        assertEquals("interface", byName["Animal"]!!.kind)
        assertEquals("class", byName["Dog"]!!.kind)
        assertEquals("object", byName["Registry"]!!.kind)
        assertEquals("function", byName["freeFun"]!!.kind)
        assertEquals("property", byName["topProp"]!!.kind)
        // 1-based lines
        assertEquals(1, byName["Animal"]!!.line)
        assertFalse(res.truncated)
        assertEquals(res.symbols.size, res.total)
    }

    private fun locator_scan(scanner: FileStructureScanner, file: com.intellij.psi.PsiFile) =
        com.intellij.openapi.application.ReadAction.compute<com.leanctx.plugin.dto.SymbolsOverviewResponse, RuntimeException> {
            scanner.scan(file)
        }

    fun testUnsupportedLanguageThrows() {
        val file = myFixture.configureByText("notes.txt", "hello world\n")
        try {
            locator_scan(FileStructureScanner(PsiLocator(project)), file)
            fail("expected UNSUPPORTED_LANGUAGE")
        } catch (e: com.leanctx.plugin.server.BackendException) {
            assertEquals("UNSUPPORTED_LANGUAGE", e.code)
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.FileStructureScannerTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL — `FileStructureScanner` does not exist.

- [ ] **Step 3: Implement `FileStructureScanner`**

Create `FileStructureScanner.kt`:

```kotlin
package com.leanctx.plugin.psi

import com.intellij.psi.PsiDocumentManager
import com.intellij.psi.PsiFile
import com.leanctx.plugin.dto.SymbolOverviewItemDTO
import com.leanctx.plugin.dto.SymbolsOverviewResponse
import com.leanctx.plugin.server.BackendException
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtObjectDeclaration
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtTypeAlias

/**
 * Flat top-level structure of a file (spec §2 #3: top-level only, no nesting). Kotlin-only
 * in Phase 4; other languages → UNSUPPORTED_LANGUAGE. Caps at MAX_SYMBOLS with `truncated`.
 * Resolve-free (pure PSI) → safe on any thread inside a ReadAction.
 */
class FileStructureScanner(private val locator: PsiLocator) {

    companion object {
        const val MAX_SYMBOLS = 500
    }

    fun scan(file: PsiFile): SymbolsOverviewResponse {
        if (file !is KtFile) {
            throw BackendException("UNSUPPORTED_LANGUAGE", "symbols_overview supports Kotlin files (Phase 4)")
        }
        val doc = PsiDocumentManager.getInstance(file.project).getDocument(file)
            ?: throw BackendException("INTERNAL", "no document for ${file.name}")
        val out = ArrayList<SymbolOverviewItemDTO>()
        var truncated = false
        for (decl in file.declarations) {
            if (out.size >= MAX_SYMBOLS) { truncated = true; break }
            val name = decl.name ?: continue
            val kind = when (decl) {
                is KtClass -> if (decl.isInterface()) "interface" else "class"
                is KtObjectDeclaration -> "object"
                is KtNamedFunction -> "function"
                is KtProperty -> "property"
                is KtTypeAlias -> "typealias"
                else -> "declaration"
            }
            val nav = decl.navigationElement ?: decl
            val line = doc.getLineNumber(nav.textRange.startOffset) + 1 // 1-based wire
            out.add(SymbolOverviewItemDTO(name, kind, line))
        }
        return SymbolsOverviewResponse(out, truncated, out.size)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `./gradlew test --tests "com.leanctx.plugin.psi.FileStructureScannerTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/FileStructureScanner.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/FileStructureScannerTest.kt
git commit -m "feat(plugin): FileStructureScanner — top-level Kotlin symbols (1-based lines)"
```

---

## Task K4: Kotlin — `StructureHandlers` (endpoint layer)

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/StructureHandlers.kt`

This task has no standalone test (it is a thin read-action wrapper, exercised by K5's router tests). It is a single small unit.

- [ ] **Step 1: Implement `StructureHandlers`**

Create `StructureHandlers.kt`:

```kotlin
package com.leanctx.plugin.endpoint

import com.intellij.openapi.project.Project
import com.leanctx.plugin.dto.FileRequest
import com.leanctx.plugin.dto.HierarchyRequest
import com.leanctx.plugin.dto.SymbolsOverviewResponse
import com.leanctx.plugin.dto.TypeHierarchyResponse
import com.leanctx.plugin.psi.FileStructureScanner
import com.leanctx.plugin.psi.PsiLocator
import com.leanctx.plugin.psi.TypeHierarchyResolver

/**
 * Endpoint layer for the two Phase-4 structure ops. Each parses an already-deserialized
 * request, runs PSI inside a smart-mode ReadAction (off the EDT in production: handlers run
 * on the background HTTP thread), and returns the wire response. BackendException (typed code)
 * propagates to the RequestRouter for the error envelope.
 */
class StructureHandlers(project: Project) {
    private val locator = PsiLocator(project)
    private val hierarchy = TypeHierarchyResolver(locator)
    private val structure = FileStructureScanner(locator)

    fun typeHierarchy(req: HierarchyRequest): TypeHierarchyResponse = locator.inSmartReadAction {
        hierarchy.resolve(locator.psiFile(req.path), req.line, req.character, req.direction, req.scope)
    }

    fun symbolsOverview(req: FileRequest): SymbolsOverviewResponse = locator.inSmartReadAction {
        structure.scan(locator.psiFile(req.path))
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: BUILD SUCCESSFUL.

- [ ] **Step 3: Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/StructureHandlers.kt
git commit -m "feat(plugin): StructureHandlers — read-action-guarded type_hierarchy/symbols_overview"
```

---

## Task K5: Kotlin — wire `RequestRouter` (2 new routes)

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt`
- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterStructureTest.kt`

- [ ] **Step 1: Write the failing tests**

Create `RequestRouterStructureTest.kt`. The `/type_hierarchy` route triggers Kotlin K2 search → the route call must run off the EDT (pooled thread). `/symbols_overview` is resolve-free but we wrap uniformly:

```kotlin
package com.leanctx.plugin.server

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.application.WriteAction
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.nio.file.Files
import java.nio.file.Paths

class RequestRouterStructureTest : BasePlatformTestCase() {

    private fun router() = RequestRouter("tok", "IC-2026.1", project.name, project)

    private fun writeSource(name: String, text: String): String {
        val base = project.basePath!!
        Files.createDirectories(Paths.get(base))
        val p = Paths.get(base, name)
        Files.writeString(p, text)
        WriteAction.computeAndWait<Unit, RuntimeException> {
            LocalFileSystem.getInstance().refreshAndFindFileByPath(p.toString())
        }
        return name
    }

    private fun routeOffEdt(method: String, path: String, body: String): HttpResult =
        ApplicationManager.getApplication().executeOnPooledThread<HttpResult> {
            router().route(method, path, "tok", body)
        }.get()

    fun testTypeHierarchySubtypesRoute() {
        val rel = writeSource("A.kt", "interface Animal\nclass Dog : Animal\nclass Cat : Animal\n")
        val col = 10 // 0-based char of "Animal" in "interface Animal"
        val body = """{"path":"$rel","line":0,"character":$col,"direction":"subtypes"}"""
        val res = routeOffEdt("POST", "/type_hierarchy", body)
        assertEquals("body=${res.body}", 200, res.status)
        assertTrue("body=${res.body}", res.body.contains("\"tree\""))
        assertTrue("body=${res.body}", res.body.contains("Dog"))
        assertTrue("body=${res.body}", res.body.contains("\"truncated\""))
    }

    fun testSymbolsOverviewRoute() {
        val rel = writeSource("B.kt", "interface Animal\nfun main() {}\n")
        val res = routeOffEdt("POST", "/symbols_overview", """{"path":"$rel"}""")
        assertEquals("body=${res.body}", 200, res.status)
        assertTrue("body=${res.body}", res.body.contains("\"symbols\""))
        assertTrue("body=${res.body}", res.body.contains("interface"))
        assertTrue("body=${res.body}", res.body.contains("\"total\""))
    }

    fun testTypeHierarchyWrongTokenIs401() {
        val res = router().route("POST", "/type_hierarchy", "WRONG", "{}")
        assertEquals(401, res.status)
        assertTrue(res.body.contains("UNAUTHORIZED"))
    }

    fun testSymbolsOverviewFileNotFoundIs200Envelope() {
        val res = routeOffEdt("POST", "/symbols_overview", """{"path":"Nope.kt"}""")
        assertEquals(200, res.status)
        assertTrue(res.body.contains("FILE_NOT_FOUND"))
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterStructureTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: FAIL — `/type_hierarchy` + `/symbols_overview` return 404 `NOT_FOUND` (routes not wired).

- [ ] **Step 3: Wire the routes in `RequestRouter`**

Edit `RequestRouter.kt` (via `ctx_edit`):

(a) Add a `StructureHandlers` field next to `handlers`:

```kotlin
    private val handlers = NavHandlers(project)
    private val structureHandlers = com.leanctx.plugin.endpoint.StructureHandlers(project)
```

(b) In `route`, add the two structure routes inside the `if (method == "POST")` block, BEFORE the nav `when`:

```kotlin
        if (method == "POST") {
            if (path == "/type_hierarchy") return dispatchHierarchy(body)
            if (path == "/symbols_overview") return dispatchOverview(body)
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
```

(c) Add the two dispatch helpers (mirroring `dispatch`'s try/catch exactly) after the existing `dispatch` method:

```kotlin
    private fun dispatchHierarchy(body: String): HttpResult = try {
        val req = JsonCodec.parseHierarchyRequest(body)
        HttpResult(200, JsonCodec.toJson(structureHandlers.typeHierarchy(req)))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("type_hierarchy endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }

    private fun dispatchOverview(body: String): HttpResult = try {
        val req = JsonCodec.parseFileRequest(body)
        HttpResult(200, JsonCodec.toJson(structureHandlers.symbolsOverview(req)))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("symbols_overview endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }
```

(Imports: `BackendException`, `JsonCodec` are already imported in the file; add `com.leanctx.plugin.dto.JsonCodec` is present. No new top-level import needed beyond the fully-qualified `StructureHandlers` used in the field, or add `import com.leanctx.plugin.endpoint.StructureHandlers` and drop the FQN.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterStructureTest"` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS (4 tests).

- [ ] **Step 5: Full Kotlin gate + commit**

Run: `./gradlew test` (cwd=`packages/jetbrains-lean-ctx`)
Expected: all tests PASS (Phase-3 suite + new ones).

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterStructureTest.kt
git commit -m "feat(plugin): route /type_hierarchy + /symbols_overview (token-guarded, 200-envelope errors)"
```

---

## Task E1: Manual `runIde` E2E gate

**Files:** none (verification only).

- [ ] **Step 1: Rebuild + install the Rust binary** (so the running MCP server has the R1–R3 wiring)

Run: `cargo install --path rust` (cwd repo root), then restart the MCP server / reconnect lean-ctx. (Phase-3 gate noted the installed binary lagged the backend — avoid that here.)

- [ ] **Step 2: Launch the sandbox IDE**

Run: `./gradlew runIde` (cwd=`packages/jetbrains-lean-ctx`). In the sandbox, **open a configured Kotlin project** (not just a folder) so the index resolves — Phase-3 gate open-point #1.

- [ ] **Step 3: Read the port file + curl both new endpoints**

Read `<data_dir>/<projectHash>.port` for port+token. Then:

```bash
# supertypes
curl -s -X POST -H "X-LeanCtx-Token: $TOK" http://127.0.0.1:$PORT/type_hierarchy \
  -d '{"path":"src/Foo.kt","line":3,"character":6,"direction":"supertypes"}'
# subtypes
curl -s -X POST -H "X-LeanCtx-Token: $TOK" http://127.0.0.1:$PORT/type_hierarchy \
  -d '{"path":"src/Foo.kt","line":3,"character":6,"direction":"subtypes"}'
# overview
curl -s -X POST -H "X-LeanCtx-Token: $TOK" http://127.0.0.1:$PORT/symbols_overview \
  -d '{"path":"src/Foo.kt"}'
```

Expected: valid `{tree,truncated}` / `{symbols,truncated,total}`; wrong token → HTTP 401; unknown path → `FILE_NOT_FOUND` envelope. Cross-check against IDE "Type Hierarchy" (Ctrl+H) and the Structure view (Alt+7).

- [ ] **Step 4: E2E through `ctx_refactor` (Backing B) + PathJail**

```
ctx_refactor(action="type_hierarchy", path="src/Foo.kt", line=4, column=6, direction="subtypes")
ctx_refactor(action="symbols_overview", path="src/Foo.kt")
ctx_refactor(action="type_hierarchy", path="../escape.kt", line=1, column=0)  # → PathJail reject before HTTP
```

Expected: first two return formatted tree/symbol list; the escape path is rejected by PathJail (MCP error) before any HTTP call.

- [ ] **Step 5: Degradation check (no IDE)**

Close the sandbox IDE, then `ctx_refactor(action="type_hierarchy", path="rust/src/lib.rs", line=1, column=0)`.
Expected: `ERROR: type_hierarchy requires the JetBrains backend` (Backing A clean degrade — no crash).

Record results in the spec's gate protocol (append a "Gate-Protokoll Phase 4" note, mirroring §17's Phase-3 entry).

---

## Task E2: Single Phase-4 commit (squash) + spec status

Phase 4 lands as **one commit** (spec §2 #1 / parent §12.3). The per-task commits above keep TDD checkpoints; squash them into one before integration.

- [ ] **Step 1: Verify both gates green**

Run: `cargo nextest run` (cwd=`rust`) → green. Run: `./gradlew test` (cwd=`packages/jetbrains-lean-ctx`) → green. Run: `cargo clippy --all-targets` (cwd=`rust`) → no new lints.

- [ ] **Step 2: Reformat changed files** (project rule, before `git add`)

For each changed `.rs`/`.kt`/spec file, run `mcp__jetbrains__reformat_file` (if the IDE bridge is up; skip with a note if 404).

- [ ] **Step 3: Squash into one Phase-4 commit**

```bash
git rebase -i <phase-3-tip-sha>   # squash the R1..K5 task commits into one
# OR, if preferred, soft-reset and recommit:
# git reset --soft <phase-3-tip-sha> && git commit -m "feat(jetbrains): Phase 4 — type_hierarchy + symbols_overview (B-only)"
```

Final commit message:

```
feat(jetbrains): Phase 4 — type_hierarchy + symbols_overview (B-only)

- Rust: ctx_refactor schema+dispatch (direction param), JetBrainsHttpBackend overrides + parsers
- Plugin: TypeHierarchyResolver (capped tree, Kotlin light classes), FileStructureScanner
  (top-level), StructureHandlers, 2 router routes
- 1-based line seam for the two ops (matches Rust struct contract); Backing-A clean degrade
- Tests: Kotlin fixtures (off-EDT for K2) + Rust mock/Stub; manual runIde gate
```

- [ ] **Step 4: Update the spec status**

Append a "Gate-Protokoll Phase 4" subsection to `docs/lean-md/specs/2026-06-08-jetbrains-phase4-type-hierarchy-symbols-overview-design.md` recording: automated gates green, runIde result, any deferred env-bound items. Commit that doc change.

---

## Self-Review (against spec)

**Spec coverage:**
- §2 #1 (both ops, one commit) → E2 squash. ✓
- §2 #2 (transitive + depth/node cap + truncated) → K2 `MAX_DEPTH=5`/`MAX_NODES=200` + `Budget.truncated`. ✓
- §2 #3 (symbols_overview top-level only) → K3 `KtFile.declarations` (no recursion). ✓
- §2 #4 (Kotlin-only) → K2/K3/K5 Kotlin fixtures; Java deferred (§6.3). ✓
- §2 #5 (`direction` default `supertypes`) → R1 schema + R2 `parse_direction`. ✓
- §2 #6 (degradation: Backing-A ERROR, unsupported → UNSUPPORTED_LANGUAGE) → R2 trait-default Err path (E1 step 5) + K2/K3 `UNSUPPORTED_LANGUAGE`. ✓
- §3 components → R1–K5 cover each (DTOs in Wire.kt per constraint 7 deviation). ✓
- §4 wire shapes (`/type_hierarchy` {tree,truncated}; `/symbols_overview` {symbols,truncated,total}) → K1 DTOs + R3 parsers. ✓
- §4 1-based line → constraint 4 + K2 `nodeOf` `+1` + K3 `getLineNumber+1` + K1 doc. ✓
- §5 Rust deltas (override + schema single-source) → R1/R2/R3. ✓
- §7 risks (K2 EDT, assertThrows, PsiLocator fixture) → constraint 5 + off-EDT test wrappers; PsiLocator reused unchanged (writeSource pattern from RequestRouterNavTest). ✓
- §8 gate → E1 (runIde) + Rust/Kotlin suites. ✓

**Type consistency:** `TypeHierarchyNodeDTO`/`SymbolOverviewItemDTO`/`HierarchyRequest`/`FileRequest`/`TypeHierarchyResponse`/`SymbolsOverviewResponse` (Kotlin) mirror Rust `TypeHierarchyNode`/`SymbolOverviewItem` field-for-field (`name`,`path`,`line`,`children` / `name`,`kind`,`line`). `parse_direction`/`format_type_hierarchy`/`format_symbols_overview`/`handle_type_hierarchy`/`handle_symbols_overview` names are used identically across R2 definitions and tests. Endpoint paths `/type_hierarchy`,`/symbols_overview` identical in R3 (Rust POST target), K5 (route), E1 (curl).

**Open deviations (intentional, noted):**
1. DTOs consolidated in `Wire.kt`, not separate `dto/*.kt` (constraint 7) — follows codebase convention.
2. `scope` not forwarded for `type_hierarchy` in R3 (plugin defaults `project`); add `body["scope"]` if `scope=all` subtypes is needed (spec §6.2 follow-up).
3. Java fixtures deferred to a follow-up (spec §2 #4 / §6.3).
