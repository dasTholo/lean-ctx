# Journey 19 — JetBrains Plugin

> You want code intelligence (navigation, structure, inspections, symbolic
> edits, refactoring) directly from a running JetBrains IDE — token-compressed
> under `ctx_refactor`, with a headless fallback for CI. This journey explains every
> function in detail: what it does, how the agent invokes it, the raw
> HTTP endpoint, and the behavior under the hood.

> Language: English. Code, parameters, endpoint/tool names, and error codes stay
> English. Concise table reference for agents:
> [appendix-jetbrains-plugin-de.md](appendix-jetbrains-plugin-de.md).

Authoritative sources:

- Plugin: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/{server,endpoint,psi,dto}/…`
- Rust backend: `rust/src/lsp/{backend,jetbrains_backend,router,edit_apply,port_discovery}.rs`
- MCP tool schema: `rust/src/tools/registered/ctx_refactor.rs`

---

## 0. Serena as Inspiration

The lean-ctx JetBrains plugin is conceptually inspired by **Serena** (Oraios'
IntelliJ-Platform MCP tool). Serena was the model because it was the only tool to
deliver the semantic core — `references`, `implementations`, `type_hierarchy` **and**
symbolic edits — directly from the IDE; the official JetBrains MCP
(`mcp__jetbrains__*`) never closed this gap.

**Clear delineation:** The plugin is an **independent reimplementation at the
architecture and class-name level — not a derivation, not decompiled
Serena code**. It is published under the lean-ctx project license and ships in the
repository (`packages/jetbrains-lean-ctx`). Goal: make Serena (and the official
JetBrains MCP) **dispensable** as a code-intelligence dependency, so that
lean-ctx becomes the sole interface for symbols, navigation, and refactoring.

### 0.1 Delineation Serena ↔ lean-ctx Plugin

| Aspect          | Serena                       | lean-ctx JetBrains plugin                                                |
| --------------- | ---------------------------- | ------------------------------------------------------------------------ |
| Hosting         | external Oraios component    | in the lean-ctx repo (`packages/jetbrains-lean-ctx`)                     |
| Interface       | several separate MCP tools   | bundled under `ctx_refactor` (token compression)                         |
| Backend model   | running IDE only             | Backing B (IDE) **+** Backing A (rust-analyzer) **+** Headless           |
| Headless / CI   | no                           | yes — tree-sitter fallback for `symbols_overview` + edits                |
| Conflict guard  | none                         | BLAKE3 `expected_hash` (edits) / `plan_hash` (refactoring), Rust-central |
| Security        | —                            | PathJail (project-root validation) + token auth per project              |
| License         | proprietary (Oraios)         | lean-ctx project license                                                 |

### 0.2 Mapping: Serena concept → `ctx_refactor` action → HTTP endpoint

| Serena concept             | `ctx_refactor` action            | HTTP endpoint                                       |
| -------------------------- | -------------------------------- | --------------------------------------------------- |
| `find_referencing_symbols` | `references`                     | `POST /references`                                  |
| `find_declaration`         | `declaration`                    | `POST /declaration`                                 |
| (goto definition)          | `definition`                     | `POST /definition`                                  |
| `find_implementations`     | `implementations`                | `POST /implementations`                             |
| `get_symbols_overview`     | `symbols_overview`               | `POST /symbols_overview`                            |
| `type_hierarchy`           | `type_hierarchy`                 | `POST /type_hierarchy`                              |
| `run_inspections` / list   | `inspections` (`mode=run\|list`) | `POST /inspections`, `POST /list_inspections`       |
| `replace_symbol_body`      | `replace_symbol_body`            | `POST /replaceSymbolBody`                           |
| `insert_before_symbol`     | `insert_before_symbol`           | `POST /insertBeforeSymbol`                          |
| `insert_after_symbol`      | `insert_after_symbol`            | `POST /insertAfterSymbol`                           |
| `rename`                   | `rename`                         | `POST /renamePreview` → `POST /renameApply`         |
| (reformat_file)            | `reformat`                       | `POST /reformat`                                    |
| `move`                     | `move`                           | `POST /movePreview` → `POST /moveApply`             |
| `safe_delete`              | `safe_delete`                    | `POST /safeDeletePreview` → `POST /safeDeleteApply` |
| `inline`                   | `inline`                         | `POST /inlinePreview` → `POST /inlineApply`         |

> `find_symbol` (pure symbol search) is not part of `ctx_refactor` but of
> `ctx_symbol` / `ctx_outline` (lean-ctx symbol index). See
> [MCP tool map](appendix-mcp-tools.md).

---

## 1. Architecture (Plugin ↔ Rust ↔ MCP tool)

```text
   Agent
     │  ctx_refactor action=… (MCP)
     ▼
  │ Rust: ctx_refactor  →  select_backend        │
        │ IDE reachable?         │ no
        ▼ yes                    ▼
  Backing B                 Headless / Backing A
  JetBrainsHttpBackend      • local_range_write (edits, atomic)
  HTTP → Plugin             • overview_from_index (tree-sitter)
        │                   • rust-analyzer (navigation)
        ▼
  │ JetBrains IDE plugin (Kotlin HTTP server)    │
  │ 127.0.0.1 · token-guarded · PSI/read-action  │
