# JetBrains-Plugin Phase 5b — `inspections` (run + list) + CI-Härtung — Implementierungsplan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `ctx_refactor action=inspections` mit `mode=run` (Diagnostik einer Datei) und `mode=list` (enabled Profil-Inspektionen) als read-only PSI-Feature über das JetBrains-Backend liefern, plus CI-Härtung (`concurrency`, `timeout-minutes`, SHA-Pinning, `actionlint`).

**Architecture:** Identisches Muster zu Phase 3 (Nav) / Phase 4 (Hierarchie/Overview). Rust-PathJail (`jail_path`) bleibt der alleinige Validierungspunkt vor jedem HTTP-Request; das Plugin re-validiert Pfade nicht. Datenfluss: `ctx_refactor` (Rust) → `with_backend` → `JetBrainsHttpBackend.inspections|list_inspections` → HTTP `POST /inspections` | `/list_inspections` → Kotlin `RequestRouter` → `InspectionHandlers` → `InspectionRunner` (PSI in SmartReadAction). Degradierung: ohne IDE/Backing A sind beide Trait-Methoden default-`Err`.

**Tech Stack:** Rust (`ureq` 3.x, `serde_json`, `lsp_types`, `cargo nextest`), Kotlin (IntelliJ Platform IC-2026.1.3, `BasePlatformTestCase`, `gson`, Gradle), GitHub Actions YAML (`actionlint`).

**Commit-Strategie (Spec §12.3 Eltern-Spec — überschreibt das writing-plans-Default):** Phase 5b = **EIN Commit**. Zwischen-Tasks werden **nicht** committet; jeder Task endet mit einem **grünen Gate** (Tests/Build laufen, nicht committen). Der finale Task (Task 12) führt das Gesamt-Gate aus und erstellt den **einen** Commit.

