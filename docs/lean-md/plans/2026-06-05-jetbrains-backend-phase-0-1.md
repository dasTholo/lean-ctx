# JetBrains-PSI-Backend — Phase 0 + 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Den Rust-Kern von lean-ctx auf ein austauschbares `LspBackend`-Trait umstellen (Phase 0, refactor-only inkl. PathJail-Härtung §4.5) und ein zweites Backend (JetBrains-HTTP, Backing B) mit Port-Datei-Discovery + B-first-Selektion und deterministischem Fallback auf rust-analyzer (Backing A) als Skeleton anlegen (Phase 1).

**Architecture:** `ctx_refactor` ruft über `lsp::router::with_backend` ein `&mut dyn LspBackend`. Phase 0 extrahiert das Trait aus dem heutigen `LspClient` (verhaltensidentisch) und schließt die §4.5-Naht, sodass PathJail garantiert vor jedem Backend-Aufruf greift. Phase 1 fügt `JetBrainsHttpBackend` (synchron via `ureq`), Port-Datei-Discovery (`port_discovery.rs`) und die Factory `select_backend` (B-first, A-Fallback) hinzu. Edits/`type_hierarchy`/`format`/`inspections` sind **nicht** Teil dieser beiden Phasen (Phase 4/5 bzw. v2).

**Tech Stack:** Rust, `lsp_types`, `serde_json`, `ureq = "3.3.0"` (blocking HTTP — **bereits Dependency**, `Cargo.toml:140` / auf `feat-jetbrains-plugin` `Cargo.toml:132`), `sha2 = "0.10"` (projecthash — **bereits Dependency**), `dirs` (vorhanden). **Kein `json`-Feature** (Repo-Konvention): JSON-Requests via `.send(&serde_json::to_vec(&body)?)` mit `Content-Type: application/json`, Antworten via `.into_body().read_to_string()` + `serde_json::from_str` — Vorbild `rust/src/cloud_client.rs:146-158`. Per-Request-Timeout (ureq 3.x): `.config().timeout_global(Some(dur)).build()`. Tests via `cargo nextest run` (niemals `cargo test`).

**Spec:** `docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md` — §4 (Rust-Seite), §6 (Wire-DTO), §9 (Phasen), §12 (Branch-Strategie).

**Branch & Commit-Disziplin (§12):** Gesamte Arbeit auf `feat-jetbrains-plugin` (geht von `main` ab, kein worktree). **Ein Commit pro Phase** — innerhalb einer Phase NICHT pro Task committen, sondern erst im finalen Schritt der Phase. Der Spec-Sync (Task 0.0) ist ein eigener, vorgelagerter Commit.