```

### 1.1 Backing choice & degradation (`backend.rs`)

`select_backend` (`rust/src/lsp/router.rs`) decides per call which path applies.
The `LspBackend` trait tiers the methods:

| Class                                       | Methods                                                                                                                      | Default without IDE                        |
| ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| **Mandatory** (both backings)              | `open_file`, `references`, `definition`, `implementations`, `rename`                                                         | served by Backing A                        |
| **Default-degrading** (Backing B preferred) | `declaration`, `type_hierarchy`, `inspections`, `list_inspections`                                                           | `Err` — "requires the JetBrains backend"   |
| **Headless-default** (lossless)            | `symbols_overview` (tree-sitter), `replace_symbol_body`, `insert_before_symbol`, `insert_after_symbol` (`local_range_write`) | works without IDE                          |
| **`BACKEND_REQUIRED`**                      | refactoring engine (`rename`, `move`, `safe_delete`, `inline`)                                                               | `Err` — no headless usage search possible  |

### 1.2 Port discovery & staleness

On project start the plugin writes a **port file** (atomic, idempotent) with
`port`, `token`, `pid`, `projectRoot`, `ideVersion`, `startedAt`
(`BackendHttpServer.kt` → `LeanCtxPaths.portFile(dataDir, projectRoot)`). On
`projectClosing` (Disposable) it is deleted.

Rust checks reachability in **three stages** (`rust/src/lsp/port_discovery.rs`):

1. Port file exists & is readable → `port`/`token`/`pid`,
2. process with `pid` is alive,
3. `GET /health` responds within the timeout.

Only when all three pass is Backing B considered reachable; otherwise Headless
or `BACKEND_REQUIRED` applies.

### 1.3 Worktrees & project windows

The HTTP server is a **project-level service** (`BackendHttpServer` as a
`Disposable`, booted by `LeanCtxStartupActivity` per `Project`, bound to
`127.0.0.1:0` = ephemeral port). The port file is keyed
**per project** via `projecthash = sha256(canonical(projectRoot))[..16]`. From this
follows for `git worktree`:

- **One dedicated port file per worktree** — but only if the worktree is opened as its
  own **project window**. Multiple terminals **within one** project window
  share **one** port file (terminals do not start a plugin).
- **One open project window serves exactly one worktree path.** A lean-ctx session
  running in a **different** worktree computes a diverging `projecthash`,
  finds **no** port file → clean **fallback to Backing A** (rust-analyzer);
  with `lsp.<lang>="jetbrains"` instead `BACKEND_REQUIRED`. **No** path collision.
- **Backing B for N worktrees in parallel:** one project window per worktree.
  A **single** IDE instance suffices — *File → Open → in new window* instantiates the
  project service again (own server, own port, own port file). **No**
  second IDE installation/process needed.
- **JetBrains VCS ↔ PSI orthogonal:** The Git tool-window confusion with worktrees
  (`.git` file → `gitdir:` indirection) concerns the **VCS layer**, not
  indexing. The Backing-B endpoints need an **indexed Cargo project**, not a
  recognized VCS root → **PSI works** even when the Git panel is acting up.
- **Per terminal** the lean-ctx session must be `cd`'d into the **matching** worktree;
  the `projecthash` match then runs automatically.

> Cost trade-off: N project windows = N× indexing/RAM (shared JVM, separate
> indexes). Worth it only with a genuine need for PSI symbolics in multiple worktrees
> **simultaneously** — otherwise leave the secondary worktree in the terminal and
> accept the rust-analyzer fallback (Backing A). The same **branch** cannot be checked
> out in two worktrees at once (git constraint).

---

## 2. Function Reference

Conventions for all endpoints:

- HTTP: `POST` to `127.0.0.1:<port>`, header `X-LeanCtx-Token: <token>`,
  body = JSON. `GET /health` is the only exception (no body).
- **Coordinates:** At the `ctx_refactor` level, `line` is **1-indexed**, `column`
  is **0-indexed**. At the **wire level** (HTTP DTO), `line`/`character` of the
  navigation/edit endpoints are **0-based** (LSP convention); the `line` fields in
  `type_hierarchy`, `symbols_overview`, and `inspections` responses are **1-based**.
- Domain negative cases arrive as an envelope `{"error":{"code","message"}}` with
  HTTP 200 (see §5).

### 2.1 Navigation (read-only)

**Actions:** `references`, `definition`, `implementations`, `declaration`
**Endpoints:** `POST /references` · `/definition` · `/implementations` · `/declaration`

**What it does:** Finds semantic occurrences of a symbol (usages,
declaration, implementations). `declaration` is only available via Backing B.

**Agent invocation:**

```text
ctx_refactor action=references path=src/Main.kt line=42 column=8 scope=project
```

**HTTP (curl):**

```bash
curl -s -X POST http://127.0.0.1:$PORT/references \
  -H "X-LeanCtx-Token: $TOKEN" -H "Content-Type: application/json" \
  -d '{"path":"src/Main.kt","line":41,"character":8,"scope":"project"}'