**Tool-Disziplin (Projekt-Hard-Rules):** Rust-Dateien (`*.rs`) **nur** via Serena-Tools editieren (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_after_symbol`, `replace_content`), **nie** native `Edit`/`ctx_edit`. Kotlin/YAML/Markdown: native `Edit`/`Write`. Vor `git add`: `mcp__jetbrains__reformat_file` auf jede geänderte Datei. Lesen via `ctx_read`, Suchen via `ctx_search`, Shell via `ctx_shell` (bare command + `cwd=`, nie `cd … &&`, nie `2>&1`). Tests: immer `cargo nextest run`, nie `cargo test`.

---

## Dateienübersicht

**Rust (Backing-/Tool-Schicht):**
- Modify: `rust/src/lsp/backend.rs` — neuer Typ `InspectionInfo`, neue default-degrading Trait-Methode `list_inspections`, Export.
- Modify: `rust/src/lsp/jetbrains_backend.rs` — Parser `parse_inspections`/`parse_inspection_list`, Methoden `inspections`/`list_inspections`, Mock-Tests.
- Modify: `rust/src/tools/ctx_refactor.rs` — Action `inspections` + `mode`-Dispatch, `handle_inspections`, `format_inspections`/`format_inspection_list`, Tests.
- Modify: `rust/src/tools/registered/ctx_refactor.rs` — Schema (`action`-Enum + `mode`-Param), `schema_test`.
- Modify: `docs/reference/generated/mcp-tools.md` (generiert), `docs/reference/appendix-mcp-tools.md` (handgepflegt).

**Kotlin (Plugin-Schicht):**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt` — vier neue DTOs.
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/InspectionRunner.kt` — PSI-Logik (run + list).
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/InspectionHandlers.kt` — dünner Handler-Wrapper.
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt` — zwei Routen + zwei Dispatcher.
- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterInspectionTest.kt` — router-getriebener End-to-End-Test.
- Modify: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt` — DTO-Round-Trip-Fälle.

**CI:**
- Modify: `.github/workflows/jetbrains-plugin.yml` — `concurrency`, `timeout-minutes`, SHA-Pinning, `actionlint`-Job.

---

## Task 0: Inspection-API gegen IC-2026.1.3 verifizieren (Spike, KEIN Commit)

**Warum:** Die IntelliJ-Inspection-API (`InspectionEngine`, `InspectionProjectProfileManager`, `Tools`, `HighlightDisplayLevel`) ist die einzige in 5b verwendete API, die NICHT bereits im Repo erprobt ist (Spec §8 nennt sie als Risiko). Vor dem Schreiben von `InspectionRunner.kt` müssen die exakten Signaturen gegen die gebundene Plattform bestätigt werden, damit Task 7 nicht auf falschen Signaturen aufsetzt.

**Files:** keine Änderung — reine Recherche.

- [ ] **Step 1: Signaturen über JetBrains-MCP bestätigen**

Die IDE läuft (siehe `.serena/project.yml` / Port 44737 aus 5a). Falls ein JetBrains-Tool deferred ist, zuerst `ToolSearch(query="select:<tool>")`. Bestätige mit `mcp__jetbrains__search_symbol` / `mcp__jetbrains__get_symbol_info` die folgenden Symbole und ihre Signaturen:

1. `com.intellij.codeInspection.InspectionEngine#runInspectionOnFile(PsiFile, InspectionToolWrapper, GlobalInspectionContext)` → erwartet `List<ProblemDescriptor>`, `@JvmStatic`.
2. `com.intellij.profile.codeInspection.InspectionProjectProfileManager#getInstance(Project)#getCurrentProfile()` → `InspectionProfileImpl`.
3. `com.intellij.codeInspection.ex.InspectionProfileImpl#getAllEnabledInspectionTools(Project)` → `List<Tools>`.
4. `com.intellij.codeInspection.ex.Tools#getTool()` → `InspectionToolWrapper<*,*>`; `Tools#getLevel()` → `HighlightDisplayLevel`.
5. `com.intellij.codeInspection.ex.InspectionToolWrapper#getShortName()` / `#getDisplayName()` → `String`.
6. `com.intellij.codeHighlighting.HighlightDisplayLevel#getSeverity()` → `com.intellij.lang.annotation.HighlightSeverity`; `HighlightSeverity` implementiert `Comparable<HighlightSeverity>` mit Konstanten `ERROR`, `WARNING`, `WEAK_WARNING`, `INFORMATION`.
7. `com.intellij.codeInspection.InspectionManager#getInstance(Project)` castbar zu `com.intellij.codeInspection.ex.InspectionManagerEx#createNewGlobalContext()` → `GlobalInspectionContextImpl`.
8. `com.intellij.codeInspection.ProblemDescriptor#getPsiElement()` → `PsiElement?`; `#getDescriptionTemplate()` → `String`.

- [ ] **Step 2: Findings festhalten**

Schreibe die bestätigten/abweichenden Signaturen via `ctx_knowledge action=remember category=api` (z. B. `key=jetbrains-inspection-engine-api`). Falls eine Signatur abweicht (z. B. `runInspectionOnFile` hat eine andere Arity oder erwartet einen `GlobalInspectionContext` statt `GlobalInspectionContextImpl`), notiere die korrekte Form — Task 7 setzt darauf auf.

Erwartetes Ergebnis: alle 8 Signaturen bestätigt ODER eine Korrekturliste, die Task 7's Code-Block anpasst.

---

## Task 1: Rust — `InspectionInfo`-Typ + `list_inspections`-Trait-Methode

**Files:**
- Modify: `rust/src/lsp/backend.rs`

> `InspectionDiag` (path/line/severity/message) und `inspections(uri)` existieren bereits (`backend.rs:42` bzw. der default-degrading Block). Neu: nur `InspectionInfo` + `list_inspections`.

- [ ] **Step 1: `InspectionInfo`-Struct einfügen**

Per Serena `insert_after_symbol` nach dem `InspectionDiag`-Struct in `rust/src/lsp/backend.rs` einfügen:

```rust
/// A single available inspection (the `list` mode of the inspections action).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectionInfo {
    /// Stable short name / id of the inspection tool.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Severity token: ERROR | WARNING | WEAK_WARNING | INFO.
    pub severity: String,
}
```

- [ ] **Step 2: `list_inspections`-Methode im default-degrading Block ergänzen**

Per Serena `insert_after_symbol` direkt nach der bestehenden `inspections`-Trait-Methode (im `pub trait LspBackend`):

```rust
    fn list_inspections(&mut self) -> Result<Vec<InspectionInfo>, String> {
        Err("list_inspections requires the JetBrains backend".to_string())
    }
```

- [ ] **Step 3: Kompilieren**

Run: `cargo build` (cwd: `rust`)
Expected: kompiliert (eventuell `unused`-Warnings für `InspectionInfo`, solange noch kein Consumer existiert — kein Fehler).

---

## Task 2: Rust — `JetBrainsHttpBackend`-Parser + Methoden (Mock-getrieben, TDD)

**Files:**
- Modify: `rust/src/lsp/jetbrains_backend.rs`

- [ ] **Step 1: Failing-Tests im `tests`-Modul ergänzen**

Per Serena `insert_after_symbol` nach `symbols_overview_parses_wire_items` im `#[cfg(test)] mod tests` einfügen. Muster = bestehende `symbols_overview_parses_wire_items` (`mock_once` + `JetBrainsHttpBackend::new`):

```rust
    #[test]
    fn inspections_parses_wire_diags() {
        let body = r#"{"diagnostics":[{"path":"A.kt","line":3,"severity":"WARNING","message":"unused variable"}],"truncated":false,"total":1}"#;
        let port = mock_once(body);
        let mut backend = JetBrainsHttpBackend::new(
            port,
            "tok".to_string(),
            "/proj".to_string(),
            std::process::id(),
        );
        let uri = file_path_to_uri("/proj/A.kt").unwrap();
        let diags = backend.inspections(&uri).expect("should parse");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].path, "A.kt");
        assert_eq!(diags[0].line, 3);
        assert_eq!(diags[0].severity, "WARNING");
        assert_eq!(diags[0].message, "unused variable");
    }

    #[test]
    fn inspections_maps_error_envelope_to_err() {
        let body = r#"{"error":{"code":"UNSUPPORTED_LANGUAGE","message":"only kotlin"}}"#;
        let port = mock_once(body);
        let mut backend = JetBrainsHttpBackend::new(
            port,
            "tok".to_string(),
            "/proj".to_string(),
            std::process::id(),
        );
        let uri = file_path_to_uri("/proj/A.kt").unwrap();
        let err = backend.inspections(&uri).expect_err("envelope → Err");
        assert_eq!(err, "UNSUPPORTED_LANGUAGE");
    }

    #[test]
    fn list_inspections_parses_wire_items() {
        let body = r#"{"inspections":[{"id":"UnusedSymbol","name":"Unused declaration","severity":"WARNING"}],"truncated":true,"total":342}"#;
        let port = mock_once(body);
        let mut backend = JetBrainsHttpBackend::new(
            port,
            "tok".to_string(),
            "/proj".to_string(),
            std::process::id(),
        );
        let items = backend.list_inspections().expect("should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "UnusedSymbol");
        assert_eq!(items[0].name, "Unused declaration");
        assert_eq!(items[0].severity, "WARNING");
        let meta = backend.last_truncation().expect("meta recorded");
        assert!(meta.truncated);
        assert_eq!(meta.total, 342);
    }
```

- [ ] **Step 2: Tests laufen lassen — müssen fehlschlagen (kompiliert nicht)**

Run: `cargo nextest run inspections` (cwd: `rust`)
Expected: Kompilierfehler — `inspections`/`list_inspections` sind auf `JetBrainsHttpBackend` noch nicht implementiert (Trait-Default `Err`, aber `inspections` ist noch nicht überschrieben; `list_inspections` ebenso). (Falls `inspections` bereits den Default-`Err` nutzt, schlägt der Test inhaltlich fehl statt zu kompilieren — beides „rot".)

- [ ] **Step 3: Imports erweitern**

In `rust/src/lsp/jetbrains_backend.rs` die `use`-Zeile für das Backend-Modul erweitern. Aktuell:

```rust
use crate::lsp::backend::{HierarchyDirection, LspBackend, SymbolOverviewItem, TypeHierarchyNode};
```

Per Serena `replace_content` ersetzen durch:

```rust
use crate::lsp::backend::{
    HierarchyDirection, InspectionDiag, InspectionInfo, LspBackend, SymbolOverviewItem,
    TypeHierarchyNode,
};
```

- [ ] **Step 4: Parser einfügen**

Per Serena `insert_after_symbol` nach `parse_symbols` (im `impl JetBrainsHttpBackend`). `parse_inspections` ist `&self` (nicht `Self::`), da es zur Methoden-Symmetrie passt; `path` kommt projekt-relativ vom Wire und wird **nicht** über `rel_to_uri` rejoined (Diagnostics zeigen den relativen Pfad direkt):

```rust
    fn parse_inspections(&self, v: &Value) -> Vec<InspectionDiag> {
        v.get("diagnostics")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| {
                        Some(InspectionDiag {
                            path: d.get("path")?.as_str()?.to_string(),
                            line: d.get("line")?.as_u64()? as u32,
                            severity: d.get("severity")?.as_str()?.to_string(),
                            message: d.get("message")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_inspection_list(v: &Value) -> Vec<InspectionInfo> {
        v.get("inspections")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|i| {
                        Some(InspectionInfo {
                            id: i.get("id")?.as_str()?.to_string(),
                            name: i.get("name")?.as_str()?.to_string(),
                            severity: i.get("severity")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
```

- [ ] **Step 5: Trait-Methoden im `impl LspBackend for JetBrainsHttpBackend` einfügen**

Per Serena `insert_after_symbol` nach der bestehenden `symbols_overview`-Methode. Error-Envelope-Mapping wie `symbols_overview` (`:263-269`); `list_inspections` sendet `{path:""}` (der Plugin-Handler nutzt den Pfad nur als No-op — die Liste ist projektweit):

```rust
    fn inspections(&mut self, uri: &Uri) -> Result<Vec<InspectionDiag>, String> {
        let body = self.path_body(uri);
        let resp = self.post("/inspections", &body)?;
        if let Some(err) = resp.get("error") {
            return Err(err
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("INTERNAL")
                .to_string());
        }
        let diags = self.parse_inspections(&resp);
        self.last_meta = Self::parse_truncation(&resp, diags.len() as u32);
        Ok(diags)
    }

    fn list_inspections(&mut self) -> Result<Vec<InspectionInfo>, String> {
        let resp = self.post("/list_inspections", &serde_json::json!({ "path": "" }))?;
        if let Some(err) = resp.get("error") {
            return Err(err
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("INTERNAL")
                .to_string());
        }
        let items = Self::parse_inspection_list(&resp);
        self.last_meta = Self::parse_truncation(&resp, items.len() as u32);
        Ok(items)
    }
```

- [ ] **Step 6: Tests laufen lassen — müssen bestehen**

Run: `cargo nextest run inspections` (cwd: `rust`)
Expected: PASS — `inspections_parses_wire_diags`, `inspections_maps_error_envelope_to_err`, `list_inspections_parses_wire_items` grün.

---

## Task 3: Rust — `ctx_refactor`-Dispatch + Formatter (TDD)

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`

- [ ] **Step 1: Failing-Tests im `tests`-Modul ergänzen**

Per Serena `insert_after_symbol` nach `references_output_surfaces_truncation_note`. Der Stub überschreibt `inspections` + `list_inspections`; alle 5 mandatory Methoden müssen (wie bei den bestehenden Stubs) Trivial-Impls haben:

```rust
    #[test]
    fn inspections_run_and_list_dispatch_and_truncation() {
        use lsp_types::Position;
        struct InspBackend;
        impl crate::lsp::backend::LspBackend for InspBackend {
            fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> {
                Ok(())
            }
            fn references(
                &mut self,
                _u: &lsp_types::Uri,
                _p: lsp_types::Position,
                _s: &str,
            ) -> Result<Vec<lsp_types::Location>, String> {
                Ok(vec![])
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
            fn inspections(
                &mut self,
                _u: &lsp_types::Uri,
            ) -> Result<Vec<crate::lsp::backend::InspectionDiag>, String> {
                Ok(vec![crate::lsp::backend::InspectionDiag {
                    path: "A.kt".into(),
                    line: 7,
                    severity: "WARNING".into(),
                    message: "unused".into(),
                }])
            }
            fn list_inspections(
                &mut self,
            ) -> Result<Vec<crate::lsp::backend::InspectionInfo>, String> {
                Ok(vec![crate::lsp::backend::InspectionInfo {
                    id: "UnusedSymbol".into(),
                    name: "Unused declaration".into(),
                    severity: "WARNING".into(),
                }])
            }
            fn last_truncation(&self) -> Option<crate::lsp::backend::Truncation> {
                Some(crate::lsp::backend::Truncation {
                    truncated: true,
                    total: 99,
                })
            }
        }
        crate::lsp::router::seed_stub_backend("rust", Box::new(InspBackend));
        let uri = crate::lsp::client::file_path_to_uri("/proj/a.rs").unwrap();

        // run mode (default): formats path:line SEVERITY message + truncation note
        let run_out = super::handle_inspections(
            &json!({"action": "inspections"}),
            "/proj/a.rs",
            "/proj",
            &uri,
        );
        assert!(run_out.contains("A.kt:7"), "run diag missing: {run_out}");
        assert!(run_out.contains("WARNING"), "run severity missing: {run_out}");
        assert!(run_out.contains("unused"), "run message missing: {run_out}");
        assert!(run_out.contains("truncated"), "run truncation missing: {run_out}");
        assert!(run_out.contains("99"), "run total missing: {run_out}");

        // list mode: formats id name severity
        let list_out = super::handle_inspections(
            &json!({"action": "inspections", "mode": "list"}),
            "/proj/a.rs",
            "/proj",
            &uri,
        );
        assert!(list_out.contains("UnusedSymbol"), "list id missing: {list_out}");
        assert!(
            list_out.contains("Unused declaration"),
            "list name missing: {list_out}"
        );

        // unknown mode → defined ERROR
        let bad_out = super::handle_inspections(
            &json!({"action": "inspections", "mode": "bogus"}),
            "/proj/a.rs",
            "/proj",
            &uri,
        );
        assert!(bad_out.contains("ERROR"), "unknown mode not rejected: {bad_out}");
        let _ = (Position::new(0, 0),); // keep import used if refactored
    }
```

- [ ] **Step 2: Test laufen lassen — muss fehlschlagen**

Run: `cargo nextest run inspections_run_and_list_dispatch_and_truncation` (cwd: `rust`)
Expected: Kompilierfehler — `handle_inspections` existiert noch nicht.

- [ ] **Step 3: Imports erweitern**

Die bestehende `use`-Zeile in `ctx_refactor.rs` lautet:

```rust
use crate::lsp::backend::{HierarchyDirection, SymbolOverviewItem, TypeHierarchyNode};
```

Per Serena `replace_content` ersetzen durch:

```rust
use crate::lsp::backend::{
    HierarchyDirection, InspectionDiag, InspectionInfo, SymbolOverviewItem, TypeHierarchyNode,
};
```

- [ ] **Step 4: Dispatch in `handle` ergänzen**

Im `match action`-Block (`ctx_refactor.rs`) den `symbols_overview`-Arm um einen `inspections`-Arm erweitern. Per Serena `replace_content`:

```rust
        "symbols_overview" => handle_symbols_overview(abs_path, project_root, &uri),
```

ersetzen durch:

```rust
        "symbols_overview" => handle_symbols_overview(abs_path, project_root, &uri),
        "inspections" => handle_inspections(args, abs_path, project_root, &uri),
```

- [ ] **Step 5: Hilfetext im unknown-action-Arm erweitern**

Per Serena `replace_content` den `_ =>`-Arm-String:

```rust
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, \
             implementations, declaration, type_hierarchy, symbols_overview."
```

ersetzen durch:

```rust
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, \
             implementations, declaration, type_hierarchy, symbols_overview, inspections."
```

- [ ] **Step 6: `handle_inspections` + Formatter einfügen**

Per Serena `insert_after_symbol` nach `handle_symbols_overview`:

```rust
fn handle_inspections(
    args: &Value,
    file_path: &str,
    project_root: &str,
    uri: &lsp_types::Uri,
) -> String {
    let mode = args.get("mode").and_then(Value::as_str).unwrap_or("run");
    match mode {
        "run" => {
            let result =
                crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
                    let diags = backend.inspections(uri)?;
                    Ok((diags, backend.last_truncation()))
                });
            match result {
                Ok((diags, meta)) => {
                    let mut out = format_inspections(&diags);
                    out.push_str(&truncation_note(diags.len(), meta));
                    out
                }
                Err(e) => format!("ERROR: {e}"),
            }
        }
        "list" => {
            let result =
                crate::lsp::router::with_backend(file_path, project_root, |backend, _| {
                    let items = backend.list_inspections()?;
                    Ok((items, backend.last_truncation()))
                });
            match result {
                Ok((items, meta)) => {
                    let mut out = format_inspection_list(&items);
                    out.push_str(&truncation_note(items.len(), meta));
                    out
                }
                Err(e) => format!("ERROR: {e}"),
            }
        }
        other => format!("ERROR: Unknown mode '{other}' for inspections. Available: run, list."),
    }
}

fn format_inspections(diags: &[InspectionDiag]) -> String {
    if diags.is_empty() {
        return "No inspection findings.".to_string();
    }
    let mut out = format!("{} finding(s):\n", diags.len());
    for d in diags {
        out.push_str(&format!(
            "  {}:{}  {}  {}\n",
            d.path, d.line, d.severity, d.message
        ));
    }
    out
}

fn format_inspection_list(items: &[InspectionInfo]) -> String {
    if items.is_empty() {
        return "No inspections enabled.".to_string();
    }
    let mut out = format!("{} inspection(s):\n", items.len());
    for i in items {
        out.push_str(&format!("  {}  {}  {}\n", i.id, i.name, i.severity));
    }
    out
}
```

- [ ] **Step 7: Test laufen lassen — muss bestehen**

Run: `cargo nextest run inspections_run_and_list_dispatch_and_truncation` (cwd: `rust`)
Expected: PASS.

- [ ] **Step 8: Bestehenden Help-Text-Test anpassen**

Der Test `unknown_action_help_lists_declaration` prüft nur `declaration`. Optional einen Assert ergänzen, dass der Help-Text jetzt `inspections` enthält. Per Serena `replace_content` den Assert-Block in `unknown_action_help_lists_declaration`:

```rust
        assert!(
            out.contains("declaration"),
            "help text missing declaration: {out}"
        );
```

ersetzen durch:

```rust
        assert!(
            out.contains("declaration"),
            "help text missing declaration: {out}"
        );
        assert!(
            out.contains("inspections"),
            "help text missing inspections: {out}"
        );
```

- [ ] **Step 9: Gesamten Tool-Test + Clippy laufen lassen**

Run: `cargo nextest run` (cwd: `rust`)
Expected: alle Tests grün.
Run: `cargo clippy --all-targets` (cwd: `rust`)
Expected: keine neuen Lints.

---

## Task 4: Rust — MCP-Schema (`action`-Enum + `mode`-Param)

**Files:**
- Modify: `rust/src/tools/registered/ctx_refactor.rs`

- [ ] **Step 1: `schema_test` erweitern (Failing-Test)**

Per Serena `replace_content` im `schema_tests`-Modul das Needle-Array:

```rust
        for needle in [
            "declaration",
            "\"scope\"",
            "type_hierarchy",
            "symbols_overview",
            "\"direction\"",
            "supertypes",
            "subtypes",
        ] {
```

ersetzen durch:

```rust
        for needle in [
            "declaration",
            "\"scope\"",
            "type_hierarchy",
            "symbols_overview",
            "\"direction\"",
            "supertypes",
            "subtypes",
            "inspections",
            "\"mode\"",
        ] {
```

- [ ] **Step 2: Test laufen lassen — muss fehlschlagen**

Run: `cargo nextest run schema_advertises` (cwd: `rust`)
Expected: FAIL — Schema enthält weder `inspections` noch `"mode"`.

- [ ] **Step 3: `action`-Enum erweitern**

Per Serena `replace_content` in `tool_def`:

```rust
                        "enum": ["rename", "references", "definition", "implementations",
                                 "declaration", "type_hierarchy", "symbols_overview"],
```

ersetzen durch:

```rust
                        "enum": ["rename", "references", "definition", "implementations",
                                 "declaration", "type_hierarchy", "symbols_overview", "inspections"],
```

- [ ] **Step 4: `mode`-Property + Beschreibung einfügen**

Per Serena `replace_content` den `direction`-Property-Block:

```rust
                    "direction": {
                        "type": "string",
                        "enum": ["supertypes", "subtypes"],
                        "description": "type_hierarchy direction (JetBrains backend). 'supertypes' (default) = parents; 'subtypes' = children/implementors."
                    }
```

ersetzen durch (Komma nach dem `direction`-Block beachten):

```rust
                    "direction": {
                        "type": "string",
                        "enum": ["supertypes", "subtypes"],
                        "description": "type_hierarchy direction (JetBrains backend). 'supertypes' (default) = parents; 'subtypes' = children/implementors."
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["run", "list"],
                        "description": "inspections mode (JetBrains backend). 'run' (default) = diagnostics for the given file; 'list' = enabled inspections of the current project profile."
                    }
```

- [ ] **Step 5: Tool-Beschreibung um `inspections` ergänzen**

Per Serena `replace_content` den `tool_def`-Beschreibungstext:

```rust
            "LSP-powered refactoring. Actions: rename, references, definition, implementations, \
             declaration, type_hierarchy, symbols_overview. Requires a running language server \
             (rust-analyzer, typescript-language-server, pylsp, gopls) or the JetBrains backend \
             (declaration, type_hierarchy, symbols_overview are JetBrains-only).",
```

ersetzen durch:

```rust
            "LSP-powered refactoring. Actions: rename, references, definition, implementations, \
             declaration, type_hierarchy, symbols_overview, inspections. Requires a running \
             language server (rust-analyzer, typescript-language-server, pylsp, gopls) or the \
             JetBrains backend (declaration, type_hierarchy, symbols_overview, inspections are \
             JetBrains-only).",
```

- [ ] **Step 6: Test laufen lassen — muss bestehen**

Run: `cargo nextest run schema_advertises` (cwd: `rust`)
Expected: PASS.

---

## Task 5: Rust — Doc-Gen regenerieren + Appendix pflegen (Drift-Gate)

**Files:**
- Modify: `docs/reference/generated/mcp-tools.md` (generiert — nicht von Hand editieren)
- Modify: `docs/reference/appendix-mcp-tools.md` (handgepflegt)

- [ ] **Step 1: Generierte Referenz + Manifest neu erzeugen**

Run: `cargo run --example gen_docs --features dev-tools` (cwd: `rust`)
Run: `cargo run --example gen_mcp_manifest --features dev-tools` (cwd: `rust`)
Expected: `docs/reference/generated/mcp-tools.md` und `website/generated/mcp-tools.json` aktualisiert (enthalten jetzt `inspections` + `mode`). Verifiziere mit `ctx_read docs/reference/generated/mcp-tools.md mode=diff`.

- [ ] **Step 2: Appendix `ctx_refactor`-Zeile aktualisieren**

In `docs/reference/appendix-mcp-tools.md` Zeile 84 (native `Edit`, da Markdown). Aktuell:

```
| `ctx_refactor` | LSP-backed refactoring | rename\|references\|definition\|implementations | S |
```

ersetzen durch (Actions-Liste vollständig, inkl. `inspections`):

```
| `ctx_refactor` | LSP-backed refactoring | rename\|references\|definition\|implementations\|declaration\|type_hierarchy\|symbols_overview\|inspections | S |
```

- [ ] **Step 3: Drift-Gate prüfen**

Run: `cargo nextest run reference_docs_drift docs_tool_counts_up_to_date mcp_manifest_up_to_date` (cwd: `rust`)
Expected: alle drei PASS (kein Drift).

---

## Task 6: Kotlin — Wire-DTOs + `JsonCodecTest` (TDD)

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt`
- Modify: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt`

- [ ] **Step 1: Failing-Tests in `JsonCodecTest.kt` ergänzen (native Edit)**

Vor der schließenden `}` von `class JsonCodecTest` einfügen:

```kotlin
    @Test
    fun inspectionsResponseRoundTrips() {
        val resp = InspectionsResponse(
            diagnostics = listOf(InspectionDiagDTO("A.kt", 3, "WARNING", "unused variable")),
            truncated = true,
            total = 42,
        )
        val json = JsonCodec.toJson(resp)
        assertTrue(json.contains("\"diagnostics\""))
        assertTrue(json.contains("\"path\":\"A.kt\""))
        assertTrue(json.contains("\"severity\":\"WARNING\""))
        assertTrue(json.contains("\"truncated\":true"))
        assertTrue(json.contains("\"total\":42"))
    }

    @Test
    fun listInspectionsResponseRoundTrips() {
        val resp = ListInspectionsResponse(
            inspections = listOf(InspectionInfoDTO("UnusedSymbol", "Unused declaration", "WARNING")),
            truncated = false,
            total = 1,
        )
        val json = JsonCodec.toJson(resp)
        assertTrue(json.contains("\"inspections\""))
        assertTrue(json.contains("\"id\":\"UnusedSymbol\""))
        assertTrue(json.contains("\"name\":\"Unused declaration\""))
        assertTrue(json.contains("\"truncated\":false"))
    }

    @Test
    fun parseFileRequestReusedForInspections() {
        // Both /inspections and /list_inspections use the {path} body → parseFileRequest.
        val req = JsonCodec.parseFileRequest("""{"path":"src/A.kt"}""")
        assertEquals("src/A.kt", req.path)
    }
```

- [ ] **Step 2: Test laufen lassen — muss fehlschlagen**

Run: `./gradlew test --tests "com.leanctx.plugin.dto.JsonCodecTest" --console=plain` (cwd: `packages/jetbrains-lean-ctx`)
Expected: Kompilierfehler — `InspectionsResponse`, `InspectionDiagDTO`, `ListInspectionsResponse`, `InspectionInfoDTO` sind unbekannt.

- [ ] **Step 3: DTOs in `Wire.kt` einfügen (native Edit)**

Direkt vor `object JsonCodec {` in `Wire.kt` einfügen:

```kotlin
/** A single inspection diagnostic. `line` is 1-BASED (matches Rust InspectionDiag.line). */
data class InspectionDiagDTO(
    val path: String,
    val line: Int,
    val severity: String,
    val message: String,
)

data class InspectionsResponse(
    val diagnostics: List<InspectionDiagDTO>,
    val truncated: Boolean,
    val total: Int,
)

/** A single available inspection (the `list` mode). */
data class InspectionInfoDTO(val id: String, val name: String, val severity: String)

data class ListInspectionsResponse(
    val inspections: List<InspectionInfoDTO>,
    val truncated: Boolean,
    val total: Int,
)

```

> Keine neue `JsonCodec`-Methode nötig: `parseFileRequest` (Body `{path}`) wird für beide Endpoints wiederverwendet; `toJson` serialisiert die neuen Responses generisch.

- [ ] **Step 4: Test laufen lassen — muss bestehen**

Run: `./gradlew test --tests "com.leanctx.plugin.dto.JsonCodecTest" --console=plain` (cwd: `packages/jetbrains-lean-ctx`)
Expected: PASS.

---

## Task 7: Kotlin — `InspectionRunner` (PSI-Logik)

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/InspectionRunner.kt`

> **Abhängigkeit:** Setzt die in Task 0 bestätigten Signaturen voraus. Falls Task 0 Abweichungen ergab, diesen Code-Block entsprechend anpassen. Severity-Mapping nutzt `HighlightSeverity`-Vergleich (Comparable) statt Namens-Strings, um gegen Anzeigename-Varianten („WEAK WARNING" mit Leerzeichen) robust zu sein.

- [ ] **Step 1: Datei anlegen (native Write)**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.codeHighlighting.HighlightDisplayLevel
import com.intellij.codeInspection.InspectionEngine
import com.intellij.codeInspection.InspectionManager
import com.intellij.codeInspection.ex.InspectionManagerEx
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.profile.codeInspection.InspectionProjectProfileManager
import com.intellij.psi.PsiDocumentManager
import com.intellij.psi.PsiFile
import com.leanctx.plugin.dto.InspectionDiagDTO
import com.leanctx.plugin.dto.InspectionInfoDTO
import com.leanctx.plugin.dto.InspectionsResponse
import com.leanctx.plugin.dto.ListInspectionsResponse
import com.leanctx.plugin.server.BackendException

/**
 * Runs / lists inspections from the current project InspectionProfile (spec §3.2, §6).
 * Read-only: never writes the file. Caps results at MAX_* with `truncated`/`total`.
 * Must be invoked inside a smart-mode ReadAction (handlers use PsiLocator.inSmartReadAction).
 */
class InspectionRunner(private val locator: PsiLocator) {

    companion object {
        const val MAX_DIAGNOSTICS = 500
        const val MAX_INSPECTIONS = 500
    }

    /** Run all enabled inspections of the project profile on [file]; [relPath] labels each diag. */
    fun runOnFile(file: PsiFile, relPath: String): InspectionsResponse {
        val project = file.project
        val doc = PsiDocumentManager.getInstance(project).getDocument(file)
            ?: throw BackendException("INTERNAL", "no document for ${file.name}")
        val profile = InspectionProjectProfileManager.getInstance(project).currentProfile
        val manager = InspectionManager.getInstance(project) as InspectionManagerEx
        val context = manager.createNewGlobalContext()

        val out = ArrayList<InspectionDiagDTO>()
        var total = 0
        for (tools in profile.getAllEnabledInspectionTools(project)) {
            val severity = mapSeverity(tools.level)
            val problems = InspectionEngine.runInspectionOnFile(file, tools.tool, context)
            for (p in problems) {
                total++
                if (out.size >= MAX_DIAGNOSTICS) continue
                val element = p.psiElement ?: continue
                val range = element.textRange ?: continue
                val line = doc.getLineNumber(range.startOffset) + 1 // 1-based wire
                out.add(InspectionDiagDTO(relPath, line, severity, p.descriptionTemplate))
            }
        }
        return InspectionsResponse(out, total > out.size, total)
    }

    /** List the enabled inspections of the current project profile (capped). */
    fun listAvailable(project: com.intellij.openapi.project.Project): ListInspectionsResponse {
        val profile = InspectionProjectProfileManager.getInstance(project).currentProfile
        val tools = profile.getAllEnabledInspectionTools(project)
        val out = ArrayList<InspectionInfoDTO>()
        var truncated = false
        for (t in tools) {
            if (out.size >= MAX_INSPECTIONS) { truncated = true; break }
            val w = t.tool
            out.add(InspectionInfoDTO(w.shortName, w.displayName ?: w.shortName, mapSeverity(t.level)))
        }
        return ListInspectionsResponse(out, truncated, tools.size)
    }

    /** Map IntelliJ HighlightDisplayLevel → fixed wire token (spec §4). */
    private fun mapSeverity(level: HighlightDisplayLevel): String {
        val sev = level.severity
        return when {
            sev >= HighlightSeverity.ERROR -> "ERROR"
            sev >= HighlightSeverity.WARNING -> "WARNING"
            sev >= HighlightSeverity.WEAK_WARNING -> "WEAK_WARNING"
            else -> "INFO"
        }
    }
}
```

- [ ] **Step 2: Kompilieren**

Run: `./gradlew compileKotlin --console=plain` (cwd: `packages/jetbrains-lean-ctx`)
Expected: kompiliert (eventuell `unused`-Warning für `locator`, solange Task 8 ihn noch nicht nutzt — kein Fehler). Falls ein Symbol nicht auflöst, gegen die Task-0-Findings abgleichen und Import/Signatur korrigieren.

---

## Task 8: Kotlin — `InspectionHandlers` (dünner Handler)

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/InspectionHandlers.kt`

> Muster = `StructureHandlers`: hält `PsiLocator` + Runner, läuft PSI in `inSmartReadAction`, gibt die Wire-Response zurück; `BackendException` propagiert an den Router.

- [ ] **Step 1: Datei anlegen (native Write)**

```kotlin
package com.leanctx.plugin.endpoint

import com.intellij.openapi.project.Project
import com.leanctx.plugin.dto.FileRequest
import com.leanctx.plugin.dto.InspectionsResponse
import com.leanctx.plugin.dto.ListInspectionsResponse
import com.leanctx.plugin.psi.InspectionRunner
import com.leanctx.plugin.psi.PsiLocator

/**
 * Endpoint layer for the Phase-5b inspections ops (run + list). Each runs PSI inside a
 * smart-mode ReadAction; BackendException (typed code) propagates to the RequestRouter
 * for the error envelope.
 */
class InspectionHandlers(private val project: Project) {
    private val locator = PsiLocator(project)
    private val runner = InspectionRunner(locator)

    fun runOnFile(req: FileRequest): InspectionsResponse = locator.inSmartReadAction {
        runner.runOnFile(locator.psiFile(req.path), req.path)
    }

    fun listAvailable(req: FileRequest): ListInspectionsResponse = locator.inSmartReadAction {
        runner.listAvailable(project)
    }
}
```

- [ ] **Step 2: Kompilieren**

Run: `./gradlew compileKotlin --console=plain` (cwd: `packages/jetbrains-lean-ctx`)
Expected: kompiliert.

---

## Task 9: Kotlin — Router-Routen + End-to-End-Test (TDD)

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt`
- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterInspectionTest.kt`

- [ ] **Step 1: Failing-Test anlegen (native Write)**

Muster = `RequestRouterStructureTest`. `runInspectionsRoute` prüft 200 + `diagnostics`-Key (nicht die konkreten Findings — die Light-Fixture-Datei ist kein indizierter Source-Root, sodass profile-getriebene Inspektionen ggf. leer bleiben; die Robustheit liegt im Wiring, nicht im Inhalt). `listInspectionsRoute` prüft eine nicht-leere Profil-Liste (das Default-Profil hat immer enabled Tools).

```kotlin
package com.leanctx.plugin.server

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.application.WriteAction
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.nio.file.Files
import java.nio.file.Paths

class RequestRouterInspectionTest : BasePlatformTestCase() {

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

    fun testRunInspectionsRoute() {
        val rel = writeSource("InspA.kt", "fun main() {\n  val x = 1\n}\n")
        val res = routeOffEdt("POST", "/inspections", """{"path":"$rel"}""")
        assertEquals("body=${res.body}", 200, res.status)
        assertTrue("body=${res.body}", res.body.contains("\"diagnostics\""))
        assertTrue("body=${res.body}", res.body.contains("\"total\""))
    }

    fun testListInspectionsRoute() {
        // path is only for backend selection; the list is project-wide.
        val res = routeOffEdt("POST", "/list_inspections", """{"path":""}""")
        assertEquals("body=${res.body}", 200, res.status)
        assertTrue("body=${res.body}", res.body.contains("\"inspections\""))
        // The default project profile always has enabled tools → non-empty id token present.
        assertTrue("body=${res.body}", res.body.contains("\"id\""))
    }

    fun testInspectionsWrongTokenIs401() {
        val res = router().route("POST", "/inspections", "WRONG", "{}")
        assertEquals(401, res.status)
        assertTrue(res.body.contains("UNAUTHORIZED"))
    }

    fun testRunInspectionsFileNotFoundIs200Envelope() {
        val res = routeOffEdt("POST", "/inspections", """{"path":"Nope.kt"}""")
        assertEquals(200, res.status)
        assertTrue(res.body.contains("FILE_NOT_FOUND"))
    }
}
```

- [ ] **Step 2: Test laufen lassen — muss fehlschlagen**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterInspectionTest" --console=plain` (cwd: `packages/jetbrains-lean-ctx`)
Expected: FAIL — die Routen `/inspections` + `/list_inspections` liefern 404 (noch nicht registriert).

- [ ] **Step 3: `InspectionHandlers`-Feld im Router instanziieren (native Edit)**

In `RequestRouter.kt` nach der `structureHandlers`-Zeile:

```kotlin
    private val structureHandlers = StructureHandlers(project)
```

ergänzen:

```kotlin
    private val structureHandlers = StructureHandlers(project)
    private val inspectionHandlers = InspectionHandlers(project)
```

Import oben ergänzen (nach `import com.leanctx.plugin.endpoint.StructureHandlers`):

```kotlin
import com.leanctx.plugin.endpoint.InspectionHandlers
```

- [ ] **Step 4: Routen im `POST`-Block einfügen (native Edit)**

In `route(...)` den Block:

```kotlin
        if (method == "POST") {
            if (path == "/type_hierarchy") return dispatchHierarchy(body)
            if (path == "/symbols_overview") return dispatchOverview(body)
```

ersetzen durch:

```kotlin
        if (method == "POST") {
            if (path == "/type_hierarchy") return dispatchHierarchy(body)
            if (path == "/symbols_overview") return dispatchOverview(body)
            if (path == "/inspections") return dispatchInspections(body)
            if (path == "/list_inspections") return dispatchListInspections(body)
```

- [ ] **Step 5: Zwei Dispatcher einfügen (native Edit)**

Nach der `dispatchOverview`-Funktion (vor `private fun q(...)`) einfügen — Muster identisch zu `dispatchOverview` (200-Envelope für `BackendException`/`IllegalArgumentException`, 500 für echte `Exception`):

```kotlin
    private fun dispatchInspections(body: String): HttpResult = try {
        val req = JsonCodec.parseFileRequest(body)
        HttpResult(200, JsonCodec.toJson(inspectionHandlers.runOnFile(req)))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("inspections endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }

    private fun dispatchListInspections(body: String): HttpResult = try {
        val req = JsonCodec.parseFileRequest(body)
        HttpResult(200, JsonCodec.toJson(inspectionHandlers.listAvailable(req)))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code))
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("list_inspections endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }
```

- [ ] **Step 6: Test laufen lassen — muss bestehen**

Run: `./gradlew test --tests "com.leanctx.plugin.server.RequestRouterInspectionTest" --console=plain` (cwd: `packages/jetbrains-lean-ctx`)
Expected: PASS (alle 4 Testmethoden). Falls `testListInspectionsRoute` eine leere Liste liefert (Headless-Profil ohne enabled Tools), den Assert auf `res.body.contains("\"inspections\"")` reduzieren und in `ctx_knowledge` notieren — die nicht-leere Erwartung gilt dann nur im manuellen `runIde`-Gate (Task 11 / Spec §5.5).

- [ ] **Step 7: Volle Kotlin-Suite (Port-Hygiene + Regression)**

Run: `./gradlew check --console=plain` (cwd: `packages/jetbrains-lean-ctx`)
Expected: alle Tests grün; Suite hinterlässt keine Port-Dateien (H5a-Hygiene, `PortFileHygieneTest`).

---

## Task 10: CI-Härtung (`jetbrains-plugin.yml`)

**Files:**
- Modify: `.github/workflows/jetbrains-plugin.yml`

> SHA-Pins werden NICHT geraten. Step 1 ermittelt die realen SHAs; Step 2–5 tragen sie ein. Format pro `uses:`: `owner/repo@<40-hex-sha> # vX.Y.Z`.

- [ ] **Step 1: Aktuelle Release-SHAs ermitteln**

Für jede verwendete Action die SHA des aktuellen Release-Tags holen (native `Bash`/`ctx_shell`):

```
gh api repos/actions/checkout/git/ref/tags/v4.2.2 --jq .object.sha
gh api repos/actions/setup-java/git/ref/tags/v4.7.0 --jq .object.sha
gh api repos/gradle/actions/git/ref/tags/v4.2.2 --jq .object.sha
gh api repos/actions/upload-artifact/git/ref/tags/v4.6.0 --jq .object.sha
```

> Tag-Namen ggf. an die jeweils neueste v4-Patch-Version anpassen (`gh api repos/<owner>/<repo>/releases/latest --jq .tag_name`). `gradle/actions/setup-gradle` und `gradle/actions/wrapper-validation` teilen sich dasselbe Repo (`gradle/actions`) → gleiche SHA, unterschiedlicher Sub-Pfad.

- [ ] **Step 2: `concurrency`-Group ergänzen (native Edit)**

Nach dem `permissions:`-Block (vor `defaults:`) einfügen:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

- [ ] **Step 3: `timeout-minutes` an alle 3 Jobs (native Edit)**

- `build`-Job: nach `runs-on: ubuntu-latest` → `timeout-minutes: 20`
- `test`-Job: nach `runs-on: ubuntu-latest` → `timeout-minutes: 30` (IC-Download ~1 GB)
- `release`-Job: nach `runs-on: ubuntu-latest` → `timeout-minutes: 20`

Beispiel `build`:

```yaml
  build:
    name: Build
    runs-on: ubuntu-latest
    timeout-minutes: 20
    steps:
```

- [ ] **Step 4: Alle `uses:` auf SHA pinnen (native Edit)**

Jede `uses:`-Zeile ersetzen (SHAs aus Step 1 einsetzen). Es gibt diese Vorkommen:
- `actions/checkout@v4` (3×, in build/test/release) → `actions/checkout@<sha> # v4.2.2`
- `gradle/actions/wrapper-validation@v4` (1×) → `gradle/actions/wrapper-validation@<sha> # v4.2.2`
- `actions/setup-java@v4` (3×) → `actions/setup-java@<sha> # v4.7.0`
- `gradle/actions/setup-gradle@v4` (3×) → `gradle/actions/setup-gradle@<sha> # v4.2.2`
- `actions/upload-artifact@v4` (1×) → `actions/upload-artifact@<sha> # v4.6.0`

Beispiel:

```yaml
      - name: Fetch Sources
        uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
```

> Die obige SHA ist ein Platzhalter-Beispiel — die realen Werte aus Step 1 verwenden.

- [ ] **Step 5: `actionlint`-Job ergänzen (native Edit)**

Als eigenen Job am Anfang der `jobs:`-Liste (kein Docker, leichtgewichtig; nutzt das offizielle Install-Skript). `working-directory`-Default greift nicht für den Download-Schritt → expliziter Pfad:

```yaml
  actionlint:
    name: Actionlint
    runs-on: ubuntu-latest
    timeout-minutes: 5
    defaults:
      run:
        working-directory: .
    steps:
      - name: Fetch Sources
        uses: actions/checkout@<sha> # v4.2.2
        with:
          persist-credentials: false
      - name: Run actionlint
        run: |
          bash <(curl -sSf https://raw.githubusercontent.com/rhysd/actionlint/main/scripts/download-actionlint.bash)
          ./actionlint -color .github/workflows/jetbrains-plugin.yml
```

- [ ] **Step 6: Workflow lokal validieren**

Falls `actionlint` lokal installiert ist (`actionlint --version`), ausführen:

Run: `actionlint .github/workflows/jetbrains-plugin.yml` (cwd: Projektwurzel)
Expected: keine Fehler. Falls nicht installiert: `go install github.com/rhysd/actionlint/cmd/actionlint@latest` oder das Download-Skript aus Step 5 lokal. YAML-Syntax mindestens via `ctx_read` gegenprüfen (korrekte Einrückung, gültige Expressions).

---

## Task 11: Manuelles `runIde`-Gate (user-gated) + Findings

**Files:** keine Änderung — Verifikation.

> Dieser Task ist **user-gated** (benötigt eine laufende IC/IU-2026.1.x-IDE auf dem Plugin-Projekt). Falls die IDE nicht verfügbar ist: Task überspringen, in `ctx_knowledge` als offenes Gate vermerken, NICHT blockieren.

- [ ] **Step 1: `run`-Pfad gegen die laufende IDE prüfen**

In einer IDE mit gestartetem Plugin-Backend: `ctx_refactor action=inspections mode=run path=<Datei mit bekanntem Problem>`.
Expected: nicht-leere, erwartete Diagnostics (`path:line SEVERITY message`).

- [ ] **Step 2: `list`-Pfad prüfen**

`ctx_refactor action=inspections mode=list path=<beliebige Projekt-Datei>`.
Expected: nicht-leere Profil-Liste (`id name severity`), ggf. `truncated`-Suffix.

- [ ] **Step 3: Sauberer Fallback nach IDE-Schließen**

IDE schließen → erneuter `mode=run`-Call.
Expected: fällt sauber auf `ERROR: inspections requires the JetBrains backend` (Backing A) bzw. `ERROR` (b_only) — kein Hänger gegen toten Endpoint.

- [ ] **Step 4: Findings festhalten**

`ctx_knowledge action=remember category=testing` — Ergebnis des manuellen Gates (Severity-Mapping korrekt, Liste nicht-leer, Fallback sauber).

---

## Task 12: Final-Gate + EIN Commit

**Files:** alle in diesem Plan geänderten/erstellten Dateien.

- [ ] **Step 1: Vollständige Rust-Gates**

Run: `cargo nextest run` (cwd: `rust`)
Expected: alle grün (inkl. `inspections*`, `schema_advertises`, `reference_docs_drift`, `docs_tool_counts_up_to_date`, `mcp_manifest_up_to_date`).
Run: `cargo clippy --all-targets` (cwd: `rust`)
Expected: keine neuen Lints.

- [ ] **Step 2: Vollständige Kotlin-Gates**

Run: `./gradlew check --console=plain` (cwd: `packages/jetbrains-lean-ctx`)
Expected: grün, keine Port-Dateien hinterlassen.

- [ ] **Step 3: Reformat aller geänderten Dateien**

Vor `git add` (Projekt-Hard-Rule): `mcp__jetbrains__reformat_file` auf jede geänderte/neue Datei (Rust + Kotlin + YAML). Falls ein Tool deferred ist: `ToolSearch(query="select:mcp__jetbrains__reformat_file")` zuerst.

- [ ] **Step 4: Status sichten**

Run: `git status --porcelain` (cwd: Projektwurzel, via `ctx_shell` bare command + `cwd=`)
Expected: nur die geplanten Dateien geändert/neu. Verifiziere, dass `docs/reference/generated/mcp-tools.md` + `website/generated/mcp-tools.json` enthalten sind (Drift-Gate-Artefakte).

- [ ] **Step 5: EIN Commit (Spec §12.3)**

```bash
git add rust/src/lsp/backend.rs rust/src/lsp/jetbrains_backend.rs \
  rust/src/tools/ctx_refactor.rs rust/src/tools/registered/ctx_refactor.rs \
  docs/reference/generated/mcp-tools.md docs/reference/appendix-mcp-tools.md \
  website/generated/mcp-tools.json \
  packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt \
  packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/InspectionRunner.kt \
  packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/InspectionHandlers.kt \
  packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt \
  packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt \
  packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterInspectionTest.kt \
  .github/workflows/jetbrains-plugin.yml
git commit -m "feat(jetbrains): Phase-5b inspections (run+list) + CI-Härtung

- ctx_refactor action=inspections, mode=run|list (Variante A dispatch)
- Rust: InspectionInfo + list_inspections trait method, JetBrains backend
  parsers/methods, tool dispatch + formatters, MCP schema + doc-gen
- Kotlin: Wire DTOs, InspectionRunner (PSI), InspectionHandlers,
  /inspections + /list_inspections routes, router-driven tests
- CI: concurrency, timeout-minutes, action SHA-pinning, actionlint gate"
```

Expected: ein Commit auf `feat-jetbrains-plugin`.

---

## Self-Review (gegen Spec geprüft)

- **F1 (`inspections mode=run`):** Task 2 (Backend), Task 3 (Dispatch/Format), Task 7–9 (Kotlin) ✓
- **F2 (`inspections mode=list`):** Task 1 (`list_inspections` Trait), Task 2/3/7–9 ✓
- **F3 (CI: concurrency/timeout/SHA-Pin):** Task 10 Steps 2–4 ✓
- **F4 (`actionlint`-Gate):** Task 10 Step 5 ✓
- **Entscheidung #2 (`mode`-Dispatch Variante A):** Task 3 `handle_inspections` + Task 4 Schema ✓
- **Entscheidung #3 (Kotlin-only Tests):** Task 6/9 Kotlin-Fixtures ✓
- **Entscheidung #6 (nur enabled Profil + Cap):** Task 7 `getAllEnabledInspectionTools` + `MAX_*`/`truncated` ✓
- **Wire §4 (zwei snake_case-Endpoints, 1-basierte Zeile, Severity-Tokens):** Task 6 DTOs, Task 7 `mapSeverity` + 1-based, Task 9 Routen ✓
- **Verifikation §5 (cargo nextest, gradlew check, Drift-Gate, actionlint, runIde, Fallback):** Task 5, 11, 12 ✓
- **Tests §6 (RequestRouterInspectionTest, JsonCodecTest, Coverage-Matrix):** Task 6, 9 ✓
- **Risiken §8 (Severity-Mapping via Comparable, INDEXING via inSmartReadAction, Volumen-Cap):** Task 7 ✓
- **`format` bleibt v2 (Entscheidung #1):** kein Task — Trait-Stub bleibt unangetastet ✓
- **Typkonsistenz:** `InspectionDiag{path,line,severity,message}` / `InspectionInfo{id,name,severity}` (Rust) ↔ `InspectionDiagDTO`/`InspectionInfoDTO` (Kotlin) durchgängig; `runOnFile`/`listAvailable`/`handle_inspections`/`parse_inspections`/`parse_inspection_list` konsistent benannt ✓
- **Commit (§12.3 ein Commit):** nur Task 12 committet ✓
