# lean-ctx JetBrains v2b — Refactoring-Engine + `rename` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Etabliere die Multi-File-Refactoring-Engine über das JetBrains-Plugin + lean-ctx und implementiere exemplarisch die `rename`-Op als Two-Phase-Protokoll (`rename_preview` + `rename_apply`).

**Architecture:** Zwei neue Actions in `ctx_refactor`. `rename_preview` resolved das Ziel-Symbol (reuse v2a `resolve_name_path`), verlangt eine **laufende** JetBrains-IDE (Backing B, kein Headless-Fallback → `BACKEND_REQUIRED`), holt vom Plugin alle semantischen Usages + Konflikte (`RenameProcessor`), bildet einen stateless `plan_hash` (BLAKE3) und liefert einen Plan. `rename_apply` wiederholt die Usage-Suche, prüft `plan_hash` (TOCTOU) + Konflikt-Gate in **Rust**, und lässt das Plugin die Multi-File-Transaktion als **einen** Undo-Eintrag ausführen.

**Tech Stack:** Rust (`ctx_refactor.rs`, `lsp/backend.rs`, `lsp/jetbrains_backend.rs`, BLAKE3 via `core::hasher`), Kotlin/IntelliJ-Plugin (`RefactoringFactory`/`RenameProcessor`, `WriteCommandAction`, gson), HTTP/JSON-Wire (127.0.0.1, Token-Header).

**Spec:** `docs/lean-md/specs/2026-06-09-leanctx-jetbrains-v2b-refactoring-rename-design.md`

---

## Wichtige Grundlagen (vor dem Start lesen)

Der Implementer kennt das Projekt nicht — diese Fakten sind verbindlich:

1. **Tests:** immer `cargo nextest run` (nie `cargo test`); Kotlin: `./gradlew test` im Verzeichnis `packages/jetbrains-lean-ctx`. Beide bare command + `cwd=`, kein `cd … &&`, kein `| tail`/`| grep`.
2. **Rust-Edits an `*.rs`:** Serena-Tools (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_after_symbol`, `insert_before_symbol`, `replace_content`) — **nie** native `Edit`/`ctx_edit` auf Rust-Dateien. Kotlin/Markdown: native `Edit`/`Write` ok.
3. **Vor `git add`:** `mcp__jetbrains__reformat_file` auf jede geänderte Datei.
4. **Fehler-Konvention (Rust):** Der Trait `LspBackend` gibt `Result<_, String>` zurück. Fachliche Fehler sind Strings im Format `"CODE: message"` (z.B. `"CONFLICT: …"`, `"BACKEND_REQUIRED: …"`, `"FILE_NOT_FOUND: …"`). **Es gibt KEINEN `BackendError`-Enum** — der Pseudocode in Spec §5.3 (`Err(BackendError::BackendRequired)`) ist illustrativ; real wird `Err("BACKEND_REQUIRED: …".to_string())` verwendet. `ctx_refactor`-Handler mappen den String zu `format!("ERROR: {e}")`.
5. **0-/1-Basierung:** Tool-Eingabe ist 1-basiert (`line`). Die Wire ist **0-basiert** (`PositionDTO`, `TextRange0Based`). `resolve_name_path` liefert 1-basiert inklusiv. Umrechnung `start_line - 1` beim Bau der `TextRange0Based`.
6. **Ein Commit pro Phase** (Spec §12, v1-§12.3). Direkt auf Branch `feat-jetbrains-plugin`, **kein** worktree.
7. **Reuse v2a (nicht neu bauen):** `resolve_name_path` (`ctx_refactor.rs`), `offset_of` (`lsp/edit_apply.rs`), `hash_hex` (`core/hasher.rs`), PathJail `core::path_resolve::resolve_tool_path`, Cache-Evict `core::cli_cache::invalidate`, Liveness `port_discovery::{read_port_file, pid_alive, health_ok}`.

---

## File Structure

**Rust (`rust/src/`):**
- `lsp/backend.rs` — MODIFY: +6 Typen (`RenameQuery`, `UsageSite`, `Conflict`, `RenamePlan`, `RenameApply`, `RenameResult`) + 2 Trait-Methoden mit `Err`-Default.
- `lsp/jetbrains_backend.rs` — MODIFY: HTTP-Override der 2 Methoden (`/renamePreview`, `/renameApply`) + Wire-Parsing.
- `tools/ctx_refactor.rs` — MODIFY: Action-Dispatch, `handle_rename_refactor`, `plan_hash`, `usage_range_text`, `live_jetbrains_backend`, Konflikt-Gate, Cache-Evict, Diff.
- `tools/registered/ctx_refactor.rs` — MODIFY: Schema (`+2` Actions, `plan_hash`/`force`/`search_comments`/`search_text_occurrences`).

**Kotlin (`packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/`):**
- `dto/Wire.kt` — MODIFY: +6 DTOs + JsonCodec-Parser.
- `psi/SymbolRefactorer.kt` — CREATE: `RenameProcessor`-Naht (Preview + Apply).
- `endpoint/RefactorHandlers.kt` — CREATE: `renamePreview`/`renameApply` (off-EDT preview, EDT apply).
- `server/RequestRouter.kt` — MODIFY: 2 Routen + Dispatch.

**Tests:**
- Rust-Unit: inline `#[cfg(test)]` in den jeweiligen Dateien.
- Kotlin: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterRefactorTest.kt` — CREATE.

**Docs:**
- `docs/reference/generated/mcp-tools.md` — regeneriert (Drift-Test).
- `docs/reference/appendix-mcp-tools.md` — MODIFY: `ctx_refactor`-Zeile.

---

# PHASE 1 — Rust Backend-Typen + Trait-Methoden

Commit-Message am Phasenende: `feat(jetbrains): v2b rename trait — Rename* types + Err-default methods`

### Task 1: Neue Backend-Typen in `backend.rs`

**Files:**
- Modify: `rust/src/lsp/backend.rs` (nach dem `EditResult`-Struct, vor `pub trait LspBackend`)
- Test: `rust/src/lsp/backend.rs` (inline `#[cfg(test)]` am Dateiende — **neu anlegen**, die Datei hat noch keinen Test-Modul)

- [ ] **Step 1: Failing test schreiben**

Serena `insert_after_symbol` am letzten Item der Datei (dem `LspBackend`-Trait) — füge am **Dateiende** an:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_types_construct_and_clone() {
        let q = RenameQuery {
            abs_path: "/proj/a.rs".into(),
            rel_path: "a.rs".into(),
            target_range: TextRange0Based { start_line: 0, start_char: 0, end_line: 0, end_char: 3 },
            new_name: "bar".into(),
            search_comments: false,
            search_text_occurrences: false,
        };
        let q2 = q.clone();
        assert_eq!(q2.new_name, "bar");

        let plan = RenamePlan {
            usages: vec![UsageSite {
                path: "a.rs".into(),
                range: TextRange0Based { start_line: 1, start_char: 4, end_line: 1, end_char: 7 },
                context: Some("foo()".into()),
            }],
            conflicts: vec![Conflict {
                path: "a.rs".into(),
                range: None,
                message: "name already exists".into(),
            }],
        };
        assert_eq!(plan.usages.len(), 1);
        assert_eq!(plan.conflicts[0].message, "name already exists");

        let apply = RenameApply {
            abs_path: "/proj/a.rs".into(),
            rel_path: "a.rs".into(),
            target_range: q.target_range,
            new_name: "bar".into(),
            force: true,
        };
        let res = RenameResult { applied: true, changed_paths: vec!["a.rs".into()] };
        assert!(apply.force);
        assert!(res.applied);
    }
}
```

- [ ] **Step 2: Test ausführen, fail bestätigen**

Run: `cargo nextest run -p lean-ctx rename_types_construct_and_clone` (cwd=`rust`)
Expected: COMPILE FAIL — `cannot find type RenameQuery`.

- [ ] **Step 3: Typen implementieren**

Serena `insert_before_symbol` vor `pub trait LspBackend` in `backend.rs` (direkt nach dem `EditResult`-Struct, das bei `/// Compact human-readable diff` endet):

```rust
/// Query for `rename_preview`: the target symbol is already resolved (name_path →
/// range) in `ctx_refactor`; the backend only ever sees an absolute + relative
/// path and a range, exactly like `RangeEdit` (no `name_path` on the wire).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameQuery {
    /// Absolute, jail-checked path of the file containing the target symbol.
    pub abs_path: String,
    /// Project-relative path (wire body sent to Backing B).
    pub rel_path: String,
    /// Declaration span of the target symbol (start is what the IDE resolves from).
    pub target_range: TextRange0Based,
    pub new_name: String,
    /// Also rename matches inside comments/strings (RenameProcessor flag).
    pub search_comments: bool,
    /// Also rename non-code text occurrences (RenameProcessor flag).
    pub search_text_occurrences: bool,
}

/// A single semantic usage of the target symbol (declaration or reference),
/// returned by Backing B's `RenameProcessor.findUsages`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageSite {
    /// Project-relative path of the file holding this usage.
    pub path: String,
    /// 0-based range of the renamed identifier at this site.
    pub range: TextRange0Based,
    /// Optional one-line context snippet (display only; NOT part of plan_hash).
    pub context: Option<String>,
}

/// A refactoring conflict surfaced by `RenameProcessor.preprocessUsages`
/// (name collision, visibility loss, override clash). `range` is optional —
/// some conflicts are scope-level, not tied to a single offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub path: String,
    pub range: Option<TextRange0Based>,
    pub message: String,
}

/// Outcome of `rename_preview`: every usage + every conflict. The `plan_hash`
/// is built in Rust from this (see `ctx_refactor::plan_hash`), never here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePlan {
    pub usages: Vec<UsageSite>,
    pub conflicts: Vec<Conflict>,
}

/// Apply request: same target addressing as `RenameQuery` plus the `force`
/// flag (passed through to `RenameProcessor`; Rust has already gated conflicts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameApply {
    pub abs_path: String,
    pub rel_path: String,
    pub target_range: TextRange0Based,
    pub new_name: String,
    pub force: bool,
}

/// Outcome of `rename_apply`: which files the IDE actually changed (no per-file
/// bodies — Multi-File would be too large; Rust re-reads via mtime validation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameResult {
    pub applied: bool,
    pub changed_paths: Vec<String>,
}
```

- [ ] **Step 4: Test ausführen, pass bestätigen**

Run: `cargo nextest run -p lean-ctx rename_types_construct_and_clone` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 5: Trait-Methoden mit `Err`-Default ergänzen**

