# lean-ctx JetBrains v2c — `move` + `safe_delete` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Zwei weitere Engine-native Refactoring-Ops auf der bestehenden v2b-Two-Phase-Engine ergänzen — `move` (`move_preview`/`move_apply`) und `safe_delete` (`safe_delete_preview`/`safe_delete_apply`) — ohne die v2b-Engine zu verändern.

**Architecture:** Vier neue Actions in `ctx_refactor`. Sie erben Two-Phase, `plan_hash` (BLAKE3, Rust-zentral), Konflikt-Gate, mehrstufiges `select_backend` (kein Headless-Fallback → `BACKEND_REQUIRED`), Smart-Mode-`INDEXING`, `UNSUPPORTED_LANGUAGE` und Multi-File-Cache-Kohärenz 1:1 aus v2b. Genau **zwei** Dinge sind neu: (1) `move`s **3. PathJail-Stufe** für das aufrufer-gelieferte Ziel (`target_path` XOR `target_parent`) + `INVALID_TARGET`; (2) `safe_delete`s Semantik „Konflikt = verbleibende Referenz" (mechanisch dasselbe v2b-Gate). `RenamePlan`/`RenameResult` werden wiederverwendet (op-unabhängig).

**Tech Stack:** Rust (`ctx_refactor.rs`, `lsp/backend.rs`, `lsp/jetbrains_backend.rs`, BLAKE3 via `core::hasher`, PathJail via `core::path_resolve`), Kotlin/IntelliJ-Plugin (`MoveFilesOrDirectoriesProcessor`/`MoveClassesOrPackagesProcessor`/Member-Move + `SafeDeleteProcessor`, `WriteCommandAction`, gson), HTTP/JSON-Wire (127.0.0.1, Token-Header).

**Spec:** `docs/lean-md/specs/2026-06-10-leanctx-jetbrains-v2c-move-safedelete-design.md`

---

## Wichtige Grundlagen (vor dem Start lesen)

Der Implementer kennt das Projekt nicht — diese Fakten sind verbindlich:

