# JetBrains Phase 5a — Härtung Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:
> executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **lean-ctx project rules (MANDATORY):**
> - **Rust (`*.rs`) edits → Serena symbolic tools only** (`mcp__serena__jet_brains_find_symbol`,
    > `replace_symbol_body`, `insert_after_symbol`/`insert_before_symbol`, `replace_content`) —
    > **never** native `Edit`/`ctx_edit` on Rust files.
> - **Tests: `cargo nextest run`** (never `cargo test`); Kotlin: `./gradlew test`.
> - **`ctx_shell`: bare command + `cwd=`** — never `cd <path> &&`; no `2>&1`; no `| tail`/`| grep` on test runners.
> - **Before `git add`:** `mcp__jetbrains__reformat_file` on every changed file.
> - **Deferred tool?** `ToolSearch(query="select:<tool>")` FIRST, never a Bash workaround.
> - One commit per task here; final squash to a single phase-commit is **optional** (§12.3 Eltern-Spec — Phase 4 kept
    per-task commits).

**Goal:** Härte die JetBrains-Backend-Integration Rust-seitig (Stale-Cache-Eviction, `project_root`-Kanonisierung,
`truncated`/`total`-Surfacing) und füge einen Plugin-CI-Job + Test-Hygiene hinzu — **keine** neuen Wire-Endpoints.

**Architecture:** Rein additive Härtung auf der bestehenden `LspBackend`-Trait-Architektur. Zwei neue
Default-Trait-Methoden (`is_stale`, `last_truncation`) lassen Backing A (`LspClient`) unberührt (Defaults), nur
`JetBrainsHttpBackend` überschreibt sie. Der Router evictet stale Cache-Einträge vor Nutzung; `ctx_refactor` hängt einen
Truncation-Hinweis an. CI läuft den bestehenden Gradle-Test-Stack headless.

**Tech Stack:** Rust (`lsp_types`, `ureq` 3.x, `serde_json`, `sha2`), Kotlin/IntelliJ-Platform-Plugin (IC 2026.1.3,
Gradle, JVM 21, `BasePlatformTestCase`), GitHub Actions.

**Spec:** `docs/lean-md/specs/2026-06-08-jetbrains-phase5a-hardening-design.md` (H1–H5).

---

## File Structure

| Datei                                                                        | Verantwortung                                                                                                               | Tasks   |
|------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------|---------|
| `rust/src/lsp/backend.rs`                                                    | `LspBackend`-Trait: +`is_stale` (H1) +`Truncation`/`last_truncation` (H3)                                                   | 1, 4    |
| `rust/src/lsp/jetbrains_backend.rs`                                          | B-Backend: pid/port-Felder + `is_stale`-Impl (H1), `project_root`-Kanonisierung (H2), `last_meta` + Truncation-Parsing (H3) | 1, 3, 4 |
| `rust/src/lsp/router.rs`                                                     | `with_backend`: Stale-Eviction vor Cache-Nutzung (H1) + `select_backend`-Call-Site (pid)                                    | 2       |
| `rust/src/tools/ctx_refactor.rs`                                             | Handler hängen Truncation-Hinweis an Output (H3)                                                                            | 4       |
| `.github/workflows/jetbrains-plugin.yml`                                     | Plugin-CI: build + test headless (H4)                                                                                       | 5       |
| `packages/jetbrains-lean-ctx/src/test/kotlin/.../PortFileTestEnv.kt` (o. ä.) | Test-Hygiene: `LEAN_CTX_DATA_DIR`→Temp + Cleanup-Assert (H5a)                                                               | 6       |

**Reihenfolge-Begründung:** H1 (Task 1+2) zuerst — Kern der Phase. H2 (Task 3) und H3 (Task 4) editieren beide
`jetbrains_backend.rs::new`/Methoden, daher nach H1. H4/H5 (Task 5+6) sind unabhängig (Infra/Kotlin).

---

## Task 1: H1 — `is_stale`-Trait-Methode + `JetBrainsHttpBackend`-Liveness-Selbstcheck

Fügt die Trait-Default-Methode `is_stale` hinzu (Backing A erbt `false`) und implementiert sie im B-Backend über einen
günstigen `pid`+Port-Datei-Vergleich (kein HTTP). Erweitert Struct + Konstruktor um `pid`/`port`.

**Files:**

- Modify: `rust/src/lsp/backend.rs:80-101` (Trait — neue Methode nach den Default-degrading-Methoden)
- Modify: `rust/src/lsp/jetbrains_backend.rs:16-30` (Struct + `new`), `:161-251` (Impl-Block — `is_stale` override)
- Modify: `rust/src/lsp/jetbrains_backend.rs:283,306,329` (3 bestehende Tests: `new`-Aufruf)
- Modify: `rust/src/lsp/router.rs:68-72` (`select_backend` Call-Site)
- Test: `rust/src/lsp/jetbrains_backend.rs` (tests-mod)

- [ ] **Step 1: Failing test für `is_stale` schreiben**

In `rust/src/lsp/jetbrains_backend.rs`, im `mod tests` (nach `symbols_overview_parses_wire_items`, vor der schließenden
`}` bei L337), via Serena `insert_after_symbol` (Anker: `symbols_overview_parses_wire_items`):