Serena `insert_after_symbol` an der Methode `insert_after_symbol` im `LspBackend`-Trait (die letzte der drei `*_symbol`-Default-Methoden, endet mit `crate::lsp::edit_apply::local_range_write(edit)` `}`). Füge **innerhalb des Traits** direkt danach an:

```rust
    /// Phase 1 of the Two-Phase rename: resolve all usages + conflicts of the
    /// target symbol. DEFAULT = `Err(BACKEND_REQUIRED)` — there is NO lossless
    /// headless usage search (spec §3); only Backing B (live IDE) overrides this.
    fn rename_preview(&mut self, _req: &RenameQuery) -> Result<RenamePlan, String> {
        Err("BACKEND_REQUIRED: rename requires a running JetBrains IDE".to_string())
    }
    /// Phase 2 of the Two-Phase rename: perform the Multi-File rename as ONE
    /// transaction (one Undo entry). DEFAULT = `Err(BACKEND_REQUIRED)`.
    fn rename_apply(&mut self, _req: &RenameApply) -> Result<RenameResult, String> {
        Err("BACKEND_REQUIRED: rename requires a running JetBrains IDE".to_string())
    }
```

Bring die neuen Typen in den `use`-Scope der `backend.rs`-Konsumenten nicht extra rein — sie sind `pub` im selben Modul.

- [ ] **Step 6: Failing test für den Default schreiben** (im selben `mod tests`)

Serena `insert_after_symbol` an `rename_types_construct_and_clone`:

```rust
    #[test]
    fn headless_rename_default_is_backend_required() {
        // HeadlessBackend inherits the Trait default → BACKEND_REQUIRED, no apply.
        let mut be = crate::lsp::edit_apply::HeadlessBackend;
        let q = RenameQuery {
            abs_path: "/x".into(), rel_path: "x".into(),
            target_range: TextRange0Based { start_line: 0, start_char: 0, end_line: 0, end_char: 1 },
            new_name: "y".into(), search_comments: false, search_text_occurrences: false,
        };
        let err = be.rename_preview(&q).unwrap_err();
        assert!(err.starts_with("BACKEND_REQUIRED"), "got: {err}");
        let a = RenameApply {
            abs_path: "/x".into(), rel_path: "x".into(),
            target_range: q.target_range, new_name: "y".into(), force: false,
        };
        assert!(be.rename_apply(&a).unwrap_err().starts_with("BACKEND_REQUIRED"));
    }
```

- [ ] **Step 7: Test ausführen, pass bestätigen**

Run: `cargo nextest run -p lean-ctx -E 'test(rename_types_construct_and_clone) + test(headless_rename_default_is_backend_required)'` (cwd=`rust`)
Expected: beide PASS.

- [ ] **Step 8: Reformat + Commit (Phase 1)**

```bash
# reformat via mcp__jetbrains__reformat_file auf rust/src/lsp/backend.rs
git add rust/src/lsp/backend.rs
git commit -m "feat(jetbrains): v2b rename trait — Rename* types + Err-default methods"
```

---

# PHASE 2 — Rust JetBrains-HTTP-Override

Commit-Message: `feat(jetbrains): v2b rename — JetBrainsHttpBackend /renamePreview /renameApply`

### Task 2: HTTP-Override + Wire-Parsing in `jetbrains_backend.rs`

**Files:**
- Modify: `rust/src/lsp/jetbrains_backend.rs`
- Test: `rust/src/lsp/jetbrains_backend.rs` (inline `mod tests` — nutzt die vorhandene `mock_once`-Helper)

- [ ] **Step 1: Failing tests schreiben**

Serena `insert_after_symbol` am letzten Test in `mod tests` (`canonical_root_falls_back_to_raw_for_nonexistent`):

```rust
    #[test]
    fn rename_preview_parses_usages_and_conflicts() {
        let body = r#"{"usages":[
            {"path":"src/a.rs","range":{"start":{"line":5,"character":4},"end":{"line":5,"character":7}},"context":"foo()"},
            {"path":"src/b.rs","range":{"start":{"line":1,"character":0},"end":{"line":1,"character":3}}}
          ],"conflicts":[
            {"path":"src/a.rs","range":{"start":{"line":9,"character":0},"end":{"line":9,"character":3}},"message":"name clash"}
          ]}"#;
        let port = mock_once(body);
        let mut be = JetBrainsHttpBackend::new(port, "tok".into(), "/proj".to_string(), 1234);
        let q = crate::lsp::backend::RenameQuery {
            abs_path: "/proj/src/a.rs".into(), rel_path: "src/a.rs".into(),
            target_range: crate::lsp::backend::TextRange0Based { start_line: 5, start_char: 4, end_line: 5, end_char: 7 },
            new_name: "bar".into(), search_comments: false, search_text_occurrences: false,
        };
        let plan = be.rename_preview(&q).unwrap();
        assert_eq!(plan.usages.len(), 2);
        assert_eq!(plan.usages[0].path, "src/a.rs");
        assert_eq!(plan.usages[0].context.as_deref(), Some("foo()"));
        assert_eq!(plan.usages[1].context, None);
        assert_eq!(plan.conflicts.len(), 1);
        assert_eq!(plan.conflicts[0].message, "name clash");
    }

    #[test]
    fn rename_preview_maps_error_envelope() {
        let port = mock_once(r#"{"error":{"code":"INDEXING","message":"busy"}}"#);
        let mut be = JetBrainsHttpBackend::new(port, "tok".into(), "/proj".to_string(), 1234);
        let q = crate::lsp::backend::RenameQuery {
            abs_path: "/proj/a.rs".into(), rel_path: "a.rs".into(),
            target_range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 0, end_line: 0, end_char: 1 },
            new_name: "y".into(), search_comments: false, search_text_occurrences: false,
        };
        assert_eq!(be.rename_preview(&q).unwrap_err(), "INDEXING");
    }

    #[test]
    fn rename_apply_parses_changed_paths() {
        let body = r#"{"applied":true,"changed_paths":["src/a.rs","src/b.rs"]}"#;
        let port = mock_once(body);
        let mut be = JetBrainsHttpBackend::new(port, "tok".into(), "/proj".to_string(), 1234);
        let a = crate::lsp::backend::RenameApply {
            abs_path: "/proj/src/a.rs".into(), rel_path: "src/a.rs".into(),
            target_range: crate::lsp::backend::TextRange0Based { start_line: 5, start_char: 4, end_line: 5, end_char: 7 },
            new_name: "bar".into(), force: false,
        };
        let res = be.rename_apply(&a).unwrap();
        assert!(res.applied);
        assert_eq!(res.changed_paths, vec!["src/a.rs", "src/b.rs"]);
    }
```

- [ ] **Step 2: Test ausführen, fail bestätigen**

Run: `cargo nextest run -p lean-ctx rename_preview_parses_usages_and_conflicts` (cwd=`rust`)
Expected: COMPILE FAIL — `no method rename_preview` für `JetBrainsHttpBackend` (Default existiert, aber Parsing-Helper fehlen → tatsächlich läuft der Default-`Err`; der Test will den Override). Hinweis: ohne Override liefert der Trait-Default `BACKEND_REQUIRED` → Test schlägt mit Assertion fehl. Beides = roter Test, ok.

- [ ] **Step 3: Parsing-Helper + Body-Builder implementieren**

Serena `insert_after_symbol` an der Methode `post_edit` (impl-Block von `JetBrainsHttpBackend`, endet mit `Ok(Self::parse_edit_result(&resp, &edit.text))` `}`):

```rust
    /// Parse a `{start,end}` range object into `TextRange0Based`.
    fn parse_range0(v: &Value) -> Option<crate::lsp::backend::TextRange0Based> {
        let start = Self::parse_position(v.get("start")?)?;
        let end = Self::parse_position(v.get("end")?)?;
        Some(crate::lsp::backend::TextRange0Based {
            start_line: start.line, start_char: start.character,
            end_line: end.line, end_char: end.character,
        })
    }

    fn parse_rename_plan(v: &Value) -> crate::lsp::backend::RenamePlan {
        use crate::lsp::backend::{Conflict, RenamePlan, UsageSite};
        let usages = v.get("usages").and_then(Value::as_array).map(|arr| {
            arr.iter().filter_map(|u| {
                Some(UsageSite {
                    path: u.get("path")?.as_str()?.to_string(),
                    range: Self::parse_range0(u.get("range")?)?,
                    context: u.get("context").and_then(Value::as_str).map(String::from),
                })
            }).collect()
        }).unwrap_or_default();
        let conflicts = v.get("conflicts").and_then(Value::as_array).map(|arr| {
            arr.iter().filter_map(|c| {
                Some(Conflict {
                    path: c.get("path")?.as_str()?.to_string(),
                    range: c.get("range").and_then(Self::parse_range0),
                    message: c.get("message")?.as_str()?.to_string(),
                })
            }).collect()
        }).unwrap_or_default();
        RenamePlan { usages, conflicts }
    }

    /// Common `{path, range, new_name, search_*}` request body for both rename endpoints.
    fn rename_body(rel_path: &str, range: crate::lsp::backend::TextRange0Based, new_name: &str) -> Value {
        serde_json::json!({
            "path": rel_path,
            "range": {
                "start": { "line": range.start_line, "character": range.start_char },
                "end":   { "line": range.end_line,   "character": range.end_char },
            },
            "new_name": new_name,
        })
    }
```

- [ ] **Step 4: Trait-Override implementieren**

Serena `insert_after_symbol` an der `insert_after_symbol`-Methode im `impl LspBackend for JetBrainsHttpBackend`-Block (endet mit `self.post_edit("/insertAfterSymbol", edit)` `}`):

```rust
    fn rename_preview(
        &mut self,
        req: &crate::lsp::backend::RenameQuery,
    ) -> Result<crate::lsp::backend::RenamePlan, String> {
        let mut body = Self::rename_body(&req.rel_path, req.target_range, &req.new_name);
        body["search_comments"] = serde_json::json!(req.search_comments);
        body["search_text_occurrences"] = serde_json::json!(req.search_text_occurrences);
        let resp = self.post("/renamePreview", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(err.get("code").and_then(Value::as_str).unwrap_or("INTERNAL").to_string());
        }
        Ok(Self::parse_rename_plan(&resp))
    }

    fn rename_apply(
        &mut self,
        req: &crate::lsp::backend::RenameApply,
    ) -> Result<crate::lsp::backend::RenameResult, String> {
        let mut body = Self::rename_body(&req.rel_path, req.target_range, &req.new_name);
        body["force"] = serde_json::json!(req.force);
        let resp = self.post("/renameApply", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(err.get("code").and_then(Value::as_str).unwrap_or("INTERNAL").to_string());
        }
        let changed_paths = resp.get("changed_paths").and_then(Value::as_array).map(|a| {
            a.iter().filter_map(|p| p.as_str().map(String::from)).collect()
        }).unwrap_or_default();
        Ok(crate::lsp::backend::RenameResult {
            applied: resp.get("applied").and_then(Value::as_bool).unwrap_or(false),
            changed_paths,
        })
    }
```