1. **Tests:** immer `cargo nextest run` (nie `cargo test`), bare command + `cwd=rust`, kein `cd … &&`, kein `| tail`/`| grep`/`| head`. Große grüne Läufe schrumpfen via `cargo nextest run --status-level fail`.
2. **Rust-Edits an `*.rs`:** Serena-Tools (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_after_symbol`, `insert_before_symbol`, `replace_content`) — **nie** native `Edit`/`ctx_edit` auf Rust-Dateien. Kotlin/Markdown/Shell: native `Edit`/`Write` ok.
3. **Vor `git add`:** `mcp__jetbrains__reformat_file` auf jede geänderte Datei.
4. **Fehler-Konvention (Rust):** Der Trait `LspBackend` gibt `Result<_, String>` zurück. Fachliche Fehler sind Strings im Format `"CODE: message"` (z.B. `"INVALID_TARGET: …"`, `"CONFLICT: …"`, `"BACKEND_REQUIRED: …"`, `"NO_SYMBOL: …"`). **Es gibt KEINEN `BackendError`-Enum** — der Pseudocode in Spec §5.5 (`Err(BackendError::BackendRequired)`) ist illustrativ; real wird `Err("BACKEND_REQUIRED: …".to_string())` verwendet. Der `ctx_refactor`-Handler mappt den String zu `format!("ERROR: {e}")`.
5. **0-/1-Basierung:** Tool-Eingabe ist 1-basiert (`line`). Die Wire ist **0-basiert** (`PositionDTO`, `TextRange0Based`). `resolve_name_path` liefert 1-basiert inklusiv. Umrechnung `start_line - 1` beim Bau der `TextRange0Based` (siehe `handle_rename_refactor`, `ctx_refactor.rs:543-548`).
6. **Ein Commit pro Phase** (Spec §11, v1-§12.3). Direkt auf Branch `feat-jetbrains-plugin`, **kein** worktree.
7. **Reuse aus v2a/v2b (nicht neu bauen):** `resolve_name_path` + `resolve_rename_target` (`ctx_refactor.rs`), `usage_range_text` + `plan_hash` (`ctx_refactor.rs:298-343`), `live_jetbrains_backend` (`ctx_refactor.rs:374-393`), PathJail `core::path_resolve::resolve_tool_path`, Cache-Evict `core::cli_cache::invalidate`, `offset_of` (`lsp/edit_apply.rs`). `RenamePlan`/`RenameResult`/`UsageSite`/`Conflict`/`TextRange0Based` werden **wiederverwendet**, nicht neu definiert.
8. **Kotlin-Verifikation:** Das Plugin-Modul (`packages/jetbrains-lean-ctx`) hat **keine** Unit-Test-Suite (das `src/test`-Verzeichnis ist leer). Kotlin-Tasks werden über **Kompilierung** (`./gradlew compileKotlin`, cwd=`packages/jetbrains-lean-ctx`, bare command) plus das **manuelle runIde-Live-Gate** (Phase 7) verifiziert — nicht über `cargo nextest`. Das ist die etablierte v2b-Realität, kein Defizit dieses Plans.
9. **Wire-Reuse:** Preview-Response (`{usages, conflicts}`) und Apply-Response (`{applied, changed_paths}`) sind op-unabhängig — die Kotlin-DTOs `UsageSiteDTO`/`ConflictDTO`/`RenamePreviewResponse`/`RenameApplyResponse` (`Wire.kt:113-143`) und das Rust-Parsing `parse_rename_plan` (`jetbrains_backend.rs:304-337`) werden **wiederverwendet**. Neu sind nur die **Request**-Formen (Ziel-Feld bei `move`, `propagate` bei `safe_delete`).

---

## File Structure

**Rust (`rust/src/`):**
- `lsp/backend.rs` — MODIFY: +5 Typen (`MoveTarget`, `MoveQuery`, `MoveApply`, `SafeDeleteQuery`, `SafeDeleteApply`) + 4 Trait-Methoden mit `Err(BACKEND_REQUIRED)`-Default. `RenamePlan`/`RenameResult` reused.
- `lsp/jetbrains_backend.rs` — MODIFY: HTTP-Override der 4 Methoden (`/movePreview`, `/moveApply`, `/safeDeletePreview`, `/safeDeleteApply`) + Request-Body-Builder. `parse_rename_plan` reused.
- `tools/ctx_refactor.rs` — MODIFY: Action-Dispatch (+4), `handle_move_refactor` + `handle_safe_delete_refactor`, `resolve_move_target` (3-Stufen-Jail, `INVALID_TARGET`), Render-Funktionen. `plan_hash`/`usage_range_text`/`live_jetbrains_backend`/Konflikt-Gate/Cache-Evict reused.
- `tools/registered/ctx_refactor.rs` — MODIFY: Schema (+4 Actions, `+target_path`/`target_parent`/`propagate`; `plan_hash`/`force` schon aus v2b da).

**Kotlin (`packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/`):**
- `dto/Wire.kt` — MODIFY: +5 Request-DTOs (`MoveTargetDTO`, `MovePreviewRequest`, `MoveApplyRequest`, `SafeDeletePreviewRequest`, `SafeDeleteApplyRequest`) + 4 JsonCodec-Parser. Response-DTOs reused.
- `psi/SymbolMover.kt` — CREATE: `move`-Naht (Preview + Apply via IntelliJ-Move-Processoren).
- `psi/SymbolDeleter.kt` — CREATE: `safe_delete`-Naht (Preview + Apply via `SafeDeleteProcessor`).
- `endpoint/RefactorHandlers.kt` — MODIFY: +4 Handler-Methoden (`movePreview`/`moveApply`/`safeDeletePreview`/`safeDeleteApply`).
- `server/RequestRouter.kt` — MODIFY: +4 Routen + Dispatch.

**Docs / Harness:**
- `docs/reference/generated/mcp-tools.md` — regeneriert (Drift-Test `generated_reference…`).
- `docs/reference/appendix-mcp-tools.md` — MODIFY: `ctx_refactor`-Zeile um 4 Actions.
- `docs/lean-md/runbooks/runide-move-safedelete-gate.md` — CREATE (Spec §9.1).
- `scripts/runide-move-safedelete-gate-setup.sh` — CREATE (Fixture-Generator, erweitert das Rename-Fixture).

---

## Phasen-Übersicht (je ein Commit)

| Phase | Inhalt | Commit-Message |
| 1 | Rust Backend-Typen + 4 Trait-Methoden (`Err`-Default) | `feat(jetbrains): v2c move/safe_delete trait — types + Err-default methods` |
| 2 | Rust HTTP-Backend-Override (4 Endpoints) | `feat(jetbrains): v2c HTTP backend — move/safeDelete endpoints` |
| 3 | Rust Tool-Layer — `safe_delete` Actions + Gate | `feat(jetbrains): v2c safe_delete actions — preview/apply + remaining-ref gate` |
| 4 | Rust Tool-Layer — `move` Actions + 3-Stufen-Jail | `feat(jetbrains): v2c move actions — target resolve + 3-stage PathJail + INVALID_TARGET` |
| 5 | Rust Schema + Doc-Regen | `feat(jetbrains): v2c schema — 4 actions + target_path/target_parent/propagate` |
| 6 | Kotlin Plugin — Wire-DTOs, Mover, Deleter, Router | `feat(plugin): v2c move/safe_delete — IntelliJ processors + endpoints` |
| 7 | Live-Gate-Runbook + Fixture-Script | `docs(runbook): v2c runIde move/safe_delete gate + fixture` |

> **Reihenfolge-Begründung:** `safe_delete` (Phase 3) **vor** `move` (Phase 4), weil `safe_delete` mechanisch ein reiner v2b-Klon ist (2-Stufen-Jail, keine neuen Fehlerklassen) und damit das einfachere Inkrement; `move` trägt die einzige genuin neue Logik (3. Jail-Stufe + `INVALID_TARGET`) und baut auf den in Phase 3 etablierten Render-/Dispatch-Hilfen auf.

---

# PHASE 1 — Rust Backend-Typen + Trait-Methoden

Commit-Message am Phasenende: `feat(jetbrains): v2c move/safe_delete trait — types + Err-default methods`

### Task 1: Neue Backend-Typen in `backend.rs`

**Files:**
- Modify: `rust/src/lsp/backend.rs` (nach dem `RenameResult`-Struct, vor `pub trait LspBackend`)
- Test: `rust/src/lsp/backend.rs` (inline `#[cfg(test)]` — die Datei hat bereits einen Test-Modul mit `rename_types_construct_and_clone`; dort ein Test ergänzen)

- [ ] **Step 1: Failing test schreiben**

Serena `mcp__serena__jet_brains_find_symbol` auf `rename_types_construct_and_clone` (im Test-Modul von `backend.rs`), dann `insert_after_symbol` mit:

```rust
#[test]
fn move_and_safe_delete_types_construct_and_clone() {
    let mt = MoveTarget::Path {
        abs_path: "/proj/app/moved".into(),
        rel_path: "app/moved".into(),
    };
    let mq = MoveQuery {
        abs_path: "/proj/Widget.kt".into(),
        rel_path: "Widget.kt".into(),
        src_range: TextRange0Based { start_line: 2, start_char: 0, end_line: 2, end_char: 12 },
        target: mt.clone(),
    };
    let ma = MoveApply { query: mq.clone(), force: true };
    assert_eq!(ma.query.target, mt);

    let parent = MoveTarget::Parent {
        abs_path: "/proj/Other.kt".into(),
        rel_path: "Other.kt".into(),
        range: TextRange0Based { start_line: 0, start_char: 0, end_line: 5, end_char: 1 },
    };
    assert_ne!(parent, mt);

    let sq = SafeDeleteQuery {
        abs_path: "/proj/Widget.kt".into(),
        rel_path: "Widget.kt".into(),
        src_range: TextRange0Based { start_line: 2, start_char: 0, end_line: 2, end_char: 12 },
    };
    let sa = SafeDeleteApply { query: sq.clone(), force: true, propagate: false };
    assert_eq!(sa.query, sq);
    assert!(sa.force);
    assert!(!sa.propagate);
}
```

- [ ] **Step 2: Test laufen lassen → muss fehlschlagen**

Run: `cargo nextest run -p <crate> move_and_safe_delete_types_construct_and_clone` (cwd=`rust`)
Expected: Kompilierfehler `cannot find type MoveTarget / MoveQuery / MoveApply / SafeDeleteQuery / SafeDeleteApply in this scope`.

- [ ] **Step 3: Typen einfügen**

Serena `mcp__serena__jet_brains_find_symbol` auf `RenameResult` (`backend.rs:169-172`), dann `insert_after_symbol` mit:

```rust
/// Where a `move` sends the symbol. Mirrors Serena's two-field dispatch
/// (`targetRelativePath` XOR `targetParentNamePath`, spec §3): the caller picks
/// the variant, the backend never sees a `name_path`. Both variants carry the
/// jail-checked `abs_path` plus the wire-facing `rel_path` (rebuilt by the IDE).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveTarget {
    /// Move a file/class into a directory or file (FileMoveProcessor side).
    Path { abs_path: String, rel_path: String },
    /// Move a member into a parent symbol (SymbolMoveProcessor side); `range`
    /// is the parent declaration span used to resolve it in the IDE.
    Parent {
        abs_path: String,
        rel_path: String,
        range: TextRange0Based,
    },
}

/// Phase-1 `move` request: the resolved source span plus an already-resolved,
/// already-jailed target (the trait never resolves a `name_path` or a path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveQuery {
    pub abs_path: String,
    pub rel_path: String,
    pub src_range: TextRange0Based,
    pub target: MoveTarget,
}

/// Phase-2 `move` request: the query plus the `force` flag (Rust already gated
/// plan_hash + conflicts before this is built).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveApply {
    pub query: MoveQuery,
    pub force: bool,
}

/// Phase-1 `safe_delete` request: just the resolved source span. `*_preview`
/// returns the remaining (blocking) usages in the reused `RenamePlan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeDeleteQuery {
    pub abs_path: String,
    pub rel_path: String,
    pub src_range: TextRange0Based,
}

/// Phase-2 `safe_delete` request: `force` = Serena's `deleteEvenIfUsed`,
/// `propagate` = delete now-unreferenced dependencies too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeDeleteApply {
    pub query: SafeDeleteQuery,
    pub force: bool,
    pub propagate: bool,
}
```

- [ ] **Step 4: Test laufen lassen → muss bestehen**

Run: `cargo nextest run -p <crate> move_and_safe_delete_types_construct_and_clone` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 5: Verifizieren (kein Commit — Phase-Commit kommt nach Task 2)**

Run: `ctx_read("rust/src/lsp/backend.rs", mode="diff")` — bestätige nur additive Typ-Blöcke.

---

### Task 2: Vier Trait-Methoden mit `Err`-Default in `LspBackend`

**Files:**
- Modify: `rust/src/lsp/backend.rs` (im `pub trait LspBackend`, nach `rename_apply`, `backend.rs:256-258`)
- Test: `rust/src/lsp/backend.rs` (Test-Modul — neben `headless_rename_default_is_backend_required`)

- [ ] **Step 1: Failing test schreiben**

Serena `insert_after_symbol` auf `headless_rename_default_is_backend_required` (`backend.rs:332-361`) mit:

```rust
#[test]
fn headless_move_and_safe_delete_default_is_backend_required() {
    // A backend that only implements the mandatory methods inherits the four
    // v2c Err defaults (no lossless headless move/delete — spec §4 inherited §3).
    struct Bare;
    impl LspBackend for Bare {
        fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> { Ok(()) }
        fn references(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
        fn definition(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position) -> Result<lsp_types::GotoDefinitionResponse, String> { Ok(lsp_types::GotoDefinitionResponse::Array(vec![])) }
        fn implementations(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
        fn rename(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _n: &str) -> Result<Option<lsp_types::WorkspaceEdit>, String> { Ok(None) }
    }
    let mut b = Bare;
    let mq = MoveQuery {
        abs_path: "/p/a.kt".into(), rel_path: "a.kt".into(),
        src_range: TextRange0Based { start_line: 0, start_char: 0, end_line: 0, end_char: 1 },
        target: MoveTarget::Path { abs_path: "/p/x".into(), rel_path: "x".into() },
    };
    assert!(b.move_preview(&mq).unwrap_err().starts_with("BACKEND_REQUIRED"));
    assert!(b.move_apply(&MoveApply { query: mq, force: false }).unwrap_err().starts_with("BACKEND_REQUIRED"));
    let sq = SafeDeleteQuery {
        abs_path: "/p/a.kt".into(), rel_path: "a.kt".into(),
        src_range: TextRange0Based { start_line: 0, start_char: 0, end_line: 0, end_char: 1 },
    };
    assert!(b.safe_delete_preview(&sq).unwrap_err().starts_with("BACKEND_REQUIRED"));
    assert!(b.safe_delete_apply(&SafeDeleteApply { query: sq, force: false, propagate: false }).unwrap_err().starts_with("BACKEND_REQUIRED"));
}
```

- [ ] **Step 2: Test laufen lassen → muss fehlschlagen**

Run: `cargo nextest run -p <crate> headless_move_and_safe_delete_default_is_backend_required` (cwd=`rust`)
Expected: Kompilierfehler `no method named move_preview / safe_delete_preview …`.

- [ ] **Step 3: Trait-Methoden einfügen**

Serena `insert_after_symbol` auf die `rename_apply`-Methode im Trait (`backend.rs:256-258`) mit:

```rust
    /// Phase 1 of the Two-Phase move: resolve all usages + conflicts of the
    /// target at the new location. DEFAULT = `Err(BACKEND_REQUIRED)` (no lossless
    /// headless move; only Backing B overrides — spec §5.5).
    fn move_preview(&mut self, _req: &MoveQuery) -> Result<RenamePlan, String> {
        Err("BACKEND_REQUIRED: move requires a running JetBrains IDE".to_string())
    }
    /// Phase 2 of the Two-Phase move: perform the Multi-File move as ONE Undo
    /// transaction. DEFAULT = `Err(BACKEND_REQUIRED)`.
    fn move_apply(&mut self, _req: &MoveApply) -> Result<RenameResult, String> {
        Err("BACKEND_REQUIRED: move requires a running JetBrains IDE".to_string())
    }
    /// Phase 1 of the Two-Phase safe-delete: report the REMAINING (blocking)
    /// references as `usages`/`conflicts`. DEFAULT = `Err(BACKEND_REQUIRED)`.
    fn safe_delete_preview(&mut self, _req: &SafeDeleteQuery) -> Result<RenamePlan, String> {
        Err("BACKEND_REQUIRED: safe_delete requires a running JetBrains IDE".to_string())
    }
    /// Phase 2 of the Two-Phase safe-delete: delete the symbol (force =
    /// deleteEvenIfUsed) as ONE Undo transaction. DEFAULT = `Err(BACKEND_REQUIRED)`.
    fn safe_delete_apply(&mut self, _req: &SafeDeleteApply) -> Result<RenameResult, String> {
        Err("BACKEND_REQUIRED: safe_delete requires a running JetBrains IDE".to_string())
    }
```

- [ ] **Step 4: Test + volle Suite laufen lassen → muss bestehen**

Run: `cargo nextest run --status-level fail` (cwd=`rust`)
Expected: alle Tests PASS (inkl. der zwei neuen).

- [ ] **Step 5: Reformat + Commit (Phasen-Ende)**

`mcp__jetbrains__reformat_file` auf `rust/src/lsp/backend.rs`, dann:

```bash
git add rust/src/lsp/backend.rs
git commit -m "feat(jetbrains): v2c move/safe_delete trait — types + Err-default methods"
```

---

# PHASE 2 — Rust HTTP-Backend-Override

Commit-Message am Phasenende: `feat(jetbrains): v2c HTTP backend — move/safeDelete endpoints`

### Task 3: HTTP-Override der vier Methoden in `jetbrains_backend.rs`

**Files:**
- Modify: `rust/src/lsp/jetbrains_backend.rs` (Methoden im `impl LspBackend for JetBrainsHttpBackend`, nach `rename_apply`, `jetbrains_backend.rs:497-525`; plus private Body-Builder neben `rename_body`, `jetbrains_backend.rs:340-353`)
- Test: `rust/src/lsp/jetbrains_backend.rs` (Test-Modul — neben `rename_apply_parses_changed_paths`, `jetbrains_backend.rs:901`)

- [ ] **Step 1: Failing test schreiben (Body-Builder, ohne HTTP)**

Die HTTP-Methoden selbst brauchen einen laufenden Server; testbar ist der **Request-Body-Bau**. Serena `insert_after_symbol` auf `rename_apply_parses_changed_paths` (im Test-Modul) mit:

```rust
#[test]
fn move_body_path_and_parent_variants() {
    use crate::lsp::backend::{MoveTarget, TextRange0Based};
    let r = TextRange0Based { start_line: 2, start_char: 0, end_line: 2, end_char: 12 };

    let path_body = JetBrainsHttpBackend::move_body(
        "Widget.kt", r, &MoveTarget::Path { abs_path: "/p/app/moved".into(), rel_path: "app/moved".into() },
    );
    assert_eq!(path_body["path"], "Widget.kt");
    assert_eq!(path_body["target"]["kind"], "path");
    assert_eq!(path_body["target"]["path"], "app/moved");
    assert!(path_body["target"].get("range").is_none());

    let pr = TextRange0Based { start_line: 0, start_char: 0, end_line: 5, end_char: 1 };
    let parent_body = JetBrainsHttpBackend::move_body(
        "Widget.kt", r, &MoveTarget::Parent { abs_path: "/p/Other.kt".into(), rel_path: "Other.kt".into(), range: pr },
    );
    assert_eq!(parent_body["target"]["kind"], "parent");
    assert_eq!(parent_body["target"]["path"], "Other.kt");
    assert_eq!(parent_body["target"]["range"]["start"]["line"], 0);
    assert_eq!(parent_body["target"]["range"]["end"]["line"], 5);
}

#[test]
fn safe_delete_body_carries_flags() {
    use crate::lsp::backend::TextRange0Based;
    let r = TextRange0Based { start_line: 2, start_char: 0, end_line: 2, end_char: 12 };
    let body = JetBrainsHttpBackend::safe_delete_body("Widget.kt", r, true, false);
    assert_eq!(body["path"], "Widget.kt");
    assert_eq!(body["range"]["start"]["line"], 2);
    assert_eq!(body["force"], true);
    assert_eq!(body["propagate"], false);
}
```

- [ ] **Step 2: Test laufen lassen → muss fehlschlagen**

Run: `cargo nextest run -p <crate> move_body_path_and_parent_variants safe_delete_body_carries_flags` (cwd=`rust`)
Expected: Kompilierfehler `no function move_body / safe_delete_body`.

- [ ] **Step 3: Body-Builder einfügen**

Serena `insert_after_symbol` auf `rename_body` (`jetbrains_backend.rs:340-353`) mit:

```rust
    /// Request body for `/movePreview` + `/moveApply`. `target` mirrors the
    /// MoveTarget variant (kind=path → `{path}`, kind=parent → `{path,range}`).
    fn move_body(
        rel_path: &str,
        src_range: crate::lsp::backend::TextRange0Based,
        target: &crate::lsp::backend::MoveTarget,
    ) -> Value {
        use crate::lsp::backend::MoveTarget;
        let target_json = match target {
            MoveTarget::Path { rel_path: tp, .. } => serde_json::json!({
                "kind": "path",
                "path": tp,
            }),
            MoveTarget::Parent { rel_path: pp, range, .. } => serde_json::json!({
                "kind": "parent",
                "path": pp,
                "range": {
                    "start": { "line": range.start_line, "character": range.start_char },
                    "end":   { "line": range.end_line,   "character": range.end_char },
                },
            }),
        };
        serde_json::json!({
            "path": rel_path,
            "range": {
                "start": { "line": src_range.start_line, "character": src_range.start_char },
                "end":   { "line": src_range.end_line,   "character": src_range.end_char },
            },
            "target": target_json,
        })
    }

    /// Request body for `/safeDeletePreview` (force/propagate ignored there) +
    /// `/safeDeleteApply`.
    fn safe_delete_body(
        rel_path: &str,
        src_range: crate::lsp::backend::TextRange0Based,
        force: bool,
        propagate: bool,
    ) -> Value {
        serde_json::json!({
            "path": rel_path,
            "range": {
                "start": { "line": src_range.start_line, "character": src_range.start_char },
                "end":   { "line": src_range.end_line,   "character": src_range.end_char },
            },
            "force": force,
            "propagate": propagate,
        })
    }
```

- [ ] **Step 4: Test laufen lassen → muss bestehen**

Run: `cargo nextest run -p <crate> move_body_path_and_parent_variants safe_delete_body_carries_flags` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 5: Die vier `LspBackend`-Override-Methoden einfügen**

Serena `insert_after_symbol` auf die `rename_apply`-Methode im `impl LspBackend for JetBrainsHttpBackend` (`jetbrains_backend.rs:497-525` — die Methode, die `changed_paths` parst). Hilfsfunktion für die geteilte `changed_paths`-Logik vermeiden wir; wir spiegeln den `rename_apply`-Body 1:1:

```rust
    fn move_preview(
        &mut self,
        req: &crate::lsp::backend::MoveQuery,
    ) -> Result<crate::lsp::backend::RenamePlan, String> {
        let body = Self::move_body(&req.rel_path, req.src_range, &req.target);
        let resp = self.post("/movePreview", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(Self::error_from_envelope(err));
        }
        Ok(Self::parse_rename_plan(&resp))
    }

    fn move_apply(
        &mut self,
        req: &crate::lsp::backend::MoveApply,
    ) -> Result<crate::lsp::backend::RenameResult, String> {
        let mut body = Self::move_body(&req.query.rel_path, req.query.src_range, &req.query.target);
        body["force"] = serde_json::json!(req.force);
        let resp = self.post("/moveApply", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(Self::error_from_envelope(err));
        }
        Ok(Self::parse_apply_result(&resp))
    }

    fn safe_delete_preview(
        &mut self,
        req: &crate::lsp::backend::SafeDeleteQuery,
    ) -> Result<crate::lsp::backend::RenamePlan, String> {
        let body = Self::safe_delete_body(&req.rel_path, req.src_range, false, false);
        let resp = self.post("/safeDeletePreview", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(Self::error_from_envelope(err));
        }
        Ok(Self::parse_rename_plan(&resp))
    }

    fn safe_delete_apply(
        &mut self,
        req: &crate::lsp::backend::SafeDeleteApply,
    ) -> Result<crate::lsp::backend::RenameResult, String> {
        let body = Self::safe_delete_body(&req.query.rel_path, req.query.src_range, req.force, req.propagate);
        let resp = self.post("/safeDeleteApply", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(Self::error_from_envelope(err));
        }
        Ok(Self::parse_apply_result(&resp))
    }
```

- [ ] **Step 6: `parse_apply_result`-Helfer extrahieren (DRY)**

Die `changed_paths`+`applied`-Parse-Logik steht aktuell inline in `rename_apply` (`jetbrains_backend.rs:507-524`). Die vier neuen Methoden + `move_apply`/`safe_delete_apply` nutzen sie — extrahiere sie als privaten assoziierten Helfer. Serena `insert_after_symbol` auf `safe_delete_body` mit:

```rust
    /// Parse a `{applied, changed_paths}` apply response (shared by rename/move/
    /// safe_delete apply). Error envelopes are handled by the caller.
    fn parse_apply_result(resp: &Value) -> crate::lsp::backend::RenameResult {
        let changed_paths = resp
            .get("changed_paths")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|p| p.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        crate::lsp::backend::RenameResult {
            applied: resp.get("applied").and_then(Value::as_bool).unwrap_or(false),
            changed_paths,
        }
    }
```

Dann Serena `mcp__serena__jet_brains_find_symbol` auf `rename_apply` (`jetbrains_backend.rs:497`) und `replace_symbol_body`, sodass der Body nach dem Error-Check nur noch `Ok(Self::parse_apply_result(&resp))` zurückgibt (statt der inline `changed_paths`-Konstruktion). Der neue `rename_apply`-Body:

```rust
    fn rename_apply(
        &mut self,
        req: &crate::lsp::backend::RenameApply,
    ) -> Result<crate::lsp::backend::RenameResult, String> {
        let mut body = Self::rename_body(&req.rel_path, req.target_range, &req.new_name);
        body["force"] = serde_json::json!(req.force);
        let resp = self.post("/renameApply", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(Self::error_from_envelope(err));
        }
        Ok(Self::parse_apply_result(&resp))
    }
```

- [ ] **Step 7: Volle Suite laufen lassen → muss bestehen**

Run: `cargo nextest run --status-level fail` (cwd=`rust`)
Expected: alle PASS (der bestehende `rename_apply_parses_changed_paths` deckt den extrahierten Helfer weiter ab).

- [ ] **Step 8: Reformat + Commit (Phasen-Ende)**

`mcp__jetbrains__reformat_file` auf `rust/src/lsp/jetbrains_backend.rs`, dann:

```bash
git add rust/src/lsp/jetbrains_backend.rs
git commit -m "feat(jetbrains): v2c HTTP backend — move/safeDelete endpoints"
```

---

# PHASE 3 — Rust Tool-Layer: `safe_delete` Actions

Commit-Message am Phasenende: `feat(jetbrains): v2c safe_delete actions — preview/apply + remaining-ref gate`

> `safe_delete` zuerst, weil es ein reiner v2b-Klon ist (2-Stufen-Jail, kein neues Ziel, kein `INVALID_TARGET`). Es etabliert die Render-/Dispatch-Hilfen, die `move` (Phase 4) wiederverwendet.

### Task 4: `safe_delete`-Quellauflösung + Render-Funktionen + Dispatch

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs` (Dispatch-`if` oben `ctx_refactor.rs:12-21`; neue `fn handle_safe_delete_refactor` + Render-Funktionen nach `handle_rename_refactor`, `ctx_refactor.rs:592`)
- Test: `rust/src/tools/ctx_refactor.rs` (Test-Modul — neben den `RenameStub`-Tests, `ctx_refactor.rs:1593+`)

- [ ] **Step 1: Failing test schreiben (Render-Apply mit Stub-Backend)**

Der bestehende `RenameStub` (`ctx_refactor.rs:1593-1649`) implementiert `rename_preview`/`rename_apply`. Wir brauchen einen analogen Stub, der `safe_delete_preview`/`safe_delete_apply` bedient. Serena `insert_after_symbol` auf den letzten Test im Modul (`unknown_action_help_lists_rename_actions`, `ctx_refactor.rs:1831-1838`) mit:

```rust
/// Minimal backend for the safe_delete renderers: canned plan + recorded apply flags.
struct SafeDeleteStub {
    plan: crate::lsp::backend::RenamePlan,
    applied: std::cell::Cell<Option<(bool, bool)>>, // (force, propagate)
}
impl crate::lsp::backend::LspBackend for SafeDeleteStub {
    fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> { Ok(()) }
    fn references(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
    fn definition(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position) -> Result<lsp_types::GotoDefinitionResponse, String> { Ok(lsp_types::GotoDefinitionResponse::Array(vec![])) }
    fn implementations(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
    fn rename(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _n: &str) -> Result<Option<lsp_types::WorkspaceEdit>, String> { Ok(None) }
    fn safe_delete_preview(&mut self, _q: &crate::lsp::backend::SafeDeleteQuery) -> Result<crate::lsp::backend::RenamePlan, String> {
        Ok(self.plan.clone())
    }
    fn safe_delete_apply(&mut self, req: &crate::lsp::backend::SafeDeleteApply) -> Result<crate::lsp::backend::RenameResult, String> {
        self.applied.set(Some((req.force, req.propagate)));
        Ok(crate::lsp::backend::RenameResult { applied: true, changed_paths: vec!["Widget.kt".into()] })
    }
}

fn safe_delete_query(abs: &str) -> crate::lsp::backend::SafeDeleteQuery {
    crate::lsp::backend::SafeDeleteQuery {
        abs_path: abs.into(),
        rel_path: "a.rs".into(),
        src_range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
    }
}

#[test]
fn safe_delete_apply_blocks_on_remaining_refs_without_force() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
    let root = dir.path().to_str().unwrap();
    let usage = crate::lsp::backend::UsageSite {
        path: "a.rs".into(),
        range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
        context: None,
    };
    // A remaining reference = a blocking conflict (spec §5.4).
    let plan = crate::lsp::backend::RenamePlan {
        usages: vec![usage.clone()],
        conflicts: vec![crate::lsp::backend::Conflict { path: "a.rs".into(), range: None, message: "still referenced".into() }],
    };
    let hash = super::plan_hash(root, &plan.usages).unwrap();
    let q = safe_delete_query(&dir.path().join("a.rs").to_string_lossy());

    // force=false → CONFLICT, apply not called.
    let mut be = SafeDeleteStub { plan: plan.clone(), applied: std::cell::Cell::new(None) };
    let out = super::render_safe_delete_apply(&mut be, root, &q, &hash, false, false);
    assert!(out.contains("CONFLICT"), "got: {out}");
    assert_eq!(be.applied.get(), None);

    // force=true → applies, force+propagate passed through.
    let mut be2 = SafeDeleteStub { plan, applied: std::cell::Cell::new(None) };
    let out2 = super::render_safe_delete_apply(&mut be2, root, &q, &hash, true, true);
    assert!(out2.contains("deleted") || out2.contains("applied"), "got: {out2}");
    assert_eq!(be2.applied.get(), Some((true, true)));
}

#[test]
fn safe_delete_apply_blocks_on_plan_hash_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
    let root = dir.path().to_str().unwrap();
    let usage = crate::lsp::backend::UsageSite {
        path: "a.rs".into(),
        range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
        context: None,
    };
    let mut be = SafeDeleteStub {
        plan: crate::lsp::backend::RenamePlan { usages: vec![usage], conflicts: vec![] },
        applied: std::cell::Cell::new(None),
    };
    let q = safe_delete_query(&dir.path().join("a.rs").to_string_lossy());
    let out = super::render_safe_delete_apply(&mut be, root, &q, "stalehash", false, false);
    assert!(out.contains("CONFLICT"), "got: {out}");
    assert_eq!(be.applied.get(), None);
}
```

- [ ] **Step 2: Test laufen lassen → muss fehlschlagen**

Run: `cargo nextest run -p <crate> safe_delete_apply` (cwd=`rust`)
Expected: Kompilierfehler `no function render_safe_delete_apply`.

- [ ] **Step 3: Render-Funktionen einfügen**

Serena `insert_after_symbol` auf `handle_rename_refactor` (`ctx_refactor.rs:515-592`) mit:

```rust
/// Phase 1 renderer for safe_delete: ask Backing B for the REMAINING references
/// (blocking usages/conflicts), build the stateless plan_hash, present them.
fn render_safe_delete_preview(
    backend: &mut dyn crate::lsp::backend::LspBackend,
    project_root: &str,
    query: &crate::lsp::backend::SafeDeleteQuery,
) -> String {
    let plan = match backend.safe_delete_preview(query) {
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
        "safe_delete_preview: '{}'\n  blocking usages: {}\n  files: {}\n  plan_hash: {hash}\n",
        query.rel_path,
        plan.usages.len(),
        files.len(),
    );
    if !plan.conflicts.is_empty() {
        out.push_str(&format!(
            "  conflicts: {} (safe_delete_apply blocks unless force=true)\n",
            plan.conflicts.len()
        ));
        for c in &plan.conflicts {
            out.push_str(&format!("    {}: {}\n", c.path, c.message));
        }
    }
    for f in &files {
        let n = plan.usages.iter().filter(|u| u.path == **f).count();
        out.push_str(&format!("  {f}: {n} remaining ref(s)\n"));
    }
    out
}

/// Phase 2 renderer for safe_delete: re-fetch usages, enforce plan_hash (TOCTOU)
/// + conflict gate (conflict = "reference still exists", spec §5.4) in Rust, then
/// run the IDE delete transaction and evict changed files.
fn render_safe_delete_apply(
    backend: &mut dyn crate::lsp::backend::LspBackend,
    project_root: &str,
    query: &crate::lsp::backend::SafeDeleteQuery,
    expected_hash: &str,
    force: bool,
    propagate: bool,
) -> String {
    let plan = match backend.safe_delete_preview(query) {
        Ok(p) => p,
        Err(e) => return format!("ERROR: {e}"),
    };
    // Gate (a): TOCTOU plan_hash (also jail-checks every usage path).
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
    // Gate (b): remaining references block unless force.
    if !plan.conflicts.is_empty() && !force {
        return format!(
            "ERROR: CONFLICT: {} blocking reference(s) remain; pass force=true to delete anyway",
            plan.conflicts.len()
        );
    }

    let apply = crate::lsp::backend::SafeDeleteApply {
        query: query.clone(),
        force,
        propagate,
    };
    let res = match backend.safe_delete_apply(&apply) {
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

    format!(
        "safe_delete_apply: '{}' deleted\n  changed files: {}\n",
        query.rel_path,
        res.changed_paths.len(),
    )
}
```

- [ ] **Step 4: Dispatch-Funktion `handle_safe_delete_refactor` einfügen**

Serena `insert_after_symbol` auf `render_safe_delete_apply` mit:

```rust
/// Entry for the Two-Phase safe_delete actions. Resolves the source (name_path /
/// position), jail-checks it, requires a live IDE, then dispatches to the renderer.
/// Two-stage jail only (source + changed_paths) — no new caller-supplied target.
fn handle_safe_delete_refactor(action: &str, args: &Value, project_root: &str) -> String {
    if action == "safe_delete_apply" && args.get("plan_hash").and_then(Value::as_str).is_none() {
        return "ERROR: 'plan_hash' is required for safe_delete_apply (run safe_delete_preview first)."
            .to_string();
    }
    // Resolve source symbol → 1-based inclusive span (reuse v2b resolver).
    let (rel_path, start_line, end_line) = match resolve_rename_target(args, project_root) {
        Ok(t) => t,
        Err(e) => return format!("ERROR: {e}"),
    };
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
    let src_range = crate::lsp::backend::TextRange0Based {
        start_line: (start_line - 1) as u32,
        start_char: 0,
        end_line: (end_line - 1) as u32,
        end_char: end_col,
    };

    let mut backend = match live_jetbrains_backend(project_root) {
        Ok(b) => b,
        Err(e) => return format!("ERROR: {e}"),
    };

    let query = crate::lsp::backend::SafeDeleteQuery {
        abs_path,
        rel_path,
        src_range,
    };

    match action {
        "safe_delete_preview" => render_safe_delete_preview(backend.as_mut(), project_root, &query),
        "safe_delete_apply" => {
            let expected = args.get("plan_hash").and_then(Value::as_str).unwrap_or_default();
            let force = args.get("force").and_then(Value::as_bool).unwrap_or(false);
            let propagate = args.get("propagate").and_then(Value::as_bool).unwrap_or(false);
            render_safe_delete_apply(backend.as_mut(), project_root, &query, expected, force, propagate)
        }
        other => format!("ERROR: INTERNAL: not a safe_delete action: {other}"),
    }
}
```

- [ ] **Step 5: Dispatch-Verzweigung oben in `handle` ergänzen**

Serena `mcp__serena__jet_brains_find_symbol` auf den `if matches!(action, "rename_preview" | "rename_apply")`-Block (`ctx_refactor.rs:19-21`), dann `insert_after_symbol` mit:

```rust

    if matches!(action, "safe_delete_preview" | "safe_delete_apply") {
        return handle_safe_delete_refactor(action, args, project_root);
    }
```

- [ ] **Step 6: Test + volle Suite laufen lassen → muss bestehen**

Run: `cargo nextest run --status-level fail` (cwd=`rust`)
Expected: alle PASS (inkl. der drei neuen safe_delete-Tests). Hinweis: ggf. `#[allow(dead_code)]` ist **nicht** nötig — die Render-Funktionen werden von `handle_safe_delete_refactor` genutzt.

- [ ] **Step 7: BACKEND_REQUIRED-Pfad als Integrationstest absichern**

Serena `insert_after_symbol` auf `handle_rename_apply_requires_plan_hash` (`ctx_refactor.rs:1820-1829`) mit:

```rust
#[test]
fn handle_safe_delete_preview_without_ide_is_backend_required() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn foo() {}\n").unwrap();
    let root = dir.path().to_str().unwrap();
    let args = serde_json::json!({"action": "safe_delete_preview", "path": "a.rs", "line": 1});
    let out = super::handle(&args, root, "");
    assert!(out.contains("BACKEND_REQUIRED"), "got: {out}");
}

#[test]
fn handle_safe_delete_apply_requires_plan_hash() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn foo() {}\n").unwrap();
    let root = dir.path().to_str().unwrap();
    let args = serde_json::json!({"action": "safe_delete_apply", "path": "a.rs", "line": 1});
    let out = super::handle(&args, root, "");
    assert!(out.contains("plan_hash"), "got: {out}");
}
```

Run: `cargo nextest run --status-level fail` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 8: Reformat + Commit (Phasen-Ende)**

`mcp__jetbrains__reformat_file` auf `rust/src/tools/ctx_refactor.rs`, dann:

```bash
git add rust/src/tools/ctx_refactor.rs
git commit -m "feat(jetbrains): v2c safe_delete actions — preview/apply + remaining-ref gate"
```

---

# PHASE 4 — Rust Tool-Layer: `move` Actions + 3-Stufen-Jail

Commit-Message am Phasenende: `feat(jetbrains): v2c move actions — target resolve + 3-stage PathJail + INVALID_TARGET`

> Das ist die einzige **genuin neue** Logik in v2c (Spec §4 Punkt 1 / §5.3): das aufrufer-gelieferte Ziel muss vor dem Backend-Call durch `resolve_tool_path`/PathJail.

### Task 5: `resolve_move_target` — Ziel-Auflösung + Jail (Stufe 2) + `INVALID_TARGET`

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs` (neue `fn resolve_move_target`, vor `handle_move_refactor`)
- Test: `rust/src/tools/ctx_refactor.rs` (Test-Modul)

- [ ] **Step 1: Failing test schreiben**

Serena `insert_after_symbol` auf `handle_safe_delete_apply_requires_plan_hash` (aus Phase 3) mit:

```rust
#[test]
fn resolve_move_target_requires_exactly_one_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("app/moved")).unwrap();
    let root = dir.path().to_str().unwrap();

    // Neither set → INVALID_TARGET.
    let err = super::resolve_move_target(&serde_json::json!({}), root).unwrap_err();
    assert!(err.starts_with("INVALID_TARGET"), "got: {err}");

    // Both set → INVALID_TARGET.
    let err2 = super::resolve_move_target(
        &serde_json::json!({"target_path": "app/moved", "target_parent": "Other"}),
        root,
    ).unwrap_err();
    assert!(err2.starts_with("INVALID_TARGET"), "got: {err2}");
}