```

**Response (`LocationsResponse`):**

```text
{"locations":[{"path":"src/Main.kt","range":{"start":{"line":41,"character":8},
 "end":{"line":41,"character":14}}}],"truncated":false,"total":1}
```

**Parameters:** `path`, `line`/`character` (0-based, wire), `scope ∈ {project, all}`
(default `project`; `all` includes libraries/SDK).
**Backing:** Backing B preferred; Backing A (rust-analyzer) as fallback for
`references`/`definition`/`implementations`. `declaration` is Backing-B-only.

### 2.2 Structure

**Actions:** `type_hierarchy`, `symbols_overview`
**Endpoints:** `POST /type_hierarchy` · `POST /symbols_overview`

**What it does:** `type_hierarchy` returns the super-/subtype tree; `symbols_overview`
lists the top-level symbols of a file.

**Agent invocation:**

```text
ctx_refactor action=type_hierarchy path=src/Main.kt line=10 column=6 direction=subtypes
ctx_refactor action=symbols_overview path=src/Main.kt
```

**HTTP (curl):**

```bash
curl -s -X POST http://127.0.0.1:$PORT/symbols_overview \
  -H "X-LeanCtx-Token: $TOKEN" -d '{"path":"src/Main.kt"}'
```

**Response (`SymbolsOverviewResponse`, `line` 1-based):**

```text
{"symbols":[{"name":"Main","kind":"class","line":3},
            {"name":"run","kind":"method","line":7}],"truncated":false,"total":2}
```

**Parameters:** `type_hierarchy`: `path`, `line`/`character`, `direction ∈
{supertypes, subtypes}` (default `supertypes`), `scope`. `symbols_overview`: `path`.
**Backing:** `type_hierarchy` is Backing-B-only. `symbols_overview` has a
**lossless headless default** via the tree-sitter symbol index
(`overview_from_index`, the same source as `ctx_symbol`/`ctx_outline`).

### 2.3 Quality — Inspections

**Action:** `inspections` (`mode=run|list`)
**Endpoints:** `POST /inspections` · `POST /list_inspections`

**What it does:** `mode=run` runs the active inspections on a file and
returns diagnostics; `mode=list` lists the inspections enabled in the project profile.

**Agent invocation:**

```text
ctx_refactor action=inspections path=src/Main.kt mode=run
ctx_refactor action=inspections path=src/Main.kt mode=list
```

**Response `run` (`InspectionsResponse`, `line` 1-based):**

```text
{"diagnostics":[{"path":"src/Main.kt","line":12,"severity":"WARNING",
 "message":"Unused symbol"}],"truncated":false,"total":1}
```

**Response `list` (`ListInspectionsResponse`):**

```text
{"inspections":[{"id":"UnusedSymbol","name":"Unused declaration",
 "severity":"WARNING"}],"truncated":false,"total":1}