- [ ] **Step 5: Tests ausführen, pass bestätigen**

Run: `cargo nextest run -p lean-ctx -E 'test(rename_preview_parses) + test(rename_preview_maps_error_envelope) + test(rename_apply_parses_changed_paths)'` (cwd=`rust`)
Expected: alle PASS.

- [ ] **Step 6: Reformat + Commit (Phase 2)**

```bash
# reformat via mcp__jetbrains__reformat_file auf rust/src/lsp/jetbrains_backend.rs
git add rust/src/lsp/jetbrains_backend.rs
git commit -m "feat(jetbrains): v2b rename — JetBrainsHttpBackend /renamePreview /renameApply"
```

---

# PHASE 3 — Rust `plan_hash` + Usage-Helfer

Commit-Message: `feat(jetbrains): v2b rename — plan_hash + usage jail/read helpers`

### Task 3: `usage_range_text` + `plan_hash` in `ctx_refactor.rs`

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`
- Test: `rust/src/tools/ctx_refactor.rs` (inline `mod tests`)

- [ ] **Step 1: Failing tests schreiben**

Serena `insert_after_symbol` am letzten Test (`inspections_run_and_list_dispatch_and_truncation`) im `mod tests` von `ctx_refactor.rs`:

```rust
    #[test]
    fn usage_range_text_reads_jailed_slice() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
        let root = dir.path().to_str().unwrap();
        let u = crate::lsp::backend::UsageSite {
            path: "a.rs".into(),
            range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
            context: None,
        };
        assert_eq!(super::usage_range_text(root, &u).unwrap(), "foo");
    }

    #[test]
    fn usage_range_text_rejects_jail_escape() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        let u = crate::lsp::backend::UsageSite {
            path: "../../etc/passwd".into(),
            range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 0, end_line: 0, end_char: 1 },
            context: None,
        };
        assert!(super::usage_range_text(root, &u).is_err());
    }

    #[test]
    fn plan_hash_is_deterministic_and_order_independent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
        let root = dir.path().to_str().unwrap();
        let u1 = crate::lsp::backend::UsageSite {
            path: "a.rs".into(),
            range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
            context: Some("ignored-in-hash".into()),
        };
        let u2 = crate::lsp::backend::UsageSite {
            path: "a.rs".into(),
            range: crate::lsp::backend::TextRange0Based { start_line: 1, start_char: 0, end_line: 1, end_char: 3 },
            context: None,
        };
        let h1 = super::plan_hash(root, &[u1.clone(), u2.clone()]).unwrap();
        let h2 = super::plan_hash(root, std::slice::from_ref(&u2)).unwrap(); // subset → differs
        let h3 = super::plan_hash(root, &[u2, u1]).unwrap(); // reversed → SAME (sorted canonical)
        assert_eq!(h1.len(), 64);
        assert_eq!(h1, h3, "hash must be order-independent");
        assert_ne!(h1, h2, "different usage set must differ");
    }
```

- [ ] **Step 2: Test ausführen, fail bestätigen**

Run: `cargo nextest run -p lean-ctx usage_range_text_reads_jailed_slice` (cwd=`rust`)
Expected: COMPILE FAIL — `cannot find function usage_range_text`.

- [ ] **Step 3: Helfer implementieren**

Serena `insert_after_symbol` an der Funktion `resolve_name_path` (endet mit dem `match leaves.len()`-Block). Füge danach an (top-level Funktionen, nicht im Trait/Impl):

```rust
/// Read the current on-disk text covered by a usage's range, jail-checking its
/// path first. Out-of-jail / unreadable / bad range → `Err` (spec §5.4 Multi-File
/// jail: every plugin-reported path is re-checked against `project_root`).
pub(crate) fn usage_range_text(
    project_root: &str,
    u: &crate::lsp::backend::UsageSite,
) -> Result<String, String> {
    let abs = crate::core::path_resolve::resolve_tool_path(Some(project_root), None, &u.path)
        .map_err(|e| format!("CONFLICT: usage path blocked by jail: {e}"))?;
    let content =
        std::fs::read_to_string(&abs).map_err(|e| format!("FILE_NOT_FOUND: {abs}: {e}"))?;
    let s = crate::lsp::edit_apply::offset_of(&content, u.range.start_line, u.range.start_char)?;
    let e = crate::lsp::edit_apply::offset_of(&content, u.range.end_line, u.range.end_char)?;
    if e < s {
        return Err("POSITION_OUT_OF_RANGE: end before start".to_string());
    }
    Ok(content[s..e].to_string())
}

/// Stateless Multi-File integrity guard (spec §5.2). BLAKE3 over the usages
/// canonicalized by sorted `(path, range)` plus each usage's *current* on-disk
/// text. `context` is display-only and intentionally excluded. Re-built in
/// `rename_apply` and compared → mismatch = `CONFLICT` (TOCTOU).
pub(crate) fn plan_hash(
    project_root: &str,
    usages: &[crate::lsp::backend::UsageSite],
) -> Result<String, String> {
    use crate::lsp::backend::TextRange0Based;
    let mut rows: Vec<(String, TextRange0Based, String)> = Vec::with_capacity(usages.len());
    for u in usages {
        let text = usage_range_text(project_root, u)?;
        rows.push((u.path.clone(), u.range, text));
    }
    rows.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.start_line.cmp(&b.1.start_line))
            .then(a.1.start_char.cmp(&b.1.start_char))
            .then(a.1.end_line.cmp(&b.1.end_line))
            .then(a.1.end_char.cmp(&b.1.end_char))
    });
    let mut canon = String::new();
    for (path, r, text) in &rows {
        canon.push_str(&format!(
            "{path}|{}:{}-{}:{}|{text}\n",
            r.start_line, r.start_char, r.end_line, r.end_char
        ));
    }
    Ok(crate::core::hasher::hash_hex(canon.as_bytes()))
}
```

- [ ] **Step 4: Tests ausführen, pass bestätigen**

Run: `cargo nextest run -p lean-ctx -E 'test(usage_range_text) + test(plan_hash_is_deterministic)'` (cwd=`rust`)
Expected: alle PASS.

- [ ] **Step 5: Reformat + Commit (Phase 3)**

```bash
# reformat via mcp__jetbrains__reformat_file auf rust/src/tools/ctx_refactor.rs
git add rust/src/tools/ctx_refactor.rs
git commit -m "feat(jetbrains): v2b rename — plan_hash + usage jail/read helpers"
```

---

# PHASE 4 — Rust Action-Handler (`rename_preview`/`rename_apply`)

Commit-Message: `feat(jetbrains): v2b rename_preview/rename_apply actions + conflict gate`

### Task 4: Live-Backend-Dispatcher + Target-Resolver

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`
- Test: `rust/src/tools/ctx_refactor.rs` (inline)

- [ ] **Step 1: Failing test schreiben** (im `mod tests`)

Serena `insert_after_symbol` an `plan_hash_is_deterministic_and_order_independent`:

```rust
    #[test]
    fn resolve_rename_target_position_fallback() {
        let (rel, sl, el) = super::resolve_rename_target(
            &serde_json::json!({"path": "a.rs", "line": 3, "end_line": 5}),
            "/proj",
        ).unwrap();
        assert_eq!(rel, "a.rs");
        assert_eq!((sl, el), (3, 5));
    }

    #[test]
    fn resolve_rename_target_requires_line_in_fallback() {
        let err = super::resolve_rename_target(
            &serde_json::json!({"path": "a.rs"}),
            "/proj",
        ).unwrap_err();
        assert!(err.contains("line"), "got: {err}");
    }

    #[test]
    fn live_backend_absent_is_backend_required() {
        // No port file under an unlikely root → deterministic BACKEND_REQUIRED, no HTTP.
        let err = super::live_jetbrains_backend("/nonexistent/leanctx/proj/zzz").unwrap_err();
        assert!(err.starts_with("BACKEND_REQUIRED"), "got: {err}");
    }
```

- [ ] **Step 2: Test ausführen, fail bestätigen**

Run: `cargo nextest run -p lean-ctx resolve_rename_target_position_fallback` (cwd=`rust`)
Expected: COMPILE FAIL — `cannot find function resolve_rename_target`.

- [ ] **Step 3: Helfer implementieren**

Serena `insert_after_symbol` an der Funktion `plan_hash` (Task 3, Step 3):

```rust
/// Resolve the rename target: `name_path` (primary, reuse v2a) or `path`+`line`
/// (+`end_line`) fallback. Returns `(rel_path, start_line, end_line)` 1-based incl.
fn resolve_rename_target(args: &Value, project_root: &str) -> Result<(String, usize, usize), String> {
    if let Some(np) = args.get("name_path").and_then(Value::as_str) {
        let r = resolve_name_path(np, project_root)?;
        Ok((r.rel_path, r.start_line, r.end_line))
    } else {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "provide 'name_path' or 'path'+'line' for rename.".to_string())?;
        let line = args.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
        let end = args.get("end_line").and_then(Value::as_u64).unwrap_or(line as u64) as usize;
        if line == 0 {
            return Err("'line' is required (1-based) when using the path fallback.".to_string());
        }
        Ok((path.to_string(), line, end))
    }
}

/// Deterministic 3-stage Backing-B reachability gate (spec §3.1, v1-§8): live
/// port file + pid alive + `/health` ping. Any miss → `BACKEND_REQUIRED` BEFORE
/// any rename HTTP call. NO fallback to Backing A (no IDE-grade rename there).
fn live_jetbrains_backend(project_root: &str) -> Result<Box<dyn LspBackend>, String> {
    use crate::lsp::port_discovery;
    if let Some(pf) = port_discovery::read_port_file(project_root) {
        if port_discovery::pid_alive(pf.pid) && port_discovery::health_ok(&pf) {
            return Ok(Box::new(crate::lsp::jetbrains_backend::JetBrainsHttpBackend::new(
                pf.port,
                pf.token,
                project_root.to_string(),
                pf.pid,
            )));
        }
    }
    Err("BACKEND_REQUIRED: rename requires a running JetBrains IDE \
         (no live port file / health check failed)"
        .to_string())
}
```