**Rust-Edit-Regel (Projekt):** `*.rs`-Änderungen ausschließlich über Serena-Tools (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_after_symbol`, …) — **nie** native `Edit`/`ctx_edit` auf Rust. `Cargo.toml` (kein Rust) via `ctx_edit`. Vor jedem `git add`: `mcp__jetbrains__reformat_file` auf alle geänderten Dateien.

---

## File Structure

**Phase 0 (refactor-only):**
- Create: `rust/src/lsp/backend.rs` — `LspBackend`-Trait + Begleittypen (`HierarchyDirection`, `TypeHierarchyNode`, `SymbolOverviewItem`, `InspectionDiag`).
- Modify: `rust/src/lsp/mod.rs` — `pub mod backend;`.
- Modify: `rust/src/lsp/client.rs` — `impl LspBackend for LspClient` (delegiert 5 vorhandene Methoden).
- Modify: `rust/src/lsp/router.rs` — `BACKENDS: HashMap<String, Box<dyn LspBackend>>`, `with_client` → `with_backend`.
- Modify: `rust/src/tools/ctx_refactor.rs` — innere `handle` nimmt gejailten `abs_path`; `with_client` → `with_backend`.
- Modify: `rust/src/tools/registered/ctx_refactor.rs` — §4.5: Pfad via `require_resolved_path` (jailt vor Backend).
- Modify: `rust/tests/lsp_integration.rs` — Test-Helper auf neue innere `handle`-Signatur.

**Phase 1 (Backing B Skeleton):**
- Create: `rust/src/lsp/port_discovery.rs` — `project_hash`, `PortFile`, `read_port_file`, `pid_alive`, `health_ok`.
- Create: `rust/src/lsp/jetbrains_backend.rs` — `JetBrainsHttpBackend` (`impl LspBackend` für refs/def/impl via `ureq`).
- Modify: `rust/src/lsp/mod.rs` — `pub mod port_discovery; pub mod jetbrains_backend;`.
- Modify: `rust/src/lsp/router.rs` — Factory `select_backend(language, project_root)`; `with_backend` nutzt sie.
- **Keine `Cargo.toml`-Änderung** — `ureq = "3.3.0"` und `sha2 = "0.10"` sind bereits Dependencies (verifiziert auf `feat-jetbrains-plugin`); `json`-Feature wird bewusst nicht verwendet.

---

## Task 0.0: Branch-Neuanlage von `origin/main` (3.7.4) + Spec-Sync (eigener Commit)

**Ziel:** Den stale lokalen `feat-jetbrains-plugin` (saß auf altem `main` 3.6.11, 231 Commits hinter `origin/main`) **verwerfen** und **frisch von `origin/main` (3.7.4)** neu anlegen; dann Spec + diesen Plan aus `feat-lmd-v1` als **einen** Commit obendrauf (§12.1/§12.2).

**Voraussetzung:** Spec **und** Plan sind auf `feat-lmd-v1` committet (korrigierte §12 mit Basis `origin/main` 3.7.4). `origin` ist gefetcht (`git fetch origin`).

**Files:**
- Add (auf neuem Zielbranch): `docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md`
- Add (auf neuem Zielbranch): `docs/lean-md/plans/2026-06-05-jetbrains-backend-phase-0-1.md` (dieser Plan)

- [ ] **Step 1: origin fetchen + Stand prüfen**

Run:
```bash
git fetch origin
git rev-list --left-right --count origin/main...feat-jetbrains-plugin
```
Expected: linke Zahl groß (≈231 = `feat-jetbrains-plugin` hinter `origin/main`), rechte = 1 → bestätigt stale Basis. `git show origin/main:rust/Cargo.toml` zeigt `version = "3.7.4"`.

- [ ] **Step 2: Lokales `main` auf `origin/main` aktualisieren**

Untracked Working-Tree-Dateien (`markdownai/`, `.serena/project.yml`) stören den Branch-Wechsel nicht (bleiben liegen). Lokales `main` ist nur ein stale Pointer ohne eigene Arbeit.
Run:
```bash
git switch main
git merge --ff-only origin/main
```
Expected: fast-forward auf 3.7.4. Falls `--ff-only` fehlschlägt (divergiert), `git reset --hard origin/main` (kein lokaler `main`-Verlust erwartet).

- [ ] **Step 3: Stale Branch löschen + frisch von `origin/main` anlegen**

Run:
```bash
git branch -D feat-jetbrains-plugin
git switch -c feat-jetbrains-plugin origin/main
```
Expected: neuer `feat-jetbrains-plugin` auf 3.7.4; `git show HEAD:rust/Cargo.toml` → `version = "3.7.4"`, enthält `ureq = "3.3.0"` + `sha2 = "0.10"`.

- [ ] **Step 4: Spec + Plan aus `feat-lmd-v1` übernehmen (Datei-Inhalt)**

Run:
```bash
git checkout feat-lmd-v1 -- docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md docs/lean-md/plans/2026-06-05-jetbrains-backend-phase-0-1.md
```
Expected: beide Dateien im Index (neu hinzugefügt, da `docs/lean-md/` auf frischem `origin/main` sonst leer/anders ist).

- [ ] **Step 5: Korrigierte §12 verifizieren**

Run: `mcp__lean-ctx__ctx_search(pattern="origin/main. = .v3.7.4|neu von .origin/main", path="docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md")`
Expected: Treffer in §12.1/§12.2 (korrigierte Basis übernommen).

- [ ] **Step 6: (Optional) Projekt-Rules übernehmen**

Falls die allgemeinen Rules (`CLAUDE.md`, `rust/CLAUDE.md`, `.claude/rules/…`) auf `origin/main` fehlen/älter sind, selektiv aus `feat-lmd-v1` holen und **lmd-Referenzen bereinigen** (§12.2). Sonst überspringen.

- [ ] **Step 7: Reformat + Commit (Spec-Sync)**

`mcp__jetbrains__reformat_file` für `.md` optional, aber Projekt-Rule — ausführen.
Run:
```bash
git add docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md docs/lean-md/plans/2026-06-05-jetbrains-backend-phase-0-1.md
git commit -m "docs(jetbrains): design spec (§12 base=origin/main 3.7.4) + phase-0/1 plan"
```
Expected: ein Commit auf frischem `feat-jetbrains-plugin` (Basis 3.7.4).

---

## Phase 0 — `LspBackend`-Trait-Extraktion (refactor-only, ein Commit)

> Gate (§9): bestehende `ctx_refactor`-Tests grün, Verhalten identisch, clippy sauber. §4.5-Pfad-Fix Pflicht. **Erst im letzten Schritt der Phase committen.**

### Task 0.1: Trait + Begleittypen anlegen

**Files:**
- Create: `rust/src/lsp/backend.rs`
- Modify: `rust/src/lsp/mod.rs`

- [ ] **Step 1: `backend.rs` schreiben**

Datei `rust/src/lsp/backend.rs` (neue Datei → native `Write` ist erlaubt, da kein bestehendes Symbol editiert wird):

```rust
//! Backend abstraction for LSP-style code intelligence.
//!
//! Two backings implement this trait:
//!   A) `LspClient` (stdio rust-analyzer) — CI/headless fallback, see client.rs
//!   B) `JetBrainsHttpBackend` (in-IDE PSI over HTTP) — preferred, see jetbrains_backend.rs
//!
//! The 5 mandatory methods exist in both backings (today's behavior must not break).
//! The default-degrading methods return a clear "unsupported" error unless a backing
//! (Backing B) overrides them.

use lsp_types::{GotoDefinitionResponse, Location, Position, TextEdit, Uri, WorkspaceEdit};

/// Direction for `type_hierarchy` queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchyDirection {
    Subtypes,
    Supertypes,
}

/// A node in a type hierarchy (super/subtype tree).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeHierarchyNode {
    pub name: String,
    /// Project-relative path of the declaring file.
    pub path: String,
    /// 1-indexed line of the declaration.
    pub line: u32,
    pub children: Vec<TypeHierarchyNode>,
}

/// A single symbol entry from a file's structure overview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolOverviewItem {
    pub name: String,
    pub kind: String,
    /// 1-indexed line.
    pub line: u32,
}

/// A single inspection/diagnostic result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectionDiag {
    /// Project-relative path.
    pub path: String,
    /// 1-indexed line.
    pub line: u32,
    pub severity: String,
    pub message: String,
}