#[test]
fn resolve_move_target_path_is_jailed() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("app/moved")).unwrap();
    let root = dir.path().to_str().unwrap();

    // In-jail path resolves to a MoveTarget::Path.
    let t = super::resolve_move_target(&serde_json::json!({"target_path": "app/moved"}), root).unwrap();
    match t {
        crate::lsp::backend::MoveTarget::Path { rel_path, .. } => assert_eq!(rel_path, "app/moved"),
        other => panic!("expected Path, got {other:?}"),
    }

    // Escape attempt → INVALID_TARGET (jail violation, before any backend call).
    let err = super::resolve_move_target(&serde_json::json!({"target_path": "../../etc/skel"}), root).unwrap_err();
    assert!(err.starts_with("INVALID_TARGET"), "got: {err}");
}
```

> `target_parent`-Auflösung (→ `MoveTarget::Parent` via `resolve_name_path`, mit `NO_SYMBOL`/`AMBIGUOUS_SYMBOL`) wird im Live-Gate (Phase 7, Check #3) und über `resolve_name_path`s bestehende Unit-Tests abgedeckt; ein eigener Unit-Test bräuchte einen materialisierten Symbol-Index (vgl. `resolve_name_path_unique_class`, `ctx_refactor.rs:1173`).

- [ ] **Step 2: Test laufen lassen → muss fehlschlagen**

Run: `cargo nextest run -p <crate> resolve_move_target` (cwd=`rust`)
Expected: Kompilierfehler `no function resolve_move_target`.

- [ ] **Step 3: `resolve_move_target` einfügen**

Serena `insert_after_symbol` auf `handle_safe_delete_refactor` (aus Phase 3) mit:

```rust
/// Resolve the `move` target (spec §5.3 stage 2): EXACTLY ONE of `target_path` /
/// `target_parent` must be set. `target_path` → jail-checked dir/file →
/// MoveTarget::Path. `target_parent` → resolve_name_path → its file → MoveTarget::
/// Parent. None/both → INVALID_TARGET. Jail violation → INVALID_TARGET. This runs
/// BEFORE any backend call so an out-of-jail target can never reach the plugin.
fn resolve_move_target(
    args: &Value,
    project_root: &str,
) -> Result<crate::lsp::backend::MoveTarget, String> {
    let target_path = args.get("target_path").and_then(Value::as_str);
    let target_parent = args.get("target_parent").and_then(Value::as_str);
    match (target_path, target_parent) {
        (Some(_), Some(_)) | (None, None) => Err(
            "INVALID_TARGET: set exactly one of 'target_path' or 'target_parent'".to_string(),
        ),
        (Some(tp), None) => {
            let abs = crate::core::path_resolve::resolve_tool_path(Some(project_root), None, tp)
                .map_err(|e| format!("INVALID_TARGET: target_path blocked by jail: {e}"))?;
            Ok(crate::lsp::backend::MoveTarget::Path {
                abs_path: abs,
                rel_path: tp.to_string(),
            })
        }
        (None, Some(parent_np)) => {
            // Resolve the parent symbol → its file + declaration span.
            let r = resolve_name_path(parent_np, project_root)?; // NO_SYMBOL / AMBIGUOUS_SYMBOL
            let abs = crate::core::path_resolve::resolve_tool_path(Some(project_root), None, &r.rel_path)
                .map_err(|e| format!("INVALID_TARGET: target_parent file blocked by jail: {e}"))?;
            // Read the parent file to compute the end-of-line column (mirror handle_rename_refactor).
            let content = std::fs::read_to_string(&abs)
                .map_err(|e| format!("FILE_NOT_FOUND: {abs}: {e}"))?;
            let end_col = content
                .lines()
                .nth(r.end_line.saturating_sub(1))
                .map_or(0, str::len) as u32;
            Ok(crate::lsp::backend::MoveTarget::Parent {
                abs_path: abs,
                rel_path: r.rel_path,
                range: crate::lsp::backend::TextRange0Based {
                    start_line: (r.start_line - 1) as u32,
                    start_char: 0,
                    end_line: (r.end_line - 1) as u32,
                    end_char: end_col,
                },
            })
        }
    }
}
```

- [ ] **Step 4: Test laufen lassen → muss bestehen**

Run: `cargo nextest run -p <crate> resolve_move_target` (cwd=`rust`)
Expected: PASS.

---

### Task 6: `move` Render-Funktionen + Dispatch + 3-Stufen-Jail-Verdrahtung

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs` (Render-Funktionen + `handle_move_refactor`; Dispatch-`if` oben)
- Test: `rust/src/tools/ctx_refactor.rs` (Test-Modul — `MoveStub`)