Beachte: `LspBackend` ist bereits via `use crate::lsp::backend::LspBackend;` im Datei-Header importiert (siehe `apply_symbol_edit`). Falls der Linter `LspBackend` als unaufgelöst meldet, nutze den vollqualifizierten Pfad `Box<dyn crate::lsp::backend::LspBackend>`.

- [ ] **Step 4: Tests ausführen, pass bestätigen**

Run: `cargo nextest run -p lean-ctx -E 'test(resolve_rename_target) + test(live_backend_absent_is_backend_required)'` (cwd=`rust`)
Expected: alle PASS.

### Task 5: `render_rename_preview` + `render_rename_apply` + Konflikt-Gate

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`
- Test: `rust/src/tools/ctx_refactor.rs` (inline, mit Stub-Backend)

- [ ] **Step 1: Failing tests mit Stub-Backend schreiben** (im `mod tests`)

Serena `insert_after_symbol` an `live_backend_absent_is_backend_required`:

```rust
    /// Minimal backend that returns canned rename plans + records apply calls.
    struct RenameStub {
        plan: crate::lsp::backend::RenamePlan,
        applied_with_force: std::cell::Cell<Option<bool>>,
    }
    impl crate::lsp::backend::LspBackend for RenameStub {
        fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> { Ok(()) }
        fn references(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
        fn definition(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position) -> Result<lsp_types::GotoDefinitionResponse, String> { Ok(lsp_types::GotoDefinitionResponse::Array(vec![])) }
        fn implementations(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
        fn rename(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _n: &str) -> Result<Option<lsp_types::WorkspaceEdit>, String> { Ok(None) }
        fn rename_preview(&mut self, _q: &crate::lsp::backend::RenameQuery) -> Result<crate::lsp::backend::RenamePlan, String> { Ok(self.plan.clone()) }
        fn rename_apply(&mut self, req: &crate::lsp::backend::RenameApply) -> Result<crate::lsp::backend::RenameResult, String> {
            self.applied_with_force.set(Some(req.force));
            Ok(crate::lsp::backend::RenameResult { applied: true, changed_paths: vec!["a.rs".into()] })
        }
    }

    fn stub_query(abs: &str) -> crate::lsp::backend::RenameQuery {
        crate::lsp::backend::RenameQuery {
            abs_path: abs.into(), rel_path: "a.rs".into(),
            target_range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
            new_name: "bar".into(), search_comments: false, search_text_occurrences: false,
        }
    }

    #[test]
    fn apply_blocks_on_plan_hash_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
        let root = dir.path().to_str().unwrap();
        let usage = crate::lsp::backend::UsageSite {
            path: "a.rs".into(),
            range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
            context: None,
        };
        let mut be = RenameStub {
            plan: crate::lsp::backend::RenamePlan { usages: vec![usage], conflicts: vec![] },
            applied_with_force: std::cell::Cell::new(None),
        };
        let q = stub_query(&dir.path().join("a.rs").to_string_lossy());
        let out = super::render_rename_apply(&mut be, root, &q, "bar", "stalehash", false);
        assert!(out.contains("CONFLICT"), "got: {out}");
        assert_eq!(be.applied_with_force.get(), None, "apply must not run on hash mismatch");
    }

    #[test]
    fn apply_blocks_on_conflicts_without_force_and_passes_with_force() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
        let root = dir.path().to_str().unwrap();
        let usage = crate::lsp::backend::UsageSite {
            path: "a.rs".into(),
            range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
            context: None,
        };
        let plan = crate::lsp::backend::RenamePlan {
            usages: vec![usage.clone()],
            conflicts: vec![crate::lsp::backend::Conflict { path: "a.rs".into(), range: None, message: "clash".into() }],
        };
        let hash = super::plan_hash(root, &plan.usages).unwrap();
        let q = stub_query(&dir.path().join("a.rs").to_string_lossy());

        // force=false → CONFLICT, apply not called.
        let mut be = RenameStub { plan: plan.clone(), applied_with_force: std::cell::Cell::new(None) };
        let out = super::render_rename_apply(&mut be, root, &q, "bar", &hash, false);
        assert!(out.contains("CONFLICT"), "got: {out}");
        assert_eq!(be.applied_with_force.get(), None);

        // force=true → applies, force passed through.
        let mut be2 = RenameStub { plan, applied_with_force: std::cell::Cell::new(None) };
        let out2 = super::render_rename_apply(&mut be2, root, &q, "bar", &hash, true);
        assert!(out2.contains("applied"), "got: {out2}");
        assert_eq!(be2.applied_with_force.get(), Some(true));
    }

    #[test]
    fn apply_success_emits_diff_and_evicts() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
        let root = dir.path().to_str().unwrap();
        let usage = crate::lsp::backend::UsageSite {
            path: "a.rs".into(),
            range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
            context: None,
        };
        let plan = crate::lsp::backend::RenamePlan { usages: vec![usage], conflicts: vec![] };
        let hash = super::plan_hash(root, &plan.usages).unwrap();
        let mut be = RenameStub { plan, applied_with_force: std::cell::Cell::new(None) };
        let q = stub_query(&dir.path().join("a.rs").to_string_lossy());
        let out = super::render_rename_apply(&mut be, root, &q, "bar", &hash, false);
        assert!(out.contains("applied"), "got: {out}");
        assert!(out.contains("\"foo\" → \"bar\""), "diff missing: {out}");
    }

    #[test]
    fn preview_renders_plan_hash_and_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
        let root = dir.path().to_str().unwrap();
        let usage = crate::lsp::backend::UsageSite {
            path: "a.rs".into(),
            range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
            context: None,
        };
        let plan = crate::lsp::backend::RenamePlan { usages: vec![usage], conflicts: vec![] };
        let mut be = RenameStub { plan, applied_with_force: std::cell::Cell::new(None) };
        let q = stub_query(&dir.path().join("a.rs").to_string_lossy());
        let out = super::render_rename_preview(&mut be, root, &q, "bar");
        assert!(out.contains("plan_hash:"), "got: {out}");
        assert!(out.contains("usages: 1"), "got: {out}");
        assert!(out.contains("a.rs: 1 usage"), "got: {out}");
    }
```

- [ ] **Step 2: Test ausführen, fail bestätigen**

Run: `cargo nextest run -p lean-ctx preview_renders_plan_hash_and_files` (cwd=`rust`)
Expected: COMPILE FAIL — `cannot find function render_rename_preview`.

- [ ] **Step 3: Render-Funktionen implementieren**

Serena `insert_after_symbol` an der Funktion `live_jetbrains_backend` (Task 4, Step 3):

```rust
/// Phase 1 renderer: ask Backing B for usages+conflicts, build the stateless
/// plan_hash, and present the blast radius (files, usage count, conflicts).
fn render_rename_preview(
    backend: &mut dyn LspBackend,
    project_root: &str,
    query: &crate::lsp::backend::RenameQuery,
    new_name: &str,
) -> String {
    let plan = match backend.rename_preview(query) {
        Ok(p) => p,
        Err(e) => return format!("ERROR: {e}"),
    };
    let hash = match plan_hash(project_root, &plan.usages) {
        Ok(h) => h,
        Err(e) => return format!("ERROR: {e}"),
    };
    let mut files: Vec<&str> = plan.usages.iter().map(|u| u.path.as_str()).collect();
    files.sort_unstable();
    files.dedup();

    let mut out = format!(
        "rename_preview: '{}' → '{new_name}'\n  usages: {}\n  files: {}\n  plan_hash: {hash}\n",
        query.rel_path,
        plan.usages.len(),
        files.len(),
    );
    if !plan.conflicts.is_empty() {
        out.push_str(&format!(
            "  conflicts: {} (rename_apply blocks unless force=true)\n",
            plan.conflicts.len()
        ));
        for c in &plan.conflicts {
            out.push_str(&format!("    {}: {}\n", c.path, c.message));
        }
    }
    for f in &files {
        let n = plan.usages.iter().filter(|u| u.path == **f).count();
        out.push_str(&format!("  {f}: {n} usage(s)\n"));
    }
    out
}

/// Phase 2 renderer: re-fetch usages, enforce the plan_hash (TOCTOU) + conflict
/// gates in Rust, then run the IDE Multi-File transaction and evict changed files.
fn render_rename_apply(
    backend: &mut dyn LspBackend,
    project_root: &str,
    query: &crate::lsp::backend::RenameQuery,
    new_name: &str,
    expected_hash: &str,
    force: bool,
) -> String {
    let plan = match backend.rename_preview(query) {
        Ok(p) => p,
        Err(e) => return format!("ERROR: {e}"),
    };
    // Capture pre-apply usage text (also jail-checks every usage path).
    let mut pre: Vec<(String, u32, String)> = Vec::with_capacity(plan.usages.len());
    for u in &plan.usages {
        match usage_range_text(project_root, u) {
            Ok(t) => pre.push((u.path.clone(), u.range.start_line + 1, t)),
            Err(e) => return format!("ERROR: {e}"),
        }
    }
    // Gate (a): TOCTOU plan_hash.
    let actual = match plan_hash(project_root, &plan.usages) {
        Ok(h) => h,
        Err(e) => return format!("ERROR: {e}"),
    };
    if actual != expected_hash {
        return format!(
            "ERROR: CONFLICT: plan_hash mismatch (source changed since preview; \
             expected={expected_hash}, actual={actual})"
        );
    }
    // Gate (b): refactoring conflicts.
    if !plan.conflicts.is_empty() && !force {
        return format!(
            "ERROR: CONFLICT: {} refactoring conflict(s); pass force=true to override",
            plan.conflicts.len()
        );
    }

    let apply = crate::lsp::backend::RenameApply {
        abs_path: query.abs_path.clone(),
        rel_path: query.rel_path.clone(),
        target_range: query.target_range,
        new_name: new_name.to_string(),
        force,
    };
    let res = match backend.rename_apply(&apply) {
        Ok(r) => r,
        Err(e) => return format!("ERROR: {e}"),
    };

    // Jail-check + cache-evict each changed file (Multi-File coherence, spec §9).
    for cp in &res.changed_paths {
        match crate::core::path_resolve::resolve_tool_path(Some(project_root), None, cp) {
            Ok(abs) => crate::core::cli_cache::invalidate(&abs),
            Err(e) => return format!("ERROR: CONFLICT: changed path blocked by jail: {e}"),
        }
    }

    let mut out = format!(
        "rename_apply: '{}' → '{new_name}' applied\n  changed files: {}\n  usages: {}\n",
        query.rel_path,
        res.changed_paths.len(),
        pre.len(),
    );
    for (path, line, old) in &pre {
        out.push_str(&format!("  {path}:{line}  \"{old}\" → \"{new_name}\"\n"));
    }
    out
}
```

- [ ] **Step 4: Tests ausführen, pass bestätigen**

Run: `cargo nextest run -p lean-ctx -E 'test(apply_blocks_on_plan_hash_mismatch) + test(apply_blocks_on_conflicts) + test(apply_success_emits_diff_and_evicts) + test(preview_renders_plan_hash_and_files)'` (cwd=`rust`)
Expected: alle PASS.

### Task 6: `handle_rename_refactor` + Dispatch-Verdrahtung

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs` (Funktion `handle`, Zeile ~14-19 + Unknown-Action-Hilfetext ~41-44)
- Test: `rust/src/tools/ctx_refactor.rs` (inline)