/// Code-intelligence backend. `Send` so instances can live in the global
/// `BACKENDS` cache (`Mutex<HashMap<String, Box<dyn LspBackend>>>`).
pub trait LspBackend: Send {
    // ── Mandatory (both backings) ──
    fn open_file(&mut self, uri: &Uri, language_id: &str, text: &str) -> Result<(), String>;
    fn references(&mut self, uri: &Uri, position: Position) -> Result<Vec<Location>, String>;
    fn definition(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<GotoDefinitionResponse, String>;
    fn implementations(&mut self, uri: &Uri, position: Position)
        -> Result<Vec<Location>, String>;
    fn rename(
        &mut self,
        uri: &Uri,
        position: Position,
        new_name: &str,
    ) -> Result<Option<WorkspaceEdit>, String>;

    // ── Default-degrading (Backing B preferred; Backing A keeps the Err) ──
    fn declaration(&mut self, _uri: &Uri, _position: Position) -> Result<Vec<Location>, String> {
        Err("declaration requires the JetBrains backend".to_string())
    }
    fn type_hierarchy(
        &mut self,
        _uri: &Uri,
        _position: Position,
        _direction: HierarchyDirection,
    ) -> Result<TypeHierarchyNode, String> {
        Err("type_hierarchy requires the JetBrains backend".to_string())
    }
    fn symbols_overview(&mut self, _uri: &Uri) -> Result<Vec<SymbolOverviewItem>, String> {
        Err("symbols_overview requires the JetBrains backend".to_string())
    }
    fn format(&mut self, _uri: &Uri) -> Result<Vec<TextEdit>, String> {
        Err("format requires the JetBrains backend".to_string())
    }
    fn inspections(&mut self, _uri: &Uri) -> Result<Vec<InspectionDiag>, String> {
        Err("inspections requires the JetBrains backend".to_string())
    }
}
```

- [ ] **Step 2: Modul registrieren**

In `rust/src/lsp/mod.rs` via Serena `insert_before_symbol` (vor `pub mod client;`) die Zeile einfügen, sodass die Datei lautet:

```rust
pub mod backend;
pub mod client;
pub mod config;
pub mod router;
```

- [ ] **Step 3: Kompiliert (Trait isoliert)**

Run: `mcp__lean-ctx__ctx_shell(command="cargo build -p lean-ctx --lib 2>&1 | tail -20")`
Expected: kompiliert (Trait noch ungenutzt → evtl. `dead_code`-Warnung auf Default-Methoden, akzeptabel; wird in Task 0.2/0.4 genutzt).

### Task 0.2: `impl LspBackend for LspClient`

**Files:**
- Modify: `rust/src/lsp/client.rs`

- [ ] **Step 1: Trait-Impl anfügen**

`LspClient` hat bereits inhärente `did_open/references/definition/implementations/rename`. Die Trait-Impl delegiert explizit (vollqualifiziert, um Methoden-Namensgleichheit eindeutig zu lösen). Via Serena `insert_after_symbol` nach dem `impl Drop for LspClient`-Block in `client.rs` einfügen:

```rust
impl crate::lsp::backend::LspBackend for LspClient {
    fn open_file(&mut self, uri: &lsp_types::Uri, language_id: &str, text: &str) -> Result<(), String> {
        LspClient::did_open(self, uri, language_id, text)
    }
    fn references(
        &mut self,
        uri: &lsp_types::Uri,
        position: lsp_types::Position,
    ) -> Result<Vec<lsp_types::Location>, String> {
        LspClient::references(self, uri, position)
    }
    fn definition(
        &mut self,
        uri: &lsp_types::Uri,
        position: lsp_types::Position,
    ) -> Result<lsp_types::GotoDefinitionResponse, String> {
        LspClient::definition(self, uri, position)
    }
    fn implementations(
        &mut self,
        uri: &lsp_types::Uri,
        position: lsp_types::Position,
    ) -> Result<Vec<lsp_types::Location>, String> {
        LspClient::implementations(self, uri, position)
    }
    fn rename(
        &mut self,
        uri: &lsp_types::Uri,
        position: lsp_types::Position,
        new_name: &str,
    ) -> Result<Option<lsp_types::WorkspaceEdit>, String> {
        LspClient::rename(self, uri, position, new_name)
    }
    // declaration/type_hierarchy/symbols_overview/format/inspections: Default-Err (Backing A).
}
```

- [ ] **Step 2: Kompiliert**

Run: `mcp__lean-ctx__ctx_shell(command="cargo build -p lean-ctx --lib 2>&1 | tail -20")`
Expected: kompiliert. `LspClient` selbst unverändert.

### Task 0.3: Router auf `Box<dyn LspBackend>`

**Files:**
- Modify: `rust/src/lsp/router.rs`

- [ ] **Step 1: Cache-Typ + Import umstellen**

In `router.rs`: Import um `use super::backend::LspBackend;` ergänzen (via Serena `replace_content` der Import-Zeile `use super::client::{file_path_to_uri, LspClient};` → zwei Zeilen):

```rust
use super::backend::LspBackend;
use super::client::{file_path_to_uri, LspClient};
```

Statisches Cache umbenennen `CLIENTS` → `BACKENDS` und Typ ändern (Serena `replace_content`):

```rust
static BACKENDS: std::sync::LazyLock<Mutex<HashMap<String, Box<dyn LspBackend>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
```

- [ ] **Step 2: `with_client` → `with_backend`**

`with_client` ersetzen durch `with_backend` (Serena `replace_symbol_body` auf `with_client`, plus Umbenennung des Symbols — falls Serena `rename` einfacher: erst `rename` `with_client`→`with_backend`, dann Body ersetzen). Neuer Body (Phase 0: konstruiert weiterhin `LspClient`, boxt ihn — `select_backend` kommt erst in Phase 1):

```rust
pub fn with_backend<F, R>(file_path: &str, project_root: &str, f: F) -> Result<R, String>
where
    F: FnOnce(&mut dyn LspBackend, &str) -> Result<R, String>,
{
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let language = language_for_extension(ext).ok_or_else(|| {
        format!(
            "No LSP server configured for extension '.{ext}'. Supported: rs, ts, tsx, js, py, go"
        )
    })?;

    let mut backends = BACKENDS.lock().map_err(|e| e.to_string())?;

    if !backends.contains_key(language) {
        let config = resolve_config_for_language(language);

        if super::config::find_binary_in_path(&config.command).is_none()
            && !Path::new(&config.command).is_file()
        {
            check_server_available(language)?;
        }

        let root_uri = file_path_to_uri(project_root)?;
        let client = LspClient::start(&config, &root_uri)?;
        backends.insert(language.to_string(), Box::new(client) as Box<dyn LspBackend>);
    }

    let backend = backends
        .get_mut(language)
        .ok_or_else(|| format!("LSP backend for '{language}' not available"))?;

    f(backend.as_mut(), language)
}
```

- [ ] **Step 3: `open_file` + `shutdown_all` anpassen**

In `open_file` (router.rs): `with_client(...)` → `with_backend(...)` und `client.did_open(...)` → `backend.open_file(...)`. Via Serena `replace_symbol_body` auf `open_file`:

```rust
pub fn open_file(file_path: &str, project_root: &str) -> Result<Uri, String> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    language_for_extension(ext).ok_or_else(|| {
        format!(
            "No LSP server configured for extension '.{ext}'. Supported: rs, ts, tsx, js, py, go"
        )
    })?;

    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Cannot read '{file_path}': {e}"))?;