```

**Backing:** Backing-B-only (no headless equivalent).

### 2.4 Symbol-body edits (write)

**Actions:** `replace_symbol_body`, `insert_before_symbol`, `insert_after_symbol`
**Endpoints:** `POST /replaceSymbolBody` · `/insertBeforeSymbol` · `/insertAfterSymbol`

**What it does:** Replaces the complete declaration of a named symbol or
inserts a sibling element before/after it. The target is addressed via `name_path`
(`'Class/method'` qualified or bare `'name'`), resolved through the
symbol index. Alternatively as a fallback via `path`+`line`(+`end_line`).

**Agent invocation:**

```text
ctx_refactor action=replace_symbol_body name_path=Main/run \
  new_body="fun run() { println(\"new\") }" expected_hash=<blake3-hex>

ctx_refactor action=insert_after_symbol name_path=Main/run \
  text="fun helper() = 42"
```

**HTTP (curl) — wire body carries `path`/`range`/`text` (no hash, see §4.1):**

```bash
curl -s -X POST http://127.0.0.1:$PORT/replaceSymbolBody \
  -H "X-LeanCtx-Token: $TOKEN" -d '{
    "path":"src/Main.kt",
    "range":{"start":{"line":6,"character":0},"end":{"line":8,"character":1}},
    "text":"fun run() { println(\"new\") }"
  }'
```

**Response (`EditResponse`):**

```text
{"applied":true,
 "newRange":{"start":{"line":6,"character":0},"end":{"line":6,"character":28}},
 "editedText":"fun run() { println(\"new\") }"}
```

**Parameters (action):** `name_path` **or** `path`+`line`(+`end_line`);
`new_body` (replace) or `text` (insert); optional `expected_hash`.
**Behavior:** Backing B executes the edit as a `WriteCommandAction` (a
single undo entry, document save). Headless writes atomically via `local_range_write`
(temp file + `rename`). **Both paths apply the same tree-sitter range
→ byte-identical result.** No automatic reformatting.

---

## 3. Refactoring Engine

All refactorings (except `reformat`) run through the **shared two-phase engine**:
`*Preview` collects usages + conflicts and forms the `plan_hash`; `*Apply`
performs the multi-file change as **one** transaction (one undo entry).
Because the semantic usage search needs the finished IDE index, there is **no**
lossless headless path — without a running IDE you get `BACKEND_REQUIRED`.

### 3.1 Rename (two-phase)

**Action:** `rename` (`new_name`)
**Endpoints:** `POST /renamePreview` → `POST /renameApply`

**What it does:** Renames a symbol project-wide — declaration **and all usages**.
Phase 1 (`/renamePreview`) collects `usages` and `conflicts` and forms the
`plan_hash` from them; Phase 2 (`/renameApply`) performs the rename as **one**
multi-file transaction.

**Agent invocation:**

```text
ctx_refactor action=rename path=src/Main.kt line=7 column=4 new_name=execute
```

**HTTP (curl) — Phase 1:**

```bash
curl -s -X POST http://127.0.0.1:$PORT/renamePreview \
  -H "X-LeanCtx-Token: $TOKEN" -d '{
    "path":"src/Main.kt",
    "range":{"start":{"line":6,"character":4},"end":{"line":6,"character":7}},
    "new_name":"execute","search_comments":false,"search_text_occurrences":false
  }'
# → {"usages":[{"path":"src/Main.kt","range":{…},"context":"run()"}],"conflicts":[]}
```

**HTTP (curl) — Phase 2:**

```bash
curl -s -X POST http://127.0.0.1:$PORT/renameApply \
  -H "X-LeanCtx-Token: $TOKEN" -d '{
    "path":"src/Main.kt","range":{…},"new_name":"execute","force":false
  }'
# → {"applied":true,"changed_paths":["src/Main.kt","src/Caller.kt"]}
```

**Parameters:** `new_name` (required); optional `search_comments`,
`search_text_occurrences` (preview); `force` (apply — skips the conflict gate).
**Behavior:** `BACKEND_REQUIRED` without a running IDE. If conflicts exist and
`force=false`, the gate blocks with `CONFLICT`. Between preview and apply the
`plan_hash` (BLAKE3, Rust-central) protects against TOCTOU drift.

### 3.2 Reformat

**Action:** `reformat`
**Endpoint:** `POST /reformat`

**What it does:** Formats a file in place according to the IDE's active code-style
profile (`CodeStyleManager` — equivalent to `mcp__jetbrains__reformat_file`).
Single-phase (no preview): formatting is idempotent and scoped to one file.

**Agent invocation:**

```text
ctx_refactor action=reformat path=src/Main.kt
```

**HTTP (curl):**

```bash
curl -s -X POST http://127.0.0.1:$PORT/reformat \
  -H "X-LeanCtx-Token: $TOKEN" -d '{"path":"src/Main.kt"}'