- [ ] **Step 1: Failing test schreiben** (im `mod tests`)

Serena `insert_after_symbol` an `preview_renders_plan_hash_and_files`:

```rust
    #[test]
    fn handle_rename_preview_without_ide_is_backend_required() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn foo() {}\n").unwrap();
        let root = dir.path().to_str().unwrap();
        // No port file under this temp root → BACKEND_REQUIRED before any HTTP.
        let args = serde_json::json!({
            "action": "rename_preview", "path": "a.rs", "line": 1, "new_name": "bar"
        });
        let out = super::handle(&args, root, "");
        assert!(out.contains("BACKEND_REQUIRED"), "got: {out}");
    }

    #[test]
    fn handle_rename_apply_requires_plan_hash() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn foo() {}\n").unwrap();
        let root = dir.path().to_str().unwrap();
        let args = serde_json::json!({
            "action": "rename_apply", "path": "a.rs", "line": 1, "new_name": "bar"
        });
        let out = super::handle(&args, root, "");
        assert!(out.contains("plan_hash"), "got: {out}");
    }

    #[test]
    fn unknown_action_help_lists_rename_actions() {
        // Resolution happens before backend selection for rename actions, so an
        // empty new_name short-circuits with a clear ERROR mentioning new_name.
        let args = serde_json::json!({"action": "rename_preview", "path": "a.rs", "line": 1});
        let out = super::handle(&args, "/proj", "");
        assert!(out.contains("new_name"), "got: {out}");
    }
```

- [ ] **Step 2: Test ausführen, fail bestätigen**

Run: `cargo nextest run -p lean-ctx handle_rename_preview_without_ide_is_backend_required` (cwd=`rust`)
Expected: FAIL — `handle` routet `rename_preview` aktuell in den Unknown-Action-Arm (oder versucht `open_file`), kein `BACKEND_REQUIRED`.

- [ ] **Step 3: Early-Dispatch in `handle` ergänzen**

In `handle` (`ctx_refactor.rs`) gibt es bereits den Block für die drei Body-Edit-Actions:

```rust
    if matches!(
        action,
        "replace_symbol_body" | "insert_before_symbol" | "insert_after_symbol"
    ) {
        return handle_symbol_edit(action, args, project_root);
    }
```

Serena `replace_content` — ersetze diesen Block durch (fügt den rename-Zweig hinzu):

```rust
    if matches!(
        action,
        "replace_symbol_body" | "insert_before_symbol" | "insert_after_symbol"
    ) {
        return handle_symbol_edit(action, args, project_root);
    }

    if matches!(action, "rename_preview" | "rename_apply") {
        return handle_rename_refactor(action, args, project_root);
    }
```

- [ ] **Step 4: `handle_rename_refactor` implementieren**

Serena `insert_after_symbol` an der Funktion `render_rename_apply` (Task 5):

```rust
/// Entry for the Two-Phase rename actions. Resolves the target (name_path / pos),
/// double-jails, requires a live IDE, then dispatches to the preview/apply renderer.
fn handle_rename_refactor(action: &str, args: &Value, project_root: &str) -> String {
    let Some(new_name) = args.get("new_name").and_then(Value::as_str) else {
        return "ERROR: 'new_name' is required for rename.".to_string();
    };
    if action == "rename_apply" && args.get("plan_hash").and_then(Value::as_str).is_none() {
        return "ERROR: 'plan_hash' is required for rename_apply (run rename_preview first)."
            .to_string();
    }

    // Resolve target symbol → 1-based inclusive span.
    let (rel_path, start_line, end_line) = match resolve_rename_target(args, project_root) {
        Ok(t) => t,
        Err(e) => return format!("ERROR: {e}"),
    };
    // PathJail stage (a): the resolved target path.
    let abs_path =
        match crate::core::path_resolve::resolve_tool_path(Some(project_root), None, &rel_path) {
            Ok(p) => p,
            Err(e) => return format!("ERROR: path blocked by jail: {e}"),
        };
    let content = match std::fs::read_to_string(&abs_path) {
        Ok(c) => c,
        Err(e) => return format!("ERROR: FILE_NOT_FOUND: {abs_path}: {e}"),
    };
    let end_col = content
        .lines()
        .nth(end_line.saturating_sub(1))
        .map_or(0, str::len) as u32;
    let target_range = crate::lsp::backend::TextRange0Based {
        start_line: (start_line - 1) as u32,
        start_char: 0,
        end_line: (end_line - 1) as u32,
        end_char: end_col,
    };
    let search_comments = args
        .get("search_comments")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let search_text_occurrences = args
        .get("search_text_occurrences")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    // Backing B is mandatory (no headless rename) → BACKEND_REQUIRED otherwise.
    let mut backend = match live_jetbrains_backend(project_root) {
        Ok(b) => b,
        Err(e) => return format!("ERROR: {e}"),
    };

    let query = crate::lsp::backend::RenameQuery {
        abs_path,
        rel_path,
        target_range,
        new_name: new_name.to_string(),
        search_comments,
        search_text_occurrences,
    };

    match action {
        "rename_preview" => render_rename_preview(backend.as_mut(), project_root, &query, new_name),
        "rename_apply" => {
            let expected = args
                .get("plan_hash")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let force = args.get("force").and_then(Value::as_bool).unwrap_or(false);
            render_rename_apply(backend.as_mut(), project_root, &query, new_name, expected, force)
        }
        other => format!("ERROR: INTERNAL: not a rename action: {other}"),
    }
}
```

- [ ] **Step 5: Unknown-Action-Hilfetext erweitern**

In `handle` endet der `_ =>`-Arm mit dem Hilfetext, der alle Actions auflistet (`replace_symbol_body, insert_before_symbol, insert_after_symbol.`). Serena `replace_content` — ergänze die zwei rename-Actions am Ende der Liste:

```rust
        _ => format!(
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, \
             implementations, declaration, type_hierarchy, symbols_overview, inspections, \
             replace_symbol_body, insert_before_symbol, insert_after_symbol, \
             rename_preview, rename_apply."
        ),
```

- [ ] **Step 6: Tests ausführen, pass bestätigen**

Run: `cargo nextest run -p lean-ctx -E 'test(handle_rename_preview_without_ide_is_backend_required) + test(handle_rename_apply_requires_plan_hash) + test(unknown_action_help_lists_rename_actions)'` (cwd=`rust`)
Expected: alle PASS.

- [ ] **Step 7: Volle Rust-Suite grün**

Run: `cargo nextest run -p lean-ctx` (cwd=`rust`)
Expected: PASS (keine Regression). Bei großem grünen Lauf: `--status-level fail`.

- [ ] **Step 8: Reformat + Commit (Phase 4)**

```bash
# reformat via mcp__jetbrains__reformat_file auf rust/src/tools/ctx_refactor.rs
git add rust/src/tools/ctx_refactor.rs
git commit -m "feat(jetbrains): v2b rename_preview/rename_apply actions + conflict gate"
```

---

# PHASE 5 — Rust Schema + Reference-Docs

Commit-Message: `feat(jetbrains): v2b rename schema + reference docs`

### Task 7: Schema-Erweiterung in `registered/ctx_refactor.rs`

**Files:**
- Modify: `rust/src/tools/registered/ctx_refactor.rs`
- Test: bestehender Drift-Test `tests/reference_docs_drift.rs` + die `consumes`/`changed`-Logik.

- [ ] **Step 1: Schema lesen + verstehen**

Run: `ctx_read("rust/src/tools/registered/ctx_refactor.rs")` — die `action`-enum-Liste, die `name_path`/`new_body`/`text`/`end_line`/`expected_hash`-Properties und das `changed: matches!(action.as_str(), "replace_symbol_body" | …)`.

- [ ] **Step 2: Action-Enum erweitern**

Im `tool_def(...)`-Schema gibt es das `"action"`-Property mit `"enum": [..., "replace_symbol_body", "insert_before_symbol", "insert_after_symbol"]`. Native `Edit` (Markdown/Schema-String, **kein** Rust-Symbol-Body — `replace_content` via Serena ist hier nicht nötig, aber da es eine `.rs`-Datei ist: **Serena `replace_content`** auf den enum-Abschnitt). Ergänze `"rename_preview", "rename_apply"`:

```
"replace_symbol_body", "insert_before_symbol", "insert_after_symbol",
"rename_preview", "rename_apply"],
```

- [ ] **Step 3: Neue Parameter-Properties ergänzen**

Nach der `"expected_hash"`-Property (Serena `replace_content`) ergänze:

```
                    "expected_hash": { "type": "string", "description": "Optional BLAKE3-hex of the current range content; mismatch → CONFLICT (no blind overwrite)." },
                    "plan_hash": { "type": "string", "description": "Required for rename_apply: the BLAKE3 plan hash returned by rename_preview (stateless TOCTOU guard; mismatch → CONFLICT)." },
                    "force": { "type": "boolean", "description": "rename_apply only: override blocking refactoring conflicts (default false → CONFLICT when conflicts exist)." },
                    "search_comments": { "type": "boolean", "description": "rename: also rename matches inside comments/strings (default false)." },
                    "search_text_occurrences": { "type": "boolean", "description": "rename: also rename non-code text occurrences (default false)." }
```

(`new_name` existiert bereits im Schema — nicht doppeln.)

- [ ] **Step 4: Tool-Beschreibung erweitern**

Die `description` (beginnt `"LSP-powered refactoring..."`, endet `"...lossless headless fallback."`). Serena `replace_content` — ergänze einen Satz:

```
... work IDE-first with a lossless headless fallback. The Two-Phase rename ops \
(rename_preview, rename_apply) are name_path-addressed, require a running JetBrains \
IDE (BACKEND_REQUIRED otherwise), use a stateless plan_hash guard, and block on \
refactoring conflicts unless force=true.
```

- [ ] **Step 5: `changed`-Flag für `rename_apply` setzen**