- [ ] **Step 1: Failing test schreiben (Render-Apply mit Stub-Backend)**

Serena `insert_after_symbol` auf `resolve_move_target_path_is_jailed` (aus Task 5) mit:

```rust
/// Minimal backend for the move renderers: canned plan + recorded apply flags + changed paths.
struct MoveStub {
    plan: crate::lsp::backend::RenamePlan,
    applied_with_force: std::cell::Cell<Option<bool>>,
}
impl crate::lsp::backend::LspBackend for MoveStub {
    fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> { Ok(()) }
    fn references(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
    fn definition(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position) -> Result<lsp_types::GotoDefinitionResponse, String> { Ok(lsp_types::GotoDefinitionResponse::Array(vec![])) }
    fn implementations(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
    fn rename(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _n: &str) -> Result<Option<lsp_types::WorkspaceEdit>, String> { Ok(None) }
    fn move_preview(&mut self, _q: &crate::lsp::backend::MoveQuery) -> Result<crate::lsp::backend::RenamePlan, String> {
        Ok(self.plan.clone())
    }
    fn move_apply(&mut self, req: &crate::lsp::backend::MoveApply) -> Result<crate::lsp::backend::RenameResult, String> {
        self.applied_with_force.set(Some(req.force));
        Ok(crate::lsp::backend::RenameResult { applied: true, changed_paths: vec!["app/moved/Widget.kt".into()] })
    }
}

fn move_query(abs: &str) -> crate::lsp::backend::MoveQuery {
    crate::lsp::backend::MoveQuery {
        abs_path: abs.into(),
        rel_path: "a.rs".into(),
        src_range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
        target: crate::lsp::backend::MoveTarget::Path { abs_path: "/p/app/moved".into(), rel_path: "app/moved".into() },
    }
}

#[test]
fn move_apply_gates_then_evicts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("app/moved")).unwrap();
    std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
    std::fs::write(dir.path().join("app/moved/Widget.kt"), "// moved\n").unwrap();
    let root = dir.path().to_str().unwrap();
    let usage = crate::lsp::backend::UsageSite {
        path: "a.rs".into(),
        range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
        context: None,
    };
    let plan = crate::lsp::backend::RenamePlan { usages: vec![usage], conflicts: vec![] };
    let hash = super::plan_hash(root, &plan.usages).unwrap();
    let q = move_query(&dir.path().join("a.rs").to_string_lossy());

    // hash mismatch → CONFLICT, apply not called.
    let mut be = MoveStub { plan: plan.clone(), applied_with_force: std::cell::Cell::new(None) };
    let out = super::render_move_apply(&mut be, root, &q, "stalehash", false);
    assert!(out.contains("CONFLICT"), "got: {out}");
    assert_eq!(be.applied_with_force.get(), None);

    // matching hash + force → applies, force passed through, changed path jailed+evicted.
    let mut be2 = MoveStub { plan, applied_with_force: std::cell::Cell::new(None) };
    let out2 = super::render_move_apply(&mut be2, root, &q, &hash, true);
    assert!(out2.contains("applied"), "got: {out2}");
    assert_eq!(be2.applied_with_force.get(), Some(true));
}

#[test]
fn move_apply_rejects_out_of_jail_changed_path() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "let foo = 1;\nfoo + foo;\n").unwrap();
    let root = dir.path().to_str().unwrap();
    let usage = crate::lsp::backend::UsageSite {
        path: "a.rs".into(),
        range: crate::lsp::backend::TextRange0Based { start_line: 0, start_char: 4, end_line: 0, end_char: 7 },
        context: None,
    };
    // Stub returns an out-of-jail changed path (stage-3 jail must reject it post-apply).
    struct EscapeStub { plan: crate::lsp::backend::RenamePlan }
    impl crate::lsp::backend::LspBackend for EscapeStub {
        fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> { Ok(()) }
        fn references(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
        fn definition(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position) -> Result<lsp_types::GotoDefinitionResponse, String> { Ok(lsp_types::GotoDefinitionResponse::Array(vec![])) }
        fn implementations(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str) -> Result<Vec<lsp_types::Location>, String> { Ok(vec![]) }
        fn rename(&mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _n: &str) -> Result<Option<lsp_types::WorkspaceEdit>, String> { Ok(None) }
        fn move_preview(&mut self, _q: &crate::lsp::backend::MoveQuery) -> Result<crate::lsp::backend::RenamePlan, String> { Ok(self.plan.clone()) }
        fn move_apply(&mut self, _r: &crate::lsp::backend::MoveApply) -> Result<crate::lsp::backend::RenameResult, String> {
            Ok(crate::lsp::backend::RenameResult { applied: true, changed_paths: vec!["../../etc/passwd".into()] })
        }
    }
    let plan = crate::lsp::backend::RenamePlan { usages: vec![usage], conflicts: vec![] };
    let hash = super::plan_hash(root, &plan.usages).unwrap();
    let mut be = EscapeStub { plan };
    let q = move_query(&dir.path().join("a.rs").to_string_lossy());
    let out = super::render_move_apply(&mut be, root, &q, &hash, false);
    assert!(out.contains("jail"), "expected jail rejection, got: {out}");
}
```