```rust
    #[test]
    fn is_stale_true_when_no_port_file() {
        // Unlikely root → no port file → cached backend is stale.
        let backend = JetBrainsHttpBackend::new(
            12345,
            "tok".to_string(),
            "/nonexistent/leanctx/proj/xyz".to_string(),
            999_999_999,
        );
        assert!(backend.is_stale("/nonexistent/leanctx/proj/xyz"));
    }

    #[test]
    fn is_stale_false_for_matching_live_pid() {
        // A port file describing THIS process (pid alive) + matching port/token
        // must be considered fresh. We stage a port file via the data-dir env.
        let tmp = std::env::temp_dir().join(format!("leanctx-stale-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_string_lossy().to_string();
        // Write a port file at the discovery path for `root`.
        std::env::set_var("LEAN_CTX_DATA_DIR", &tmp);
        let pf_path = crate::lsp::port_discovery::port_file_path(&root).unwrap();
        let pid = std::process::id();
        std::fs::write(
            &pf_path,
            format!(
                r#"{{"port":4567,"token":"tok","pid":{pid},"project_root":"{root}","ide_version":"x"}}"#
            ),
        )
        .unwrap();
        let backend = JetBrainsHttpBackend::new(4567, "tok".to_string(), root.clone(), pid);
        assert!(!backend.is_stale(&root), "matching live pid+port must be fresh");
        // Different cached pid → stale even though the file is live.
        let other = JetBrainsHttpBackend::new(4567, "tok".to_string(), root.clone(), pid + 1);
        assert!(other.is_stale(&root), "pid mismatch must be stale");
        std::env::remove_var("LEAN_CTX_DATA_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }
```

- [ ] **Step 2: Test ausführen — muss fehlschlagen (Kompilierfehler: `new` 4. Arg + `is_stale` fehlt)**

Run: `cargo nextest run -p lean-ctx is_stale` (cwd `rust`)
Expected: FAIL — `new` erwartet 3 Argumente / Methode `is_stale` nicht gefunden.

- [ ] **Step 3: Trait-Default `is_stale` ergänzen**

In `rust/src/lsp/backend.rs`, via Serena `replace_symbol_body` auf das Trait `LspBackend` ODER `insert`-Edit: direkt vor
der schließenden `}` des Traits (nach `inspections`, L100) einfügen:

```rust
    // ── Self-management (liveness) ──
    /// Whether a cached instance of this backend is no longer valid and must be
    /// evicted + re-selected. Backing A (in-process LSP) is never stale → default `false`.
    /// Backing B overrides: the IDE may have closed/restarted since caching.
    fn is_stale(&self, _project_root: &str) -> bool {
        false
    }
```

- [ ] **Step 4: Struct + `new` um `pid`/`port` erweitern**

In `rust/src/lsp/jetbrains_backend.rs`, via Serena `replace_symbol_body` auf `JetBrainsHttpBackend` (Struct) — Felder
ergänzen:

```rust
pub struct JetBrainsHttpBackend {
    base_url: String,
    token: String,
    /// Absolute project root, to rejoin project-relative wire paths.
    project_root: String,
    /// IDE process id from the discovered port file — for cheap staleness checks.
    pid: u32,
    /// IDE listen port — re-compared against the port file to detect restarts.
    port: u16,
}
```

Dann via Serena `replace_symbol_body` auf `new`:

```rust
    pub fn new(port: u16, token: String, project_root: String, pid: u32) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            token,
            project_root,
            pid,
            port,
        }
    }
```

- [ ] **Step 5: `is_stale`-Override im Impl-Block ergänzen**

In `rust/src/lsp/jetbrains_backend.rs`, via Serena `insert_after_symbol` (Anker: `rename` im
`impl LspBackend for JetBrainsHttpBackend`-Block, L242-250) — neue Methode innerhalb des Impl-Blocks:

```rust
    fn is_stale(&self, project_root: &str) -> bool {
        // Cheap re-check: port file gone, or pid/port changed (IDE restarted),
        // or our cached pid is dead → stale. NO HTTP (health is not pinged per call).
        match crate::lsp::port_discovery::read_port_file(project_root) {
            Some(pf) => {
                pf.pid != self.pid
                    || pf.port != self.port
                    || !crate::lsp::port_discovery::pid_alive(self.pid)
            }
            None => true,
        }
    }
```

- [ ] **Step 6: 3 bestehende Tests + Router-Call-Site an die neue `new`-Signatur anpassen**

In `rust/src/lsp/jetbrains_backend.rs` (tests-mod), via Serena `replace_content` die 3 Vorkommen anpassen — jeweils
`, std::process::id()` als 4. Argument:

```rust
        let mut backend =
            JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string(), std::process::id());
```

(in `references_parses_wire_locations`, `type_hierarchy_parses_wire_tree`, `symbols_overview_parses_wire_items`)

In `rust/src/lsp/router.rs`, via Serena `replace_content` die `select_backend`-Konstruktion (L68-72):

```rust
                return Ok(Box::new(JetBrainsHttpBackend::new(
                    pf.port,
                    pf.token,
                    project_root.to_string(),
                    pf.pid,
                )));
```

- [ ] **Step 7: Tests ausführen — müssen grün sein**

Run: `cargo nextest run -p lean-ctx jetbrains_backend` (cwd `rust`)
Expected: PASS — `is_stale_true_when_no_port_file`, `is_stale_false_for_matching_live_pid`, + die 3 Parser-Tests grün.

- [ ] **Step 8: clippy + fmt**

Run: `cargo clippy -p lean-ctx --all-targets` (cwd `rust`)
Expected: keine neuen Lints in `backend.rs`/`jetbrains_backend.rs`/`router.rs`.
Dann `mcp__jetbrains__reformat_file` auf die 3 geänderten `.rs`.

- [ ] **Step 9: Commit**

```bash
git add rust/src/lsp/backend.rs rust/src/lsp/jetbrains_backend.rs rust/src/lsp/router.rs
git commit -m "feat(lsp): is_stale trait method + JetBrains pid/port liveness self-check (H1)"
```

---

## Task 2: H1 — Router-Eviction stale Cache-Einträge

Verdrahtet `is_stale` in `with_backend`: ein gecachter Backend wird **vor** Nutzung auf Staleness geprüft und ggf.
evictet, sodass der bestehende `!contains_key`-Pfad ihn neu selektiert (auto → Fallback A; b_only → `Err`).

**Files:**

- Modify: `rust/src/lsp/router.rs:95-122` (`with_backend` + neuer Helper `evict_if_stale`)
- Test: `rust/src/lsp/router.rs` (tests-mod)

- [ ] **Step 1: Failing test für `evict_if_stale` schreiben**

