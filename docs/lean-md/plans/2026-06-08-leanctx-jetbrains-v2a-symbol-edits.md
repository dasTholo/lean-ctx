# lean-ctx JetBrains v2a — Symbolische Body-Edits — Implementierungsplan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serenas `replace_symbol_body` / `insert_before_symbol` / `insert_after_symbol` durch drei neue `ctx_refactor`-Actions ablösen — `name_path`-adressiert (Rust-Symbol-Index), IDE-first mit verlustfreiem Headless-Fallback, byte-identisch in beiden Pfaden.

**Architecture:** `ctx_refactor` löst `name_path` Rust-seitig über den bestehenden tree-sitter-Symbol-Index (`graph_provider`) auf eine `(path, range)` auf, jailt den Pfad (PathJail) und baut den finalen Wire-Text (inkl. Einrückung) **vor** dem Apply. Der Apply-Pfad wird über die Port-Datei entschieden: lebende IDE → `JetBrainsHttpBackend` (HTTP → `WriteCommandAction`, VFS-Kohärenz + Undo); sonst → `local_range_write` (lokaler atomarer Range-Write, identische Bytes). Die kanonische Edit-Grenze ist in **beiden** Pfaden dieselbe tree-sitter-Range → IDE-Pfad ≡ Headless-Pfad. Zusätzlich bekommt `symbols_overview` einen tree-sitter-Default-Impl (verlustfreier Headless-Read).

**Tech Stack:** Rust (`serde_json`, `lsp_types`, `ureq` 3.x, `crate::core::hasher::hash_hex` = md5-hex, `cargo nextest`), Kotlin (IntelliJ Platform IC-2026.1.3, `WriteCommandAction`, `PsiDocumentManager`, `FileDocumentManager`, `gson`, `BasePlatformTestCase`), Gradle.

**Commit-Strategie (Eltern-Spec §12, überschreibt das writing-plans-Default):** v2a = **EIN Commit** auf `feat-jetbrains-plugin`. Zwischen-Tasks werden **nicht** committet; jeder Task endet mit einem **grünen Gate** (Build/Tests laufen). Der finale Task (Task 16) führt das Gesamt-Gate aus, regeneriert die Schema-Docs und erstellt den **einen** Commit. **Kein worktree** (Projekt-Rule).