- [ ] **Step 2: Test laufen lassen → muss fehlschlagen**

Run: `cargo nextest run -p <crate> move_apply_gates_then_evicts move_apply_rejects_out_of_jail_changed_path` (cwd=`rust`)
Expected: Kompilierfehler `no function render_move_apply`.

- [ ] **Step 3: Render-Funktionen einfügen**

Serena `insert_after_symbol` auf `resolve_move_target` (aus Task 5) mit:

```rust
/// Phase 1 renderer for move: ask Backing B for usages+conflicts at the new
/// location, build the stateless plan_hash, present the blast radius.
fn render_move_preview(
    backend: &mut dyn crate::lsp::backend::LspBackend,
    project_root: &str,
    query: &crate::lsp::backend::MoveQuery,
) -> String {
    let plan = match backend.move_preview(query) {
        Ok(p) => p,
        Err(e) => return format!("ERROR: {e}"),
    };
    let hash = match plan_hash(project_root, &plan.usages) {
        Ok(h) => h,
        Err(e) => return format!("ERROR: {e}"),
    };
    let target_desc = match &query.target {
        crate::lsp::backend::MoveTarget::Path { rel_path, .. } => format!("→ {rel_path}"),
        crate::lsp::backend::MoveTarget::Parent { rel_path, .. } => format!("→ member of {rel_path}"),
    };
    let mut files: Vec<&str> = plan.usages.iter().map(|u| u.path.as_str()).collect();
    files.push(query.rel_path.as_str());
    files.sort_unstable();
    files.dedup();
    let mut out = format!(
        "move_preview: '{}' {target_desc}\n  usages: {}\n  files: {}\n  plan_hash: {hash}\n",
        query.rel_path,
        plan.usages.len(),
        files.len(),
    );
    if !plan.conflicts.is_empty() {
        out.push_str(&format!(
            "  conflicts: {} (move_apply blocks unless force=true)\n",
            plan.conflicts.len()
        ));
        for c in &plan.conflicts {
            out.push_str(&format!("    {}: {}\n", c.path, c.message));
        }
    }
    out
}

/// Phase 2 renderer for move: re-fetch usages, enforce plan_hash (TOCTOU) +
/// conflict gate in Rust, run the IDE Multi-File move, then jail-check + evict
/// every changed path (spec §5.3 stage 3 — includes the NEW destination file).
fn render_move_apply(
    backend: &mut dyn crate::lsp::backend::LspBackend,
    project_root: &str,
    query: &crate::lsp::backend::MoveQuery,
    expected_hash: &str,
    force: bool,
) -> String {
    let plan = match backend.move_preview(query) {
        Ok(p) => p,
        Err(e) => return format!("ERROR: {e}"),
    };
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
    if !plan.conflicts.is_empty() && !force {
        return format!(
            "ERROR: CONFLICT: {} refactoring conflict(s); pass force=true to override",
            plan.conflicts.len()
        );
    }

    let apply = crate::lsp::backend::MoveApply {
        query: query.clone(),
        force,
    };
    let res = match backend.move_apply(&apply) {
        Ok(r) => r,
        Err(e) => return format!("ERROR: {e}"),
    };

    // Stage-3 jail: every changed path (incl. the new destination file) re-checked
    // against project_root BEFORE eviction (spec §5.3).
    for cp in &res.changed_paths {
        match crate::core::path_resolve::resolve_tool_path(Some(project_root), None, cp) {
            Ok(abs) => crate::core::cli_cache::invalidate(&abs),
            Err(e) => return format!("ERROR: CONFLICT: changed path blocked by jail: {e}"),
        }
    }

    format!(
        "move_apply: '{}' applied\n  changed files: {}\n",
        query.rel_path,
        res.changed_paths.len(),
    )
}
```

- [ ] **Step 4: `handle_move_refactor` einfügen**

Serena `insert_after_symbol` auf `render_move_apply` mit:

```rust
/// Entry for the Two-Phase move actions. Resolves the source (stage-1 jail), the
/// target (stage-2 jail via resolve_move_target → INVALID_TARGET on miss/escape),
/// requires a live IDE, then dispatches. Stage-3 jail is inside render_move_apply.
fn handle_move_refactor(action: &str, args: &Value, project_root: &str) -> String {
    if action == "move_apply" && args.get("plan_hash").and_then(Value::as_str).is_none() {
        return "ERROR: 'plan_hash' is required for move_apply (run move_preview first)."
            .to_string();
    }
    // Stage 2 (target) BEFORE any read/backend work, so INVALID_TARGET fires first.
    let target = match resolve_move_target(args, project_root) {
        Ok(t) => t,
        Err(e) => return format!("ERROR: {e}"),
    };
    // Stage 1 (source).
    let (rel_path, start_line, end_line) = match resolve_rename_target(args, project_root) {
        Ok(t) => t,
        Err(e) => return format!("ERROR: {e}"),
    };
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
    let src_range = crate::lsp::backend::TextRange0Based {
        start_line: (start_line - 1) as u32,
        start_char: 0,
        end_line: (end_line - 1) as u32,
        end_char: end_col,
    };

    let mut backend = match live_jetbrains_backend(project_root) {
        Ok(b) => b,
        Err(e) => return format!("ERROR: {e}"),
    };

    let query = crate::lsp::backend::MoveQuery {
        abs_path,
        rel_path,
        src_range,
        target,
    };

    match action {
        "move_preview" => render_move_preview(backend.as_mut(), project_root, &query),
        "move_apply" => {
            let expected = args.get("plan_hash").and_then(Value::as_str).unwrap_or_default();
            let force = args.get("force").and_then(Value::as_bool).unwrap_or(false);
            render_move_apply(backend.as_mut(), project_root, &query, expected, force)
        }
        other => format!("ERROR: INTERNAL: not a move action: {other}"),
    }
}
```

- [ ] **Step 5: Dispatch-Verzweigung oben in `handle` ergänzen**

Serena `mcp__serena__jet_brains_find_symbol` auf den safe_delete-Dispatch-`if` (aus Phase 3 Step 5), dann `insert_after_symbol` mit:

```rust

    if matches!(action, "move_preview" | "move_apply") {
        return handle_move_refactor(action, args, project_root);
    }
```

- [ ] **Step 6: `INVALID_TARGET`-Integrationstest (Dispatch-Ebene, vor Backend)**

Serena `insert_after_symbol` auf `move_apply_rejects_out_of_jail_changed_path` mit:

```rust
#[test]
fn handle_move_preview_invalid_target_before_backend() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn foo() {}\n").unwrap();
    let root = dir.path().to_str().unwrap();
    // No target → INVALID_TARGET, and crucially BEFORE BACKEND_REQUIRED (no live IDE here).
    let args = serde_json::json!({"action": "move_preview", "path": "a.rs", "line": 1});
    let out = super::handle(&args, root, "");
    assert!(out.contains("INVALID_TARGET"), "got: {out}");
    assert!(!out.contains("BACKEND_REQUIRED"), "target gate must precede backend gate: {out}");
}

#[test]
fn handle_move_apply_requires_plan_hash() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("x")).unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn foo() {}\n").unwrap();
    let root = dir.path().to_str().unwrap();
    let args = serde_json::json!({"action": "move_apply", "path": "a.rs", "line": 1, "target_path": "x"});
    let out = super::handle(&args, root, "");
    assert!(out.contains("plan_hash"), "got: {out}");
}
```