In `rust/src/lsp/router.rs`, im `mod tests` via Serena `insert_after_symbol` (Anker: `no_port_file_means_no_backing_b`):

```rust
    struct StaleStub(bool);
    impl LspBackend for StaleStub {
        fn open_file(&mut self, _u: &Uri, _l: &str, _t: &str) -> Result<(), String> {
            Ok(())
        }
        fn references(
            &mut self,
            _u: &Uri,
            _p: lsp_types::Position,
            _s: &str,
        ) -> Result<Vec<lsp_types::Location>, String> {
            Ok(vec![])
        }
        fn definition(
            &mut self,
            _u: &Uri,
            _p: lsp_types::Position,
        ) -> Result<lsp_types::GotoDefinitionResponse, String> {
            Ok(lsp_types::GotoDefinitionResponse::Array(vec![]))
        }
        fn implementations(
            &mut self,
            _u: &Uri,
            _p: lsp_types::Position,
            _s: &str,
        ) -> Result<Vec<lsp_types::Location>, String> {
            Ok(vec![])
        }
        fn rename(
            &mut self,
            _u: &Uri,
            _p: lsp_types::Position,
            _n: &str,
        ) -> Result<Option<lsp_types::WorkspaceEdit>, String> {
            Ok(None)
        }
        fn is_stale(&self, _project_root: &str) -> bool {
            self.0
        }
    }

    #[test]
    fn evict_if_stale_removes_stale_keeps_fresh() {
        let mut map: HashMap<String, Box<dyn LspBackend>> = HashMap::new();
        map.insert("stale".to_string(), Box::new(StaleStub(true)));
        map.insert("fresh".to_string(), Box::new(StaleStub(false)));
        evict_if_stale(&mut map, "stale", "/any");
        evict_if_stale(&mut map, "fresh", "/any");
        assert!(!map.contains_key("stale"), "stale entry must be evicted");
        assert!(map.contains_key("fresh"), "fresh entry must remain");
    }
```

- [ ] **Step 2: Test ausführen — muss fehlschlagen (`evict_if_stale` nicht definiert)**

Run: `cargo nextest run -p lean-ctx evict_if_stale` (cwd `rust`)
Expected: FAIL — cannot find function `evict_if_stale`.

- [ ] **Step 3: `evict_if_stale`-Helper + Verdrahtung in `with_backend`**

In `rust/src/lsp/router.rs`, via Serena `insert_before_symbol` (Anker: `with_backend`) den Helper einfügen:

```rust
/// Evicts a cached backend whose liveness check (`is_stale`) failed, so the next
/// lookup re-selects (auto → Backing A fallback; b_only → Err). Backing A never stale.
fn evict_if_stale(
    backends: &mut HashMap<String, Box<dyn LspBackend>>,
    language: &str,
    project_root: &str,
) {
    if backends
        .get(language)
        .is_some_and(|b| b.is_stale(project_root))
    {
        backends.remove(language);
    }
}
```

Dann via Serena `replace_symbol_body` auf `with_backend` — den Cache-Block (L110-119) um den Eviction-Call ergänzen:

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

    // Drop a cached entry whose IDE went away / restarted before reusing it.
    evict_if_stale(&mut backends, language, project_root);

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

- [ ] **Step 4: Tests ausführen — müssen grün sein**

Run: `cargo nextest run -p lean-ctx evict_if_stale` (cwd `rust`)
Expected: PASS.

- [ ] **Step 5: clippy + fmt + Commit**

Run: `cargo clippy -p lean-ctx --all-targets` (cwd `rust`) → keine neuen Lints.
`mcp__jetbrains__reformat_file` auf `rust/src/lsp/router.rs`.

```bash
git add rust/src/lsp/router.rs
git commit -m "feat(lsp): evict stale Backing-B cache entries before reuse (H1)"
```

---

## Task 3: H2 — `project_root`-Kanonisierung im B-Backend (§5.5-Trap)

Kanonisiert `project_root` **einmalig** in `new` (realpath via `std::fs::canonicalize`, identisch zur `project_hash`
-Ableitung in `port_discovery.rs:29`) + Trailing-`/`-Trim, sodass `position_body`-`strip_prefix` und `rel_to_uri`
byte-identisch zur Kotlin-Seite arbeiten. Fehler-Guard: nicht-existenter Pfad → Roh-Root (erhält bestehende `/proj`
-Tests).

**Files:**

- Modify: `rust/src/lsp/jetbrains_backend.rs` (`new` — Kanonisierung; neuer privater Helper `canonical_root`)
- Test: `rust/src/lsp/jetbrains_backend.rs` (tests-mod)

- [ ] **Step 1: Failing test schreiben**

In `rust/src/lsp/jetbrains_backend.rs` (tests-mod) via Serena `insert_after_symbol` (Anker:
`is_stale_false_for_matching_live_pid`):

```rust
    #[test]
    fn canonical_root_strips_trailing_slash_and_resolves_realpath() {
        // Existing dir with a trailing slash → canonical form has no trailing slash
        // and matches sha2's canonicalize (port_discovery::project_hash parity).
        let tmp = std::env::temp_dir();
        let with_slash = format!("{}/", tmp.to_string_lossy());
        let backend =
            JetBrainsHttpBackend::new(1, "t".to_string(), with_slash.clone(), std::process::id());
        let expected = std::fs::canonicalize(&tmp).unwrap().to_string_lossy().to_string();
        assert_eq!(backend.project_root_for_test(), expected);
        assert!(!backend.project_root_for_test().ends_with('/'));
    }

    #[test]
    fn canonical_root_falls_back_to_raw_for_nonexistent() {
        let raw = "/nonexistent/leanctx/xyz";
        let backend =
            JetBrainsHttpBackend::new(1, "t".to_string(), raw.to_string(), std::process::id());
        assert_eq!(backend.project_root_for_test(), raw);
    }
```

Außerdem im `impl JetBrainsHttpBackend` (nicht-Trait-Block) via Serena `insert_after_symbol` (Anker: `new`) einen
Test-Accessor ergänzen:

```rust
    #[cfg(test)]
    fn project_root_for_test(&self) -> &str {
        &self.project_root
    }
```

- [ ] **Step 2: Test ausführen — muss fehlschlagen**

Run: `cargo nextest run -p lean-ctx canonical_root` (cwd `rust`)
Expected: FAIL — Trailing-Slash bleibt (keine Kanonisierung) bzw. realpath weicht ab.

- [ ] **Step 3: Kanonisierung in `new` + Helper**

In `rust/src/lsp/jetbrains_backend.rs` via Serena `insert_before_symbol` (Anker: `new`) den Helper in den
`impl JetBrainsHttpBackend`-Block:

```rust
    /// Canonicalize the project root ONCE so project-relative wire paths rejoin
    /// byte-identically with the Kotlin side (port-file key = sha256(realpath)[..16]).
    /// Mirrors `port_discovery::project_hash` canonicalization. On error (e.g. path
    /// does not exist), fall back to the raw root with a trailing-slash trim.
    fn canonical_root(project_root: &str) -> String {
        let canonical = std::fs::canonicalize(project_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| project_root.to_string());
        canonical.strip_suffix('/').unwrap_or(&canonical).to_string()
    }
```

Dann via Serena `replace_symbol_body` auf `new` — `project_root` durch den kanonisierten Wert ersetzen:

```rust
    pub fn new(port: u16, token: String, project_root: String, pid: u32) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            token,
            project_root: Self::canonical_root(&project_root),
            pid,
            port,
        }
    }
```

- [ ] **Step 4: Tests ausführen — müssen grün sein (inkl. der bestehenden Parser-Tests mit `/proj`)**

Run: `cargo nextest run -p lean-ctx jetbrains_backend` (cwd `rust`)
Expected: PASS — `canonical_root_*` grün; `references_parses_wire_locations` weiterhin grün (`/proj` existiert nicht →
Fallback Roh-Root → URI endet auf `/proj/src/main.rs`).

- [ ] **Step 5: clippy + fmt + Commit**

Run: `cargo clippy -p lean-ctx --all-targets` (cwd `rust`) → keine neuen Lints.
`mcp__jetbrains__reformat_file` auf `rust/src/lsp/jetbrains_backend.rs`.

```bash
git add rust/src/lsp/jetbrains_backend.rs
git commit -m "fix(lsp): canonicalize project_root in JetBrains backend (realpath parity, H2)"
```

---

## Task 4: H3 — `truncated`/`total` Rust-seitig surfacen

Ein `Truncation`-Typ + Trait-Default `last_truncation` (Backing A → `None`). `JetBrainsHttpBackend` merkt sich die Meta
des letzten Calls (`last_meta`) und `ctx_refactor` hängt bei `truncated=true` einen Hinweis an den Output. Capped Ops:
`references`, `implementations`, `type_hierarchy`, `symbols_overview`.

**Files:**

- Modify: `rust/src/lsp/backend.rs` (`Truncation`-Struct + Trait-Default `last_truncation`)
- Modify: `rust/src/lsp/jetbrains_backend.rs` (`last_meta`-Feld + `new` + `parse_truncation`-Helper + 4 Methoden +
  `last_truncation`-Override)
- Modify: `rust/src/tools/ctx_refactor.rs` (4 Handler + `truncation_note`-Helper)
- Test: `rust/src/lsp/jetbrains_backend.rs` + `rust/src/tools/ctx_refactor.rs`

- [ ] **Step 1: Failing test (Backend-Meta) schreiben**

In `rust/src/lsp/jetbrains_backend.rs` (tests-mod) via Serena `insert_after_symbol` (Anker:
`symbols_overview_parses_wire_items`):

```rust
    #[test]
    fn references_records_truncation_meta() {
        let body = r#"{"locations":[{"path":"a.rs","range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}}}],"truncated":true,"total":742}"#;
        let port = mock_once(body);
        let mut backend =
            JetBrainsHttpBackend::new(port, "tok".to_string(), "/proj".to_string(), std::process::id());
        let uri = file_path_to_uri("/proj/a.rs").unwrap();
        let _ = backend
            .references(&uri, Position { line: 0, character: 0 }, "project")
            .unwrap();
        let meta = backend.last_truncation().expect("meta recorded");
        assert!(meta.truncated);
        assert_eq!(meta.total, 742);
    }
```

- [ ] **Step 2: Test ausführen — muss fehlschlagen**

Run: `cargo nextest run -p lean-ctx references_records_truncation_meta` (cwd `rust`)
Expected: FAIL — `last_truncation` / `Truncation` nicht vorhanden.

- [ ] **Step 3: `Truncation` + Trait-Default in `backend.rs`**

In `rust/src/lsp/backend.rs` via Serena `insert_before_symbol` (Anker: Trait `LspBackend`) den Typ einfügen:

```rust
/// Truncation metadata for capped result sets (Backing B caps; spec Phase 3/4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Truncation {
    pub truncated: bool,
    /// Total available matches/items (≥ returned count when truncated).
    pub total: u32,
}
```

Dann via Serena `insert_after_symbol` (Anker: `is_stale` im Trait — die in Task 1 ergänzte Methode) den Default-Accessor
in den Trait:

```rust
    /// Truncation metadata of the most recent capped call, or `None` (Backing A,
    /// or no capped call yet). Lets `ctx_refactor` surface "(truncated …)".
    fn last_truncation(&self) -> Option<Truncation> {
        None
    }
```

- [ ] **Step 4: `last_meta`-Feld + Parsing im B-Backend**

In `rust/src/lsp/jetbrains_backend.rs`:

a) via Serena `replace_symbol_body` auf `JetBrainsHttpBackend` (Struct) — Feld ergänzen (nach `port`):