**Tool-Disziplin (Projekt-Hard-Rules):** Rust-Dateien (`*.rs`) **nur** via Serena-Tools editieren (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_after_symbol`, `insert_before_symbol`, `replace_content`), **nie** native `Edit`/`ctx_edit`. Kotlin/Markdown: native `Edit`/`Write`. Vor `git add`: `mcp__jetbrains__reformat_file` auf jede geänderte Datei. Lesen via `ctx_read`, Suchen via `ctx_search`, Shell via `ctx_shell` (bare command + `cwd=`, nie `cd … &&`, nie `2>&1`). Tests: immer `cargo nextest run`, nie `cargo test`. Bei deferred Tool: zuerst `ToolSearch(query="select:<tool>")`.

---

## Dateienübersicht

**Rust (Backing-/Tool-Schicht):**
- Modify: `rust/src/lsp/backend.rs` — neue Typen `TextRange0Based`, `RangeEdit`, `EditResult`; drei neue default-Apply-Trait-Methoden (`replace_symbol_body`/`insert_before_symbol`/`insert_after_symbol`, Default = `local_range_write`); `symbols_overview`-Default-Impl von `Err` → tree-sitter.
- Create: `rust/src/lsp/edit_apply.rs` — `local_range_write` (gemeinsamer headless Range-Write + Diff + `expected_hash`-Guard) und `offset_of` (0-based line/char → Byte-Offset).
- Modify: `rust/src/lsp/mod.rs` — `pub mod edit_apply;`.
- Modify: `rust/src/lsp/jetbrains_backend.rs` — Override der drei Edit-Methoden (HTTP), Parser `parse_edit_result`, Mock-Tests.
- Modify: `rust/src/tools/ctx_refactor.rs` — `name_path`-Auflösung (`resolve_name_path`), Einrück-Berechnung, drei Action-Handler, Apply-Dispatch (`apply_symbol_edit`), Diff-Format, Tests.
- Modify: `rust/src/tools/registered/ctx_refactor.rs` — Schema (`action`-Enum + `name_path`/`new_body`/`text`/`expected_hash`), `schema_test`.
- Modify: `docs/reference/generated/mcp-tools.md` (generiert), `docs/reference/appendix-mcp-tools.md` (handgepflegt).

**Kotlin (Plugin-Schicht):**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt` — `EditRequest`/`EditResponse`-DTOs + `JsonCodec.parseEditRequest`.
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolEditor.kt` — `WriteCommandAction`-Range-Write.
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/EditHandlers.kt` — drei Handler.
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt` — drei Routen + drei Dispatcher.
- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterEditTest.kt` — router-getriebener End-to-End-Edit-Test.
- Modify: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt` — Edit-DTO-Round-Trip.

---

## Architektur-Hinweis: warum der Edit-Dispatch NICHT durch `with_backend` läuft

`router::with_backend` ruft `select_backend`, das bei fehlender IDE **Backing A** (rust-analyzer o. ä.) startet bzw. für nicht-LSP-Sprachen (z. B. `.java`) hart `Err`t (`router.rs:120-124`, `select_backend` `router.rs:58-94`). Der Headless-Edit muss aber **ohne** jeden Sprachserver laufen. Lösung (Task 11): Der Edit-Dispatch `apply_symbol_edit` spiegelt nur die **Backing-B-Erkennung** von `select_backend` (Port-Datei + `pid_alive` + `health_ok`) und fällt sonst direkt auf die freie Funktion `local_range_write` zurück — `with_backend`/`select_backend` werden für Edits **nicht** verwendet. Die drei Trait-Default-Methoden (Task 4) rufen ebenfalls `local_range_write` und erfüllen damit das Spec-§5.1-Versprechen „Backing A erbt den Default", werden aber in Produktion nur über den Trait erreicht, wenn ein Backend bereits instanziiert ist (Unit-Tests decken diesen Pfad ab).

---

## Task 0: Spike — `name_path`-Auflösbarkeit + IntelliJ-Write-API verifizieren (KEIN Commit)

**Warum:** Der gesamte v2a-Hebel beruht auf Spec §3 (`name_path` ist Rust-seitig über den vorhandenen Index auflösbar). Das ist gegen **dieses** Repo zu bestätigen, bevor `resolve_name_path` (Task 9) darauf aufsetzt. Zusätzlich ist die IntelliJ-Write-API (`WriteCommandAction`, `Document.replaceString`, `FileDocumentManager.saveDocument`) die einzige neue Plugin-API.

**Files:** keine Änderung — reine Recherche.

- [ ] **Step 1: `name_path`-Auflösung gegen das Repo prüfen**

Führe aus (lean-ctx-eigenes `ctx_symbol`):

```
ctx_symbol(name="InspectionRunner")     → erwartet 1 Treffer mit (class, file, L-range)
ctx_symbol(name="runOnFile")            → erwartet ≥2 Treffer (mehrdeutig)
```

Bestätige: (a) ein eindeutiger Klassenname liefert genau eine `(file, start_line, end_line)`; (b) ein bare Methodenname liefert mehrere Treffer (→ Disambiguierung über Enclosing-Range nötig). Falls `ctx_symbol` deferred ist: `ToolSearch(query="select:mcp__lean-ctx__ctx_symbol")`.

- [ ] **Step 2: IntelliJ-Write-API-Signaturen bestätigen**

Die IDE läuft (Port-Datei aus 5x). Bestätige via `mcp__jetbrains__get_symbol_info` (bei deferred zuerst `ToolSearch(query="select:mcp__jetbrains__get_symbol_info")`):

1. `com.intellij.openapi.command.WriteCommandAction#runWriteCommandAction(Project, Runnable)` → `void`, `@JvmStatic`.
2. `com.intellij.openapi.editor.Document#replaceString(int startOffset, int endOffset, CharSequence)` → `void`.
3. `com.intellij.psi.PsiDocumentManager#getInstance(Project)#commitDocument(Document)` → `void`.
4. `com.intellij.openapi.fileEditor.FileDocumentManager#getInstance()#saveDocument(Document)` → `void`.
5. `com.intellij.openapi.editor.Document#getLineStartOffset(int)` / `#getLineCount()` → `int`.

- [ ] **Step 3: Findings festhalten**

`ctx_knowledge action=remember category=api key=jetbrains-write-api content="<bestätigte/abweichende Signaturen>"`. Bei Abweichung: korrigierte Form notieren — Task 13 setzt darauf auf.

Erwartetes Ergebnis: `name_path`-Disambiguierung über Enclosing-Range bestätigt + 5 Write-Signaturen bestätigt ODER Korrekturliste.

---

## Task 1: Rust — Edit-Typen `TextRange0Based` / `RangeEdit` / `EditResult`

**Files:**
- Modify: `rust/src/lsp/backend.rs`

- [ ] **Step 1: Typen nach dem `Truncation`-Struct einfügen**

Per Serena `insert_after_symbol` (Anker-Symbol `Truncation` in `rust/src/lsp/backend.rs`) einfügen:

```rust
/// A 0-based, half-open text range (LSP/wire convention: start inclusive, end exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange0Based {
    pub start_line: u32,
    pub start_char: u32,
    pub end_line: u32,
    pub end_char: u32,
}

/// A resolved, ready-to-apply edit. The `name_path` → range resolution has already
/// happened in `ctx_refactor`; the backend only ever sees an absolute path + range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeEdit {
    /// Absolute, jail-checked path of the file to edit.
    pub abs_path: String,
    /// Project-relative path (for the wire body sent to Backing B).
    pub rel_path: String,
    /// The canonical edit boundary (same in IDE and headless paths).
    pub range: TextRange0Based,
    /// Final text to write into `range` (indentation already baked in by Rust).
    pub text: String,
    /// Optional md5-hex of the current content of `range`; mismatch → CONFLICT.
    pub expected_hash: Option<String>,
}

/// Outcome of applying a `RangeEdit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditResult {
    pub applied: bool,
    /// Range covering the newly written text after the edit.
    pub new_range: TextRange0Based,
    /// The text that now occupies `new_range`.
    pub edited_text: String,
    /// Compact human-readable diff (removed/added lines).
    pub diff: String,
}
```

- [ ] **Step 2: Exporte ergänzen**

Per Serena `replace_content` die `//! ... exports`-Kopfzeile NICHT anfassen (auto). Stattdessen prüfen, dass die Typen `pub` sind (sie sind es). Keine weitere Änderung.

- [ ] **Step 3: Kompilieren**

Run: `ctx_shell("cargo build -p lean-ctx", cwd="rust")`
Expected: PASS (nur „unused"-Warnungen für die noch ungenutzten Typen sind ok).

---

## Task 2: Rust — `offset_of` (0-based line/char → Byte-Offset)

**Files:**
- Create: `rust/src/lsp/edit_apply.rs`
- Modify: `rust/src/lsp/mod.rs`

- [ ] **Step 1: Modul registrieren**

Per Serena `insert_after_symbol` in `rust/src/lsp/mod.rs` nach der letzten `pub mod …;`-Zeile:

```rust
pub mod edit_apply;
```

(Falls Serena keinen passenden Anker findet: `replace_content` mit der bestehenden Modulliste + neuer Zeile.)

- [ ] **Step 2: Failing test schreiben**

`rust/src/lsp/edit_apply.rs` anlegen (native `Write` — Datei existiert noch nicht, daher Serena nicht anwendbar):

```rust
//! Shared headless apply path for symbol-body edits (spec v2a §5.1).
//!
//! `local_range_write` is the Trait-default for `replace_symbol_body` /
//! `insert_before_symbol` / `insert_after_symbol`: it writes a resolved range
//! to disk atomically, so edits work without any running language server / IDE.
//! `JetBrainsHttpBackend` overrides the Trait methods with the in-IDE HTTP path;
//! both paths apply the *same* tree-sitter range → byte-identical result.

use crate::lsp::backend::{EditResult, RangeEdit, TextRange0Based};

/// Convert a 0-based (line, character) coordinate to a byte offset into `content`.
/// `line`/`character` count UTF-8 *bytes* per line (wire convention here is byte
/// columns, matching how Rust slices `&str`). Out-of-range → `Err`.
pub fn offset_of(content: &str, line: u32, character: u32) -> Result<usize, String> {
    let mut offset = 0usize;
    let mut cur_line = 0u32;
    for l in content.split_inclusive('\n') {
        if cur_line == line {
            let line_len = l.trim_end_matches('\n').len();
            if character as usize > line_len {
                return Err(format!(
                    "POSITION_OUT_OF_RANGE: character {character} past end of line {line}"
                ));
            }
            return Ok(offset + character as usize);
        }
        offset += l.len();
        cur_line += 1;
    }
    // Allow the position one past the last line (line == cur_line, character 0):
    if line == cur_line && character == 0 {
        return Ok(offset);
    }
    Err(format!("POSITION_OUT_OF_RANGE: line {line} past end of file"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_of_maps_lines_and_columns() {
        let s = "ab\ncde\nf";
        assert_eq!(offset_of(s, 0, 0).unwrap(), 0);
        assert_eq!(offset_of(s, 0, 2).unwrap(), 2); // end of "ab"
        assert_eq!(offset_of(s, 1, 0).unwrap(), 3); // start of "cde"
        assert_eq!(offset_of(s, 1, 3).unwrap(), 6); // end of "cde"
        assert_eq!(offset_of(s, 2, 1).unwrap(), 8); // end of "f"
    }

    #[test]
    fn offset_of_one_past_last_line_is_eof() {
        let s = "ab\ncde\n";
        assert_eq!(offset_of(s, 2, 0).unwrap(), s.len());
    }

    #[test]
    fn offset_of_rejects_overrun() {
        let s = "ab\ncde";
        assert!(offset_of(s, 0, 5).is_err());
        assert!(offset_of(s, 9, 0).is_err());
    }
}
```

- [ ] **Step 3: Run test to verify it fails-then-passes**

Run: `ctx_shell("cargo nextest run -p lean-ctx offset_of", cwd="rust")`
Expected: PASS (3 Tests). `local_range_write` folgt in Task 3.

---

## Task 3: Rust — `local_range_write` (headless Range-Write + Diff + Hash-Guard)

**Files:**
- Modify: `rust/src/lsp/edit_apply.rs`

- [ ] **Step 1: Failing test schreiben**

Per native `Edit` (Datei wurde in Task 2 mit `Write` angelegt, ist also gelesen) den `mod tests`-Block in `rust/src/lsp/edit_apply.rs` um diese Tests **vor** der schließenden `}` erweitern:

```rust
    fn tmp_file(content: &str) -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Foo.txt");
        std::fs::write(&path, content).unwrap();
        (dir, path.to_string_lossy().to_string())
    }

    fn edit(abs: &str, r: TextRange0Based, text: &str, hash: Option<String>) -> RangeEdit {
        RangeEdit {
            abs_path: abs.to_string(),
            rel_path: "Foo.txt".to_string(),
            range: r,
            text: text.to_string(),
            expected_hash: hash,
        }
    }

    #[test]
    fn local_range_write_replaces_range() {
        let (_d, p) = tmp_file("aaa\nBODY\nccc\n");
        let r = TextRange0Based { start_line: 1, start_char: 0, end_line: 1, end_char: 4 };
        let res = local_range_write(&edit(&p, r, "NEW", None)).unwrap();
        assert!(res.applied);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "aaa\nNEW\nccc\n");
        assert_eq!(res.edited_text, "NEW");
    }

    #[test]
    fn local_range_write_zero_width_insert() {
        let (_d, p) = tmp_file("aaa\nccc\n");
        let r = TextRange0Based { start_line: 1, start_char: 0, end_line: 1, end_char: 0 };
        local_range_write(&edit(&p, r, "bbb\n", None)).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "aaa\nbbb\nccc\n");
    }

    #[test]
    fn local_range_write_hash_match_and_mismatch() {
        let (_d, p) = tmp_file("aaa\nBODY\nccc\n");
        let r = TextRange0Based { start_line: 1, start_char: 0, end_line: 1, end_char: 4 };
        let good = crate::core::hasher::hash_hex(b"BODY");
        local_range_write(&edit(&p, r, "X", Some(good))).unwrap();
        // second write with stale hash → CONFLICT, file unchanged
        let err = local_range_write(&edit(&p, r, "Y", Some("deadbeef".into()))).unwrap_err();
        assert!(err.starts_with("CONFLICT"), "got: {err}");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "aaa\nX\nccc\n");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `ctx_shell("cargo nextest run -p lean-ctx local_range_write", cwd="rust")`
Expected: FAIL — `local_range_write` not found.

- [ ] **Step 3: `local_range_write` implementieren**

Per native `Edit` in `rust/src/lsp/edit_apply.rs` **vor** den `#[cfg(test)] mod tests` einfügen:

```rust
/// Apply a resolved `RangeEdit` to disk (headless). Reads the file, optionally
/// verifies `expected_hash` against the *current* bytes covered by `range`
/// (mismatch → `CONFLICT`), replaces the range with `text`, writes atomically,
/// and returns the post-edit range + a compact diff.
pub fn local_range_write(edit: &RangeEdit) -> Result<EditResult, String> {
    let content = std::fs::read_to_string(&edit.abs_path)
        .map_err(|e| format!("FILE_NOT_FOUND: {}: {e}", edit.abs_path))?;

    let start = offset_of(&content, edit.range.start_line, edit.range.start_char)?;
    let end = offset_of(&content, edit.range.end_line, edit.range.end_char)?;
    if end < start {
        return Err("POSITION_OUT_OF_RANGE: end before start".to_string());
    }
    let old = &content[start..end];

    if let Some(expected) = edit.expected_hash.as_deref() {
        let actual = crate::core::hasher::hash_hex(old.as_bytes());
        if expected != actual {
            return Err(format!(
                "CONFLICT: range hash mismatch (expected={expected}, actual={actual})"
            ));
        }
    }

    let mut new_content = String::with_capacity(content.len() - old.len() + edit.text.len());
    new_content.push_str(&content[..start]);
    new_content.push_str(&edit.text);
    new_content.push_str(&content[end..]);

    write_file_atomic(&edit.abs_path, &new_content)?;

    let new_range = range_after_write(&content[..start], &edit.text);
    Ok(EditResult {
        applied: true,
        new_range,
        edited_text: edit.text.clone(),
        diff: build_range_diff(&edit.rel_path, old, &edit.text),
    })
}

/// Compute the 0-based range the freshly written `text` now occupies, given the
/// `prefix` (everything before the insertion point).
fn range_after_write(prefix: &str, text: &str) -> TextRange0Based {
    let (sl, sc) = line_col_at_end(prefix);
    let (dl, dc) = line_col_at_end(text);
    let end_line = sl + dl;
    let end_char = if dl == 0 { sc + dc } else { dc };
    TextRange0Based { start_line: sl, start_char: sc, end_line, end_char }
}

/// (line, character) of the position *after* the last byte of `s` (0-based).
fn line_col_at_end(s: &str) -> (u32, u32) {
    let line = s.matches('\n').count() as u32;
    let col = match s.rfind('\n') {
        Some(i) => (s.len() - i - 1) as u32,
        None => s.len() as u32,
    };
    (line, col)
}

fn build_range_diff(path: &str, old: &str, new: &str) -> String {
    let mut out = format!("--- {path}\n");
    for l in old.lines() {
        out.push_str(&format!("- {l}\n"));
    }
    for l in new.lines() {
        out.push_str(&format!("+ {l}\n"));
    }
    out
}

fn write_file_atomic(path: &str, content: &str) -> Result<(), String> {
    let p = std::path::Path::new(path);
    let parent = p
        .parent()
        .ok_or_else(|| "invalid path (no parent directory)".to_string())?;
    let filename = p
        .file_name()
        .ok_or_else(|| "invalid path (no filename)".to_string())?
        .to_string_lossy();
    let pid = std::process::id();
    let tmp = parent.join(format!(".{filename}.lean-ctx.v2a.tmp.{pid}"));
    std::fs::write(&tmp, content.as_bytes())
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, p).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("atomic write failed: {e}")
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `ctx_shell("cargo nextest run -p lean-ctx edit_apply", cwd="rust")`
Expected: PASS (alle `edit_apply`-Tests inkl. Task-2-Offset-Tests).

---

## Task 4: Rust — drei Edit-Trait-Methoden (Default = `local_range_write`) + `symbols_overview`-Default

**Files:**
- Modify: `rust/src/lsp/backend.rs`

- [ ] **Step 1: Drei Edit-Methoden im default-degrading Block ergänzen**

Per Serena `insert_after_symbol` (Anker: die `list_inspections`-Methode im Trait `LspBackend`) einfügen:

```rust
    /// Replace a symbol's full declaration range with `edit.text`.
    /// DEFAULT = headless local range write; `JetBrainsHttpBackend` overrides.
    fn replace_symbol_body(&mut self, edit: &RangeEdit) -> Result<EditResult, String> {
        crate::lsp::edit_apply::local_range_write(edit)
    }
    /// Insert a new sibling before the anchor symbol (range is zero-width at the
    /// anchor start line; indentation already baked into `edit.text`).
    fn insert_before_symbol(&mut self, edit: &RangeEdit) -> Result<EditResult, String> {
        crate::lsp::edit_apply::local_range_write(edit)
    }
    /// Insert a new sibling after the anchor symbol (range is zero-width at the
    /// line following the anchor; indentation already baked into `edit.text`).
    fn insert_after_symbol(&mut self, edit: &RangeEdit) -> Result<EditResult, String> {
        crate::lsp::edit_apply::local_range_write(edit)
    }
```

- [ ] **Step 2: Import für `RangeEdit`/`EditResult` im Trait-Block sicherstellen**

Die Typen liegen im selben Modul (`backend.rs`) — kein `use` nötig. Prüfen, dass die Methoden-Signaturen `RangeEdit`/`EditResult` unqualifiziert referenzieren (tun sie).

- [ ] **Step 3: `symbols_overview`-Default-Impl auf tree-sitter umstellen**

Per Serena `replace_symbol_body` (Symbol: `symbols_overview` im Trait) den Body ersetzen:

```rust
    fn symbols_overview(&mut self, uri: &Uri) -> Result<Vec<SymbolOverviewItem>, String> {
        // v2a §5.2: lossless headless default via the tree-sitter symbol index
        // (same source as ctx_symbol/ctx_outline). Backing B overrides with PSI.
        let abs = crate::lsp::client::uri_to_file_path(uri)
            .ok_or_else(|| "symbols_overview: bad uri".to_string())?;
        Ok(crate::lsp::edit_apply::overview_from_index(&abs))
    }
```

> `uri_to_file_path` Rückgabetyp prüfen: in `client.rs` ist es `Option<String>` (siehe `ctx_refactor.rs:4`-Import). Falls es `Result` ist, `.map_err`/`?` statt `ok_or_else` verwenden — an die tatsächliche Signatur anpassen (`ctx_read(rust/src/lsp/client.rs, mode=signatures)`).

- [ ] **Step 4: `overview_from_index` implementieren**

Per native `Edit` in `rust/src/lsp/edit_apply.rs` (nach `local_range_write`):

```rust
/// Build a file's structure overview from the tree-sitter symbol index
/// (headless `symbols_overview` default, spec v2a §5.2). Best-effort: returns
/// an empty vec when no graph is available.
pub fn overview_from_index(abs_path: &str) -> Vec<crate::lsp::backend::SymbolOverviewItem> {
    use crate::core::graph_provider;
    let Some(project_root) = crate::core::pathutil::nearest_project_root(abs_path) else {
        return Vec::new();
    };
    let Some(open) = graph_provider::open_or_build(&project_root) else {
        return Vec::new();
    };
    let rel = abs_path
        .strip_prefix(&project_root)
        .map(|s| s.trim_start_matches('/'))
        .unwrap_or(abs_path);
    let mut items: Vec<_> = open
        .provider
        .find_symbols("", Some(rel), None)
        .into_iter()
        .map(|s| crate::lsp::backend::SymbolOverviewItem {
            name: s.name,
            kind: s.kind,
            line: s.start_line as u32,
        })
        .collect();
    items.sort_by_key(|i| i.line);
    items
}
```

> **Verifizieren vor dem Schreiben:** (a) `find_symbols("", Some(rel), None)` mit leerem `name` — der `GraphIndex`-Zweig filtert `name.to_lowercase().contains("")` = immer true → alle Symbole der Datei (`graph_provider.rs:148-164`). Den `PropertyGraph`-Zweig prüfen (`graph_provider.rs:135` `find_symbols` mit leerem Namen) — falls er bei leerem Namen nichts liefert, stattdessen alle Symbole über eine vorhandene „alle Symbole einer Datei"-API holen (`ctx_search`/`ctx_read(rust/src/core/graph_provider.rs, mode=signatures)` nach passender Methode wie `symbols_in_file`). (b) `crate::core::pathutil::nearest_project_root` existiert — sonst die im Repo übliche Projektwurzel-Ermittlung verwenden (`ctx_search("fn nearest_project_root|fn project_root", "rust/src/core")`).

- [ ] **Step 5: Unit-Test für headless overview**

Per native `Edit` in `rust/src/lsp/edit_apply.rs` `mod tests` ergänzen:

```rust
    #[test]
    fn overview_from_index_is_empty_without_graph() {
        // A path outside any project root must degrade to empty, not panic.
        let items = overview_from_index("/nonexistent/Nope.rs");
        assert!(items.is_empty());
    }
```

- [ ] **Step 6: Run tests + build**

Run: `ctx_shell("cargo nextest run -p lean-ctx edit_apply", cwd="rust")`
Expected: PASS. Falls `find_symbols`/`nearest_project_root` angepasst werden mussten, Build prüfen: `ctx_shell("cargo build -p lean-ctx", cwd="rust")` → PASS.

---

## Task 5: Rust — JetBrains-Override der drei Edit-Methoden + `parse_edit_result`

**Files:**
- Modify: `rust/src/lsp/jetbrains_backend.rs`

- [ ] **Step 1: Failing Mock-Test schreiben**

Per native `Edit` (Datei via `ctx_read` gelesen) im `#[cfg(test)] mod tests` von `rust/src/lsp/jetbrains_backend.rs` ergänzen (Muster: `inspections_parses_wire_diags`, `mock_once`):

```rust
    #[test]
    fn replace_symbol_body_parses_wire_result() {
        let port = mock_once(
            r#"{"applied":true,
                "newRange":{"start":{"line":1,"character":0},"end":{"line":1,"character":3}},
                "editedText":"NEW"}"#,
        );
        let mut be = JetBrainsHttpBackend::new(port, "tok".into(), "/tmp/proj", 1234);
        let edit = crate::lsp::backend::RangeEdit {
            abs_path: "/tmp/proj/Foo.kt".into(),
            rel_path: "Foo.kt".into(),
            range: crate::lsp::backend::TextRange0Based {
                start_line: 1, start_char: 0, end_line: 1, end_char: 4,
            },
            text: "NEW".into(),
            expected_hash: None,
        };
        let res = be.replace_symbol_body(&edit).unwrap();
        assert!(res.applied);
        assert_eq!(res.edited_text, "NEW");
        assert_eq!(res.new_range.end_char, 3);
    }

    #[test]
    fn edit_maps_error_envelope_to_err() {
        let port = mock_once(r#"{"error":{"code":"CONFLICT","message":"stale"}}"#);
        let mut be = JetBrainsHttpBackend::new(port, "tok".into(), "/tmp/proj", 1234);
        let edit = crate::lsp::backend::RangeEdit {
            abs_path: "/tmp/proj/Foo.kt".into(), rel_path: "Foo.kt".into(),
            range: crate::lsp::backend::TextRange0Based {
                start_line: 0, start_char: 0, end_line: 0, end_char: 0,
            },
            text: "x".into(), expected_hash: None,
        };
        assert_eq!(be.replace_symbol_body(&edit).unwrap_err(), "CONFLICT");
    }
```

> `JetBrainsHttpBackend::new`-Signatur gegen `jetbrains_backend.rs:50` abgleichen (Arg-Reihenfolge `port, token, project_root, pid`); bestehende Tests (`references_parses_wire_locations`) als Vorlage für `mock_once`-Nutzung lesen.

- [ ] **Step 2: Run test to verify it fails**

Run: `ctx_shell("cargo nextest run -p lean-ctx replace_symbol_body", cwd="rust")`
Expected: FAIL — Methode existiert nur als Trait-Default (lokaler Write greift auf `/tmp/proj/Foo.kt` zu → Datei fehlt → FILE_NOT_FOUND, nicht der Mock). Das bestätigt, dass der Override noch fehlt.

- [ ] **Step 3: `parse_edit_result` als assoziierte Fn einfügen**

Per Serena `insert_after_symbol` (Anker: `parse_truncation` in `impl JetBrainsHttpBackend`):

```rust
    fn parse_edit_result(v: &Value, fallback_text: &str) -> EditResult {
        let pos = |obj: &Value, key: &str| -> (u32, u32) {
            let p = obj.get(key);
            let line = p.and_then(|p| p.get("line")).and_then(Value::as_u64).unwrap_or(0) as u32;
            let ch = p.and_then(|p| p.get("character")).and_then(Value::as_u64).unwrap_or(0) as u32;
            (line, ch)
        };
        let nr = v.get("newRange");
        let (sl, sc) = nr.map(|r| pos(r, "start")).unwrap_or((0, 0));
        let (el, ec) = nr.map(|r| pos(r, "end")).unwrap_or((0, 0));
        EditResult {
            applied: v.get("applied").and_then(Value::as_bool).unwrap_or(false),
            new_range: TextRange0Based {
                start_line: sl, start_char: sc, end_line: el, end_char: ec,
            },
            edited_text: v
                .get("editedText")
                .and_then(Value::as_str)
                .unwrap_or(fallback_text)
                .to_string(),
            diff: String::new(), // Rust builds the diff in ctx_refactor from old/new
        }
    }
```

> Import-Block am Kopf von `jetbrains_backend.rs` um `EditResult, RangeEdit, TextRange0Based` erweitern (zur bestehenden `use crate::lsp::backend::{…}`-Zeile per Serena `replace_content` ergänzen).

- [ ] **Step 4: Drei Override-Methoden einfügen**

Per Serena `insert_after_symbol` (Anker: `list_inspections` in `impl LspBackend for JetBrainsHttpBackend`):

```rust
    fn replace_symbol_body(&mut self, edit: &RangeEdit) -> Result<EditResult, String> {
        self.post_edit("/replaceSymbolBody", edit)
    }
    fn insert_before_symbol(&mut self, edit: &RangeEdit) -> Result<EditResult, String> {
        self.post_edit("/insertBeforeSymbol", edit)
    }
    fn insert_after_symbol(&mut self, edit: &RangeEdit) -> Result<EditResult, String> {
        self.post_edit("/insertAfterSymbol", edit)
    }
```

- [ ] **Step 5: `post_edit`-Helfer einfügen**

Per Serena `insert_after_symbol` (Anker: `position_body` in `impl JetBrainsHttpBackend`):

```rust
    /// POST a resolved edit to the plugin and parse the result. The wire range is
    /// the canonical tree-sitter range (byte-identical to the headless path).
    fn post_edit(&self, endpoint: &str, edit: &RangeEdit) -> Result<EditResult, String> {
        let mut body = serde_json::json!({
            "path": edit.rel_path,
            "range": {
                "start": { "line": edit.range.start_line, "character": edit.range.start_char },
                "end":   { "line": edit.range.end_line,   "character": edit.range.end_char },
            },
            "text": edit.text,
        });
        if let Some(h) = &edit.expected_hash {
            body["expected_hash"] = serde_json::json!(h);
        }
        let resp = self.post(endpoint, &body)?;
        if let Some(err) = resp.get("error") {
            return Err(err
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("INTERNAL")
                .to_string());
        }
        Ok(Self::parse_edit_result(&resp, &edit.text))
    }
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `ctx_shell("cargo nextest run -p lean-ctx jetbrains_backend", cwd="rust")`
Expected: PASS (inkl. der zwei neuen Edit-Tests).

---

## Task 6: Rust — `name_path`-Auflösung (`resolve_name_path`)

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`

- [ ] **Step 1: Failing test schreiben**

Per native `Edit` im `#[cfg(test)] mod tests` von `rust/src/tools/ctx_refactor.rs` ergänzen. Der Test nutzt `ctx_symbol`s vorhandene Test-GraphProvider-Helfer als Vorlage — hier aber gegen das echte Repo (Integrationsstil):

```rust
    #[test]
    fn resolve_name_path_unique_class() {
        // Resolve a class known to be unique in this repo's index.
        let root = env!("CARGO_MANIFEST_DIR"); // rust/
        let project_root = std::path::Path::new(root).parent().unwrap().to_str().unwrap();
        match super::resolve_name_path("JetBrainsHttpBackend", project_root) {
            Ok(r) => {
                assert!(r.rel_path.ends_with("jetbrains_backend.rs"));
                assert!(r.end_line >= r.start_line && r.start_line > 0);
            }
            Err(e) => panic!("expected unique resolution, got: {e}"),
        }
    }

    #[test]
    fn resolve_name_path_unknown_is_no_symbol() {
        let root = env!("CARGO_MANIFEST_DIR");
        let project_root = std::path::Path::new(root).parent().unwrap().to_str().unwrap();
        let err = super::resolve_name_path("ZzzNoSuchSymbol123", project_root).unwrap_err();
        assert!(err.starts_with("NO_SYMBOL"), "got: {err}");
    }
```

> Falls `find_symbols` substring-matcht (`GraphIndex`-Zweig nutzt `.contains`), kann „JetBrainsHttpBackend" auch Teiltreffer liefern. In `resolve_name_path` daher **exakte** Namensgleichheit auf den Leaf erzwingen (Step 3). Den Test ggf. auf ein Symbol mit eindeutigem exakten Namen setzen, das im Index als exakter Treffer existiert (per Step-0-Spike bestätigt).

- [ ] **Step 2: Run test to verify it fails**

Run: `ctx_shell("cargo nextest run -p lean-ctx resolve_name_path", cwd="rust")`
Expected: FAIL — `resolve_name_path` not found.

- [ ] **Step 3: `resolve_name_path` + `Resolved`-Typ implementieren**

Per Serena `insert_before_symbol` (Anker: `fn parse_direction` in `ctx_refactor.rs`) einfügen:

```rust
/// A resolved symbol location (project-relative path + 1-based inclusive line span).
pub(crate) struct Resolved {
    pub rel_path: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Resolve a `name_path` (`Class/method` or bare `name`) to a single symbol via
/// the tree-sitter index (spec v2a §3/§5.3). Disambiguates a qualified path by
/// enclosing-range containment (ancestor symbol's line span contains the leaf's).
pub(crate) fn resolve_name_path(name_path: &str, project_root: &str) -> Result<Resolved, String> {
    use crate::core::graph_provider;
    let open = graph_provider::open_or_build(project_root)
        .ok_or_else(|| "NO_SYMBOL: no symbol index available".to_string())?;
    let gp = &open.provider;

    let segments: Vec<&str> = name_path.split('/').filter(|s| !s.is_empty()).collect();
    let leaf = *segments
        .last()
        .ok_or_else(|| "NO_SYMBOL: empty name_path".to_string())?;

    // Exact-name leaf candidates (case-sensitive — the index may substring-match).
    let mut leaves: Vec<_> = gp
        .find_symbols(leaf, None, None)
        .into_iter()
        .filter(|s| s.name == leaf)
        .collect();

    // Qualify by the immediate ancestor segment, if present.
    if segments.len() >= 2 {
        let ancestor = segments[segments.len() - 2];
        let parents: Vec<_> = gp
            .find_symbols(ancestor, None, None)
            .into_iter()
            .filter(|s| s.name == ancestor)
            .collect();
        leaves.retain(|leaf_sym| {
            parents.iter().any(|p| {
                p.file == leaf_sym.file
                    && p.start_line <= leaf_sym.start_line
                    && leaf_sym.end_line <= p.end_line
            })
        });
    }

    match leaves.len() {
        0 => Err(format!("NO_SYMBOL: '{name_path}' did not resolve to any indexed symbol")),
        1 => Ok(Resolved {
            rel_path: leaves[0].file.clone(),
            start_line: leaves[0].start_line,
            end_line: leaves[0].end_line,
        }),
        _ => {
            let mut msg = format!(
                "AMBIGUOUS_SYMBOL: '{name_path}' matches {} symbols; qualify it:\n",
                leaves.len()
            );
            for s in leaves.iter().take(10) {
                msg.push_str(&format!("  {}:{} (L{}-{})\n", s.file, s.name, s.start_line, s.end_line));
            }
            Err(msg)
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `ctx_shell("cargo nextest run -p lean-ctx resolve_name_path", cwd="rust")`
Expected: PASS. Falls der Unique-Test wegen Substring-Matching mehrdeutig wird, im Test ein im Index nachweislich eindeutiges Symbol wählen (Step-0-Befund).

---

## Task 7: Rust — Einrück-Berechnung (`anchor_indent` + `reindent_first_line`)

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`

- [ ] **Step 1: Failing test schreiben**

Per native `Edit` im `mod tests` von `ctx_refactor.rs`:

```rust
    #[test]
    fn anchor_indent_reads_leading_whitespace() {
        let content = "class A {\n    fun b() {}\n}\n";
        assert_eq!(super::anchor_indent(content, 2), "    "); // line 2 (1-based) → 4 spaces
        assert_eq!(super::anchor_indent(content, 1), "");     // line 1 → none
    }

    #[test]
    fn reindent_prefixes_first_line_only() {
        assert_eq!(super::reindent_first_line("fun x() {}", "    "), "    fun x() {}");
        // Already-indented text is left untouched.
        assert_eq!(super::reindent_first_line("    fun x()", "    "), "    fun x()");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `ctx_shell("cargo nextest run -p lean-ctx anchor_indent", cwd="rust")`
Expected: FAIL — Funktionen fehlen.

- [ ] **Step 3: Implementieren**

Per Serena `insert_before_symbol` (Anker: `fn resolve_name_path`) einfügen:

```rust
/// Leading whitespace of the 1-based `line` in `content` (anchor indentation).
pub(crate) fn anchor_indent(content: &str, line: usize) -> String {
    content
        .lines()
        .nth(line.saturating_sub(1))
        .map(|l| l.chars().take_while(|c| *c == ' ' || *c == '\t').collect())
        .unwrap_or_default()
}

/// Prefix `indent` to the first line of `text` iff that line has no leading
/// whitespace of its own (deterministic; the same Rust computes it for both
/// apply paths, so the wire text is byte-identical).
pub(crate) fn reindent_first_line(text: &str, indent: &str) -> String {
    if text.starts_with(' ') || text.starts_with('\t') || indent.is_empty() {
        return text.to_string();
    }
    format!("{indent}{text}")
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `ctx_shell("cargo nextest run -p lean-ctx anchor_indent reindent", cwd="rust")`
Expected: PASS (beide Tests).

---

## Task 8: Rust — Apply-Dispatch (`apply_symbol_edit`)

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`

- [ ] **Step 1: Failing test schreiben**

Per native `Edit` im `mod tests` von `ctx_refactor.rs` (headless-Pfad: keine IDE → `local_range_write`):

```rust
    #[test]
    fn apply_symbol_edit_headless_replaces_range() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Foo.txt"), "aaa\nBODY\nccc\n").unwrap();
        let abs = dir.path().join("Foo.txt").to_string_lossy().to_string();
        let edit = crate::lsp::backend::RangeEdit {
            abs_path: abs.clone(),
            rel_path: "Foo.txt".into(),
            range: crate::lsp::backend::TextRange0Based {
                start_line: 1, start_char: 0, end_line: 1, end_char: 4,
            },
            text: "NEW".into(),
            expected_hash: None,
        };
        // No port file under this temp dir → headless apply.
        let res = super::apply_symbol_edit(
            "replace_symbol_body",
            dir.path().to_str().unwrap(),
            edit,
        )
        .unwrap();
        assert!(res.applied);
        assert_eq!(std::fs::read_to_string(&abs).unwrap(), "aaa\nNEW\nccc\n");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `ctx_shell("cargo nextest run -p lean-ctx apply_symbol_edit", cwd="rust")`
Expected: FAIL — `apply_symbol_edit` not found.

- [ ] **Step 3: Implementieren**

Per Serena `insert_before_symbol` (Anker: `fn anchor_indent`) einfügen. Spiegelt die Backing-B-Erkennung aus `router::select_backend` (Port-Datei + Liveness), fällt sonst auf `local_range_write`:

```rust
/// Apply a resolved edit. IDE-first: a live JetBrains backend (port file +
/// liveness, mirroring router::select_backend) handles it via WriteCommandAction;
/// otherwise the headless local_range_write applies the identical bytes.
pub(crate) fn apply_symbol_edit(
    action: &str,
    project_root: &str,
    edit: crate::lsp::backend::RangeEdit,
) -> Result<crate::lsp::backend::EditResult, String> {
    use crate::lsp::backend::LspBackend;
    use crate::lsp::port_discovery;

    let mut backend: Box<dyn LspBackend> =
        if let Some(pf) = port_discovery::read_port_file(project_root) {
            if port_discovery::pid_alive(pf.pid) && port_discovery::health_ok(&pf) {
                Box::new(crate::lsp::jetbrains_backend::JetBrainsHttpBackend::new(
                    pf.port,
                    pf.token,
                    project_root.to_string(),
                    pf.pid,
                ))
            } else {
                Box::new(crate::lsp::edit_apply::HeadlessBackend)
            }
        } else {
            Box::new(crate::lsp::edit_apply::HeadlessBackend)
        };

    match action {
        "replace_symbol_body" => backend.replace_symbol_body(&edit),
        "insert_before_symbol" => backend.insert_before_symbol(&edit),
        "insert_after_symbol" => backend.insert_after_symbol(&edit),
        other => Err(format!("INTERNAL: not an edit action: {other}")),
    }
}
```

- [ ] **Step 4: `HeadlessBackend` als Trait-Default-Träger anlegen**

Per native `Edit` in `rust/src/lsp/edit_apply.rs` (vor `mod tests`): ein Nullobjekt, das nur die fünf Pflicht-Trait-Methoden mit `Err` stubbt und die drei Edit-Defaults erbt — so trägt der Headless-Pfad denselben Default-Apply:

```rust
/// Zero-dependency backend that carries only the Trait default-apply for the
/// three edit methods (used by ctx_refactor when no IDE is reachable). The five
/// mandatory read methods are unsupported here (edits never call them).
pub struct HeadlessBackend;

impl crate::lsp::backend::LspBackend for HeadlessBackend {
    fn open_file(&mut self, _u: &lsp_types::Uri, _l: &str, _t: &str) -> Result<(), String> {
        Ok(())
    }
    fn references(
        &mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str,
    ) -> Result<Vec<lsp_types::Location>, String> {
        Err("references requires a backend".into())
    }
    fn definition(
        &mut self, _u: &lsp_types::Uri, _p: lsp_types::Position,
    ) -> Result<lsp_types::GotoDefinitionResponse, String> {
        Err("definition requires a backend".into())
    }
    fn implementations(
        &mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _s: &str,
    ) -> Result<Vec<lsp_types::Location>, String> {
        Err("implementations requires a backend".into())
    }
    fn rename(
        &mut self, _u: &lsp_types::Uri, _p: lsp_types::Position, _n: &str,
    ) -> Result<Option<lsp_types::WorkspaceEdit>, String> {
        Err("rename requires a backend".into())
    }
    // replace_symbol_body / insert_before_symbol / insert_after_symbol inherit
    // the Trait default → local_range_write.
}
```

> `port_discovery`-API (`read_port_file`/`pid_alive`/`health_ok`) gegen `router.rs:66-67` abgleichen (`ctx_read(rust/src/lsp/port_discovery.rs, mode=signatures)`); Feldnamen `pf.port`/`pf.token`/`pf.pid` bestätigen.

- [ ] **Step 5: Run tests to verify they pass**

Run: `ctx_shell("cargo nextest run -p lean-ctx apply_symbol_edit", cwd="rust")`
Expected: PASS (Datei wurde headless verändert).

---

## Task 9: Rust — drei Action-Handler in `ctx_refactor::handle`

**Files:**
- Modify: `rust/src/tools/ctx_refactor.rs`

- [ ] **Step 1: Handler-Funktion `handle_symbol_edit` schreiben**

Per Serena `insert_before_symbol` (Anker: `fn handle_inspections`) einfügen. Sie löst `name_path` ODER Position auf, baut die `RangeEdit` (inkl. Einrückung für insert) und ruft `apply_symbol_edit`:

```rust
fn handle_symbol_edit(action: &str, args: &Value, project_root: &str) -> String {
    // 1) Resolve target: name_path (primary) or path+line(+column) fallback.
    let (rel_path, start_line, end_line) = match args.get("name_path").and_then(Value::as_str) {
        Some(np) => match resolve_name_path(np, project_root) {
            Ok(r) => (r.rel_path, r.start_line, r.end_line),
            Err(e) => return format!("ERROR: {e}"),
        },
        None => {
            let Some(path) = args.get("path").and_then(Value::as_str) else {
                return "ERROR: provide 'name_path' or 'path'+'line' for symbol edits.".to_string();
            };
            // Position fallback: caller gives the symbol's own line span explicitly.
            let line = args.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
            let end = args.get("end_line").and_then(Value::as_u64).unwrap_or(line as u64) as usize;
            if line == 0 {
                return "ERROR: 'line' is required (1-based) when using the path fallback.".to_string();
            }
            (path.to_string(), line, end)
        }
    };

    // 2) PathJail on the resolved path (v1 §4.5 seam — critical before writes).
    let abs_path = match crate::core::path_resolve::resolve_tool_path(
        Some(project_root),
        None,
        &rel_path,
    ) {
        Ok(p) => p,
        Err(e) => return format!("ERROR: path blocked by jail: {e}"),
    };

    let content = match std::fs::read_to_string(&abs_path) {
        Ok(c) => c,
        Err(e) => return format!("ERROR: FILE_NOT_FOUND: {abs_path}: {e}"),
    };

    // 3) Build the canonical range + final wire text per action.
    let expected_hash = args.get("expected_hash").and_then(Value::as_str).map(String::from);
    let (range, text) = match action {
        "replace_symbol_body" => {
            let Some(new_body) = args.get("new_body").and_then(Value::as_str) else {
                return "ERROR: 'new_body' is required for replace_symbol_body.".to_string();
            };
            // Full declaration range: (start_line-1, 0) .. (end_line-1, len(end_line)).
            let end_col = content.lines().nth(end_line.saturating_sub(1)).map_or(0, str::len) as u32;
            (
                crate::lsp::backend::TextRange0Based {
                    start_line: (start_line - 1) as u32,
                    start_char: 0,
                    end_line: (end_line - 1) as u32,
                    end_char: end_col,
                },
                new_body.to_string(),
            )
        }
        "insert_before_symbol" | "insert_after_symbol" => {
            let Some(t) = args.get("text").and_then(Value::as_str) else {
                return format!("ERROR: 'text' is required for {action}.");
            };
            let indent = anchor_indent(&content, start_line);
            let final_text = format!("{}\n", reindent_first_line(t, &indent));
            let insert_line = if action == "insert_before_symbol" {
                (start_line - 1) as u32 // zero-width at anchor start line
            } else {
                end_line as u32 // zero-width at the line AFTER the anchor's last line
            };
            (
                crate::lsp::backend::TextRange0Based {
                    start_line: insert_line, start_char: 0,
                    end_line: insert_line, end_char: 0,
                },
                final_text,
            )
        }
        other => return format!("ERROR: INTERNAL: not an edit action: {other}"),
    };

    let edit = crate::lsp::backend::RangeEdit {
        abs_path,
        rel_path,
        range,
        text,
        expected_hash,
    };

    // 4) Dispatch (IDE-first, headless fallback) + format.
    match apply_symbol_edit(action, project_root, edit) {
        Ok(res) => format_edit_result(action, &res),
        Err(e) => format!("ERROR: {e}"),
    }
}

fn format_edit_result(action: &str, res: &crate::lsp::backend::EditResult) -> String {
    if !res.applied {
        return format!("{action}: not applied.");
    }
    let r = res.new_range;
    let body = if res.diff.is_empty() {
        res.edited_text.clone()
    } else {
        res.diff.clone()
    };
    format!(
        "{action} applied (L{}:{}-L{}:{}):\n{}",
        r.start_line + 1, r.start_char, r.end_line + 1, r.end_char, body
    )
}
```

> Für den IDE-Pfad ist `res.diff` leer (Plugin liefert keinen Diff). Optional: in `handle_symbol_edit` den Diff Rust-seitig aus `old` (vor dem Apply gelesener Range-Inhalt) + `res.edited_text` bauen und in `res` setzen, bevor formatiert wird. Für v2a genügt `edited_text` als Body (headless liefert ohnehin einen Diff). Konsistenz beibehalten: wenn gewünscht, in beiden Pfaden den Diff in `handle_symbol_edit` bauen statt in `local_range_write` — dann ist der Output backendunabhängig identisch. **Empfehlung:** Diff in `handle_symbol_edit` aus dem vorab gelesenen `old`-Slice bauen (eine Quelle), `local_range_write`s internen Diff ignorieren.

- [ ] **Step 2: Dispatch in `handle` ergänzen**

Per Serena `replace_symbol_body` (Symbol: `handle` in `ctx_refactor.rs`) den `match action`-Block um die drei Actions erweitern (vor dem `_ =>`-Arm) und den Hilfetext im `_`-Arm ergänzen:

```rust
        "replace_symbol_body" | "insert_before_symbol" | "insert_after_symbol" => {
            handle_symbol_edit(action, args, project_root)
        }
```

und den Fallback-Hilfetext anpassen auf:

```rust
        _ => format!(
            "ERROR: Unknown action '{action}'. Available: rename, references, definition, \
             implementations, declaration, type_hierarchy, symbols_overview, inspections, \
             replace_symbol_body, insert_before_symbol, insert_after_symbol."
        ),
```

> `handle` öffnet aktuell die Datei via `router::open_file` und baut `position` aus `line`/`column` (`ctx_refactor.rs:19-24`). Die drei Edit-Actions brauchen das **nicht** (sie lösen selbst auf). Den Edit-Arm daher **vor** die `open_file`-Logik ziehen ODER `open_file` für Edit-Actions überspringen: am saubersten den Edit-Dispatch am Anfang von `handle` per Frühausstieg behandeln:

Per Serena `replace_symbol_body` den Anfang von `handle` so umbauen, dass Edit-Actions vor `open_file` abgefangen werden:

```rust
    let action = args.get("action").and_then(Value::as_str).unwrap_or("references");

    if matches!(
        action,
        "replace_symbol_body" | "insert_before_symbol" | "insert_after_symbol"
    ) {
        return handle_symbol_edit(action, args, project_root);
    }

    let line = args.get("line").and_then(Value::as_u64).unwrap_or(1) as u32;
    // … (Rest unverändert: column, scope, open_file, position, match) …
```

(Den bestehenden `match action`-Block dann NICHT zusätzlich um die drei Actions erweitern — der Frühausstieg deckt sie ab. Hilfetext im `_`-Arm trotzdem wie oben ergänzen, damit er die neuen Actions listet.)

- [ ] **Step 3: Build + Test**

Run: `ctx_shell("cargo nextest run -p lean-ctx ctx_refactor", cwd="rust")`
Expected: PASS (inkl. `apply_symbol_edit`-, `resolve_name_path`-, `anchor_indent`-Tests). Build-Check: `ctx_shell("cargo build -p lean-ctx", cwd="rust")` → PASS.

- [ ] **Step 4: End-to-End-Handler-Test (headless replace via name_path)**

Per native `Edit` im `mod tests` von `ctx_refactor.rs`:

```rust
    #[test]
    fn handle_replace_symbol_body_via_position_fallback() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn old() {\n  1\n}\n").unwrap();
        let args = serde_json::json!({
            "action": "replace_symbol_body",
            "path": "a.rs",
            "line": 1,
            "end_line": 3,
            "new_body": "fn new() {\n  2\n}"
        });
        let out = super::handle(&args, dir.path().to_str().unwrap(), "");
        assert!(out.contains("replace_symbol_body applied"), "got: {out}");
        let after = std::fs::read_to_string(dir.path().join("a.rs")).unwrap();
        assert!(after.contains("fn new()"), "file: {after}");
    }
```

> `handle(args, project_root, abs_path)` — der dritte Param `abs_path` wird für Edit-Actions ignoriert (Frühausstieg vor `open_file`); leeren String übergeben ist ok.

Run: `ctx_shell("cargo nextest run -p lean-ctx handle_replace_symbol_body", cwd="rust")`
Expected: PASS.

---

## Task 10: Rust — Schema-Erweiterung (`registered/ctx_refactor.rs`)

**Files:**
- Modify: `rust/src/tools/registered/ctx_refactor.rs`

- [ ] **Step 1: Failing schema-test erweitern**

Per native `Edit` den `schema_advertises_declaration_and_scope`-Test in `registered/ctx_refactor.rs` um die neuen Needles ergänzen (in das `for needle in [...]`-Array):

```rust
            "replace_symbol_body",
            "insert_before_symbol",
            "insert_after_symbol",
            "name_path",
            "new_body",
            "expected_hash",
```

- [ ] **Step 2: Run test to verify it fails**

Run: `ctx_shell("cargo nextest run -p lean-ctx -E 'test(schema_advertises)'", cwd="rust")`
Expected: FAIL — neue Needles fehlen im Schema.

- [ ] **Step 3: Schema + Beschreibung anpassen**

Per Serena `replace_symbol_body` (Symbol: `tool_def` in `impl McpTool for CtxRefactorTool`) den `action`-Enum, die Beschreibung und die `properties` erweitern:

- `action.enum` um `"replace_symbol_body", "insert_before_symbol", "insert_after_symbol"` ergänzen.
- Description-String ergänzen um: `Symbol-body edits (replace_symbol_body, insert_before_symbol, insert_after_symbol) are name_path-addressed and work IDE-first with a lossless headless fallback.`
- Neue `properties` einfügen:

```rust
                    "name_path": {
                        "type": "string",
                        "description": "Symbol path for body edits: 'Class/method' (qualified) or bare 'name'. Resolved via the symbol index; ambiguous → AMBIGUOUS_SYMBOL with candidates."
                    },
                    "new_body": {
                        "type": "string",
                        "description": "Full replacement declaration text (replace_symbol_body)."
                    },
                    "text": {
                        "type": "string",
                        "description": "Sibling text to insert (insert_before_symbol/insert_after_symbol); indentation is applied automatically."
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "1-based last line of the symbol (only for the path+line fallback when name_path is omitted)."
                    },
                    "expected_hash": {
                        "type": "string",
                        "description": "Optional md5-hex of the current range content; mismatch → CONFLICT (no blind overwrite)."
                    }
```

> `required` bleibt `["action", "path"]` NICHT — `path` ist für name_path-Edits optional. **Anpassen** auf `"required": ["action"]`, da Edits per `name_path` ohne `path` laufen. Prüfen, dass die bestehenden Read-Actions (`references` etc.) weiterhin `path` über `require_resolved_path` erzwingen — das passiert im `handle` der `McpTool`-Impl (`require_resolved_path(ctx, args, "path")?`). **Wichtig:** Für name_path-Edits darf `require_resolved_path` nicht hart fehlschlagen, wenn `path` fehlt. Daher in der `handle`-Methode der `McpTool`-Impl `path` nur auflösen, wenn vorhanden:

Per Serena `replace_symbol_body` (Symbol: `handle` in `impl McpTool for CtxRefactorTool`) den Body anpassen:

```rust
    fn handle(&self, args: &Map<String, Value>, ctx: &ToolContext) -> Result<ToolOutput, ErrorData> {
        // name_path edits resolve their own path; only require/resolve `path`
        // when it is actually provided (read actions + position-fallback edits).
        let has_path = args.get("path").and_then(Value::as_str).is_some();
        let abs_path = if has_path {
            require_resolved_path(ctx, args, "path")?
        } else {
            String::new()
        };

        let args_value = Value::Object(args.clone());
        let result = crate::tools::ctx_refactor::handle(&args_value, &ctx.project_root, &abs_path);

        let action = get_str(args, "action").unwrap_or_default();
        Ok(ToolOutput {
            text: result,
            original_tokens: 0,
            saved_tokens: 0,
            mode: Some(action),
            path: get_str(args, "path"),
            changed: matches!(
                action.as_str(),
                "replace_symbol_body" | "insert_before_symbol" | "insert_after_symbol"
            ),
        })
    }
```

> `require_resolved_path`-Rückgabetyp (`String`) gegen `tool_trait.rs` abgleichen; `get_str` Rückgabe (`Option<String>`) prüfen — `action.as_str()` ggf. anpassen, falls `get_str` `String` liefert.

- [ ] **Step 4: Run schema-test to verify it passes**

Run: `ctx_shell("cargo nextest run -p lean-ctx -E 'test(schema_advertises)'", cwd="rust")`
Expected: PASS.

---

## Task 11: Rust — Gate (ganze Rust-Seite grün)

**Files:** keine Änderung — Verifikation.

- [ ] **Step 1: Volle Test-Suite**

Run: `ctx_shell("cargo nextest run -p lean-ctx", cwd="rust")`
Expected: PASS. Bekannte, unabhängige Vorab-Fails (`hn_hardening`) sind zulässig, solange keine **neuen** Fails durch v2a entstehen — Ergebnis mit dem Baseline-Stand vergleichen.

- [ ] **Step 2: Clippy (keine neuen Lints)**

Run: `ctx_shell("cargo clippy -p lean-ctx --all-targets", cwd="rust")`
Expected: 0 neue Warnungen/Fehler.

- [ ] **Step 3: Format**

Run: `ctx_shell("cargo fmt -p lean-ctx", cwd="rust")`
Expected: keine Diffs außer in den geänderten Dateien.

---

## Task 12: Kotlin — Wire-DTOs `EditRequest` / `EditResponse`

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt`
- Modify: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt`

- [ ] **Step 1: Failing round-trip test schreiben**

Per native `Edit` in `JsonCodecTest.kt` einen Test ergänzen (Muster: bestehende DTO-Round-Trip-Tests):

```kotlin
    @Test
    fun parseEditRequest_roundTrips() {
        val json = """
            {"path":"Foo.kt",
             "range":{"start":{"line":1,"character":0},"end":{"line":1,"character":4}},
             "text":"NEW","expected_hash":"abc"}
        """.trimIndent()
        val req = JsonCodec.parseEditRequest(json)
        assertEquals("Foo.kt", req.path)
        assertEquals(1, req.range.start.line)
        assertEquals(4, req.range.end.character)
        assertEquals("NEW", req.text)
        assertEquals("abc", req.expectedHash)
    }

    @Test
    fun editResponse_serializes() {
        val resp = EditResponse(
            applied = true,
            newRange = TextRangeDTO(PositionDTO(1, 0), PositionDTO(1, 3)),
            editedText = "NEW",
        )
        val json = JsonCodec.toJson(resp)
        assertTrue(json.contains("\"applied\":true"))
        assertTrue(json.contains("\"editedText\":\"NEW\""))
    }
```

- [ ] **Step 2: DTOs + Parser einfügen**

Per native `Edit` in `Wire.kt` nach `ListInspectionsResponse` die DTOs einfügen:

```kotlin
/** Request body for /replaceSymbolBody|/insertBeforeSymbol|/insertAfterSymbol. */
data class EditRequest(
    val path: String,
    val range: TextRangeDTO,
    val text: String,
    /** Maps the JSON key `expected_hash` (snake_case from Rust). */
    @com.google.gson.annotations.SerializedName("expected_hash")
    val expectedHash: String? = null,
)

/** Response body for the three edit endpoints. */
data class EditResponse(
    val applied: Boolean,
    val newRange: TextRangeDTO,
    val editedText: String,
)
```

und in `object JsonCodec` den Parser ergänzen (nach `parseFileRequest`):

```kotlin
    fun parseEditRequest(body: String): EditRequest =
        gson.fromJson(body, EditRequest::class.java)
            ?: throw IllegalArgumentException("empty request body")
```

- [ ] **Step 3: Run test to verify it passes**

Run: `ctx_shell("./gradlew :test --tests '*JsonCodecTest*'", cwd="packages/jetbrains-lean-ctx")`
Expected: PASS (neue + bestehende Codec-Tests). Falls Gradle-Task-Name abweicht: `ctx_shell("./gradlew tasks --group verification", cwd="packages/jetbrains-lean-ctx")` zur Klärung.

---

## Task 13: Kotlin — `SymbolEditor.kt` (WriteCommandAction-Range-Write)

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolEditor.kt`

- [ ] **Step 1: `SymbolEditor.kt` schreiben (native `Write`)**

```kotlin
package com.leanctx.plugin.psi

import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.editor.Document
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiDocumentManager
import com.leanctx.plugin.dto.EditRequest
import com.leanctx.plugin.dto.EditResponse
import com.leanctx.plugin.dto.PositionDTO
import com.leanctx.plugin.dto.TextRangeDTO
import com.leanctx.plugin.server.BackendException

/**
 * Applies a resolved range edit through the IDE so the change carries VFS
 * coherence + a single Undo entry. The edit boundary is the *wire range*
 * (the canonical tree-sitter range computed in Rust) — the plugin does NOT
 * re-resolve the symbol, so this path is byte-identical to the headless path.
 */
class SymbolEditor(private val project: Project) {
    private val locator = PsiLocator(project)

    fun apply(req: EditRequest): EditResponse {
        val file = locator.psiFile(req.path)
        val doc: Document = PsiDocumentManager.getInstance(project).getDocument(file)
            ?: throw BackendException("INTERNAL", "no document for ${req.path}")

        val startOffset = locator.offsetOf(file, req.range.start.line, req.range.start.character)
        val endOffset = locator.offsetOf(file, req.range.end.line, req.range.end.character)
        if (endOffset < startOffset) {
            throw BackendException("POSITION_OUT_OF_RANGE", "end before start")
        }

        // Optional CONFLICT guard: hash the current range content (md5 hex) and
        // compare to expected_hash before writing.
        req.expectedHash?.let { expected ->
            val current = doc.getText(com.intellij.openapi.util.TextRange(startOffset, endOffset))
            val actual = md5Hex(current)
            if (expected != actual) {
                throw BackendException("CONFLICT", "range hash mismatch")
            }
        }

        WriteCommandAction.runWriteCommandAction(project) {
            doc.replaceString(startOffset, endOffset, req.text)
            PsiDocumentManager.getInstance(project).commitDocument(doc)
            FileDocumentManager.getInstance().saveDocument(doc) // persist to disk for lean-ctx
        }

        // Post-edit range: start stays, end = start + text length.
        val newEndOffset = startOffset + req.text.length
        val newStart = positionOf(doc, startOffset)
        val newEnd = positionOf(doc, newEndOffset)
        return EditResponse(
            applied = true,
            newRange = TextRangeDTO(newStart, newEnd),
            editedText = req.text,
        )
    }

    private fun positionOf(doc: Document, offset: Int): PositionDTO {
        val line = doc.getLineNumber(offset)
        return PositionDTO(line, offset - doc.getLineStartOffset(line))
    }

    private fun md5Hex(s: String): String {
        val md = java.security.MessageDigest.getInstance("MD5")
        val bytes = md.digest(s.toByteArray(Charsets.UTF_8))
        return bytes.joinToString("") { "%02x".format(it) }
    }
}
```

> **Hash-Konsistenz prüfen:** Rust nutzt `crate::core::hasher::hash_hex` für `expected_hash`. Bestätigen, dass `hash_hex` md5-hex (lowercase) ist (`ctx_read(rust/src/core/hasher.rs, mode=signatures)`). Falls es **nicht** md5 ist (z. B. blake3/sha256), den Kotlin-`md5Hex` an denselben Algorithmus anpassen, sonst schlägt der CONFLICT-Guard im IDE-Pfad fälschlich an. **Headless** nutzt denselben `hash_hex` → dort ist Konsistenz automatisch gegeben.

- [ ] **Step 2: Kompilieren**

Run: `ctx_shell("./gradlew :compileKotlin", cwd="packages/jetbrains-lean-ctx")`
Expected: PASS.

---

## Task 14: Kotlin — `EditHandlers.kt` + Router-Routen

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/EditHandlers.kt`
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt`

- [ ] **Step 1: `EditHandlers.kt` schreiben (native `Write`)**

```kotlin
package com.leanctx.plugin.endpoint

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.leanctx.plugin.dto.EditRequest
import com.leanctx.plugin.dto.EditResponse
import com.leanctx.plugin.psi.SymbolEditor

/**
 * Endpoint layer for the three v2a body-edit ops. Unlike the read endpoints
 * (off-EDT ReadAction), writes go through WriteCommandAction, which dispatches
 * to the EDT itself. The handler invokes the editor on the EDT and blocks for
 * the result.
 */
class EditHandlers(project: Project) {
    private val editor = SymbolEditor(project)

    fun replaceSymbolBody(req: EditRequest): EditResponse = onEdt { editor.apply(req) }
    fun insertBeforeSymbol(req: EditRequest): EditResponse = onEdt { editor.apply(req) }
    fun insertAfterSymbol(req: EditRequest): EditResponse = onEdt { editor.apply(req) }

    /** Run [body] synchronously on the EDT, propagating exceptions to the caller. */
    private fun <T> onEdt(body: () -> T): T {
        var result: T? = null
        var error: Throwable? = null
        ApplicationManager.getApplication().invokeAndWait {
            try {
                result = body()
            } catch (t: Throwable) {
                error = t
            }
        }
        error?.let { throw it }
        @Suppress("UNCHECKED_CAST")
        return result as T
    }
}
```

> `WriteCommandAction.runWriteCommandAction` dispatcht intern bereits auf die EDT; das zusätzliche `invokeAndWait` stellt sicher, dass auch die PSI-Vorbereitung (offset, hash) konsistent auf der EDT läuft und Exceptions sauber propagieren. Falls der Step-0-Spike zeigt, dass `runWriteCommandAction` von einem Background-Thread direkt aufrufbar ist und Exceptions korrekt durchreicht, kann `onEdt` entfallen — dann `editor.apply(req)` direkt zurückgeben. Spike-Ergebnis entscheidet.

- [ ] **Step 2: Router um drei Routen + Dispatcher erweitern**

Per native `Edit` in `RequestRouter.kt`:

(a) Feld ergänzen (nach `inspectionHandlers`):

```kotlin
    private val editHandlers = EditHandlers(project)
```

(b) Import ergänzen:

```kotlin
import com.leanctx.plugin.endpoint.EditHandlers
```

(c) Im `POST`-Block (nach der `/list_inspections`-Zeile) drei Routen einfügen:

```kotlin
            if (path == "/replaceSymbolBody") return dispatchEdit(body, editHandlers::replaceSymbolBody)
            if (path == "/insertBeforeSymbol") return dispatchEdit(body, editHandlers::insertBeforeSymbol)
            if (path == "/insertAfterSymbol") return dispatchEdit(body, editHandlers::insertAfterSymbol)
```

(d) Den generischen Edit-Dispatcher einfügen (nach `dispatchListInspections`):

```kotlin
    private fun dispatchEdit(
        body: String,
        handler: (com.leanctx.plugin.dto.EditRequest) -> com.leanctx.plugin.dto.EditResponse,
    ): HttpResult = try {
        val req = JsonCodec.parseEditRequest(body)
        HttpResult(200, JsonCodec.toJson(handler(req)))
    } catch (e: BackendException) {
        HttpResult(200, JsonCodec.error(e.code, e.message ?: e.code)) // fachlicher Negativfall = 200
    } catch (e: IllegalArgumentException) {
        HttpResult(200, JsonCodec.error("INTERNAL", e.message ?: "bad request"))
    } catch (e: Exception) {
        log.warn("edit endpoint failed", e)
        HttpResult(500, JsonCodec.error("INTERNAL", e.message ?: "internal error"))
    }
```

- [ ] **Step 3: Kompilieren**

Run: `ctx_shell("./gradlew :compileKotlin", cwd="packages/jetbrains-lean-ctx")`
Expected: PASS.

---

## Task 15: Kotlin — Router-getriebener Edit-Test (`RequestRouterEditTest`)

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterEditTest.kt`

- [ ] **Step 1: Test schreiben (native `Write`)**

Muster: `RequestRouterInspectionTest` (BasePlatformTestCase, echtes Test-PSI-Projekt). Der Test legt eine Datei an, fährt `replaceSymbolBody` über den Router und prüft den geschriebenen Inhalt + die `applied`-Response.

```kotlin
package com.leanctx.plugin.server

import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.leanctx.plugin.dto.JsonCodec

class RequestRouterEditTest : BasePlatformTestCase() {

    private fun router(): RequestRouter =
        RequestRouter(
            token = "tok",
            ideVersion = "test",
            projectName = "p",
            project = project,
        )

    fun testReplaceSymbolBodyWritesRange() {
        val psi = myFixture.configureByText(
            "Foo.kt",
            "class A {\n    fun b() { 1 }\n}\n",
        )
        val rel = psi.virtualFile.path.removePrefix(project.basePath ?: "").removePrefix("/")
        // Replace line 2 (0-based line 1), full line range.
        val body = """
            {"path":"$rel",
             "range":{"start":{"line":1,"character":0},"end":{"line":1,"character":17}},
             "text":"    fun b() { 2 }"}
        """.trimIndent()

        val res = router().route("POST", "/replaceSymbolBody", "tok", body)
        assertEquals(200, res.status)
        assertTrue(res.body, res.body.contains("\"applied\":true"))

        val after = String(psi.virtualFile.contentsToByteArray(), Charsets.UTF_8)
        assertTrue(after, after.contains("fun b() { 2 }"))
    }

    fun testReplaceSymbolBodyConflictOnStaleHash() {
        val psi = myFixture.configureByText("Bar.kt", "val x = 1\n")
        val rel = psi.virtualFile.path.removePrefix(project.basePath ?: "").removePrefix("/")
        val body = """
            {"path":"$rel",
             "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":9}},
             "text":"val x = 2","expected_hash":"deadbeef"}
        """.trimIndent()
        val res = router().route("POST", "/replaceSymbolBody", "tok", body)
        assertEquals(200, res.status)
        assertTrue(res.body, res.body.contains("CONFLICT"))
    }
}
```

> `RequestRouter`-Konstruktorparameter gegen `RequestRouter.kt` abgleichen (`token, ideVersion, projectName, project`). `configureByText`-Pfad: `myFixture` legt Dateien in einem temp-Quellroot ab; `project.basePath` zeigt dorthin — `PsiLocator.psiFile(rel)` muss die Datei finden. Falls die Relativierung im Fixture nicht greift (in-memory VFS), den Test mit `myFixture.tempDirFixture` / `copyFileToProject` auf eine echte Plattendatei stützen (Muster aus `RequestRouterInspectionTest` übernehmen).

- [ ] **Step 2: Run test**

Run: `ctx_shell("./gradlew :test --tests '*RequestRouterEditTest*'", cwd="packages/jetbrains-lean-ctx")`
Expected: PASS (beide Test-Methoden).

> Falls `saveDocument` im Headless-Test-Framework nicht auf eine echte Platte schreibt (in-memory VFS), prüft der Test den **Document**-Inhalt statt `contentsToByteArray` — der Edit-Effekt ist in beiden Fällen über das Document sichtbar. Assertion entsprechend auf `psi.text` umstellen, falls nötig.

---

## Task 16: Final-Gate + Docs-Regen + EIN Commit

**Files:**
- Modify: `docs/reference/generated/mcp-tools.md` (generiert)
- Modify: `docs/reference/appendix-mcp-tools.md` (handgepflegt)

- [ ] **Step 1: Schema-Docs regenerieren (Drift-Gate, Spec §12)**

Run: `ctx_shell("cargo run --example gen_docs --features dev-tools", cwd="rust")`
Expected: aktualisiert `docs/reference/generated/mcp-tools.md` mit den drei neuen Actions + Params.

- [ ] **Step 2: Appendix handnachziehen**

Per native `Edit` in `docs/reference/appendix-mcp-tools.md` den `ctx_refactor`-Abschnitt um die drei Edit-Actions ergänzen (Actions-Liste + `name_path`/`new_body`/`text`/`expected_hash`/`end_line`-Params + Fehlercodes `CONFLICT`/`AMBIGUOUS_SYMBOL`/`NO_SYMBOL`). `ctx_search("ctx_refactor", "docs/reference/appendix-mcp-tools.md")` zum Lokalisieren.

- [ ] **Step 3: Drift-Test**

Run: `ctx_shell("cargo nextest run -p lean-ctx -E 'test(generated_reference)'", cwd="rust")`
Expected: PASS (generierte Docs ≡ Schema).

- [ ] **Step 4: Rust-Gesamt-Gate**

Run: `ctx_shell("cargo nextest run -p lean-ctx", cwd="rust")`
Expected: PASS (nur bekannte unabhängige `hn_hardening`-Fails, keine neuen).
Run: `ctx_shell("cargo clippy -p lean-ctx --all-targets", cwd="rust")` → 0 neue Lints.

- [ ] **Step 5: Kotlin-Gesamt-Gate**

Run: `ctx_shell("./gradlew check", cwd="packages/jetbrains-lean-ctx")`
Expected: PASS (compile + alle Unit-Tests).

- [ ] **Step 6: runIde-E2E-Gate (manuell, IDE≡Headless byte-identisch)**

Run: `ctx_shell("./gradlew runIde --rerun-tasks --no-configuration-cache --run_in_background=true", cwd="packages/jetbrains-lean-ctx")` und gegen ein echtes Projekt:
1. `ctx_refactor action=replace_symbol_body name_path="<Klasse/Methode>" new_body="…"` bei **offener** IDE → Edit erscheint mit **einem** Undo-Eintrag, Datei auf Platte aktualisiert.
2. Dieselbe Eingabe gegen ein Backup der Datei bei **geschlossener** IDE (headless) → **byte-identisches** Ergebnis (`diff` der beiden Ergebnisdateien = leer). Das ist das Kern-Gate aus Spec §10 (Entscheidung 4).
3. `name_path`-Mehrdeutigkeit → `AMBIGUOUS_SYMBOL` mit Kandidatenliste; unbekannt → `NO_SYMBOL`; stale `expected_hash` → `CONFLICT`.

Erwartetes Ergebnis: IDE- und Headless-Ausgabe identisch; alle drei Fehlerfälle korrekt.

- [ ] **Step 7: Reformat aller geänderten Dateien (vor `git add`)**

Für jede geänderte Datei `mcp__jetbrains__reformat_file` (bei deferred: `ToolSearch(query="select:mcp__jetbrains__reformat_file")`). Geänderte Dateien: `ctx_shell("git status --porcelain", cwd=".")`.

- [ ] **Step 8: EIN Commit**

```bash
git add rust/src/lsp/backend.rs rust/src/lsp/edit_apply.rs rust/src/lsp/mod.rs \
        rust/src/lsp/jetbrains_backend.rs rust/src/tools/ctx_refactor.rs \
        rust/src/tools/registered/ctx_refactor.rs \
        docs/reference/generated/mcp-tools.md docs/reference/appendix-mcp-tools.md \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/dto/Wire.kt \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/SymbolEditor.kt \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/endpoint/EditHandlers.kt \
        packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/dto/JsonCodecTest.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterEditTest.kt
git commit -m "feat(jetbrains): v2a symbol-body edits (replace/insert_before/after) + overview headless fallback

- ctx_refactor: name_path-addressed replace_symbol_body / insert_before_symbol /
  insert_after_symbol; Rust-side index resolution (AMBIGUOUS_SYMBOL/NO_SYMBOL),
  PathJail, indentation; IDE-first apply with byte-identical headless fallback.
- LspBackend: 3 default-apply edit methods (local_range_write) + tree-sitter
  symbols_overview headless default; JetBrainsHttpBackend HTTP overrides.
- Plugin: SymbolEditor (WriteCommandAction range write) + EditHandlers + 3 routes.
- expected_hash CONFLICT guard; schema + generated/appendix docs."
```

Expected: ein Commit auf `feat-jetbrains-plugin`.

---

## Self-Review-Notizen (vom Plan-Autor geprüft)

- **Spec-Coverage:** §3 name_path-Auflösung → Task 6; §4 Schichtung/Dispatch → Task 8; §5.1 Edit-Methoden + local_range_write → Tasks 1-4; §5.2 overview-Default → Task 4; §5.3 Actions+PathJail+Einrückung → Tasks 6,7,9; §5.4 alle Rust-Änderungsstellen → Tasks 1-10; §6 Plugin (SymbolEditor/EditHandlers/Router/Threading) → Tasks 13,14; §6.1 Reformat-Entkopplung → `text` verbatim (kein Auto-Reformat in SymbolEditor); §7 Wire-DTOs+Endpoints+Fehlercodes → Tasks 5,12,14; §8 Body-Semantik (Range/Insert/Einrückung) → Task 9; §9 Cache-Kohärenz → saveDocument (IDE) / atomarer Write (headless) + mtime-Auto-Validierung; §10 Verifikation → Tasks 11,15,16; §12 EIN-Commit + Schema-Drift → Task 16.
- **Offene Verifikationspunkte (im Plan markiert):** (a) `uri_to_file_path` Rückgabetyp (Task 4); (b) `find_symbols` Substring- vs. Exact-Match + leerer Name für overview (Tasks 4,6); (c) `nearest_project_root`-API (Task 4); (d) `port_discovery`-Feldnamen (Task 8); (e) `hash_hex`-Algorithmus = md5 (Task 13 — sonst CONFLICT-Drift IDE↔headless); (f) `runWriteCommandAction`-Threading (Task 14); (g) Fixture-Pfad-Relativierung im Router-Test (Task 15). Jeder Punkt hat eine konkrete Fallback-Anweisung.
- **Typ-Konsistenz:** `RangeEdit`/`EditResult`/`TextRange0Based` durchgängig identisch benannt (Rust); Wire `EditRequest`/`EditResponse` mit `expected_hash`↔`expectedHash`-SerializedName-Mapping; Endpoints `/replaceSymbolBody`/`/insertBeforeSymbol`/`/insertAfterSymbol` in Rust `post_edit`-Aufrufen ≡ Router-Routen ≡ Test.