# → {"reformatted":true,"path":"src/Main.kt"}
```

**Behavior:** Backing-B-only (`WriteCommandAction` → `CodeStyleManager.reformat` →
`saveDocument`). Deliberately **decoupled** from the edit ops: symbol-body edits
do not reformat automatically; `reformat` is applied afterward when needed.

### 3.3 Move

**Action:** `move`
**Endpoints:** `POST /movePreview` → `POST /moveApply`

**What it does:** Moves a symbol (class/file/member) into another
package/target and adjusts all references + imports. Same two-phase mechanic as
`rename`: preview reports affected files + conflicts (`plan_hash`), apply performs
the multi-file transaction. `BACKEND_REQUIRED` without IDE.

### 3.4 Safe Delete

**Action:** `safe_delete`
**Endpoints:** `POST /safeDeletePreview` → `POST /safeDeleteApply`

**What it does:** Deletes a symbol only if no blocking usages
exist. Preview reports the found usages as conflicts; apply deletes (or
blocks with `CONFLICT` unless `force`). Same engine as `rename`.

### 3.5 Inline

**Action:** `inline`
**Endpoints:** `POST /inlinePreview` → `POST /inlineApply`

**What it does:** Replaces a symbol with its body at all call sites and
removes the declaration. Preview reports the affected sites + conflicts; apply
performs the multi-file replacement. Same engine as `rename`.

---

## 4. Behavioral Guarantees & Guards

### 4.1 BLAKE3 conflict guard (Rust-central)

The `expected_hash` (edits) or `plan_hash` (refactoring) is a **BLAKE3 hex**
(`crate::core::hasher::hash_hex`) and is checked **exclusively in Rust** — the
plugin does not hash and does not know the field in the wire protocol (`EditRequest`
carries only `path`/`range`/`text`).

- **Headless:** `local_range_write` reads the current bytes of the range, compares
  against `expected_hash`, and aborts on divergence with `CONFLICT: range hash
  mismatch` — the file stays unchanged.
- **IDE (Backing B):** Rust checks the same hash against the disk bytes **before** the
  HTTP POST. So the guard is identical on both paths (same disk bytes,
  same BLAKE3 check).

This prevents blindly overwriting externally modified locations.

### 4.2 Smart mode, language, PathJail

- **Smart mode:** If the IDE is in dumb mode (index being built),
  PSI operations return `INDEXING` instead of a partial result (no automatic waiting).
  For the refactoring engine this is mandatory: an incomplete usage set would be
  a broken refactoring.
- **Language:** If an LSP configuration is missing (Backing A) or a PSI processor
  (Backing B), `UNSUPPORTED_LANGUAGE` is returned (defensive, nullable EP resolution).
- **PathJail:** Every file operation is validated against the `project_root` before
  execution — both the name_path/position resolution and every
  `usage`/`changed_path` returned by the plugin.

### 4.3 Idempotency & atomicity

| Operation                                    | Transaction                                    | Idempotent                    |
| -------------------------------------------- | ---------------------------------------------- | ----------------------------- |
| Navigation, structure, inspections           | smart-mode read action                         | yes (index-stable)            |
| Symbol-body edits                            | `WriteCommandAction` (IDE) / atomic (headless) | protected via `expected_hash` |
| Refactoring (rename/move/safe_delete/inline) | multi-file `WriteCommandAction`                | protected via `plan_hash`     |
| Reformat                                     | `WriteCommandAction` (single file)             | yes (formatting-stable)       |

Headless writes are atomic (temp file `.<name>.lean-ctx.tmp.<pid>` + `rename`).

### 4.4 Cache coherence

After every write, lean-ctx evicts the file from the cache; the next `ctx_read`
re-validates via mtime (~13 tokens). The `editedText` of the `EditResponse` allows an
immediate rewarm; for multi-file refactoring each `changed_path` is mtime-checked.

---

## 5. Authentication & Security

- **Token per project:** On start the plugin generates a random token
  (`SecureRandom`, hex), stored in the port file. It is checked on every HTTP request
  via the header **`X-LeanCtx-Token`**.
- **401 on missing/mismatch:** `headerToken != token` →
  `HttpResult(401, {"error":{"code":"UNAUTHORIZED",…}})` — no processing.
- **Loopback only:** The HTTP server listens on `127.0.0.1` (not exposed on the
  network) and runs in the IDE user context.
- **Rotation:** On IDE restart a new port file with a new token is created.

See also [Journey 13 — Security & Governance](13-security-and-governance.md).

---

## 6. Error Catalog

**HTTP status:** `200` = success **or** domain negative case (envelope); `401`
= token missing/wrong; `404` = no route for `METHOD /path`; `500` = a real,
unexpected exception. (An `IllegalArgumentException`, e.g. an empty body, is returned
as `200` + `INTERNAL`.)

**Envelope:** `{"error":{"code":"<CODE>","message":"<text>"}}`

| Code                    | Trigger                                                         | Source                       | Remedy                                                    |
| ----------------------- | -------------------------------------------------------------- | ---------------------------- | --------------------------------------------------------- |
| `UNAUTHORIZED`          | token missing/wrong (401)                                      | plugin (`RequestRouter`)     | send a valid `X-LeanCtx-Token`                            |
| `NOT_FOUND`             | unknown route (404)                                            | plugin                       | check the endpoint path                                   |
| `FILE_NOT_FOUND`        | file not readable                                              | Rust (`edit_apply`) / plugin | verify the path with `ctx_tree`                           |
| `POSITION_OUT_OF_RANGE` | line/column past EOF / `end < start`                            | Rust / plugin                | re-resolve the range (`ctx_read`)                         |
| `CONFLICT`              | `expected_hash`/`plan_hash` mismatch; or conflicts ∧ `!force`  | Rust                         | read fresh, refresh the hash; if needed `force`           |
| `AMBIGUOUS_SYMBOL`      | `name_path` matches >1 symbol                                  | Rust (`ctx_refactor`)        | qualify (`Class/method`) — note the candidate list        |
| `NO_SYMBOL`             | `name_path` matches 0 symbols                                  | Rust / plugin                | correct the name/path                                     |
| `INDEXING`             | IDE in dumb mode                                               | plugin (`PsiLocator`)        | wait until indexing is finished, retry                    |
| `UNSUPPORTED_LANGUAGE`  | no LSP config / no PSI processor                               | Rust / plugin                | language is not (yet) supported                           |
| `BACKEND_REQUIRED`      | refactoring without a running IDE                              | Rust (trait default)         | start the IDE with an open project                        |
| `INTERNAL`              | other error / parse                                            | both                         | check `message`; report a bug if needed                   |

---

## 7. End-to-End Examples

**Example 1 — Replace a function body conflict-safely.**

```text
# 1. fetch the current range + hash (ctx_read delivers bytes; hash = BLAKE3 of the range)
ctx_refactor action=symbols_overview path=src/Main.kt        # find symbol + line
# 2. replace, secured against the expected hash
ctx_refactor action=replace_symbol_body name_path=Main/run \
  new_body="fun run() { println(\"v2\") }" expected_hash=<blake3-hex>