```rust
pub struct JetBrainsHttpBackend {
    base_url: String,
    token: String,
    /// Absolute project root, to rejoin project-relative wire paths.
    project_root: String,
    /// IDE process id from the discovered port file — for cheap staleness checks.
    pid: u32,
    /// IDE listen port — re-compared against the port file to detect restarts.
    port: u16,
    /// Truncation meta of the most recent capped call (references/implementations/
    /// type_hierarchy/symbols_overview), surfaced by ctx_refactor.
    last_meta: Option<crate::lsp::backend::Truncation>,
}
```

b) via Serena `replace_symbol_body` auf `new` — `last_meta: None` initialisieren:

```rust
    pub fn new(port: u16, token: String, project_root: String, pid: u32) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            token,
            project_root: Self::canonical_root(&project_root),
            pid,
            port,
            last_meta: None,
        }
    }
```

c) via Serena `insert_after_symbol` (Anker: `parse_symbols`) den Truncation-Parser in den `impl JetBrainsHttpBackend`
-Block:

```rust
    fn parse_truncation(v: &Value, shown: u32) -> Option<crate::lsp::backend::Truncation> {
        let truncated = v.get("truncated").and_then(Value::as_bool)?;
        let total = v
            .get("total")
            .and_then(Value::as_u64)
            .map_or(shown, |n| n as u32);
        Some(crate::lsp::backend::Truncation { truncated, total })
    }
```

d) via Serena `replace_symbol_body` auf die 4 Methoden — `last_meta` setzen. `references`:

```rust
    fn references(
        &mut self,
        uri: &Uri,
        position: Position,
        scope: &str,
    ) -> Result<Vec<Location>, String> {
        let mut body = self.position_body(uri, position);
        body["scope"] = serde_json::json!(scope);
        let resp = self.post("/references", &body)?;
        let locs = self.parse_locations(&resp);
        self.last_meta = Self::parse_truncation(&resp, locs.len() as u32);
        Ok(locs)
    }
```

`implementations`:

```rust
    fn implementations(
        &mut self,
        uri: &Uri,
        position: Position,
        scope: &str,
    ) -> Result<Vec<Location>, String> {
        let mut body = self.position_body(uri, position);
        body["scope"] = serde_json::json!(scope);
        let resp = self.post("/implementations", &body)?;
        let locs = self.parse_locations(&resp);
        self.last_meta = Self::parse_truncation(&resp, locs.len() as u32);
        Ok(locs)
    }
```

`type_hierarchy` (total nicht im Wire → `0`):

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
            return Err(err
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("INTERNAL")
                .to_string());
        }
        self.last_meta = Self::parse_truncation(&resp, 0);
        Ok(Self::parse_type_hierarchy(&resp))
    }
```

`symbols_overview`:

```rust
    fn symbols_overview(&mut self, uri: &Uri) -> Result<Vec<SymbolOverviewItem>, String> {
        let body = self.path_body(uri);
        let resp = self.post("/symbols_overview", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(err
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("INTERNAL")
                .to_string());
        }
        let items = Self::parse_symbols(&resp);
        self.last_meta = Self::parse_truncation(&resp, items.len() as u32);
        Ok(items)
    }
```

e) via Serena `insert_after_symbol` (Anker: `is_stale` im Impl-Block) den Override:

```rust
    fn last_truncation(&self) -> Option<crate::lsp::backend::Truncation> {
        self.last_meta
    }
```

- [ ] **Step 5: Backend-Test ausführen — muss grün sein**

Run: `cargo nextest run -p lean-ctx jetbrains_backend` (cwd `rust`)
Expected: PASS — `references_records_truncation_meta` grün; bestehende Tests grün (truncated false/absent → `last_meta`
Some/None, unkritisch).

- [ ] **Step 6: Failing test (ctx_refactor-Surfacing) schreiben**

In `rust/src/tools/ctx_refactor.rs` (tests-mod) via Serena `insert_after_symbol` (Anker:
`parse_direction_defaults_to_supertypes`) — Test über einen Stub mit `last_truncation`:

```rust
    #[test]
    fn references_output_surfaces_truncation_note() {
        struct TruncBackend;
        impl crate::lsp::backend::LspBackend for TruncBackend {
            fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> {
                Ok(())
            }
            fn references(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                let uri = crate::lsp::client::file_path_to_uri("/proj/a.rs").unwrap();
                Ok(vec![lsp_types::Location {
                    uri,
                    range: lsp_types::Range::default(),
                }])
            }
            fn definition(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
            ) -> Result<lsp_types::GotoDefinitionResponse, String> {
                Ok(lsp_types::GotoDefinitionResponse::Array(vec![]))
            }
            fn implementations(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
            }
            fn rename(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _n: &str,
            ) -> Result<Option<lsp_types::WorkspaceEdit>, String> {
                Ok(None)
            }
            fn last_truncation(&self) -> Option<crate::lsp::backend::Truncation> {
                Some(crate::lsp::backend::Truncation { truncated: true, total: 742 })
            }
        }
        crate::lsp::router::seed_stub_backend("rust", Box::new(TruncBackend));
        let uri = crate::lsp::client::file_path_to_uri("/proj/a.rs").unwrap();
        let out = handle_references(
            "/proj/a.rs",
            "/proj",
            &uri,
            Position { line: 0, character: 0 },
            "project",
        );
        assert!(out.contains("truncated"), "expected truncation note, got: {out}");
        assert!(out.contains("742"), "expected total in note, got: {out}");
    }