    let uri = file_path_to_uri(file_path)?;

    with_backend(file_path, project_root, |backend, language| {
        backend.open_file(&uri, language, &content)?;
        Ok(uri.clone())
    })
}
```

`shutdown_all`: `CLIENTS` → `BACKENDS` (Serena `replace_symbol_body`):

```rust
pub fn shutdown_all() {
    if let Ok(mut backends) = BACKENDS.lock() {
        for (_, backend) in backends.drain() {
            drop(backend);
        }
    }
}
```

- [ ] **Step 4: Kompiliert (router intern)**

Run: `mcp__lean-ctx__ctx_shell(command="cargo build -p lean-ctx --lib 2>&1 | tail -30")`
Expected: Fehler NUR noch in `ctx_refactor.rs` (Call-Sites `with_client`) — wird in Task 0.4 behoben.

### Task 0.4: §4.5-PathJail-Fix + Call-Sites umstellen

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`
- Modify: `rust/src/tools/registered/ctx_refactor.rs`
- Modify: `rust/tests/lsp_integration.rs`

> **§4.5-Kern:** Der Dispatcher jailt `path` bereits **vor** `handle` (PATH_LIKE_KEYS, `dispatch/mod.rs:151,324`) → Ergebnis steht in `ctx.resolved_path("path")` bzw. der Fehler in `ctx.path_error("path")`. Die heutige innere `handle` ignoriert das und baut `abs_path` selbst aus rohem `project_root + path` (Jail-Umgehung). Fix: Wrapper reicht den **gejailten** Pfad über `require_resolved_path` durch; innere `handle` nimmt ihn als Parameter und baut **nichts** mehr selbst.

- [ ] **Step 1: Charakterisierungs-Test schreiben (failing)**

Via Serena `insert_after_symbol` ans Ende von `rust/src/tools/ctx_refactor.rs` ein Test-Modul anhängen:

```rust
#[cfg(test)]
mod tests {
    use serde_json::json;

    /// §4.5: inner handle MUST use the (already jailed) abs_path it is given,
    /// never re-derive a path from raw args. A raw "../escape.rs" must never
    /// reach the filesystem layer; only the provided abs_path does.
    #[test]
    fn inner_handle_uses_provided_abs_path_not_raw_args() {
        let args = json!({"action": "references", "path": "../escape.rs", "line": 1, "column": 0});
        let out = super::handle(&args, "/proj", "/proj/jailed.rs");
        // open_file fails reading the (nonexistent) jailed file → error names abs_path.
        assert!(out.contains("/proj/jailed.rs"), "abs_path not used: {out}");
        assert!(!out.contains("../escape.rs"), "raw path leaked to fs layer: {out}");
    }
}
```

- [ ] **Step 2: Test ausführen → MUSS fehlschlagen (Signatur)**

Run: `mcp__lean-ctx__ctx_shell(command="cargo nextest run -p lean-ctx inner_handle_uses_provided_abs_path 2>&1 | tail -20")`
Expected: Compile-FAIL — `super::handle` nimmt aktuell nur 2 Argumente.

- [ ] **Step 3: Innere `handle`-Signatur ändern (abs_path-Parameter)**

Via Serena `replace_symbol_body` auf `handle` in `ctx_refactor.rs` — neuer Body ohne Selbstbau von `abs_path`, Presence-Check entfällt (Wrapper garantiert ihn):

```rust
pub fn handle(args: &Value, project_root: &str, abs_path: &str) -> String {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("references");

    let line = args.get("line").and_then(Value::as_u64).unwrap_or(1) as u32;
    let column = args.get("column").and_then(Value::as_u64).unwrap_or(0) as u32;

    let uri = match crate::lsp::router::open_file(abs_path, project_root) {
        Ok(u) => u,
        Err(e) => return format!("ERROR: {e}"),
    };

    let position = Position::new(line.saturating_sub(1), column);

    match action {
        "rename" => handle_rename(args, abs_path, project_root, &uri, position),
        "references" => handle_references(abs_path, project_root, &uri, position),
        "definition" => handle_definition(abs_path, project_root, &uri, position),
        "implementations" => handle_implementations(abs_path, project_root, &uri, position),
        _ => format!(
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, implementations."
        ),
    }
}
```

Die `Path`-Nutzung für den entfallenen `abs_path`-Aufbau wird damit ungenutzt → unbenutzten Import `use std::path::Path;` via Serena `replace_content` entfernen, falls sonst nirgends genutzt (clippy-Gate). (Prüfen: `mcp__lean-ctx__ctx_search(pattern="Path::", path="rust/src/tools/ctx_refactor.rs")` — falls 0 weitere Treffer, Import streichen.)

- [ ] **Step 4: `with_client` → `with_backend` in allen 4 Call-Sites**

In `ctx_refactor.rs` rufen `handle_rename/handle_references/handle_definition/handle_implementations` je `crate::lsp::router::with_client(...)` mit Closure `|client, _|`. Jeweils via Serena `replace_symbol_body` auf `with_backend(...)` und Closure-Param `|backend, _|` umstellen; Methodenaufrufe bleiben gleich (`backend.references(uri, position)` etc.). Beispiel `handle_references`:

```rust
fn handle_references(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        backend.references(uri, position)
    });

    match result {
        Ok(locations) => format_locations(&locations, project_root),
        Err(e) => format!("ERROR: {e}"),
    }
}
```

Analog für `handle_definition` (`backend.definition(uri, position)`), `handle_implementations` (`backend.implementations(uri, position)`), `handle_rename` (`backend.rename(uri, position, new_name)`).