Im Block `changed: matches!(action.as_str(), "replace_symbol_body" | "insert_before_symbol" | "insert_after_symbol")` — Serena `replace_content`, ergänze `"rename_apply"` (NICHT `rename_preview` — Preview schreibt nicht):

```rust
            changed: matches!(
                action.as_str(),
                "replace_symbol_body"
                    | "insert_before_symbol"
                    | "insert_after_symbol"
                    | "rename_apply"
            ),
```

- [ ] **Step 6: `consumes`-Liste erweitern (falls vorhanden)**

Die Datei listet konsumierte Parameter-Keys (`"name_path", "new_body", "expected_hash", …`). Serena `replace_content` — ergänze `"plan_hash"`, `"force"`, `"search_comments"`, `"search_text_occurrences"` (und `"new_name"` falls dort noch nicht enthalten). Prüfe via `ctx_read` welche Keys gelistet sind, ergänze die fehlenden.

- [ ] **Step 7: Build prüfen**

Run: `cargo build -p lean-ctx` (cwd=`rust`)
Expected: kompiliert.

### Task 8: Reference-Docs regenerieren

**Files:**
- Generated: `docs/reference/generated/mcp-tools.md`
- Modify: `docs/reference/appendix-mcp-tools.md` (Zeile 84)
- Test: `tests/reference_docs_drift.rs::generated_reference_docs_are_committed_and_current`

- [ ] **Step 1: Drift-Test rot bestätigen**

Run: `cargo nextest run -p lean-ctx generated_reference_docs_are_committed_and_current` (cwd=`rust`)
Expected: FAIL — generated doc out of date (neue Actions/Params fehlen).

- [ ] **Step 2: Generierte Docs schreiben**

Run: `cargo run --example gen_docs --features dev-tools` (cwd=`rust`)
Expected: schreibt `docs/reference/generated/mcp-tools.md` (Header: „GENERATED FILE — do not edit by hand").

- [ ] **Step 3: Drift-Test grün bestätigen**

Run: `cargo nextest run -p lean-ctx generated_reference_docs_are_committed_and_current` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 4: Appendix (human map) aktualisieren**

Native `Edit` auf `docs/reference/appendix-mcp-tools.md` Zeile 84 — ergänze die zwei Actions in der Pipe-Liste:

```
| `ctx_refactor` | LSP-backed refactoring + name_path symbol-body edits + Two-Phase rename (IDE-first, lossless headless fallback for edits; rename needs a live IDE → BACKEND_REQUIRED; `expected_hash`/`plan_hash` BLAKE3 CONFLICT guards) | rename\|references\|definition\|implementations\|declaration\|type_hierarchy\|symbols_overview\|inspections\|replace_symbol_body\|insert_before_symbol\|insert_after_symbol\|rename_preview\|rename_apply | S |
```

- [ ] **Step 5: Reformat + Commit (Phase 5)**

```bash
# reformat via mcp__jetbrains__reformat_file auf rust/src/tools/registered/ctx_refactor.rs
git add rust/src/tools/registered/ctx_refactor.rs docs/reference/generated/mcp-tools.md docs/reference/appendix-mcp-tools.md
git commit -m "feat(jetbrains): v2b rename schema + reference docs"
```

---

# PHASE 6 — Kotlin Wire-DTOs

Commit-Message: `feat(jetbrains): v2b rename wire DTOs`

### Task 9: DTOs + JsonCodec in `Wire.kt`

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt`

- [ ] **Step 1: Failing test schreiben**

Native `Edit` — füge in `JsonCodecTest.kt` eine neue Test-Methode hinzu (am Klassen-Ende, vor der schließenden `}`):

```kotlin
    @Test
    fun parsesRenamePreviewRequest() {
        val body = """{"path":"a.kt","range":{"start":{"line":1,"character":4},"end":{"line":1,"character":7}},"new_name":"bar","search_comments":true}"""
        val req = JsonCodec.parseRenamePreviewRequest(body)
        assertEquals("a.kt", req.path)
        assertEquals(4, req.range.start.character)
        assertEquals("bar", req.new_name)
        assertTrue(req.search_comments)
        assertFalse(req.search_text_occurrences) // default
    }

    @Test
    fun parsesRenameApplyRequestWithForceDefault() {
        val body = """{"path":"a.kt","range":{"start":{"line":1,"character":4},"end":{"line":1,"character":7}},"new_name":"bar"}"""
        val req = JsonCodec.parseRenameApplyRequest(body)
        assertEquals("bar", req.new_name)
        assertFalse(req.force) // default false
    }

    @Test
    fun serializesRenamePreviewResponse() {
        val resp = RenamePreviewResponse(
            usages = listOf(UsageSiteDTO("a.kt", TextRangeDTO(PositionDTO(1, 4), PositionDTO(1, 7)), "foo()")),
            conflicts = listOf(ConflictDTO("a.kt", null, "name clash")),
        )
        val json = JsonCodec.toJson(resp)
        assertTrue(json, json.contains("\"usages\""))
        assertTrue(json, json.contains("\"name clash\""))
    }
```

Prüfe die Imports/Annotations am Datei-Kopf von `JsonCodecTest.kt` (`org.junit.Test`, `kotlin.test.*` o.ä.) und ergänze fehlende.

- [ ] **Step 2: Test ausführen, fail bestätigen**

Run: `./gradlew test --tests '*JsonCodecTest'` (cwd=`packages/jetbrains-lean-ctx`)
Expected: COMPILE FAIL — `unresolved reference RenamePreviewResponse / parseRenamePreviewRequest`.

- [ ] **Step 3: DTOs ergänzen**

Native `Edit` — füge in `Wire.kt` nach dem `EditResponse`-Block (vor `object JsonCodec`) ein:

```kotlin
/** Request body for /renamePreview. range = target symbol declaration span (0-based). */
data class RenamePreviewRequest(
    val path: String,
    val range: TextRangeDTO,
    val new_name: String,
    val search_comments: Boolean = false,
    val search_text_occurrences: Boolean = false,
)

/** A single semantic usage of the renamed symbol (declaration or reference). */
data class UsageSiteDTO(
    val path: String,
    val range: TextRangeDTO,
    val context: String? = null,
)

/** A refactoring conflict. `range` is nullable (some conflicts are scope-level). */
data class ConflictDTO(
    val path: String,
    val range: TextRangeDTO?,
    val message: String,
)

data class RenamePreviewResponse(
    val usages: List<UsageSiteDTO>,
    val conflicts: List<ConflictDTO>,
)

/** Request body for /renameApply. force = override blocking conflicts (Rust already gated). */
data class RenameApplyRequest(
    val path: String,
    val range: TextRangeDTO,
    val new_name: String,
    val force: Boolean = false,
)