```

- [ ] **Step 7: Test ausführen — muss fehlschlagen (kein Hinweis im Output)**

Run: `cargo nextest run -p lean-ctx references_output_surfaces_truncation_note` (cwd `rust`)
Expected: FAIL — Output enthält keinen „truncated"-Hinweis.

- [ ] **Step 8: `truncation_note`-Helper + 4 Handler anpassen**

In `rust/src/tools/ctx_refactor.rs` via Serena `insert_before_symbol` (Anker: `format_type_hierarchy`) den Helper:

```rust
fn truncation_note(shown: usize, meta: Option<crate::lsp::backend::Truncation>) -> String {
    match meta {
        Some(m) if m.truncated => {
            format!("\n(truncated — showing {shown} of {})\n", m.total)
        }
        _ => String::new(),
    }
}
```

Dann via Serena `replace_symbol_body` die 4 Handler — Meta aus dem Backend ziehen + Hinweis anhängen.
`handle_references`:

```rust
fn handle_references(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
    scope: &str,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        let locs = backend.references(uri, position, scope)?;
        Ok((locs, backend.last_truncation()))
    });

    match result {
        Ok((locations, meta)) => {
            let mut out = format_locations(&locations, project_root);
            out.push_str(&truncation_note(locations.len(), meta));
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}
```

`handle_implementations`:

```rust
fn handle_implementations(
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
    scope: &str,
) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        let locs = backend.implementations(uri, position, scope)?;
        Ok((locs, backend.last_truncation()))
    });

    match result {
        Ok((locations, meta)) => {
            let mut out = format_locations(&locations, project_root);
            out.push_str(&truncation_note(locations.len(), meta));
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}
```

`handle_symbols_overview`:

```rust
fn handle_symbols_overview(file_path: &str, project_root: &str, uri: &lsp_types::Uri) -> String {
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        let items = backend.symbols_overview(uri)?;
        Ok((items, backend.last_truncation()))
    });
    match result {
        Ok((items, meta)) => {
            let mut out = format_symbols_overview(&items);
            out.push_str(&truncation_note(items.len(), meta));
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}
```

`handle_type_hierarchy` (Hierarchie hat kein `total` → schlichter `(truncated)`-Hinweis):

```rust
fn handle_type_hierarchy(
    args: &Value,
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
    position: Position,
) -> String {
    let direction = parse_direction(args);
    let result = crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
        let tree = backend.type_hierarchy(uri, position, direction)?;
        Ok((tree, backend.last_truncation()))
    });
    match result {
        Ok((tree, meta)) => {
            let mut out = format_type_hierarchy(&tree);
            if matches!(meta, Some(m) if m.truncated) {
                out.push_str("\n(truncated — depth/node cap reached)\n");
            }
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}
```

- [ ] **Step 9: Tests ausführen — müssen grün sein**

Run: `cargo nextest run -p lean-ctx ctx_refactor` (cwd `rust`)
Expected: PASS — `references_output_surfaces_truncation_note` grün; bestehende `type_hierarchy_formats_indented_tree` /
`unknown_action_help_lists_declaration` grün (HierBackend/StubBackend → `last_truncation()` Default `None` → kein
Hinweis).

- [ ] **Step 10: Voller Rust-Lauf + clippy + fmt + Commit**

Run: `cargo nextest run -p lean-ctx` (cwd `rust`)
Expected: PASS (Baseline: 2 vorbestehende, unabhängige `hn_hardening`-Shell-Compression-Fails dürfen bleiben — sonst
keine neuen Fails).
Run: `cargo clippy -p lean-ctx --all-targets` (cwd `rust`) → keine neuen Lints.
`mcp__jetbrains__reformat_file` auf `backend.rs`, `jetbrains_backend.rs`, `ctx_refactor.rs`.

```bash
git add rust/src/lsp/backend.rs rust/src/lsp/jetbrains_backend.rs rust/src/tools/ctx_refactor.rs
git commit -m "feat(refactor): surface truncated/total in ctx_refactor output (H3)"
```

---

## Task 5: H4 — Plugin-CI-Workflow

Eigener GitHub-Actions-Workflow (GitHub erkennt **nur** Dateien direkt in `.github/workflows/`, keine Unterordner) für
das JetBrains-Plugin: Build + headless `check` (= `test`). Orientiert am `slint-idea-plugin/build.yml`, angepasst auf *
*JVM 21** (Plugin-Target) und das **Unterverzeichnis** `packages/jetbrains-lean-ctx` (Gradle-Wrapper liegt dort).
Pfad-Filter, damit der Job nur bei Plugin-Änderungen läuft.

**Files:**

- Create: `.github/workflows/jetbrains-plugin.yml`

- [ ] **Step 1: Workflow-Datei anlegen**

Erstelle `.github/workflows/jetbrains-plugin.yml` (native `Write` — kein Rust):

```yaml
name: JetBrains Plugin

on:
  push:
    branches: [main]
    paths:
      - 'packages/jetbrains-lean-ctx/**'
      - '.github/workflows/jetbrains-plugin.yml'
  pull_request:
    branches: [main]
    paths:
      - 'packages/jetbrains-lean-ctx/**'
      - '.github/workflows/jetbrains-plugin.yml'

permissions:
  contents: read

defaults:
  run:
    working-directory: packages/jetbrains-lean-ctx

jobs:
  build:
    name: Build
    runs-on: ubuntu-latest
    steps:
      - name: Fetch Sources
        uses: actions/checkout@v4
        with:
          persist-credentials: false

      - name: Gradle Wrapper Validation
        uses: gradle/actions/wrapper-validation@v4

      - name: Setup Java
        uses: actions/setup-java@v4
        with:
          distribution: zulu
          java-version: 21

      - name: Setup Gradle
        uses: gradle/actions/setup-gradle@v4

      - name: Build plugin
        run: ./gradlew buildPlugin --console=plain

  test:
    name: Test
    needs: [build]
    runs-on: ubuntu-latest
    steps:
      - name: Fetch Sources
        uses: actions/checkout@v4
        with:
          persist-credentials: false

      - name: Setup Java
        uses: actions/setup-java@v4
        with:
          distribution: zulu
          java-version: 21

      - name: Setup Gradle
        uses: gradle/actions/setup-gradle@v4
        with:
          gradle-home-cache-cleanup: true

      - name: Run Tests
        run: ./gradlew check --console=plain

      - name: Collect Tests Result
        if: ${{ failure() }}
        uses: actions/upload-artifact@v4
        with:
          name: jetbrains-tests-result
          path: packages/jetbrains-lean-ctx/build/reports/tests
```

- [ ] **Step 2: YAML-Syntax lokal validieren**

Run: `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/jetbrains-plugin.yml')); print('ok')"` (cwd
Repo-Root)
Expected: `ok` (keine YAML-Fehler).