- [ ] **Step 5: Wrapper §4.5 — gejailten Pfad durchreichen**

In `rust/src/tools/registered/ctx_refactor.rs` den Import um `require_resolved_path` ergänzen und `handle` umstellen. Import (Serena `replace_content`):

```rust
use crate::server::tool_trait::{get_str, require_resolved_path, McpTool, ToolContext, ToolOutput};
```

`handle` via Serena `replace_symbol_body`:

```rust
    fn handle(
        &self,
        args: &Map<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, ErrorData> {
        // §4.5: PathJail runs in the dispatcher BEFORE this handle. require_resolved_path
        // surfaces a jail rejection / missing / non-string `path` as an MCP error here,
        // so no relative/escaping path is ever rebuilt or sent to a backend.
        let abs_path = require_resolved_path(ctx, args, "path")?.to_string();

        let args_value = Value::Object(args.clone());
        let result = crate::tools::ctx_refactor::handle(&args_value, &ctx.project_root, &abs_path);

        let action = get_str(args, "action").unwrap_or_default();
        Ok(ToolOutput {
            text: result,
            original_tokens: 0,
            saved_tokens: 0,
            mode: Some(action),
            path: get_str(args, "path"),
            changed: false,
        })
    }
```

- [ ] **Step 6: Integrationstest-Helper anpassen**

In `rust/tests/lsp_integration.rs` ruft `call_refactor` die innere `handle` mit 2 Args. Da die Tests bereits **absolute** Pfade in `args["path"]` übergeben, den Helper via `mcp__lean-ctx__ctx_edit` (Test-Datei, aber `.rs` → Serena bevorzugt; nutze Serena `replace_symbol_body` auf `call_refactor`):

```rust
fn call_refactor(args: &serde_json::Value, root: &str) -> String {
    let abs_path = args.get("path").and_then(|v| v.as_str()).unwrap_or_default();
    lean_ctx::tools::ctx_refactor::handle(args, root, abs_path)
}
```

- [ ] **Step 7: Charakterisierungs-Test grün**

Run: `mcp__lean-ctx__ctx_shell(command="cargo nextest run -p lean-ctx inner_handle_uses_provided_abs_path 2>&1 | tail -20")`
Expected: PASS.

- [ ] **Step 8: Volles Phasen-Gate — Build, Tests, clippy**

Run:
```bash
cargo build -p lean-ctx --lib 2>&1 | tail -10
cargo nextest run -p lean-ctx 2>&1 | tail -25
cargo clippy -p lean-ctx --lib --tests 2>&1 | tail -25
```
Expected: Build ok; alle Tests grün (LSP-Integrationstests sind `#[ignore]`, kompilieren aber); clippy ohne Warnungen. (Verhalten identisch zu vorher; einzige bewusste Änderung: Jail-Rejection liefert jetzt sauberen Fehler vor Backend-Aufruf.)

- [ ] **Step 9: Reformat + Phase-0-Commit (EINZIGER Commit der Phase)**

`mcp__jetbrains__reformat_file` auf alle geänderten `.rs`-Dateien:
`rust/src/lsp/backend.rs`, `rust/src/lsp/mod.rs`, `rust/src/lsp/client.rs`, `rust/src/lsp/router.rs`, `rust/src/tools/ctx_refactor.rs`, `rust/src/tools/registered/ctx_refactor.rs`, `rust/tests/lsp_integration.rs`.

Run:
```bash
git add rust/src/lsp/backend.rs rust/src/lsp/mod.rs rust/src/lsp/client.rs rust/src/lsp/router.rs rust/src/tools/ctx_refactor.rs rust/src/tools/registered/ctx_refactor.rs rust/tests/lsp_integration.rs
git commit -m "feat(lsp): extract LspBackend trait + harden ctx_refactor PathJail (§4.5) [Phase 0]"
```
Expected: ein Commit; `git status` sauber (außer untracked Nicht-Phase-Dateien).

---

## Phase 1 — Port-Discovery + HTTP-Backend-Skeleton (ein Commit)

> Gate (§9): gegen Mock-Server parsebar; ohne Port-Datei deterministischer Fallback A. **Erst im letzten Schritt der Phase committen.**

### Task 1.1: ~~Dependencies hinzufügen~~ — ENTFÄLLT

`ureq = "3.3.0"` und `sha2 = "0.10"` sind **bereits** Dependencies auf `feat-jetbrains-plugin`. Keine `Cargo.toml`-Änderung. Das `json`-Feature wird **nicht** aktiviert — JSON läuft über `serde_json` + `ureq`-Body-API (Repo-Konvention, siehe Tech-Stack-Notiz). Direkt mit Task 1.2 fortfahren.

### Task 1.2: `port_discovery.rs`

**Files:**
- Create: `rust/src/lsp/port_discovery.rs`
- Modify: `rust/src/lsp/mod.rs`

- [ ] **Step 1: Modul schreiben**

Neue Datei `rust/src/lsp/port_discovery.rs` (native `Write` ok — neue Datei):