# → applied:true ; on a concurrent change → CONFLICT (file untouched)
```

**Example 2 — Project-wide rename (two-phase).**

```text
# Phase 1: preview — see usages + conflicts
ctx_refactor action=rename path=src/Main.kt line=7 column=4 new_name=execute
#   internal: POST /renamePreview → {usages:[…], conflicts:[]}
# Phase 2: with empty conflicts, apply automatically (one transaction, one undo)
#   internal: POST /renameApply → {applied:true, changed_paths:[…]}
```

**Example 3 — Reformat a file (after an edit).**

```text
ctx_refactor action=replace_symbol_body name_path=Main/run new_body="…"
ctx_refactor action=reformat path=src/Main.kt    # apply code style afterward
# → {"reformatted":true,"path":"src/Main.kt"}
```

---

## 8. Cross-references & Sources

- [Concise agent reference](appendix-jetbrains-plugin-de.md) — tables for quick lookup
- [Per-IDE quickstarts](appendix-ide-quickstarts.md) — setup for JetBrains IDEs
- [MCP tool map](appendix-mcp-tools.md) — all MCP tools incl. `ctx_refactor`, `ctx_symbol`
- [Journey 4 — Code Intelligence](04-code-intelligence.md)
- [Journey 13 — Security & Governance](13-security-and-governance.md) — PathJail, auth
- Source code: `rust/src/lsp/{backend,jetbrains_backend,router,edit_apply,port_discovery}.rs`,
  `rust/src/tools/registered/ctx_refactor.rs`,
  `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/{server,endpoint,psi,dto}/…`