- [ ] **Step 3: Gradle-Tasks lokal verifizieren (headless, falls IC-Cache vorhanden)**

Run: `./gradlew check --console=plain --offline` (cwd `packages/jetbrains-lean-ctx`)
Expected: `BUILD SUCCESSFUL`, 54 Tests grün (entspricht dem aktuellen Stand). Falls `--offline` mangels Cache scheitert:
einmal online `./gradlew check --console=plain`.
_(Setzt Task 6 voraus, damit der Lauf keine Port-Dateien ins reale Data-Dir leakt — Reihenfolge bei der Ausführung: Task
6 vor diesem Verify-Step.)_

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/jetbrains-plugin.yml
git commit -m "ci(jetbrains): plugin build + headless test workflow (JVM 21, H4)"
```

---

## Task 6: H5a — Test-Hygiene: kein Port-Datei-Leak

`BasePlatformTestCase` bootet via `LeanCtxStartupActivity` einen echten `BackendHttpServer`, der eine Port-Datei ins
reale Data-Dir (`~/.lean-ctx` bzw. `LEAN_CTX_DATA_DIR`) schreibt. Test-Setup leitet das Data-Dir auf ein
Test-Temp-Verzeichnis um (`LEAN_CTX_DATA_DIR`) und prüft im Teardown, dass keine `jetbrains-*.port` zurückbleibt.

> **Hinweis:** Die konkrete Test-Basisklasse/Setup-Stelle in `packages/jetbrains-lean-ctx/src/test/kotlin/` ist beim
> Ausführen zu ermitteln (`ctx_search "BasePlatformTestCase" packages/jetbrains-lean-ctx/src/test`). `LeanCtxPaths` liest
`LEAN_CTX_DATA_DIR` (Phase-2-Parität). Setze die Env **vor** dem `super.setUp()`-Boot.

**Files:**

- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/PortFileHygieneTest.kt`
- (ggf.) Modify: vorhandene Test-Basisklasse, falls ein gemeinsamer `setUp` existiert

- [ ] **Step 1: Test-Setup + Hygiene-Assertion schreiben**

Ermittle zuerst die Data-Dir-Auflösung:

Run: `ctx_search "fun.*[dD]ataDir|LEAN_CTX_DATA_DIR" packages/jetbrains-lean-ctx/src/main/kotlin`
Expected: Fundstelle in `LeanCtxPaths` (Env-Lookup), die der Test übersteuert.

Erstelle `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/PortFileHygieneTest.kt` (native `Write` —
Kotlin, kein Serena-Zwang):

```kotlin
package com.leanctx.plugin

import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.nio.file.Files
import java.nio.file.Path

/**
 * Guards that a test run does not leak `jetbrains-*.port` files into the real
 * data dir: the startup activity boots a real BackendHttpServer (writes a port
 * file). We redirect LEAN_CTX_DATA_DIR to a temp dir and assert cleanup.
 */
class PortFileHygieneTest : BasePlatformTestCase() {

    private lateinit var tempDataDir: Path
    private var prevEnv: String? = null

    override fun setUp() {
        tempDataDir = Files.createTempDirectory("leanctx-test-datadir")
        // Route the plugin's data-dir resolution to the temp dir for this run.
        prevEnv = System.getProperty("LEAN_CTX_DATA_DIR")
        System.setProperty("LEAN_CTX_DATA_DIR", tempDataDir.toString())
        super.setUp()
    }

    override fun tearDown() {
        try {
            super.tearDown()
        } finally {
            if (prevEnv != null) {
                System.setProperty("LEAN_CTX_DATA_DIR", prevEnv!!)
            } else {
                System.clearProperty("LEAN_CTX_DATA_DIR")
            }
        }
    }

    fun testNoPortFileLeftInRealDataDir() {
        val realHome = System.getProperty("user.home")
        val leanCtxDir = Path.of(realHome, ".lean-ctx")
        val leaked = if (Files.isDirectory(leanCtxDir)) {
            Files.list(leanCtxDir).use { stream ->
                stream.filter { it.fileName.toString().startsWith("jetbrains-") &&
                    it.fileName.toString().endsWith(".port") }
                    .anyMatch { it.fileName.toString().contains("unitTest") ||
                        it.fileName.toString().contains("/tmp") }
            }
        } else {
            false
        }
        assertFalse("test run leaked a port file into ~/.lean-ctx", leaked)
        // The temp data dir may hold a transient port file during the run; the
        // server lifecycle (dispose) must remove it by teardown.
        val tempLeaks = Files.list(tempDataDir).use { s -> s.count() }
        assertEquals("temp data dir must be clean after teardown of fixtures", 0L, tempLeaks)
    }
}
```

> **Falls `LeanCtxPaths` Umgebungs-Variablen statt System-Properties liest** (`System.getenv("LEAN_CTX_DATA_DIR")`): der
> Test kann `getenv` nicht setzen. Dann ist die korrekte Maßnahme, `LeanCtxPaths` test-freundlich zu machen —
`System.getProperty("LEAN_CTX_DATA_DIR") ?: System.getenv("LEAN_CTX_DATA_DIR")` (Property-Override vor Env). Diese
> kleine Anpassung in `LeanCtxPaths` (main) ist Teil dieses Tasks, falls nötig; sie ändert das Produktivverhalten nicht (
> Property normalerweise ungesetzt).

- [ ] **Step 2: Test ausführen — etabliert die Hygiene-Erwartung**

Run: `./gradlew test --tests "com.leanctx.plugin.PortFileHygieneTest" --console=plain` (cwd
`packages/jetbrains-lean-ctx`)
Expected: Falls der Leak existiert → FAIL (Port-Datei im realen `~/.lean-ctx` bzw. Temp-Dir nicht leer). Das ist der
rote TDD-Zustand.