```rust
//! Discovery of the in-IDE JetBrains backend via a per-project port file.
//!
//! The plugin writes `~/.lean-ctx/jetbrains-<projecthash>.port` (JSON, 0600).
//! `projecthash = sha256(canonical(project_root))[..16]` — Rust and Kotlin MUST
//! canonicalize identically (symlink / trailing-slash trap, spec §5.5).

use std::time::Duration;

use serde::Deserialize;

/// Contents of the per-project port file (subset Rust needs).
#[derive(Debug, Clone, Deserialize)]
pub struct PortFile {
    pub port: u16,
    pub token: String,
    pub pid: u32,
    #[serde(default)]
    pub project_root: String,
    #[serde(default)]
    pub ide_version: String,
}

/// `sha256(canonical(project_root))[..16]` as lowercase hex (first 8 bytes → 16 chars).
pub fn project_hash(project_root: &str) -> String {
    use sha2::{Digest, Sha256};
    let canonical = std::fs::canonicalize(project_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| project_root.to_string());
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// `~/.lean-ctx/jetbrains-<projecthash>.port`.
pub fn port_file_path(project_root: &str) -> Option<std::path::PathBuf> {
    let home = dirs::home_dir()?;
    Some(
        home.join(".lean-ctx")
            .join(format!("jetbrains-{}.port", project_hash(project_root))),
    )
}

/// Reads + parses the port file, or `None` if absent/unreadable/malformed.
pub fn read_port_file(project_root: &str) -> Option<PortFile> {
    let path = port_file_path(project_root)?;
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// Liveness check for the IDE process. Linux: `/proc/<pid>`. Other OSes:
/// optimistic `true` (the `/health` ping is the authoritative reachability gate).
pub fn pid_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        true
    }
}

/// `GET /health` with token header and a tight timeout (~300ms, spec §4.3).
/// ureq 3.x: per-request timeout via `.config().timeout_global(..).build()`.
pub fn health_ok(pf: &PortFile) -> bool {
    let url = format!("http://127.0.0.1:{}/health", pf.port);
    ureq::get(&url)
        .config()
        .timeout_global(Some(Duration::from_millis(300)))
        .build()
        .header("X-LeanCtx-Token", &pf.token)
        .call()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_hash_is_stable_and_16_hex() {
        let h1 = project_hash("/some/project");
        let h2 = project_hash("/some/project");
        assert_eq!(h1, h2, "hash must be deterministic");
        assert_eq!(h1.len(), 16, "expected 16 hex chars (8 bytes)");
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn port_file_absent_for_unlikely_root() {
        // A path that has no port file → None (never panics).
        assert!(read_port_file("/nonexistent/lean-ctx/project/xyz").is_none());
    }
}
```

- [ ] **Step 2: Modul registrieren**

In `rust/src/lsp/mod.rs` via Serena `insert_after_symbol` nach `pub mod config;` einfügen, Ziel:

```rust
pub mod backend;
pub mod client;
pub mod config;
pub mod jetbrains_backend;
pub mod port_discovery;
pub mod router;
```
(`jetbrains_backend` wird in Task 1.3 erstellt — Reihenfolge egal; falls Build vor Task 1.3 nötig, diese Zeile erst dort hinzufügen.)

- [ ] **Step 3: Tests grün**

Run: `mcp__lean-ctx__ctx_shell(command="cargo nextest run -p lean-ctx port_discovery 2>&1 | tail -20")`
Expected: `project_hash_is_stable_and_16_hex` + `port_file_absent_for_unlikely_root` PASS.

### Task 1.3: `jetbrains_backend.rs` (refs/def/impl via `ureq`)

**Files:**
- Create: `rust/src/lsp/jetbrains_backend.rs`

> Wire-DTO (§6): Request `{path, line, character}` (Pfad **relativ** zu project_root, Position **0-basiert**); Response `{locations:[{path, range:{start:{line,character}, end:{...}}}]}`. Rust joint relative Pfade zurück zu absoluten file-URIs.

- [ ] **Step 1: Test zuerst — Mock-Server-Parsing (failing)**

Neue Datei `rust/src/lsp/jetbrains_backend.rs` mit Implementierung **und** Test (native `Write` ok — neue Datei). Inhalt:

```rust
//! Backing B: in-IDE JetBrains PSI backend over HTTP/JSON (127.0.0.1).
//! Synchronous (`ureq`) — matches the synchronous `McpTool::handle` path and does
//! not block the Tokio runtime. Phase 1 implements references/definition/
//! implementations; rename + the degrading ops follow in later phases.

use std::time::Duration;

use lsp_types::{GotoDefinitionResponse, Location, Position, Range, Uri, WorkspaceEdit};
use serde_json::Value;

use crate::lsp::backend::LspBackend;
use crate::lsp::client::file_path_to_uri;

const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct JetBrainsHttpBackend {
    base_url: String,
    token: String,
    /// Absolute project root, to rejoin project-relative wire paths.
    project_root: String,
}

impl JetBrainsHttpBackend {
    pub fn new(port: u16, token: String, project_root: String) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            token,
            project_root,
        }
    }

    fn post(&self, endpoint: &str, body: Value) -> Result<Value, String> {
        let url = format!("{}{endpoint}", self.base_url);
        // ureq 3.x + repo convention (NO `json` feature): serialize via serde_json,
        // send raw bytes, read response body as string, parse. Per-request timeout via
        // `.config().timeout_global(..).build()`. Pattern mirrors cloud_client.rs.
        let payload =
            serde_json::to_vec(&body).map_err(|e| format!("serialize request: {e}"))?;
        let resp = ureq::post(&url)
            .config()
            .timeout_global(Some(Duration::from_secs(REQUEST_TIMEOUT_SECS)))
            .build()
            .header("X-LeanCtx-Token", &self.token)
            .header("Content-Type", "application/json")
            .send(&payload)
            .map_err(|e| format!("JetBrains backend request to {endpoint} failed: {e}"))?;
        let text = resp
            .into_body()
            .read_to_string()
            .map_err(|e| format!("JetBrains backend: read response: {e}"))?;
        serde_json::from_str(&text)
            .map_err(|e| format!("JetBrains backend: parse response: {e}"))
    }

    /// Project-relative path → absolute file URI (Rust rejoins, spec §6).
    fn rel_to_uri(&self, rel: &str) -> Option<Uri> {
        let abs = format!("{}/{}", self.project_root, rel);
        file_path_to_uri(&abs).ok()
    }

    fn parse_position(v: &Value) -> Option<Position> {
        let line = v.get("line")?.as_u64()? as u32;
        let character = v.get("character")?.as_u64()? as u32;
        Some(Position { line, character })
    }

    fn parse_locations(&self, v: &Value) -> Vec<Location> {
        v.get("locations")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|loc| {
                        let rel = loc.get("path")?.as_str()?;
                        let uri = self.rel_to_uri(rel)?;
                        let range = loc.get("range")?;
                        let start = Self::parse_position(range.get("start")?)?;
                        let end = Self::parse_position(range.get("end")?)?;
                        Some(Location {
                            uri,
                            range: Range { start, end },
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Build the `{path, line, character}` request body. `position` is already
    /// 0-based (LSP convention) — sent verbatim. `uri` → project-relative path.
    fn position_body(&self, uri: &Uri, position: Position) -> Value {
        let abs = crate::lsp::client::uri_to_file_path(uri).unwrap_or_default();
        let rel = abs
            .strip_prefix(&self.project_root)
            .map(|s| s.strip_prefix('/').unwrap_or(s).to_string())
            .unwrap_or(abs);
        serde_json::json!({
            "path": rel,
            "line": position.line,
            "character": position.character,
        })
    }
}

impl LspBackend for JetBrainsHttpBackend {
    fn open_file(&mut self, _uri: &Uri, _language_id: &str, _text: &str) -> Result<(), String> {
        // The IDE already has the file in its VFS/index — no explicit open needed.
        Ok(())
    }

    fn references(&mut self, uri: &Uri, position: Position) -> Result<Vec<Location>, String> {
        let body = self.position_body(uri, position);
        let resp = self.post("/references", body)?;
        Ok(self.parse_locations(&resp))
    }

    fn definition(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<GotoDefinitionResponse, String> {
        let body = self.position_body(uri, position);
        let resp = self.post("/definition", body)?;
        Ok(GotoDefinitionResponse::Array(self.parse_locations(&resp)))
    }

    fn implementations(&mut self, uri: &Uri, position: Position) -> Result<Vec<Location>, String> {
        let body = self.position_body(uri, position);
        let resp = self.post("/implementations", body)?;
        Ok(self.parse_locations(&resp))
    }

    fn rename(
        &mut self,
        _uri: &Uri,
        _position: Position,
        _new_name: &str,
    ) -> Result<Option<WorkspaceEdit>, String> {
        // Symbolic edits are v2 (spec §9 v2-Ausblick). Phase 1 skeleton: not yet.
        Err("rename via JetBrains backend is not implemented yet (v2 edit spec)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Spins up a one-shot TCP server returning a canned HTTP/JSON response,
    /// so we can assert the wire→Location mapping without a real IDE.
    fn mock_once(json_body: &'static str) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 2048];
                let _ = stream.read(&mut buf); // drain request
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    json_body.len(),
                    json_body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        port
    }

    #[test]
    fn references_parses_wire_locations() {
        let body = r#"{"locations":[{"path":"src/main.rs","range":{"start":{"line":5,"character":13},"end":{"line":5,"character":18}}}]}"#;
        let port = mock_once(body);
        let mut backend =
            JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string());
        let uri = file_path_to_uri("/proj/src/main.rs").unwrap();
        let locs = backend
            .references(&uri, Position { line: 5, character: 13 })
            .expect("should parse");
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].range.start.line, 5);
        assert_eq!(locs[0].range.start.character, 13);
        assert!(locs[0].uri.as_str().ends_with("/proj/src/main.rs"));
    }
}
```

- [ ] **Step 2: Test → erst rot, dann grün**

Run: `mcp__lean-ctx__ctx_shell(command="cargo nextest run -p lean-ctx references_parses_wire_locations 2>&1 | tail -25")`
Expected: kompiliert und PASS. (Falls `mod.rs` `jetbrains_backend` noch nicht eingetragen ist → erst Task 1.2 Step 2 abschließen.)

### Task 1.4: `select_backend`-Factory (B-first, A-Fallback)

**Files:**
- Modify: `rust/src/lsp/router.rs`

- [ ] **Step 1: Factory einfügen**

Via Serena `insert_before_symbol` vor `with_backend` in `router.rs`. Importe um die neuen Module ergänzen (Serena `replace_content` der `use super::backend::LspBackend;`-Zeile → mehrere Zeilen):

```rust
use super::backend::LspBackend;
use super::client::{file_path_to_uri, LspClient};
use super::jetbrains_backend::JetBrainsHttpBackend;
use super::port_discovery;
```

Factory:

```rust
/// Selects a code-intelligence backend for `language` (§4.3).
///
/// Config `cfg.lsp[language]` (HashMap<String,String>):
///   - absent  → "auto" = B-first (JetBrains if reachable, else rust-analyzer)
///   - "auto"      → same as absent
///   - "jetbrains" → B only (error if the IDE is not reachable; no fallback)
///   - anything else → treated as an explicit rust-analyzer binary path = A only
///
/// Reachability = live port file + pid alive + `/health` ping. On any miss in
/// "auto" mode we fall back to Backing A deterministically (one ~300ms timeout max).
fn select_backend(language: &str, project_root: &str) -> Result<Box<dyn LspBackend>, String> {
    let cfg = crate::core::config::Config::load();
    let mode = cfg.lsp.get(language).map(String::as_str);

    let want_b = matches!(mode, None | Some("auto") | Some("jetbrains"));
    let b_only = mode == Some("jetbrains");

    if want_b {
        if let Some(pf) = port_discovery::read_port_file(project_root) {
            if port_discovery::pid_alive(pf.pid) && port_discovery::health_ok(&pf) {
                return Ok(Box::new(JetBrainsHttpBackend::new(
                    pf.port,
                    pf.token,
                    project_root.to_string(),
                )));
            }
        }
        if b_only {
            return Err(format!(
                "LSP backend 'jetbrains' configured for '{language}' but the IDE is not reachable \
                 (no live port file / health check failed)"
            ));
        }
    }

    // Backing A: rust-analyzer (today's behavior).
    let config = resolve_config_for_language(language);
    if super::config::find_binary_in_path(&config.command).is_none()
        && !Path::new(&config.command).is_file()
    {
        check_server_available(language)?;
    }
    let root_uri = file_path_to_uri(project_root)?;
    let client = LspClient::start(&config, &root_uri)?;
    Ok(Box::new(client) as Box<dyn LspBackend>)
}
```

- [ ] **Step 2: `with_backend` auf Factory umstellen**

Via Serena `replace_symbol_body` auf `with_backend` — den inline-LspClient-Aufbau durch `select_backend` ersetzen:

```rust
pub fn with_backend<F, R>(file_path: &str, project_root: &str, f: F) -> Result<R, String>
where
    F: FnOnce(&mut dyn LspBackend, &str) -> Result<R, String>,
{
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let language = language_for_extension(ext).ok_or_else(|| {
        format!(
            "No LSP server configured for extension '.{ext}'. Supported: rs, ts, tsx, js, py, go"
        )
    })?;

    let mut backends = BACKENDS.lock().map_err(|e| e.to_string())?;

    if !backends.contains_key(language) {
        let backend = select_backend(language, project_root)?;
        backends.insert(language.to_string(), backend);
    }

    let backend = backends
        .get_mut(language)
        .ok_or_else(|| format!("LSP backend for '{language}' not available"))?;

    f(backend.as_mut(), language)
}
```

- [ ] **Step 3: Determinismus-Test — ohne Port-Datei kein B**

Da ein Volltest von `with_backend` einen echten rust-analyzer bräuchte, testen wir die Selektions-**Vorentscheidung** isoliert: ohne Port-Datei liefert die Discovery `None`. Via Serena `insert_after_symbol` ein `#[cfg(test)] mod tests` ans Ende von `router.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_port_file_means_no_backing_b() {
        // With no IDE port file for an unlikely root, discovery yields None →
        // select_backend would deterministically fall through to Backing A.
        let pf = port_discovery::read_port_file("/nonexistent/leanctx/proj/xyz");
        assert!(pf.is_none(), "unexpected port file for nonexistent root");
    }
}
```

- [ ] **Step 4: Tests + clippy**

Run:
```bash
cargo nextest run -p lean-ctx 2>&1 | tail -25
cargo clippy -p lean-ctx --lib --tests 2>&1 | tail -25
```
Expected: alle Tests grün (inkl. `references_parses_wire_locations`, `project_hash_*`, `no_port_file_means_no_backing_b`); clippy sauber.

### Task 1.5: Phasen-Gate + Phase-1-Commit (EINZIGER Commit der Phase)

- [ ] **Step 1: Voller Build + Test + clippy**

Run:
```bash
cargo build -p lean-ctx --lib 2>&1 | tail -10
cargo nextest run -p lean-ctx 2>&1 | tail -25
cargo clippy -p lean-ctx --lib --tests 2>&1 | tail -20
```
Expected: alles grün/sauber. Ohne laufende IDE wählt `select_backend` deterministisch Backing A (Regressionsschutz; LSP-Integrationstests bleiben `#[ignore]`).

- [ ] **Step 2: Reformat + Commit**

`mcp__jetbrains__reformat_file` auf: `rust/src/lsp/port_discovery.rs`, `rust/src/lsp/jetbrains_backend.rs`, `rust/src/lsp/mod.rs`, `rust/src/lsp/router.rs`. (`Cargo.toml` braucht kein reformat.)

Run:
```bash
git add rust/Cargo.toml rust/src/lsp/port_discovery.rs rust/src/lsp/jetbrains_backend.rs rust/src/lsp/mod.rs rust/src/lsp/router.rs
git commit -m "feat(lsp): JetBrains HTTP backend skeleton + port discovery + B-first select_backend [Phase 1]"
```
Expected: ein Commit; `git status` sauber (außer untracked Nicht-Phase-Dateien). `Cargo.lock` ggf. mitcommitten, falls vom Repo getrackt (`git status` prüfen).

---

## Self-Review (gegen Spec)

**Spec-Abdeckung Phase 0 (§9):** Trait-Extraktion (Task 0.1) ✓; `impl für LspClient` (0.2) ✓; Router auf `Box<dyn LspBackend>` (0.3) ✓; §4.5-Pfad-Fix (0.4) ✓; Gate „Tests grün/Verhalten identisch/clippy" (0.4 Step 8) ✓.

**Spec-Abdeckung Phase 1 (§9):** `port_discovery.rs` (1.2) ✓; `jetbrains_backend.rs` refs/def/impl via `ureq` (1.3) ✓; `select_backend` mit Fallback (1.4) ✓; Gate „gegen Mock parsebar / ohne Port-Datei Fallback A" (1.3 Step 2, 1.4 Step 3) ✓.

**Bewusst NICHT in Phase 0/1 (spätere Phasen):** `type_hierarchy`/`overview`/`format`/`inspections`-Actions + Schema-Erweiterung in `registered/ctx_refactor.rs` (Phase 4/5); `rename`-apply über Backing B (v2); Kotlin-Plugin (Phase 2/3). Die Default-degradierenden Trait-Methoden sind bereits angelegt (additiv, kein Breaking Change).

**Typ-Konsistenz:** Trait-Methodennamen (`open_file/references/definition/implementations/rename`) identisch in `backend.rs`, `client.rs`-Impl, `jetbrains_backend.rs`-Impl und Router-Closures. `with_backend` (nicht `with_client`) durchgängig. `JetBrainsHttpBackend::new(port, token, project_root)` einheitlich in `select_backend` und Test. `PortFile`-Felder (`port/token/pid`) konsistent zwischen `read_port_file`, `health_ok`, `select_backend`.

**Offene Punkte (in späteren Phasen zu prüfen):** (1) Plugin und Rust müssen `projecthash` **byte-identisch** canonicalisieren (§5.5) — beim Plugin-Bau (Phase 2) gegen `project_hash` hier verifizieren. (2) `ureq`-API: Plan nutzt **3.x** (`origin/main` 3.7.4 = `ureq "3.3.0"`), **ohne** `json`-Feature → JSON via `serde_json` + `.send(bytes)`/`.into_body().read_to_string()`, Per-Request-Timeout via `.config().timeout_global(..).build()` (verifiziert gegen docs.rs/ureq 3.3 + Repo-Muster `cloud_client.rs`).

**Commit-Disziplin (§12):** Drei Commits gesamt — Spec-Sync (0.0), Phase 0 (0.4 Step 9), Phase 1 (1.5 Step 2). Finaler Squash erst beim PR-Merge nach `main`.