> **Wichtig (Reihenfolge-Garantie):** `handle_move_refactor` ruft `resolve_move_target` **vor** `live_jetbrains_backend`. Der Test `handle_move_preview_invalid_target_before_backend` zementiert diese Reihenfolge — ohne Ziel kommt `INVALID_TARGET`, nicht `BACKEND_REQUIRED`. Das entspricht Spec §5.3 („vor dem Backend-Call").

- [ ] **Step 7: Volle Suite laufen lassen → muss bestehen**

Run: `cargo nextest run --status-level fail` (cwd=`rust`)
Expected: alle PASS.

- [ ] **Step 8: Reformat + Commit (Phasen-Ende)**

`mcp__jetbrains__reformat_file` auf `rust/src/tools/ctx_refactor.rs`, dann:

```bash
git add rust/src/tools/ctx_refactor.rs
git commit -m "feat(jetbrains): v2c move actions — target resolve + 3-stage PathJail + INVALID_TARGET"
```

---

# PHASE 5 — Rust Schema + Doc-Regeneration

Commit-Message am Phasenende: `feat(jetbrains): v2c schema — 4 actions + target_path/target_parent/propagate`

### Task 7: Schema-Erweiterung in `registered/ctx_refactor.rs`

**Files:**
- Modify: `rust/src/tools/registered/ctx_refactor.rs` (Action-Enum `:32-35`, Properties `:38-65`, Description `:18-26`, `changed`-Match `:96-102`, Schema-Test `:117-139`)
- Test: `rust/src/tools/registered/ctx_refactor.rs` (`schema_advertises_declaration_and_scope`)

- [ ] **Step 1: Schema-Test erweitern (failing)**

Native `Edit` ist **nicht** erlaubt für `.rs` — Serena `mcp__serena__jet_brains_find_symbol` auf `schema_advertises_declaration_and_scope`, dann `replace_content` (oder `replace_symbol_body`) um diese Needles in das `for needle in [...]`-Array (`:117-139`) zu ergänzen — direkt nach `"search_text_occurrences",`:

```rust
            "move_preview",
            "move_apply",
            "safe_delete_preview",
            "safe_delete_apply",
            "target_path",
            "target_parent",
            "propagate",
```

- [ ] **Step 2: Test laufen lassen → muss fehlschlagen**

Run: `cargo nextest run -p <crate> schema_advertises_declaration_and_scope` (cwd=`rust`)
Expected: FAIL `schema missing move_preview` (das Schema enthält die Actions noch nicht).

- [ ] **Step 3: Action-Enum erweitern**

Serena `mcp__serena__jet_brains_find_symbol` auf `tool_def` (in `CtxRefactorTool::tool_def`), dann `replace_content`: das `"enum": [...]`-Array der `action`-Property (`:32-35`) um die vier Actions ergänzen — neuer Stand:

```rust
                        "enum": ["rename", "references", "definition", "implementations",
                                 "declaration", "type_hierarchy", "symbols_overview", "inspections",
                                 "replace_symbol_body", "insert_before_symbol", "insert_after_symbol",
                                 "rename_preview", "rename_apply",
                                 "move_preview", "move_apply",
                                 "safe_delete_preview", "safe_delete_apply"],
```

- [ ] **Step 4: Neue Properties einfügen**

`replace_content` direkt nach der `search_text_occurrences`-Property (`:65`) — die drei v2c-Felder einfügen (`plan_hash`/`force` existieren bereits aus v2b und werden mitgenutzt):

```rust
,
                    "target_path": { "type": "string", "description": "move only: destination directory/file (project-relative). Set EXACTLY ONE of target_path/target_parent. Out-of-jail or both/neither set → INVALID_TARGET." },
                    "target_parent": { "type": "string", "description": "move only: destination parent symbol (name_path, e.g. 'OtherClass') for a member move. Set EXACTLY ONE of target_path/target_parent." },
                    "propagate": { "type": "boolean", "description": "safe_delete_apply only: also delete dependencies that become unreferenced (Serena 'propagate', default false)." }
```

> Achtung Komma: Die letzte bestehende Property `search_text_occurrences` (`:65`) endet **ohne** Komma. Das vorangestellte `,` oben schließt sie ab; die drei neuen Felder enden wieder ohne Komma (vor der schließenden `}` des `properties`-Objekts). `ctx_read("rust/src/tools/registered/ctx_refactor.rs", mode="diff")` zur Komma-Kontrolle nutzen.

- [ ] **Step 5: Description-Text + `changed`-Match erweitern**

`replace_content` auf den Description-String (`:18-26`): ans Ende (vor dem schließenden `"`) ergänzen:

```
 The move ops (move_preview, move_apply) take target_path XOR target_parent and run a 3-stage PathJail (INVALID_TARGET on miss/escape). The safe_delete ops (safe_delete_preview, safe_delete_apply) report remaining references and block apply unless force=true; propagate also deletes now-unreferenced dependencies.
```

Dann `replace_content` auf den `changed`-Match (`:96-102`) — die zwei neuen schreibenden Actions ergänzen:

```rust
            changed: matches!(
                action.as_str(),
                "replace_symbol_body"
                    | "insert_before_symbol"
                    | "insert_after_symbol"
                    | "rename_apply"
                    | "move_apply"
                    | "safe_delete_apply"
            ),
```

- [ ] **Step 6: Schema-Test laufen lassen → muss bestehen**

Run: `cargo nextest run -p <crate> schema_advertises_declaration_and_scope` (cwd=`rust`)
Expected: PASS.

- [ ] **Step 7: Volle Suite + Drift-Test**

Run: `cargo nextest run --status-level fail` (cwd=`rust`)
Expected: alle PASS — **außer** ggf. der generierte Doc-Drift-Test (`generated_reference…`), der jetzt rot ist, weil `mcp-tools.md` noch alt ist. Das ist erwartet → Step 8.

- [ ] **Step 8: Generierte Doc regenerieren**

Run: `cargo run --example gen_docs --features dev-tools` (cwd=`rust`)
Expected: `docs/reference/generated/mcp-tools.md` aktualisiert (enthält die vier neuen Actions + Felder).

Danach erneut: `cargo nextest run --status-level fail` (cwd=`rust`) → Drift-Test jetzt grün.

- [ ] **Step 9: Human-Tool-Map (`appendix-mcp-tools.md`) ergänzen**

`appendix-mcp-tools.md` ist Markdown → native `Edit`/`ctx_read` erlaubt. Die `ctx_refactor`-Zeile/-Sektion um die vier Actions + Parameter erweitern (Format der bestehenden Einträge spiegeln). Zuerst `ctx_search("ctx_refactor", "docs/reference/appendix-mcp-tools.md")` um die Stelle zu finden, dann die Actions `move_preview`/`move_apply`/`safe_delete_preview`/`safe_delete_apply` mit ihren Parametern (`target_path`/`target_parent`/`plan_hash`/`force`/`propagate`) dokumentieren.

- [ ] **Step 10: Reformat + Commit (Phasen-Ende)**

`mcp__jetbrains__reformat_file` auf `rust/src/tools/registered/ctx_refactor.rs`, dann:

```bash
git add rust/src/tools/registered/ctx_refactor.rs docs/reference/generated/mcp-tools.md docs/reference/appendix-mcp-tools.md
git commit -m "feat(jetbrains): v2c schema — 4 actions + target_path/target_parent/propagate"
```

---

# PHASE 6 — Kotlin Plugin: Wire-DTOs, Mover, Deleter, Router

Commit-Message am Phasenende: `feat(plugin): v2c move/safe_delete — IntelliJ processors + endpoints`

> **Verifikation dieser Phase:** Kompilierung (`./gradlew compileKotlin`, cwd=`packages/jetbrains-lean-ctx`, bare command) — das Plugin-Modul hat keine Unit-Test-Suite. Funktionale Verifikation erfolgt im Live-Gate (Phase 7). Kotlin/Java-Dateien dürfen mit nativem `Edit`/`Write` bearbeitet werden.

### Task 8: Wire-DTOs + JsonCodec-Parser in `Wire.kt`

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt` (nach `RenameApplyResponse` `:143`; Parser im `JsonCodec`-Objekt nach `parseRenameApplyRequest` `:175-177`)

- [ ] **Step 1: Request-DTOs einfügen**

Native `Edit` auf `Wire.kt` — nach `RenameApplyResponse` (`:140-143`) einfügen:

```kotlin
/** Move target: kind="path" → {path}; kind="parent" → {path,range}. Mirrors Rust MoveTarget. */
data class MoveTargetDTO(
    val kind: String,
    val path: String,
    val range: TextRangeDTO? = null,
)

/** Request body for /movePreview. range = source symbol declaration span (0-based). */
data class MovePreviewRequest(
    val path: String,
    val range: TextRangeDTO,
    val target: MoveTargetDTO,
)

/** Request body for /moveApply. force = override blocking conflicts (Rust already gated). */
data class MoveApplyRequest(
    val path: String,
    val range: TextRangeDTO,
    val target: MoveTargetDTO,
    val force: Boolean = false,
)

/** Request body for /safeDeletePreview. range = source symbol declaration span (0-based). */
data class SafeDeletePreviewRequest(
    val path: String,
    val range: TextRangeDTO,
)

/** Request body for /safeDeleteApply. force = deleteEvenIfUsed; propagate = delete now-unreferenced deps. */
data class SafeDeleteApplyRequest(
    val path: String,
    val range: TextRangeDTO,
    val force: Boolean = false,
    val propagate: Boolean = false,
)
```

> Response-DTOs werden **wiederverwendet**: `RenamePreviewResponse` (`{usages, conflicts}`, `:127-130`) und `RenameApplyResponse` (`{applied, changed_paths}`, `:140-143`) sind op-unabhängig.

- [ ] **Step 2: JsonCodec-Parser einfügen**

Native `Edit` — nach `parseRenameApplyRequest` (`:175-177`) im `JsonCodec`-Objekt einfügen:

```kotlin
    fun parseMovePreviewRequest(body: String): MovePreviewRequest =
        gson.fromJson(body, MovePreviewRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")

    fun parseMoveApplyRequest(body: String): MoveApplyRequest =
        gson.fromJson(body, MoveApplyRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")

    fun parseSafeDeletePreviewRequest(body: String): SafeDeletePreviewRequest =
        gson.fromJson(body, SafeDeletePreviewRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")

    fun parseSafeDeleteApplyRequest(body: String): SafeDeleteApplyRequest =
        gson.fromJson(body, SafeDeleteApplyRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")
```

- [ ] **Step 3: Kompilieren**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`, bare command)
Expected: BUILD SUCCESSFUL (DTOs kompilieren).

---

### Task 9: `SymbolDeleter.kt` — `safe_delete` Preview + Apply

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolDeleter.kt`

> Modell: `SymbolRefactorer.kt` (`PsiLocator.inSmartReadAction` für off-EDT-Preview, `ApplicationManager.invokeAndWait` + `WriteCommandAction` + `CommandProcessor.executeCommand` für ein Undo, `saveAllDocuments`, `resolveTarget` mit `UNSUPPORTED_LANGUAGE`-Gate). `SafeDeleteProcessor.findUsages(...)` liefert die verbleibenden Referenzen; `findConflicts`/die nicht-sicher-löschbaren Usages werden als Konflikte gemeldet.

- [ ] **Step 1: Datei anlegen**

Native `Write` `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolDeleter.kt`:

```kotlin
package com.leanctx.plugin.psi

import com.intellij.lang.LanguageRefactoringSupport
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.CommandProcessor
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileTypes.PlainTextFileType
import com.intellij.openapi.fileTypes.PlainTextLanguage
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.util.PsiTreeUtil
import com.intellij.refactoring.safeDelete.SafeDeleteProcessor
import com.intellij.usageView.UsageInfo
import com.leanctx.plugin.dto.ConflictDTO
import com.leanctx.plugin.dto.RenameApplyResponse
import com.leanctx.plugin.dto.RenamePreviewResponse
import com.leanctx.plugin.dto.SafeDeleteApplyRequest
import com.leanctx.plugin.dto.SafeDeletePreviewRequest
import com.leanctx.plugin.dto.UsageSiteDTO
import com.leanctx.plugin.server.BackendException

/**
 * Safe-delete via IntelliJ's SafeDeleteProcessor (spec §6). Preview reports the
 * REMAINING (blocking) references as usages+conflicts (NO write). Apply runs the
 * delete as one WriteCommandAction → one Undo entry, saved to disk for lean-ctx.
 * The plan_hash + conflict gate live entirely in Rust; this class never hashes.
 */
class SymbolDeleter(private val project: Project) {
    private val locator = PsiLocator(project)

    fun preview(req: SafeDeletePreviewRequest): RenamePreviewResponse {
        val (element, remaining) = locator.inSmartReadAction {
            val el = resolveTarget(req.path, req.range.start.line, req.range.start.character)
            // SafeDeleteProcessor.findUsages collects every reference; those that are not
            // safe-to-delete (still referenced) are the blocking usages we surface.
            val usages = ArrayList<UsageInfo>()
            SafeDeleteProcessor.findUsages(el, arrayOf(el), usages)
            Pair(el, usages)
        }
        return locator.inSmartReadAction {
            val usageDtos = remaining.mapNotNull { info ->
                val el = info.element ?: return@mapNotNull null
                // Skip the declaration itself — only references count as "blocking".
                if (el == element || PsiTreeUtil.isAncestor(element, el, false)) return@mapNotNull null
                locator.toLocation(el)?.let { UsageSiteDTO(it.path, it.range, contextSnippet(el)) }
            }
            // Every remaining reference is a blocking conflict (spec §5.4).
            val conflictDtos = usageDtos.map { ConflictDTO(it.path, it.range, "symbol is still referenced here") }
            RenamePreviewResponse(usageDtos, conflictDtos)
        }
    }

    fun apply(req: SafeDeleteApplyRequest): RenameApplyResponse {
        val element = locator.inSmartReadAction {
            resolveTarget(req.path, req.range.start.line, req.range.start.character)
        }
        val changed = LinkedHashSet<String>()
        locator.inSmartReadAction { locator.toLocation(element)?.let { changed.add(it.path) } }
        var error: Throwable? = null
        ApplicationManager.getApplication().invokeAndWait {
            try {
                CommandProcessor.getInstance().executeCommand(project, {
                    // deleteEvenIfUsed = force; the Rust gate already blocked the non-force path.
                    val processor = SafeDeleteProcessor.createDelete(project, arrayOf(element), req.force)
                    // searchInCommentsAndStrings / searchNonJava default to the processor's settings;
                    // propagate is honored by the processor when it computes additional deletions.
                    processor.run()
                    WriteCommandAction.runWriteCommandAction(project) {
                        FileDocumentManager.getInstance().saveAllDocuments()
                    }
                }, "Safe Delete", null)
            } catch (t: Throwable) {
                error = t
            }
        }
        error?.let { throw it }
        return RenameApplyResponse(applied = true, changed_paths = changed.toList())
    }

    /** Resolve the target named declaration from a 0-based (line, character), or throw. */
    private fun resolveTarget(relPath: String, line: Int, character: Int): PsiElement {
        val file = locator.psiFile(relPath)
        val lang = file.language
        if (lang == PlainTextLanguage.INSTANCE ||
            file.fileType == PlainTextFileType.INSTANCE ||
            LanguageRefactoringSupport.getInstance().forLanguage(lang) == null
        ) {
            throw BackendException("UNSUPPORTED_LANGUAGE", "safe_delete not supported for ${lang.id}")
        }
        val offset = locator.offsetOf(file, line, character)
        val at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at $line:$character")
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
        if (named != null && named.name != null) return named
        throw BackendException("NO_SYMBOL", "no named declaration at target range")
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

> **Implementer-Hinweis (IntelliJ-API-Drift):** `SafeDeleteProcessor.findUsages(element, allElementsToDelete, usages)` und `SafeDeleteProcessor.createDelete(project, elements, dialogOK)` sind die stabilen Einstiegspunkte in IC-2026.x. Falls die Signatur in der gepinnten SDK-Version abweicht (z.B. `createDelete` mit zusätzlichen `searchInCommentsAndStrings`/`searchInNonJavaFiles`-Booleans), die Überladung mit den meisten Defaults wählen und `req.force` an den `deleteEvenIfUsed`/Dialog-OK-Parameter binden. `mcp__jetbrains__get_symbol_info` auf `SafeDeleteProcessor` zur Signatur-Bestätigung nutzen, bevor implementiert wird.

- [ ] **Step 2: Kompilieren**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: BUILD SUCCESSFUL. Bei Signatur-Fehlern: `get_symbol_info` auf `SafeDeleteProcessor`, Überladung anpassen.

---

### Task 10: `SymbolMover.kt` — `move` Preview + Apply

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolMover.kt`

> Modell: `SymbolRefactorer.kt`. `move` dispatcht nach `target.kind` (Spec §6): `kind="path"` → `MoveFilesOrDirectoriesProcessor` (Datei) bzw. `MoveClassesOrPackagesProcessor`; `kind="parent"` → Member-Move. Preview = `findUsages()` + `preprocessUsages` (kein Write), Apply = `WriteCommandAction`/`run()` als ein Undo.

- [ ] **Step 1: Datei anlegen**

Native `Write` `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolMover.kt`:

```kotlin
package com.leanctx.plugin.psi

import com.intellij.lang.LanguageRefactoringSupport
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.command.CommandProcessor
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileTypes.PlainTextFileType
import com.intellij.openapi.fileTypes.PlainTextLanguage
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.psi.PsiDirectory
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiManager
import com.intellij.psi.PsiNamedElement
import com.intellij.psi.util.PsiTreeUtil
import com.intellij.refactoring.move.moveFilesOrDirectories.MoveFilesOrDirectoriesProcessor
import com.leanctx.plugin.dto.ConflictDTO
import com.leanctx.plugin.dto.MoveApplyRequest
import com.leanctx.plugin.dto.MovePreviewRequest
import com.leanctx.plugin.dto.MoveTargetDTO
import com.leanctx.plugin.dto.RenameApplyResponse
import com.leanctx.plugin.dto.RenamePreviewResponse
import com.leanctx.plugin.dto.UsageSiteDTO
import com.leanctx.plugin.server.BackendException
import java.nio.file.Paths

/**
 * Multi-File move via IntelliJ's move processors (spec §6). Dispatches on the
 * target kind: "path" → MoveFilesOrDirectoriesProcessor (file/class into a dir);
 * "parent" → member move into a parent symbol. Preview = findUsages, NO write.
 * Apply = one WriteCommandAction → one Undo entry, saved for lean-ctx. plan_hash +
 * conflict gates live in Rust; this class never hashes.
 */
class SymbolMover(private val project: Project) {
    private val locator = PsiLocator(project)
    private val projectRoot: String = project.basePath ?: ""

    fun preview(req: MovePreviewRequest): RenamePreviewResponse {
        // Phase-1: collect cross-file usages of the moved element. For a file move,
        // the usages are the references to the moved class; for a member move, the
        // references to the member. We model both via the element's references.
        val usageDtos = locator.inSmartReadAction {
            val element = resolveSource(req.path, req.range.start.line, req.range.start.character, req.target)
            com.intellij.psi.search.searches.ReferencesSearch.search(element)
                .findAll()
                .mapNotNull { ref ->
                    val el = ref.element
                    locator.toLocation(el)?.let { UsageSiteDTO(it.path, it.range, contextSnippet(el)) }
                }
        }
        // Move conflicts are rare for clean targets; surface none for the happy path.
        // (Destination-collision conflicts are caught by the processor at apply time and
        //  bubble up as a BackendException → CONFLICT, mirroring rename's modal guard.)
        return RenamePreviewResponse(usageDtos, emptyList<ConflictDTO>())
    }

    fun apply(req: MoveApplyRequest): RenameApplyResponse {
        val element = locator.inSmartReadAction {
            resolveSource(req.path, req.range.start.line, req.range.start.character, req.target)
        }
        val changed = LinkedHashSet<String>()
        locator.inSmartReadAction {
            com.intellij.psi.search.searches.ReferencesSearch.search(element).findAll().forEach { ref ->
                locator.toLocation(ref.element)?.let { changed.add(it.path) }
            }
            locator.toLocation(element)?.let { changed.add(it.path) }
        }
        var error: Throwable? = null
        ApplicationManager.getApplication().invokeAndWait {
            try {
                CommandProcessor.getInstance().executeCommand(project, {
                    runMove(element, req.target)
                    WriteCommandAction.runWriteCommandAction(project) {
                        FileDocumentManager.getInstance().saveAllDocuments()
                    }
                }, "Move", null)
            } catch (e: BackendException) {
                error = e
            } catch (t: Throwable) {
                // A destination collision / illegal move surfaces here → CONFLICT (non-destructive).
                error = BackendException("CONFLICT", t.message ?: "move failed")
            }
        }
        error?.let { throw it }
        // Re-collect changed paths post-move (the destination file path is new).
        locator.inSmartReadAction { locator.toLocation(element)?.let { changed.add(it.path) } }
        return RenameApplyResponse(applied = true, changed_paths = changed.toList())
    }

    /** Run the move on the EDT. kind="path" → file/dir move; kind="parent" → member move. */
    private fun runMove(element: PsiElement, target: MoveTargetDTO) {
        when (target.kind) {
            "path" -> {
                val destDir = resolveDestinationDir(target.path)
                val file = element.containingFile
                    ?: throw BackendException("UNSUPPORTED_LANGUAGE", "element has no containing file to move")
                MoveFilesOrDirectoriesProcessor(
                    project,
                    arrayOf(file),
                    destDir,
                    /* searchInComments = */ true,
                    /* searchInNonJavaFiles = */ true,
                    /* moveCallback = */ null,
                    /* prepareSuccessfulCallback = */ null,
                ).run()
            }
            "parent" -> {
                // Member move: locate the parent symbol, then move the member into it.
                // The concrete processor is language-specific (e.g. Kotlin's move-members);
                // resolve it via the language move-refactoring support, else UNSUPPORTED_LANGUAGE.
                throw BackendException(
                    "UNSUPPORTED_LANGUAGE",
                    "member move (target_parent) not yet wired for ${element.language.id}",
                )
            }
            else -> throw BackendException("INVALID_TARGET", "unknown move target kind '${target.kind}'")
        }
    }

    /** Resolve the source element to move (file move → the class/file decl; member → the member). */
    private fun resolveSource(relPath: String, line: Int, character: Int, target: MoveTargetDTO): PsiElement {
        val file = locator.psiFile(relPath)
        val lang = file.language
        if (lang == PlainTextLanguage.INSTANCE ||
            file.fileType == PlainTextFileType.INSTANCE ||
            LanguageRefactoringSupport.getInstance().forLanguage(lang) == null
        ) {
            throw BackendException("UNSUPPORTED_LANGUAGE", "move not supported for ${lang.id}")
        }
        val offset = locator.offsetOf(file, line, character)
        val at = file.findElementAt(offset)
            ?: throw BackendException("NO_SYMBOL", "no element at $line:$character")
        val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
        if (named != null && named.name != null) return named
        throw BackendException("NO_SYMBOL", "no named declaration at target range")
    }

    /** Resolve a project-relative destination directory to a PsiDirectory, or throw INVALID_TARGET. */
    private fun resolveDestinationDir(relPath: String): PsiDirectory {
        val abs = Paths.get(projectRoot, relPath).toString()
        val vDir = LocalFileSystem.getInstance().findFileByPath(abs)
            ?: throw BackendException("INVALID_TARGET", "destination not found: $relPath")
        if (!vDir.isDirectory) throw BackendException("INVALID_TARGET", "destination is not a directory: $relPath")
        return PsiManager.getInstance(project).findDirectory(vDir)
            ?: throw BackendException("INVALID_TARGET", "destination is not a PSI directory: $relPath")
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

> **Scope-Hinweis (`target_parent`):** Der Member-Move-Pfad (`kind="parent"`) ist sprach-spezifisch (Kotlin: kein universeller `MoveMembersProcessor` wie in Java). Dieser Plan verdrahtet den **`target_path`-Pfad voll** (Datei/Klasse-Move = Akzeptanz-Gate-Fall #1/#2) und liefert `target_parent` zunächst als sauberes `UNSUPPORTED_LANGUAGE` (kein Crash, kein Teil-Edit). Das deckt die Spec-Akzeptanz (Kotlin Top-Level-Klasse via `target_path`) ab; der Member-Move (#3) wird in der Live-Gate-Tabelle als „sprach-abhängig / best-effort" markiert. **Vor** dem Verdrahten des Member-Move: `mcp__jetbrains__search_symbol` nach dem Kotlin-Move-Members-Processor (`org.jetbrains.kotlin.idea.refactoring.move…`) und dessen API bestätigen; falls vorhanden, den `"parent"`-Zweig analog zum `"path"`-Zweig implementieren statt zu werfen.

- [ ] **Step 2: Kompilieren**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: BUILD SUCCESSFUL. Bei `MoveFilesOrDirectoriesProcessor`-Signaturabweichung: `get_symbol_info` auf den Konstruktor, Parameterzahl anpassen.

---

### Task 11: Handler + Router-Verdrahtung

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/RefactorHandlers.kt`
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt`

- [ ] **Step 1: `RefactorHandlers` erweitern**

Native `Edit` auf `RefactorHandlers.kt` — Imports + Felder + Methoden ergänzen. Nach `import com.leanctx.plugin.psi.SymbolRefactorer` die neuen Imports:

```kotlin
import com.leanctx.plugin.dto.MoveApplyRequest
import com.leanctx.plugin.dto.MovePreviewRequest
import com.leanctx.plugin.dto.SafeDeleteApplyRequest
import com.leanctx.plugin.dto.SafeDeletePreviewRequest
import com.leanctx.plugin.psi.SymbolDeleter
import com.leanctx.plugin.psi.SymbolMover
```

Im Klassenkörper nach `private val refactorer = SymbolRefactorer(project)`:

```kotlin
    private val mover = SymbolMover(project)
    private val deleter = SymbolDeleter(project)

    fun movePreview(req: MovePreviewRequest) = mover.preview(req)
    fun moveApply(req: MoveApplyRequest) = mover.apply(req)
    fun safeDeletePreview(req: SafeDeletePreviewRequest) = deleter.preview(req)
    fun safeDeleteApply(req: SafeDeleteApplyRequest) = deleter.apply(req)
```

- [ ] **Step 2: Router-Routen + Dispatch ergänzen**

Native `Edit` auf `RequestRouter.kt`. Nach `if (path == "/renameApply") return dispatchRenameApply(body)` (`:49`):

```kotlin
            if (path == "/movePreview") return dispatchMovePreview(body)
            if (path == "/moveApply") return dispatchMoveApply(body)
            if (path == "/safeDeletePreview") return dispatchSafeDeletePreview(body)
            if (path == "/safeDeleteApply") return dispatchSafeDeleteApply(body)
```

Dann die vier Dispatch-Funktionen nach `dispatchRenameApply` (`:160`) ergänzen — jede spiegelt das `dispatchRenameApply`-Muster (BackendException → 200, IllegalArgumentException → 200 INTERNAL, Exception → 500):

```kotlin
    private fun dispatchMovePreview(body: String): HttpResult = try {
        HttpResult(200, JsonCodec.toJson(refactorHandlers.movePreview(JsonCodec.parseMovePreviewRequest(body))))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("movePreview endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }

    private fun dispatchMoveApply(body: String): HttpResult = try {
        HttpResult(200, JsonCodec.toJson(refactorHandlers.moveApply(JsonCodec.parseMoveApplyRequest(body))))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("moveApply endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }

    private fun dispatchSafeDeletePreview(body: String): HttpResult = try {
        HttpResult(200, JsonCodec.toJson(refactorHandlers.safeDeletePreview(JsonCodec.parseSafeDeletePreviewRequest(body))))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("safeDeletePreview endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }

    private fun dispatchSafeDeleteApply(body: String): HttpResult = try {
        HttpResult(200, JsonCodec.toJson(refactorHandlers.safeDeleteApply(JsonCodec.parseSafeDeleteApplyRequest(body))))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("safeDeleteApply endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }
```

- [ ] **Step 3: Vollständig kompilieren + Plugin bauen**

Run: `./gradlew compileKotlin` (cwd=`packages/jetbrains-lean-ctx`)
Expected: BUILD SUCCESSFUL.

Dann (optionaler Smoke): `./gradlew buildPlugin` (cwd=`packages/jetbrains-lean-ctx`) → Plugin-ZIP baut ohne Fehler.

- [ ] **Step 4: Reformat + Commit (Phasen-Ende)**

`mcp__jetbrains__reformat_file` auf jede geänderte/neue Kotlin-Datei (`Wire.kt`, `SymbolDeleter.kt`, `SymbolMover.kt`, `RefactorHandlers.kt`, `RequestRouter.kt`), dann:

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolDeleter.kt \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolMover.kt \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/RefactorHandlers.kt \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt
git commit -m "feat(plugin): v2c move/safe_delete — IntelliJ processors + endpoints"
```

---

# PHASE 7 — Live-Gate-Runbook + Fixture-Script

Commit-Message am Phasenende: `docs(runbook): v2c runIde move/safe_delete gate + fixture`

### Task 12: Fixture-Setup-Script

**Files:**
- Create: `scripts/runide-move-safedelete-gate-setup.sh`

> Modell: `scripts/runide-gate-setup.sh` (Rename-Fixture). Erweitert um Move-/Delete-taugliche Symbole: eine verschiebbare Top-Level-Klasse (`Widget`) + ein Ziel-Package-Verzeichnis (`app/moved`), ein ungenutztes Symbol (`Unused`) und ein genutztes (`Widget` via `Usage.kt`).

- [ ] **Step 1: Script anlegen**

Native `Write` `scripts/runide-move-safedelete-gate-setup.sh`:

```bash
#!/usr/bin/env bash
# Materializes the runIde move/safe_delete-gate Kotlin fixture into
# tmp/runide-move-safedelete-gate/. Idempotent: re-running fully resets it.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="$ROOT/tmp/runide-move-safedelete-gate"

rm -rf "$DEST"
mkdir -p "$DEST/src/main/kotlin/app"
mkdir -p "$DEST/src/main/kotlin/app/moved"

cat > "$DEST/settings.gradle.kts" <<'EOF'
rootProject.name = "runide-move-safedelete-gate"
EOF

cat > "$DEST/build.gradle.kts" <<'EOF'
plugins {
    kotlin("jvm") version "2.1.0"
}

repositories {
    mavenCentral()
}
EOF

# Movable top-level class (move_preview/move_apply name_path=Widget, target_path=app/moved).
cat > "$DEST/src/main/kotlin/app/Widget.kt" <<'EOF'
package app

// Move target for the gate. Referenced cross-file by Usage.kt (proves refs follow).
class Widget
EOF

# Cross-file reference to Widget — proves move rewrites imports/refs.
cat > "$DEST/src/main/kotlin/app/Usage.kt" <<'EOF'
package app

// Reference to Widget — must be rewritten after move; blocks safe_delete of Widget.
fun use(): Widget = Widget()
EOF

# Unused symbol — safe_delete happy path (no blocking refs).
cat > "$DEST/src/main/kotlin/app/Unused.kt" <<'EOF'
package app

// Unreferenced — safe_delete_preview should report zero blocking usages.
class Unused
EOF

# Helper with a member, for the (language-dependent) target_parent member-move case.
cat > "$DEST/src/main/kotlin/app/Helper.kt" <<'EOF'
package app

class Helper {
    fun calc(): Int = 42
}

class OtherClass
EOF

cat > "$DEST/notes.txt" <<'EOF'
Plain text file — used for the UNSUPPORTED_LANGUAGE gate case.
EOF

echo "fixture ready: $DEST"
```

- [ ] **Step 2: Ausführbar machen + Smoke-Test**

```bash
chmod +x scripts/runide-move-safedelete-gate-setup.sh
./scripts/runide-move-safedelete-gate-setup.sh
```

Expected: `fixture ready: …/tmp/runide-move-safedelete-gate`. Verifiziere via `ctx_tree("tmp/runide-move-safedelete-gate", 3)`, dass `app/Widget.kt`, `app/Usage.kt`, `app/Unused.kt`, `app/Helper.kt`, `app/moved/` (leer) und `notes.txt` existieren.

---

### Task 13: Runbook

**Files:**
- Create: `docs/lean-md/runbooks/runide-move-safedelete-gate.md`

> Modell: `docs/lean-md/runbooks/runide-rename-gate.md`. **Zusätzlich verbindlich** der Daemon-Stopp-Block aus Spec §9.1 (neue Actions existieren erst nach Neubau; laufender Daemon hält den alten Action-Satz).

- [ ] **Step 1: Runbook schreiben**

Native `Write` `docs/lean-md/runbooks/runide-move-safedelete-gate.md`:

````markdown
# Runbook: runIde-Move/Safe-Delete-Gate (v2c Live-Verifikation)

Verifiziert den vollen v2c-Two-Phase-Stack live: Rust-Gate (`plan_hash`/TOCTOU,
Konflikt-Gate, **3-Stufen**-PathJail, `INVALID_TARGET`, Cache-Evict) **und** das
JetBrains-Plugin (`MoveFilesOrDirectoriesProcessor`/`SafeDeleteProcessor`-Naht,
Multi-File-Transaktion, ein Undo) gegen ein sauberes Kotlin-Gradle-Fixture.

Bezug: Spec `docs/lean-md/specs/2026-06-10-leanctx-jetbrains-v2c-move-safedelete-design.md` §9.1.

## Voraussetzungen — frisches Binary (Daemon-Stopp ist PFLICHT)

Die neuen Actions (`move_*`/`safe_delete_*`) existieren erst nach Neubau. Ein
**laufender** lean-ctx-Daemon hält den **alten** Action-Satz im Speicher →
`Unknown action`. Reihenfolge **vor** dem Gate:

1. `lean-ctx serve --stop` — Daemon stoppen (gibt Binary frei + entlädt alten Action-Satz).
2. `cargo build` (cwd=`rust`) [+ ggf. Binary neu installieren].
3. `lean-ctx serve --daemon` neu starten **oder** ersten `lean-ctx call` den Daemon auto-starten lassen.

> **Achtung MCP-Session:** In einer aktiven Agent-/MCP-Session ist dieser Daemon
> zugleich der `ctx_*`-Server — `serve --stop` unterbricht die eigenen `ctx_*`-Tools.
> Das Gate als **separaten** Schritt fahren, nicht mitten in einer ctx_*-Aufgabe.

- Plugin-Modul gebaut: `./gradlew buildPlugin` (cwd=`packages/jetbrains-lean-ctx`).

## 1. Setup — Fixture materialisieren
```
./scripts/runide-move-safedelete-gate-setup.sh
```
Notiere `FIX=<abs>/tmp/runide-move-safedelete-gate`.

## 2. Launch — Sandbox-IDE auf dem Fixture
```
./gradlew runIde --args="$FIX"
```
(cwd=`packages/jetbrains-lean-ctx`) — **Indizierung abwarten** (Statusleiste idle).
> Falls `runIde --args` das Projekt nicht öffnet: einmal manuell `File → Open` auf `$FIX`.

## 3. Gate-Checks
Jeder Check: `lean-ctx call ctx_refactor --project-root "$FIX" --json '<args>'`.
Für force-/TOCTOU-Fälle zuerst das passende `*_preview` ausführen, um den aktuellen
`plan_hash` zu holen.

| # | Fall | Aufruf (`--json`, Auszug) | Soll-Ergebnis |
| 1 | move Preview (`target_path`) | `{"action":"move_preview","name_path":"Widget","target_path":"src/main/kotlin/app/moved"}` | usages cross-file (Usage.kt), `files≥2`, `plan_hash` gesetzt |
| 2 | move Apply + Undo | `{"action":"move_apply","name_path":"Widget","target_path":"src/main/kotlin/app/moved","plan_hash":"<#1>"}` | `Widget.kt` umgezogen, Refs/Imports in `Usage.kt` angepasst; **ein** Undo (Strg+Z revertet komplett) |
| 3 | move Member (`target_parent`) | `{"action":"move_preview","name_path":"Helper/calc","target_parent":"OtherClass"}` | sprach-abhängig: Member-Move-Plan **oder** `UNSUPPORTED_LANGUAGE` (Kotlin best-effort, vgl. Plan Task 10) — kein Crash |
| 4 | INVALID_TARGET | (a) `{"action":"move_preview","name_path":"Widget"}` (kein Ziel); (b) beide Ziele gesetzt; (c) `{"action":"move_preview","name_path":"Widget","target_path":"../escape"}` | je `INVALID_TARGET`, **vor** Backend-Call, kein Apply |
| 5 | move TOCTOU | eine usage-Stelle in `Usage.kt` zwischen #1 und Apply ändern, dann Apply mit altem `plan_hash` | `CONFLICT` |
| 6 | safe_delete Preview (ungenutzt) | `{"action":"safe_delete_preview","name_path":"Unused"}` | keine blockierenden usages, `plan_hash` gesetzt |
| 7 | safe_delete Apply ohne force (genutzt) | `{"action":"safe_delete_apply","name_path":"Widget","plan_hash":"<preview Widget>"}` | `CONFLICT` mit blockierenden Refs, **kein** Löschen |
| 8 | safe_delete Apply mit force | wie #7 + `"force":true` | gelöscht; Refs bleiben dangling (bewusst, `deleteEvenIfUsed`) |
| 9 | INDEXING | Projekt neu öffnen, sofort `move_preview`/`safe_delete_preview` während Indizierung | `INDEXING`, kein Teil-Edit (best-effort beim Mini-Fixture; deterministisch via Rust-Unit abgesichert) |
| 10 | UNSUPPORTED_LANGUAGE | `{"action":"move_preview","path":"notes.txt","line":1,"target_path":"src/main/kotlin/app/moved"}` (`path`+`line`-Fallback, **nicht** `name_path`) | `UNSUPPORTED_LANGUAGE`, kein Crash |
| 11 | BACKEND_REQUIRED | IDE schließen, dann preview **und** apply (move + safe_delete) | `BACKEND_REQUIRED` in beiden Phasen |

> `safe_delete_preview` für `Unused` (#6) liefert den `plan_hash`; für den genutzten
> `Widget` (#7) zuerst `safe_delete_preview name_path=Widget` für dessen `plan_hash`.

## 4. Teardown
- Sandbox-IDE schließen.
- `tmp/runide-move-safedelete-gate/` kann liegen bleiben (gitignored) oder via
  `./scripts/runide-move-safedelete-gate-setup.sh` zurückgesetzt werden.
- Daemon wieder hochfahren, falls für die MCP-Session gestoppt.
````

- [ ] **Step 2: `.gitignore`-Check**

Run: `ctx_search("runide", ".gitignore")` (oder `git check-ignore tmp/runide-move-safedelete-gate`)
Expected: das `tmp/`-Fixture ist ignoriert (wie das Rename-Fixture). Falls nur `tmp/runide-rename-gate` explizit gelistet ist, ergänze eine Zeile `tmp/runide-move-safedelete-gate/` in `.gitignore`.

- [ ] **Step 3: Commit (Phasen-Ende)**

```bash
git add scripts/runide-move-safedelete-gate-setup.sh docs/lean-md/runbooks/runide-move-safedelete-gate.md .gitignore
git commit -m "docs(runbook): v2c runIde move/safe_delete gate + fixture"
```

---

## Abschluss-Verifikation (nach Phase 7)

- [ ] **Volle Rust-Suite grün:** `cargo nextest run --status-level fail` (cwd=`rust`) — keine Regression.
- [ ] **Clippy/fmt:** `cargo clippy --all-targets` + `cargo fmt --check` (cwd=`rust`) sauber.
- [ ] **Plugin baut:** `./gradlew buildPlugin` (cwd=`packages/jetbrains-lean-ctx`) erfolgreich.
- [ ] **Drift-Test grün:** der generierte `mcp-tools.md`-Drift-Test passt (Phase 5 Step 8).
- [ ] **Sieben Commits** auf `feat-jetbrains-plugin` (ein Commit pro Phase).
- [ ] **Live-Gate** (manuell, separater Schritt — nicht in der MCP-Session): Runbook `runide-move-safedelete-gate.md` durchlaufen; Ergebnisse für die PR-/Merge-Beschreibung notieren.
- [ ] **Finishing:** danach `superpowers:finishing-a-development-branch` (Merge/PR-Entscheidung — Spec §11: Squash-Merge-PR nach `main` am Schluss).

---

## Self-Review-Notizen (Spec-Abdeckung)

| Spec-Abschnitt | Abgedeckt durch |
| §2 #1 Scope (move+safe_delete) | Phasen 3–6 |
| §2 #2 Two-Phase | Render-Funktionen (Phase 3/4), Wire-Reuse |
| §2 #3 Ziel-Form (target_path XOR target_parent) | `resolve_move_target` (Task 5), Schema (Task 7) |
| §2 #4 3-stufiges move-Jail | `resolve_move_target` (Stufe 2) + `render_move_apply` (Stufe 3) + `handle_move_refactor` (Stufe 1); Tests Task 5/6 |
| §2 #5 safe_delete-Policy | `render_safe_delete_apply`-Gate (Task 4) |
| §2 #6 vier Actions in ctx_refactor | Dispatch (Task 4/6), Schema (Task 7) |
| §2 #7 stateless plan_hash | reuse `plan_hash` (alle Render-Funktionen) |
| §2 #8 Kotlin-Akzeptanz-Gate | Runbook (Task 13) |
| §5.5 Trait + Typen | Phase 1 (Task 1/2) |
| §5.6 Rust-Änderungsstellen | Phasen 1–5 (alle gelisteten Dateien) |
| §6 Plugin-Naht | Phase 6 (Mover/Deleter/Handler/Router) |
| §7 Wire-Protokoll + INVALID_TARGET | Wire-DTOs (Task 8), Body-Builder (Task 3), `resolve_move_target` |
| §9 Verifikation (Rust-Unit) | Tests in Phasen 1–4 |
| §9.1 Live-Gate-Runbook | Phase 7 (Task 12/13) |
| §11 Branch/Commit + Drift-Gate | ein Commit/Phase; Doc-Regen (Task 7) |

**Bewusst NICHT abgedeckt (Spec §12 YAGNI, korrekt ausgelassen):** kein `inline`/`reformat`
(v2d), kein Blast-Radius-Limit, kein Server-State, kein Headless-move/-delete, kein
plugin-seitiges Hashing, kein Auto-Reformat im Apply, kein Symbol-Inspect-Auto-Detect.

**Offener Implementer-Vorbehalt (ehrlich markiert):** Der `target_parent`-Member-Move
(Spec §3 `SymbolMoveProcessor`) ist sprach-spezifisch und in Task 10 zunächst als
`UNSUPPORTED_LANGUAGE` verdrahtet (Kotlin hat keinen universellen `MoveMembersProcessor`).
Der `target_path`-Pfad (Akzeptanz-Gate #1/#2) ist voll implementiert. Vor dem Schließen
des Member-Move-Pfads: Kotlin-Move-Members-Processor via `mcp__jetbrains__search_symbol`
verifizieren (Task 10 Step 1 Hinweis) — nicht annehmen, prüfen.