- [ ] **Step 3: Fix — Data-Dir-Override + Dispose-Cleanup sicherstellen**

Falls Step 2 fehlschlägt, weil `LeanCtxPaths` `getenv` statt Property liest: in
`packages/jetbrains-lean-ctx/src/main/kotlin/.../LeanCtxPaths.kt` die Auflösung um den Property-Override erweitern (
native Kotlin-Edit):

```kotlin
// Resolve data dir: test-overridable system property first, then env, then default.
private fun dataDirEnv(): String? =
    System.getProperty("LEAN_CTX_DATA_DIR") ?: System.getenv("LEAN_CTX_DATA_DIR")
```

(an der Stelle einsetzen, wo bisher `System.getenv("LEAN_CTX_DATA_DIR")` direkt gelesen wird).

Stelle außerdem sicher, dass die Test-Fixture die `BackendHttpServer`-Disposable korrekt schließt (Server `dispose()`
löscht die Port-Datei — bereits implementiert in `BackendHttpServer.dispose`, Commit `d2fd93f9`). Der
Temp-Data-Dir-Override genügt i. d. R.

- [ ] **Step 4: Test ausführen — muss grün sein**

Run: `./gradlew test --tests "com.leanctx.plugin.PortFileHygieneTest" --console=plain` (cwd
`packages/jetbrains-lean-ctx`)
Expected: PASS — kein Leak im realen Data-Dir, Temp-Dir nach Teardown leer.

- [ ] **Step 5: Voller Kotlin-Lauf — keine Regression**

Run: `./gradlew test --console=plain` (cwd `packages/jetbrains-lean-ctx`)
Expected: PASS — 55 Tests grün (54 bestehende + neuer Hygiene-Test).

- [ ] **Step 6: reformat + Commit**

`mcp__jetbrains__reformat_file` auf die neue Test-Datei (+ ggf. `LeanCtxPaths.kt`).

```bash
git add packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/PortFileHygieneTest.kt
# falls LeanCtxPaths angepasst:
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxPaths.kt
git commit -m "test(plugin): port-file hygiene — no leak into real data dir (H5a)"
```

---

## Task 7: Final-Gate + Spec-Gate-Protokoll

Verifiziert alle automatisierten Gates zusammen und füllt §9 des Specs (Gate-Protokoll). Das manuelle `runIde`-Gate (IDE
auf → B gecacht → IDE zu → sauberer A-Fallback) ist **User-gated** (separates Terminal, ~1 GB IC) und wird im Protokoll
als PENDING markiert, falls nicht live ausgeführt.

**Files:**

- Modify: `docs/lean-md/specs/2026-06-08-jetbrains-phase5a-hardening-design.md` (§9)

- [ ] **Step 1: Voller Rust-Gate**

Run: `cargo nextest run -p lean-ctx` (cwd `rust`)
Expected: PASS — neue Tests grün; Baseline (2 `hn_hardening`-Fails) unverändert, keine neuen Fails.
Run: `cargo clippy -p lean-ctx --all-targets` (cwd `rust`)
Expected: keine neuen Lints in den vier geänderten Rust-Dateien.

- [ ] **Step 2: Voller Kotlin-Gate**

Run: `./gradlew check --console=plain` (cwd `packages/jetbrains-lean-ctx`)
Expected: `BUILD SUCCESSFUL`, 55 Tests grün.

- [ ] **Step 3: Drift-Gate (Doku-Generator)**

Da `ctx_refactor` keine neuen Actions/Schema bekommt (H3 ändert nur Output-Text), darf das generierte Tool-Doc
unverändert bleiben.
Run: `ctx_read docs/reference/generated/mcp-tools.md mode=signatures` und prüfe, dass `ctx_refactor`-Schema (Actions)
unverändert ist. Falls ein Drift-Test existiert: `cargo nextest run -p lean-ctx drift` → grün.

- [ ] **Step 4: §9-Gate-Protokoll ins Spec schreiben**

In `docs/lean-md/specs/2026-06-08-jetbrains-phase5a-hardening-design.md`, §9 ersetzen (native Markdown-Edit) — mit den
realen Commit-Hashes (H1a, H1b, H2, H3, H4, H5a), Gate-Ergebnissen (Rust nextest N passed, Kotlin 55 grün, clippy
clean), und dem manuellen `runIde`-Status (live verifiziert ODER PENDING/User-gated).

- [ ] **Step 5: Memory + Commit**

`ctx_knowledge action=remember category=decision` — Phase-5a-Abschluss (Commits, Gates, offene runIde-/E2E-Punkte).

```bash
git add docs/lean-md/specs/2026-06-08-jetbrains-phase5a-hardening-design.md
git commit -m "docs(spec): Phase-5a Gate-Protokoll — Härtung H1–H5a results"
```

---

## Self-Review-Notiz (für den Ausführenden)

- **Spec-Coverage:** H1→Task 1+2, H2→Task 3, H3→Task 4, H4→Task 5, H5a→Task 6, H5b→nur Doku (Spec §6, kein Task —
  korrekt). Gate §5→Task 7. Keine neuen Wire-Endpoints (Spec §4) — kein Task, korrekt.
- **Typ-Konsistenz:** `Truncation { truncated: bool, total: u32 }` (backend.rs) einheitlich in `jetbrains_backend.rs` (
  `last_meta`, `parse_truncation`) und `ctx_refactor.rs` (`truncation_note`, Stub).
  `is_stale(&self, project_root: &str) -> bool` und `last_truncation(&self) -> Option<Truncation>` als Trait-Defaults —
  Backing A (`LspClient`) braucht **keine** Änderung.
- **`new`-Signatur:** überall `new(port, token, project_root, pid)` — Call-Sites: `router.rs` (select_backend) + 3
  Backend-Tests + neue Tests konsistent angepasst.
- **Rust-Edits ausschließlich via Serena**; Kotlin/YAML via native `Write`/Edit. `cargo nextest`, nie `cargo test`.
  `ctx_shell` bare + `cwd=`.