data class RenameApplyResponse(
    val applied: Boolean,
    val changed_paths: List<String>,
)
```

- [ ] **Step 4: JsonCodec-Parser ergänzen**

Native `Edit` — füge in `object JsonCodec` nach `parseEditRequest` ein:

```kotlin
    fun parseRenamePreviewRequest(body: String): RenamePreviewRequest =
        gson.fromJson(body, RenamePreviewRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")

    fun parseRenameApplyRequest(body: String): RenameApplyRequest =
        gson.fromJson(body, RenameApplyRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")
```

- [ ] **Step 5: Test ausführen, pass bestätigen**

Run: `./gradlew test --tests '*JsonCodecTest'` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS.

- [ ] **Step 6: Reformat + Commit (Phase 6)**

```bash
# reformat via mcp__jetbrains__reformat_file auf Wire.kt + JsonCodecTest.kt
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt
git commit -m "feat(jetbrains): v2b rename wire DTOs"
```

---

# PHASE 7 — Kotlin `SymbolRefactorer`

Commit-Message: `feat(jetbrains): v2b SymbolRefactorer — RenameProcessor preview/apply`

> **SDK-Naht (am `runIde`-Gate, Task 14, zu verifizieren):** `RenameProcessor`
> erbt von `BaseRefactoringProcessor`; `findUsages()` ist `protected`, ebenso
> `preprocessUsages(Ref<UsageInfo[]>)`. Der Zugriff erfolgt über eine **Subklasse**,
> die zusätzlich `showConflicts(...)` überschreibt, um Konflikte ohne Dialog
> abzugreifen und immer `true` (proceed) zurückzugeben — das Rust-Gate entscheidet,
> nicht der Dialog. Sollte sich die Sichtbarkeit/Signatur in der Ziel-SDK-Version
> unterscheiden, ist `RefactoringFactory.createRename(...).findUsages()` (public)
> der dokumentierte Fallback für die Usage-Liste; Konflikt-Erkennung dann via
> `RenameUtil`/Name-Kollisionsprüfung. Diese Naht ist bewusst der einzige Punkt,
> der echte IDE-Runtime braucht (Unit-Tests via `BasePlatformTestCase`, Task 14).

### Task 10: `SymbolRefactorer.kt` — Preview

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolRefactorer.kt`
- Test: via `RequestRouterRefactorTest.kt` (Task 14) — `SymbolRefactorer` wird off-EDT/EDT über die Handler getestet; ein direkter Unit-Test ist hier optional, weil PSI-Resolution einen `BasePlatformTestCase`-Fixture braucht.

- [ ] **Step 1: Datei anlegen (Preview-Pfad)**

Native `Write` — `SymbolRefactorer.kt`:

```kotlin
package com.leanctx.plugin.psi

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Ref
import com.intellij.psi.PsiDocumentManager
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiNamedElement
import com.intellij.refactoring.rename.RenameProcessor
import com.intellij.usageView.UsageInfo
import com.intellij.util.containers.MultiMap
import com.leanctx.plugin.dto.ConflictDTO
import com.leanctx.plugin.dto.RenameApplyRequest
import com.leanctx.plugin.dto.RenameApplyResponse
import com.leanctx.plugin.dto.RenamePreviewRequest
import com.leanctx.plugin.dto.RenamePreviewResponse
import com.leanctx.plugin.dto.UsageSiteDTO
import com.leanctx.plugin.server.BackendException

/**
 * Multi-File rename via IntelliJ's RenameProcessor — the canonical compiler-semantic
 * (resolve-based) usage search the headless lean-ctx stack cannot provide (spec §3).
 *
 * Preview: findUsages + conflict collection, NO write. Apply: one WriteCommandAction
 * → one Undo entry, saved to disk for lean-ctx. The plan_hash CONFLICT guard lives
 * entirely in Rust; this class never hashes.
 */
class SymbolRefactorer(private val project: Project) {
    private val locator = PsiLocator(project)

    /** Subclass exposing protected findUsages + capturing conflicts without a dialog. */
    private class CapturingProcessor(
        project: Project,
        element: PsiElement,
        newName: String,
        searchInComments: Boolean,
        searchTextOccurrences: Boolean,
    ) : RenameProcessor(project, element, newName, searchInComments, searchTextOccurrences) {
        val captured = MultiMap<PsiElement, String>()

        fun usages(): Array<UsageInfo> = findUsages()

        /** Collect conflicts via preprocessUsages → showConflicts hook, then proceed. */
        fun collectConflicts(usages: Array<UsageInfo>) {
            preprocessUsages(Ref.create(usages))
        }

        public override fun showConflicts(
            conflicts: MultiMap<PsiElement, String>,
            usages: Array<out UsageInfo>?,
        ): Boolean {
            captured.putAllValues(conflicts)
            return true // never block here — the Rust gate decides
        }
    }

    fun preview(req: RenamePreviewRequest): RenamePreviewResponse = locator.inSmartReadAction {
        val element = resolveTarget(req)
        val processor = CapturingProcessor(
            project, element, req.new_name, req.search_comments, req.search_text_occurrences,
        )
        val usages = processor.usages()
        processor.collectConflicts(usages)

        val usageDtos = usages.mapNotNull { info ->
            val el = info.element ?: return@mapNotNull null
            locator.toLocation(el)?.let { UsageSiteDTO(it.path, it.range, contextSnippet(el)) }
        }
        val conflictDtos = processor.captured.entrySet().flatMap { entry ->
            val loc = locator.toLocation(entry.key)
            entry.value.map { msg -> ConflictDTO(loc?.path ?: "", loc?.range, msg) }
        }
        RenamePreviewResponse(usageDtos, conflictDtos)
    }

    /** Resolve the target PsiElement from the declaration range start (walk to a named decl). */
    private fun resolveTarget(req: RenamePreviewRequest): PsiElement {
        val file = locator.psiFile(req.path)
        val offset = locator.offsetOf(file, req.range.start.line, req.range.start.character)
        val at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at ${req.range.start.line}:${req.range.start.character}")
        return generateSequence(at) { it.parent }
            .firstOrNull { it is PsiNamedElement && (it as PsiNamedElement).name != null }
            ?: throw BackendException("NO_SYMBOL", "no named declaration at target range")
    }

    private fun contextSnippet(el: PsiElement): String? {
        val text = el.containingFile?.text ?: return null
        val range = el.textRange ?: return null
        val lineStart = text.lastIndexOf('\n', range.startOffset).let { if (it < 0) 0 else it + 1 }
        val lineEnd = text.indexOf('\n', range.endOffset).let { if (it < 0) text.length else it }
        return text.substring(lineStart, lineEnd).trim().take(200)
    }
}
```

- [ ] **Step 2: Build prüfen**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: kompiliert (SDK-Symbole `RenameProcessor`/`UsageInfo`/`MultiMap` aufgelöst). Falls `showConflicts`-Signatur abweicht → siehe SDK-Naht-Hinweis oben; passe Override-Signatur an die tatsächliche `BaseRefactoringProcessor`-Deklaration der Ziel-SDK an.

### Task 11: `SymbolRefactorer.kt` — Apply

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolRefactorer.kt`

- [ ] **Step 1: Apply-Methode ergänzen**

Native `Edit` — füge in `SymbolRefactorer` nach `preview(...)` (vor `resolveTarget`) ein:

```kotlin
    fun apply(req: RenameApplyRequest): RenameApplyResponse {
        // Resolve + findUsages in a read action; run the transaction on the EDT.
        val element = locator.inSmartReadAction {
            resolveTarget(
                RenamePreviewRequest(req.path, req.range, req.new_name, false, false)
            )
        }
        val processor = locator.inSmartReadAction {
            CapturingProcessor(project, element, req.new_name, false, false)
        }
        val usages = locator.inSmartReadAction { processor.usages() }

        // Distinct changed files = every usage's file (+ the declaration file).
        val changed = LinkedHashSet<String>()
        locator.inSmartReadAction {
            usages.forEach { info -> info.element?.let { el -> locator.toLocation(el)?.let { changed.add(it.path) } } }
            locator.toLocation(element)?.let { changed.add(it.path) }
        }

        // RenameProcessor.run() performs its own WriteCommandAction → one Undo entry.
        var error: Throwable? = null
        ApplicationManager.getApplication().invokeAndWait {
            try {
                processor.setPreviewUsages(false)
                processor.run()
                // Persist every changed document to disk so lean-ctx (reads from disk) sees it.
                WriteCommandAction.runWriteCommandAction(project) {
                    val fdm = FileDocumentManager.getInstance()
                    PsiDocumentManager.getInstance(project).let { /* commits handled by run() */ }
                    fdm.saveAllDocuments()
                }
            } catch (t: Throwable) {
                error = t
            }
        }
        error?.let { throw it }

        return RenameApplyResponse(applied = true, changed_paths = changed.toList())
    }
```

> **Note (Apply-Persistenz):** `RenameProcessor.run()` committet die PSI-Änderungen
> selbst; `saveAllDocuments()` schreibt sie auf Platte. `saveAllDocuments` ist
> bewusst breiter als nötig, aber korrekt — nur tatsächlich veränderte Dokumente
> werden geschrieben. Falls am `runIde`-Gate ein gezielteres Speichern gewünscht
> ist, ersetze durch eine Schleife über die `changed`-Pfade mit
> `LocalFileSystem.findFileByPath` → `getDocument` → `saveDocument`.

- [ ] **Step 2: Build prüfen**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: kompiliert.

- [ ] **Step 3: Reformat + Commit (Phase 7)**

```bash
# reformat via mcp__jetbrains__reformat_file auf SymbolRefactorer.kt
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolRefactorer.kt
git commit -m "feat(jetbrains): v2b SymbolRefactorer — RenameProcessor preview/apply"
```

---

# PHASE 8 — Kotlin Handler + Router-Verdrahtung

Commit-Message: `feat(jetbrains): v2b RefactorHandlers + router wiring`

### Task 12: `RefactorHandlers.kt`

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/RefactorHandlers.kt`

- [ ] **Step 1: Datei anlegen**

Native `Write` — `RefactorHandlers.kt`:

```kotlin
package com.leanctx.plugin.endpoint

import com.intellij.openapi.project.Project
import com.leanctx.plugin.dto.RenameApplyRequest
import com.leanctx.plugin.dto.RenameApplyResponse
import com.leanctx.plugin.dto.RenamePreviewRequest
import com.leanctx.plugin.dto.RenamePreviewResponse
import com.leanctx.plugin.psi.SymbolRefactorer

/**
 * Endpoint layer for the Two-Phase rename. Preview runs PSI off-EDT in a smart-mode
 * read action (SymbolRefactorer.preview). Apply runs the Multi-File transaction on
 * the EDT (SymbolRefactorer.apply handles invokeAndWait + WriteCommandAction).
 */
class RefactorHandlers(project: Project) {
    private val refactorer = SymbolRefactorer(project)

    fun renamePreview(req: RenamePreviewRequest): RenamePreviewResponse = refactorer.preview(req)

    fun renameApply(req: RenameApplyRequest): RenameApplyResponse = refactorer.apply(req)
}
```

- [ ] **Step 2: Build prüfen**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: kompiliert.

### Task 13: `RequestRouter` verdrahten

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt`

- [ ] **Step 1: Handler-Feld + Routen ergänzen**

Native `Edit` — füge das Feld nach `private val editHandlers = EditHandlers(project)` ein:

```kotlin
    private val refactorHandlers = RefactorHandlers(project)
```

Und den Import oben:

```kotlin
import com.leanctx.plugin.endpoint.RefactorHandlers
```

- [ ] **Step 2: POST-Routen ergänzen**

Native `Edit` — füge im `if (method == "POST") {`-Block nach der `/insertAfterSymbol`-Zeile ein:

```kotlin
            if (path == "/renamePreview") return dispatchRenamePreview(body)
            if (path == "/renameApply") return dispatchRenameApply(body)
```

- [ ] **Step 3: Dispatch-Funktionen ergänzen**

Native `Edit` — füge nach `dispatchEdit(...)` (vor `private fun q(...)`) ein:

```kotlin
    private fun dispatchRenamePreview(body: String): HttpResult = try {
        val req = JsonCodec.parseRenamePreviewRequest(body)
        HttpResult(200, JsonCodec.toJson(refactorHandlers.renamePreview(req)))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code)) // fachlicher Negativfall = 200
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("renamePreview endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }

    private fun dispatchRenameApply(body: String): HttpResult = try {
        val req = JsonCodec.parseRenameApplyRequest(body)
        HttpResult(200, JsonCodec.toJson(refactorHandlers.renameApply(req)))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("renameApply endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }
```

- [ ] **Step 4: Build prüfen**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: kompiliert.

- [ ] **Step 5: Reformat + Commit (Phase 8)**

```bash
# reformat via mcp__jetbrains__reformat_file auf RefactorHandlers.kt + RequestRouter.kt
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/RefactorHandlers.kt \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt
git commit -m "feat(jetbrains): v2b RefactorHandlers + router wiring"
```

---

# PHASE 9 — Kotlin Akzeptanz-Tests + manuelles `runIde`-Gate

Commit-Message: `test(jetbrains): v2b rename router + acceptance tests`

### Task 14: `RequestRouterRefactorTest.kt` — Multi-File-Rename (Kotlin-Akzeptanz-Gate)

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterRefactorTest.kt`

- [ ] **Step 1: Akzeptanz-Test schreiben (Multi-File-Rename über die Router-Naht)**

Native `Write` — Muster aus `RequestRouterEditTest.kt` (on-disk Fixture via `project.basePath`, `LocalFileSystem.refreshAndFindFileByPath`):

```kotlin
package com.leanctx.plugin.server

import com.intellij.openapi.application.WriteAction
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.nio.file.Files
import java.nio.file.Paths

class RequestRouterRefactorTest : BasePlatformTestCase() {

    private fun router() = RequestRouter(
        token = "tok",
        ideVersion = "IC-2026.1",
        projectName = project.name,
        project = project,
    )

    private fun writeFile(rel: String, content: String): String {
        val base = project.basePath!!
        val p = Paths.get(base, rel)
        Files.createDirectories(p.parent)
        Files.writeString(p, content)
        WriteAction.computeAndWait<Unit, RuntimeException> {
            LocalFileSystem.getInstance().refreshAndFindFileByPath(p.toString())
        }
        return p.toString()
    }

    fun testRenamePreviewReturnsUsagesAndPlanHash() {
        // Declaration in A.kt + a usage in B.kt (same package).
        writeFile("A.kt", "package p\nclass Widget\n")
        writeFile("B.kt", "package p\nfun use(): Widget = Widget()\n")

        // Target = the `Widget` class declaration: line 1 (0-based), char 6 (after "class ").
        val body = """
            {"path":"A.kt",
             "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
             "new_name":"Gadget"}
        """.trimIndent()

        val res = router().route("POST", "/renamePreview", "tok", body)
        assertEquals(res.body, 200, res.status)
        assertTrue(res.body, res.body.contains("\"usages\""))
        // At least the declaration + the two B.kt references appear.
        assertTrue(res.body, res.body.contains("B.kt"))
    }

    fun testRenameApplyRenamesAcrossFiles() {
        val aPath = writeFile("A.kt", "package p\nclass Widget\n")
        val bPath = writeFile("B.kt", "package p\nfun use(): Widget = Widget()\n")

        val body = """
            {"path":"A.kt",
             "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
             "new_name":"Gadget","force":false}
        """.trimIndent()

        val res = router().route("POST", "/renameApply", "tok", body)
        assertEquals(res.body, 200, res.status)
        assertTrue(res.body, res.body.contains("\"applied\":true"))

        // Re-read both files from disk: declaration + usages must be Gadget now.
        WriteAction.computeAndWait<Unit, RuntimeException> {
            LocalFileSystem.getInstance().refreshAndFindFileByPath(aPath)
            LocalFileSystem.getInstance().refreshAndFindFileByPath(bPath)
        }
        val a = Files.readString(Paths.get(aPath))
        val b = Files.readString(Paths.get(bPath))
        assertTrue(a, a.contains("class Gadget"))
        assertTrue(b, b.contains("Gadget"))
        assertFalse(b, b.contains("Widget"))
    }

    fun testUnauthorizedTokenRejected() {
        val res = router().route("POST", "/renamePreview", "wrong", "{}")
        assertEquals(401, res.status)
    }
}
```

- [ ] **Step 2: Test ausführen, beobachten**

Run: `./gradlew test --tests '*RequestRouterRefactorTest'` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS. **Falls die `RenameProcessor`-Naht (showConflicts/preprocessUsages-Sichtbarkeit) in der Test-SDK abweicht**, schlägt der Compile/Run hier zuerst sichtbar fehl → folge dem SDK-Naht-Hinweis (Phase 7): auf `RefactoringFactory.createRename(...).findUsages()` + `run()` umstellen, Konflikt-Sammlung via `RenameUtil`. Danach Test erneut laufen lassen.

> Dies ist das **primäre Akzeptanz-Gate Kotlin** (Spec §10): Rename eines Symbols
> mit Usages über **mehrere** Dateien → alle Deklarationen + Referenzen korrekt
> umbenannt, Ergebnis von Platte verifiziert.

- [ ] **Step 3: Konflikt- + INDEXING-Fälle ergänzen (sofern im Fixture darstellbar)**

Native `Edit` — füge in `RequestRouterRefactorTest.kt` einen Konflikt-Test hinzu (konstruierte Namenskollision):

```kotlin
    fun testRenamePreviewSurfacesConflict() {
        // Rename `Widget` → `Existing`, where `Existing` already exists in scope.
        writeFile("C.kt", "package p\nclass Widget\nclass Existing\n")
        val body = """
            {"path":"C.kt",
             "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":12}},
             "new_name":"Existing"}
        """.trimIndent()
        val res = router().route("POST", "/renamePreview", "tok", body)
        assertEquals(res.body, 200, res.status)
        assertTrue(res.body, res.body.contains("\"conflicts\""))
    }
```

Run: `./gradlew test --tests '*RequestRouterRefactorTest'` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS. Falls IntelliJ die konkrete Kollision nicht als `preprocessUsages`-Konflikt meldet (sprach-/typabhängig), markiere diesen Sub-Test als `@Ignore` mit Begründung und verschiebe die Konflikt-Verifikation ins manuelle `runIde`-Gate (Step 5).

- [ ] **Step 4: Reformat + Commit (Phase 9)**

```bash
# reformat via mcp__jetbrains__reformat_file auf RequestRouterRefactorTest.kt
git add packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterRefactorTest.kt
git commit -m "test(jetbrains): v2b rename router + acceptance tests"
```

- [ ] **Step 5: Manuelles `runIde`-Gate (nicht automatisiert — Checkliste)**

Diese Schritte erfordern eine laufende IDE (Spec §10 „manuelles `runIde`-Gate"). Dokumentiere die Ergebnisse in der finalen Commit-/PR-Beschreibung:

1. `./gradlew runIde` (cwd=`packages/jetbrains-lean-ctx`) startet eine Sandbox-IDE; öffne ein Kotlin-Multi-Modul-Projekt.
2. Via `ctx_refactor action=rename_preview name_path=Widget new_name=Gadget` → Plan mit Usages über mehrere Dateien + `plan_hash`.
3. `ctx_refactor action=rename_apply name_path=Widget new_name=Gadget plan_hash=<aus preview>` → alle Stellen umbenannt; **ein** Undo-Eintrag (Strg+Z macht den kompletten Rename rückgängig).
4. Quelle zwischen preview und apply ändern → `rename_apply` mit altem `plan_hash` → `CONFLICT` (TOCTOU).
5. Konstruierte Namenskollision → `rename_preview` zeigt `conflicts`; `rename_apply` ohne `force` → `CONFLICT`; mit `force=true` → durchgereicht.
6. Während laufender Indizierung (großes Projekt frisch öffnen) → `rename_preview` → `INDEXING`, **kein** Teil-Rename.
7. Datei in Sprache ohne `RenamePsiElementProcessor` (z.B. plain text) → `UNSUPPORTED_LANGUAGE`, kein Crash.
8. IDE schließen, `rename_preview`/`rename_apply` erneut → `BACKEND_REQUIRED` in beiden Phasen.
9. Java optionaler Sekundär-Check (nicht akzeptanzkritisch): ein Java-Symbol-Rename über mehrere Dateien.

---

# PHASE 10 — Final-Gate + Merge-Vorbereitung

### Task 15: Gesamtverifikation

- [ ] **Step 1: Volle Rust-Suite**

Run: `cargo nextest run -p lean-ctx` (cwd=`rust`)
Expected: PASS, keine Regression.

- [ ] **Step 2: Volle Kotlin-Suite**

Run: `./gradlew test` (cwd=`packages/jetbrains-lean-ctx`)
Expected: PASS.

- [ ] **Step 3: Clippy sauber**

Run: `cargo clippy -p lean-ctx --all-targets` (cwd=`rust`)
Expected: keine neuen Warnungen (Projekt hält pedantic clippy silent — siehe Commit `24a342e9`).

- [ ] **Step 4: Drift-Test final**

Run: `cargo nextest run -p lean-ctx generated_reference_docs_are_committed_and_current` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 5: Commit-Historie prüfen**

Run: `git log --oneline -10` (cwd=Projekt-Root, via `ctx_shell`) — neun saubere Phasen-Commits, einer pro Phase (Phase 9+10 teilen sich ggf. den Test-Commit).

- [ ] **Step 6: Branch-Abschluss**

Per Spec §12: finaler Merge nach `main` via Squash-Merge-PR (am Schluss, **nach** Review). Nutze `superpowers:finishing-a-development-branch` für die Integrations-Entscheidung. v2c (`move`/`safe_delete`/`inline`) folgt als eigener Spec auf derselben Engine (Spec §11) — **nicht** Teil dieses Plans.

---

## Self-Review (Spec-Abdeckung)

| Spec-Abschnitt | Abgedeckt durch |
| §2 Entscheidung 1 (Engine + rename) | Alle Phasen; v2c explizit ausgeschlossen (Task 15 Step 6) |
| §2 Entscheidung 2 (Two-Phase) | Task 6 (`rename_preview`/`rename_apply`), Task 14 |
| §2 Entscheidung 3 (Plan meldet, Apply blockt) | Task 5 (Konflikt-Gate `conflicts≠∅ ∧ ¬force`) |
| §2 Entscheidung 4 (stateless `plan_hash` BLAKE3) | Task 3 (`plan_hash`), Task 5 (TOCTOU-Gate) |
| §2 Entscheidung 5/6 (zwei Actions in `ctx_refactor`) | Task 6 (Dispatch), Task 7 (Schema) |
| §3.1 mehrstufiges `select_backend`/`BACKEND_REQUIRED` | Task 4 (`live_jetbrains_backend`), Task 1 (Trait-Default) |
| §5.1 Auflösung + PathJail (zweistufig) | Task 6 (Stage a), Task 3 `usage_range_text` (Stage b) |
| §5.2 `plan_hash` Bildung/Prüfung | Task 3, Task 5 |
| §5.3 Trait-Methoden (Default = `Err`) | Task 1 |
| §5.4 Änderungsstellen Rust | Tasks 1-7 (alle 5 Dateien) |
| §6 Plugin (SymbolRefactorer, Handler, Threading, Sprach-Fallback) | Tasks 10-13; Sprach-Fallback via `UNSUPPORTED_LANGUAGE` (SDK-Naht) + INDEXING via `inSmartReadAction` |
| §7 Wire-Protokoll + Fehler | Task 9 (DTOs), Task 2 (Rust-Parsing), Fehler-Codes durchgängig |
| §8 rename-Semantik (Serena-Parität) | Task 6 (name_path primär), Task 14 |
| §9 Multi-File-Cache-Kohärenz | Task 5 (`cli_cache::invalidate` je `changed_path`) |
| §10 Verifikation | Tasks 1-14 (Unit) + Task 14 Step 5 (manuelles Gate) |
| §12 Branch/Commit + Schema-Drift-Gate | Phasen-Commits, Task 8 (gen_docs) |
| §13 YAGNI (kein Limit/State/Headless/Plugin-Hash/Auto-Reformat) | Eingehalten: kein Blast-Limit, stateless, kein Headless-Apply, Hash nur in Rust, kein Reformat im Apply |

**Offene SDK-Naht (bewusst):** Die genaue Sichtbarkeit von `RenameProcessor.findUsages`/`preprocessUsages`/`showConflicts` in der Ziel-SDK ist erst am `runIde`/`BasePlatformTestCase`-Gate (Task 10 Step 2, Task 14) verifizierbar. Der Plan gibt den primären Pfad (Subklasse) **und** den dokumentierten Fallback (`RefactoringFactory.createRename`) vor — kein Platzhalter, sondern eine benannte Integrations-Entscheidung mit zwei konkreten Optionen.
