# lmd Phase 3 — `@graph` (statische Code-Intelligence-Direktive) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Die `@graph`-Direktive mit 7 statischen Ops (dependents, dependencies, related, callers, callees, context, recent-neighbors) als dünne R(+Anreicherung)-Bridge in die bestehenden lean-ctx-Graph-Core-APIs einbauen — kein LSP, deterministisch, CI-tauglich.

**Architecture:** Eine neue `GraphBridge` (`src/lmd/bridges/graph.rs`) routet je Op in `graph_index` (Datei-Deps), `call_graph` (Symbol-Calls) und `graph_context` (PageRank-Kontext). Der `EngineContext` bekommt ein lazy In-Memory-Memo (`ProjectIndex`/`CallGraph` je Render einmal gebaut, §4.2a-analog). Jede Op ist eine freie, mit Fixtures testbare Format-Funktion; `execute` ruft das Memo + die Format-Funktion. Eine fehlende forward-deps-Core-Methode (`get_forward_deps`) wird symmetrisch zu `get_reverse_deps` ergänzt.

**Tech Stack:** Rust (edition 2021), `crate::core::graph_index` (`ProjectIndex`), `crate::core::call_graph` (`CallGraph`), `crate::core::graph_context`; lmd `DirectiveBridge`-Trait; Tests via `cargo nextest run`.

**Spec-Referenz:** `docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md` §4.5 (`@graph`-Design), §6 (Phase 3), §3.1 (Direktiven-Klasse R).

---

## Tool-Disziplin für die Ausführung (verbindlich)

- **`*.rs`-Edits NUR via Serena** (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_after_symbol`/`insert_before_symbol`) — **nie** native `Edit`/`ctx_edit` auf Rust. Markdown (dieser Plan, Spec) via `ctx_edit`/Edit.
- **Lesen/Suchen** via `ctx_read`/`ctx_search`/`ctx_tree` (nie native `Read`/`grep`/`cat`); re-read nach eigener Edit via `ctx_read mode=diff`/`ctx_delta`, **nie `fresh`**.
- **Tests:** immer `cargo nextest run` (nie `cargo test`), via `ctx_shell`.
- **Vor `git add`:** `mcp__jetbrains__reformat_file` auf jede geänderte `*.rs` — **aber** das CI-Gate ist `cargo fmt --check`; nach reformat **immer** `cargo fmt` laufen lassen (jetbrains-reformat ≠ rustfmt, bekannter Phase-2-Befund).
- **Deferred-Tool-Reflex:** zeigt ein MCP-Tool als deferred → `ToolSearch(query="select:<tool>")` FIRST, nie Bash-Workaround.
- Arbeiten direkt auf Branch `feat-lmd-v1`, keine Worktrees.

## Referenz-Fakten (verifiziert 2026-06-05)

Bridge-Trait (`src/lmd/bridges/mod.rs`):
```rust
pub trait DirectiveBridge {
    fn name(&self) -> &'static str;
    fn execute(&self, ctx: &Rc<EngineContext>, args: &DirectiveArgs) -> Result<String, BridgeError>;
}
pub enum BridgeError { MissingArg(&'static str), Resolve(String), Io(String), DepthExceeded, ShellDenied, ShellRejected(String) }
```

`DirectiveArgs` (`src/lmd/args.rs`): `positional(i) -> Option<&str>`, `get(key) -> Option<&str>`, `raw() -> &str`.

Graph-Core-APIs (verifizierte Signaturen):
- `crate::core::graph_index::load_or_build(project_root: &str) -> ProjectIndex` (Disk-cached + Staleness-Check)
- `ProjectIndex { pub files: HashMap<String,FileEntry>, pub edges: Vec<IndexEdge>, pub symbols: HashMap<String,SymbolEntry>, .. }`
- `IndexEdge { pub from: String, pub to: String, pub kind: String, pub weight: f32 }`
- `ProjectIndex::new(project_root: &str) -> Self` (leerer Index — für Test-Fixtures)
- `ProjectIndex::file_count(&self) -> usize`, `edge_count(&self) -> usize`
- `ProjectIndex::get_reverse_deps(&self, path: &str, depth: usize) -> Vec<String>` (BFS über `edge.to == current && kind=="import"`)
- `ProjectIndex::get_related(&self, path: &str, depth: usize) -> Vec<String>` (BFS bidirektional)
- `graph_index::graph_relative_key(path: &str, root: &str) -> String` (Pfad → Repo-relativer Key)
- `crate::core::call_graph::CallGraph::load_or_build(project_root: &str, index: &ProjectIndex) -> CallGraph`
- `CallGraph::new(project_root: &str) -> Self` (leer — für Fixtures), `pub edges: Vec<CallEdge>`, `CallGraph::save(&self) -> Result<(),String>`
- `CallGraph::callers_of(&self, symbol: &str) -> Vec<&CallEdge>`, `callees_of(&self, symbol: &str) -> Vec<&CallEdge>`
- `CallEdge { pub caller_file: String, pub caller_symbol: String, pub caller_line: usize, pub callee_name: String }`
- `crate::core::graph_context::build_graph_context(file_path: &str, project_root: &str, options: Option<GraphContextOptions>) -> Option<GraphContext>`
- `graph_context::format_graph_context(ctx: &GraphContext) -> String`
- `graph_context::graph_neighbor_ranks_for_recent_files(project_root: &str, recent_repo_paths: &[String], per_seed_limit: usize, max_ranked: usize) -> Option<HashMap<String, usize>>`

**Op → Backing (final):**

| Op | Backing | Ziel-Arg |
|----|---------|----------|
| `dependents` | `ProjectIndex::get_reverse_deps` | Datei-Pfad |
| `dependencies` | `ProjectIndex::get_forward_deps` (**neu, Task 1**) | Datei-Pfad |
| `related` | `ProjectIndex::get_related` | Datei-Pfad |
| `callers` | `CallGraph::callers_of` | Symbol-Name |
| `callees` | `CallGraph::callees_of` | Symbol-Name |
| `context` | `build_graph_context` + `format_graph_context` | Datei-Pfad |
| `recent-neighbors` | `graph_neighbor_ranks_for_recent_files` (explizite Seeds, v1) | Seed-Pfade |

---

## Task 1: Core-Methode `get_forward_deps` (symmetrisch zu `get_reverse_deps`)

`@graph dependencies` braucht forward-deps (was importiert *diese* Datei), die es als Core-API noch nicht gibt. Wir spiegeln `get_reverse_deps` (1:1, nur `edge.from == current` statt `edge.to == current`). Hält die Bridge ein dünner Router.

**Files:**
- Modify: `rust/src/core/graph_index/mod.rs` (neue Methode in `impl ProjectIndex`, nach `get_reverse_deps` ~L354)
- Test: gleiche Datei, `#[cfg(test)] mod tests`

- [ ] **Step 1: Failing test schreiben**

Finde mit `mcp__serena__jet_brains_find_symbol` (name_path `tests`, relative_path `rust/src/core/graph_index/mod.rs`) den Test-Mod (oder lege via `insert_after_symbol` nach der letzten Funktion einen an). Füge mit `mcp__serena__insert_after_symbol` diesen Test ein:

```rust
    #[test]
    fn get_forward_deps_follows_import_edges_outward() {
        let mut idx = ProjectIndex::new("/tmp/fwd");
        idx.edges.push(IndexEdge { from: "a.rs".into(), to: "b.rs".into(), kind: "import".into(), weight: 1.0 });
        idx.edges.push(IndexEdge { from: "b.rs".into(), to: "c.rs".into(), kind: "import".into(), weight: 1.0 });
        let deps = idx.get_forward_deps("a.rs", 2);
        assert!(deps.contains(&"b.rs".to_string()), "got: {deps:?}");
        assert!(deps.contains(&"c.rs".to_string()), "got: {deps:?}");
        // reverse direction must NOT appear
        assert!(idx.get_forward_deps("c.rs", 2).is_empty(), "leaf has no forward deps");
    }
```

- [ ] **Step 2: Test laufen lassen — muss fehlschlagen**

Run: `cargo nextest run -p lean-ctx get_forward_deps_follows_import_edges_outward` (via `ctx_shell`)
Expected: FAIL — `no method named get_forward_deps`.

- [ ] **Step 3: Methode implementieren**

Mit `mcp__serena__insert_after_symbol` (name_path `ProjectIndex/get_reverse_deps`, relative_path `rust/src/core/graph_index/mod.rs`):

```rust
    /// Forward import dependencies: files that `path` (transitively) imports.
    /// Mirror of `get_reverse_deps` with the edge direction flipped.
    pub fn get_forward_deps(&self, path: &str, depth: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue: Vec<(String, usize)> = vec![(path.to_string(), 0)];

        while let Some((current, d)) = queue.pop() {
            if d > depth || visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            if current != path {
                result.push(current.clone());
            }

            for edge in &self.edges {
                if edge.from == current && edge.kind == "import" && !visited.contains(&edge.to) {
                    queue.push((edge.to.clone(), d + 1));
                }
            }
        }
        result
    }
```

- [ ] **Step 4: Test laufen lassen — muss bestehen**

Run: `cargo nextest run -p lean-ctx get_forward_deps_follows_import_edges_outward`
Expected: PASS.

- [ ] **Step 5: Reformat + commit**

`mcp__jetbrains__reformat_file` auf `rust/src/core/graph_index/mod.rs`, dann `cargo fmt` (via `ctx_shell`), dann:
```bash
git add rust/src/core/graph_index/mod.rs
git commit -m "feat(graph): add ProjectIndex::get_forward_deps (symmetric to get_reverse_deps)"
```

---

## Task 2: `EngineContext` um lazy Graph-Memo erweitern

Ein `ProjectIndex`/`CallGraph` soll pro Render **einmal** gebaut und über alle `@graph`-Ops geteilt werden (§4.2a-analog: ein Build, geteilter Zustand). Lazy via `RefCell<Option<Rc<…>>>`.

**Files:**
- Modify: `rust/src/lmd/engine.rs` (Struct `EngineContext` ~L19, `EngineContext::new` ~L33, neue Methoden im `impl`)
- Test: gleiche Datei, `#[cfg(test)] mod tests`

- [ ] **Step 1: Failing test schreiben**

Mit `mcp__serena__find_symbol` den Test-Mod in `rust/src/lmd/engine.rs` finden; via `insert_after_symbol` (an eine bestehende Test-Fn) einfügen:

```rust
    #[test]
    fn index_memo_returns_same_handle_twice() {
        let ctx = Rc::new(EngineContext::new(LeanMdHeader::default(), PathBuf::from(".")));
        let a = ctx.index();
        let b = ctx.index();
        assert!(Rc::ptr_eq(&a, &b), "index() must memoize one build per render");
    }
```

(Test-Imports `LeanMdHeader`, `PathBuf` sind im Test-Mod bereits vorhanden — siehe bestehende Engine-Tests.)

- [ ] **Step 2: Test laufen lassen — muss fehlschlagen**

Run: `cargo nextest run -p lean-ctx index_memo_returns_same_handle_twice`
Expected: FAIL — `no method named index`.

- [ ] **Step 3: Imports + Felder + Methoden ergänzen**

a) Imports oben in `engine.rs` ergänzen (via `mcp__serena__insert_after_symbol` an den bestehenden `use super::header::...`-Block oder `replace_symbol_body` falls als Block erfasst):
```rust
use crate::core::call_graph::CallGraph;
use crate::core::graph_index::{self, ProjectIndex};
```

b) Zwei Felder in `struct EngineContext` ergänzen (via `replace_symbol_body` auf `EngineContext`-Struct — gesamte Struct neu setzen, mit den neuen Feldern nach `depth`):
```rust
pub struct EngineContext {
    pub header: LeanMdHeader,
    pub jail_root: PathBuf,
    pub fragments: FragmentRegistry,
    pub registry: BridgeRegistry,
    pub cache: RefCell<SessionCache>,
    pub max_chain_depth: usize,
    depth: Cell<usize>,
    /// Lazy per-render memo of the static graph index (one build, shared by
    /// every `@graph` op in this render — §4.2a-analog).
    graph_index: RefCell<Option<Rc<ProjectIndex>>>,
    /// Lazy per-render memo of the call graph (built from `graph_index`).
    call_graph: RefCell<Option<Rc<CallGraph>>>,
}
```
(Behalte die Doc-Kommentare der bestehenden Felder bei — übernimm sie aus der aktuellen Struct, nur die zwei neuen Felder kommen hinzu.)

c) `EngineContext::new` um die Feld-Initialisierung ergänzen (im Struct-Literal nach `depth: Cell::new(0),`):
```rust
            graph_index: RefCell::new(None),
            call_graph: RefCell::new(None),
```

d) Zwei Methoden in `impl EngineContext` ergänzen (via `insert_after_symbol` nach `leave`):
```rust
    /// Lazy-build + memoize the static project index for this render.
    pub fn index(&self) -> Rc<ProjectIndex> {
        if let Some(existing) = self.graph_index.borrow().as_ref() {
            return existing.clone();
        }
        let root = self.jail_root.to_str().unwrap_or(".");
        let built = Rc::new(graph_index::load_or_build(root));
        *self.graph_index.borrow_mut() = Some(built.clone());
        built
    }

    /// Lazy-build + memoize the call graph (depends on `index()`).
    pub fn call_graph(&self) -> Rc<CallGraph> {
        if let Some(existing) = self.call_graph.borrow().as_ref() {
            return existing.clone();
        }
        let index = self.index();
        let root = self.jail_root.to_str().unwrap_or(".");
        let built = Rc::new(CallGraph::load_or_build(root, &index));
        let _ = built.save();
        *self.call_graph.borrow_mut() = Some(built.clone());
        built
    }
```

- [ ] **Step 4: Test laufen lassen — muss bestehen**

Run: `cargo nextest run -p lean-ctx index_memo_returns_same_handle_twice`
Expected: PASS.

- [ ] **Step 5: Reformat + commit**

reformat + `cargo fmt`, dann:
```bash
git add rust/src/lmd/engine.rs
git commit -m "feat(lmd): add lazy graph_index/call_graph memo to EngineContext (§4.2a-analog)"
```

---

## Task 3: `GraphBridge`-Skelett + Registrierung + `dependents`-Op

Neue Bridge-Datei mit Dispatch-Gerüst und der ersten Op. Muster exakt wie `count.rs`.

**Files:**
- Create: `rust/src/lmd/bridges/graph.rs`
- Modify: `rust/src/lmd/bridges/mod.rs` (`pub mod graph;` + Registrierung + Registry-Test)
- Test: in `graph.rs`

- [ ] **Step 1: Failing test schreiben** (Bridge-Datei mit Skelett + Test anlegen)

Lege `rust/src/lmd/bridges/graph.rs` an (via `Write`):

```rust
//! `@graph` Router bridge → static code-intelligence over the lean-ctx graph
//! APIs (spec §4.5). 7 ops, no LSP: dependents/dependencies/related (file deps),
//! callers/callees (call graph), context (PageRank), recent-neighbors.
use std::rc::Rc;

use super::{BridgeError, DirectiveBridge};
use crate::core::graph_index::{self, ProjectIndex};
use crate::lmd::args::DirectiveArgs;
use crate::lmd::engine::EngineContext;

pub struct GraphBridge;

impl DirectiveBridge for GraphBridge {
    fn name(&self) -> &'static str {
        "graph"
    }

    fn execute(&self, ctx: &Rc<EngineContext>, args: &DirectiveArgs) -> Result<String, BridgeError> {
        let op = args.positional(0).ok_or(BridgeError::MissingArg("op"))?;
        let root = ctx.jail_root.to_str().unwrap_or(".");
        match op {
            "dependents" => {
                let target = args.positional(1).ok_or(BridgeError::MissingArg("path"))?;
                let key = graph_index::graph_relative_key(target, root);
                Ok(fmt_dependents(&ctx.index(), &key, depth_arg(args)))
            }
            other => Err(BridgeError::Resolve(format!(
                "unknown @graph op '{other}'. Use: dependents|dependencies|related|callers|callees|context|recent-neighbors"
            ))),
        }
    }
}

/// `depth=N` named arg, default 2, clamped to 1..=5.
fn depth_arg(args: &DirectiveArgs) -> usize {
    args.get("depth")
        .and_then(|d| d.parse::<usize>().ok())
        .unwrap_or(2)
        .clamp(1, 5)
}

fn fmt_dependents(index: &ProjectIndex, key: &str, depth: usize) -> String {
    let deps = index.get_reverse_deps(key, depth);
    if deps.is_empty() {
        return format!(
            "No dependents of '{key}' ({} files, {} edges indexed)",
            index.file_count(),
            index.edge_count()
        );
    }
    let mut out = format!("{} dependent(s) of '{key}' (depth≤{depth}):\n", deps.len());
    for d in &deps {
        out.push_str(&format!("  {d}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph_index::IndexEdge;

    fn index_a_imports_b() -> ProjectIndex {
        let mut idx = ProjectIndex::new("/tmp/g");
        idx.edges.push(IndexEdge { from: "a.rs".into(), to: "b.rs".into(), kind: "import".into(), weight: 1.0 });
        idx
    }

    #[test]
    fn fmt_dependents_lists_importers() {
        let out = fmt_dependents(&index_a_imports_b(), "b.rs", 2);
        assert!(out.contains("a.rs"), "got: {out}");
        assert!(out.contains("dependent"), "got: {out}");
    }

    #[test]
    fn fmt_dependents_empty_is_explained() {
        let out = fmt_dependents(&index_a_imports_b(), "nope.rs", 2);
        assert!(out.contains("No dependents"), "got: {out}");
    }

    #[test]
    fn graph_is_registered() {
        assert!(super::super::default_registry().get("graph").is_some());
    }
}
```

- [ ] **Step 2: `mod.rs` verdrahten**

In `rust/src/lmd/bridges/mod.rs`:
- via `mcp__serena__insert_after_symbol` (oder `replace_content`) `pub mod graph;` zur Modul-Liste (alphabetisch nach `pub mod env;`).
- in `default_registry()` (via `replace_symbol_body` auf `default_registry`) `reg.register(Box::new(graph::GraphBridge));` ergänzen.
- im Test `default_registry_has_all_core_bridges` das Namens-Array um `"graph"` erweitern (via `replace_symbol_body`).

- [ ] **Step 3: Tests laufen lassen — `graph_is_registered` muss zunächst evtl. fehlschlagen, dann grün**

Run: `cargo nextest run -p lean-ctx fmt_dependents graph_is_registered default_registry_has_all_core_bridges`
Expected: nach Step 1+2 alle PASS. (Wird `graph_is_registered` rot, fehlt die Registrierung aus Step 2.)

- [ ] **Step 4: Reformat + commit**

reformat (`graph.rs` + `mod.rs`) + `cargo fmt`, dann:
```bash
git add rust/src/lmd/bridges/graph.rs rust/src/lmd/bridges/mod.rs
git commit -m "feat(lmd): @graph bridge skeleton + dependents op + registry wiring"
```

---

## Task 4: Ops `dependencies` + `related`

**Files:**
- Modify: `rust/src/lmd/bridges/graph.rs` (zwei Match-Arme + zwei Format-Fns + Tests)

- [ ] **Step 1: Failing tests schreiben**

Via `mcp__serena__insert_before_symbol` (vor `fn graph_is_registered` im Test-Mod) ergänzen:
```rust
    #[test]
    fn fmt_dependencies_lists_imported() {
        let out = fmt_dependencies(&index_a_imports_b(), "a.rs", 2);
        assert!(out.contains("b.rs"), "got: {out}");
        assert!(out.contains("dependenc"), "got: {out}");
    }

    #[test]
    fn fmt_related_lists_either_direction() {
        let out = fmt_related(&index_a_imports_b(), "a.rs", 2);
        assert!(out.contains("b.rs"), "got: {out}");
    }
```

- [ ] **Step 2: Tests laufen lassen — müssen fehlschlagen**

Run: `cargo nextest run -p lean-ctx fmt_dependencies fmt_related`
Expected: FAIL — `cannot find function fmt_dependencies`.

- [ ] **Step 3: Format-Fns + Dispatch-Arme implementieren**

Via `insert_after_symbol` (nach `fmt_dependents`) zwei Funktionen ergänzen:
```rust
fn fmt_dependencies(index: &ProjectIndex, key: &str, depth: usize) -> String {
    let deps = index.get_forward_deps(key, depth);
    if deps.is_empty() {
        return format!(
            "No dependencies of '{key}' ({} files, {} edges indexed)",
            index.file_count(),
            index.edge_count()
        );
    }
    let mut out = format!("{} dependenc(ies) of '{key}' (depth≤{depth}):\n", deps.len());
    for d in &deps {
        out.push_str(&format!("  {d}\n"));
    }
    out
}

fn fmt_related(index: &ProjectIndex, key: &str, depth: usize) -> String {
    let rel = index.get_related(key, depth);
    if rel.is_empty() {
        return format!(
            "No related files for '{key}' ({} files, {} edges indexed)",
            index.file_count(),
            index.edge_count()
        );
    }
    let mut out = format!("{} related file(s) for '{key}' (depth≤{depth}):\n", rel.len());
    for d in &rel {
        out.push_str(&format!("  {d}\n"));
    }
    out
}
```

Im `execute`-Match (via `replace_symbol_body` auf `GraphBridge/execute`) zwei Arme vor dem `other =>` einsetzen:
```rust
            "dependencies" => {
                let target = args.positional(1).ok_or(BridgeError::MissingArg("path"))?;
                let key = graph_index::graph_relative_key(target, root);
                Ok(fmt_dependencies(&ctx.index(), &key, depth_arg(args)))
            }
            "related" => {
                let target = args.positional(1).ok_or(BridgeError::MissingArg("path"))?;
                let key = graph_index::graph_relative_key(target, root);
                Ok(fmt_related(&ctx.index(), &key, depth_arg(args)))
            }
```

- [ ] **Step 4: Tests laufen lassen — müssen bestehen**

Run: `cargo nextest run -p lean-ctx fmt_dependencies fmt_related`
Expected: PASS.

- [ ] **Step 5: Reformat + commit**

```bash
git add rust/src/lmd/bridges/graph.rs
git commit -m "feat(lmd): @graph dependencies + related ops"
```

---

## Task 5: Ops `callers` + `callees`

Routen in `CallGraph` (Symbol-Ebene). Format analog `ctx_callgraph::format_callers`/`format_callees`, aber eigenständig (memo-konsistent über `ctx.call_graph()`).

**Files:**
- Modify: `rust/src/lmd/bridges/graph.rs`

- [ ] **Step 1: Failing tests schreiben**

Test-Fixture braucht einen `CallGraph`. Via `insert_before_symbol` (vor `fn graph_is_registered`) ergänzen:
```rust
    fn call_graph_a_calls_b() -> crate::core::call_graph::CallGraph {
        use crate::core::call_graph::{CallEdge, CallGraph};
        let mut g = CallGraph::new("/tmp/g");
        g.edges.push(CallEdge {
            caller_file: "a.rs".into(),
            caller_symbol: "fn_a".into(),
            caller_line: 10,
            callee_name: "fn_b".into(),
        });
        g
    }

    #[test]
    fn fmt_callers_lists_calling_symbols() {
        let out = fmt_callers(&call_graph_a_calls_b(), "fn_b");
        assert!(out.contains("fn_a"), "got: {out}");
        assert!(out.contains("caller"), "got: {out}");
    }

    #[test]
    fn fmt_callees_lists_called_symbols() {
        let out = fmt_callees(&call_graph_a_calls_b(), "fn_a");
        assert!(out.contains("fn_b"), "got: {out}");
        assert!(out.contains("callee"), "got: {out}");
    }
```

- [ ] **Step 2: Tests laufen lassen — müssen fehlschlagen**

Run: `cargo nextest run -p lean-ctx fmt_callers fmt_callees`
Expected: FAIL — `cannot find function fmt_callers`.

- [ ] **Step 3: Format-Fns + Dispatch + Import ergänzen**

Import oben in `graph.rs` (nach den bestehenden `use`-Zeilen, via `insert_after_symbol` an den letzten `use`):
```rust
use crate::core::call_graph::CallGraph;
```

Format-Fns (via `insert_after_symbol` nach `fmt_related`):
```rust
fn fmt_callers(graph: &CallGraph, symbol: &str) -> String {
    let callers = graph.callers_of(symbol);
    if callers.is_empty() {
        return format!("No callers of '{symbol}' ({} edges in call graph)", graph.edges.len());
    }
    let mut out = format!("{} caller(s) of '{symbol}':\n", callers.len());
    for e in &callers {
        out.push_str(&format!("  {} → {}  (L{})\n", e.caller_file, e.caller_symbol, e.caller_line));
    }
    out
}

fn fmt_callees(graph: &CallGraph, symbol: &str) -> String {
    let callees = graph.callees_of(symbol);
    if callees.is_empty() {
        return format!("No callees of '{symbol}' ({} edges in call graph)", graph.edges.len());
    }
    let mut out = format!("{} callee(s) of '{symbol}':\n", callees.len());
    for e in &callees {
        out.push_str(&format!("  → {}  ({}:L{})\n", e.callee_name, e.caller_file, e.caller_line));
    }
    out
}
```

Dispatch-Arme (in `GraphBridge/execute` via `replace_symbol_body`, vor `other =>`):
```rust
            "callers" => {
                let sym = args.positional(1).ok_or(BridgeError::MissingArg("symbol"))?;
                Ok(fmt_callers(&ctx.call_graph(), sym))
            }
            "callees" => {
                let sym = args.positional(1).ok_or(BridgeError::MissingArg("symbol"))?;
                Ok(fmt_callees(&ctx.call_graph(), sym))
            }
```

- [ ] **Step 4: Tests laufen lassen — müssen bestehen**

Run: `cargo nextest run -p lean-ctx fmt_callers fmt_callees`
Expected: PASS.

- [ ] **Step 5: Reformat + commit**

```bash
git add rust/src/lmd/bridges/graph.rs
git commit -m "feat(lmd): @graph callers + callees ops (CallGraph)"
```

---

## Task 6: Op `context`

Routet in `build_graph_context` + `format_graph_context` (PageRank-Nachbarschaft mit Token-Budget).

**Files:**
- Modify: `rust/src/lmd/bridges/graph.rs`

- [ ] **Step 1: Failing test schreiben**

`build_graph_context` braucht ein reales Projekt (es liest Dateien). Test gegen das Repo selbst (Smoke: kein Panic, plausible Ausgabe). Via `insert_before_symbol` (vor `fn graph_is_registered`):
```rust
    #[test]
    fn context_op_renders_for_a_real_file() {
        use crate::lmd::header::LeanMdHeader;
        use std::path::PathBuf;
        let ctx = Rc::new(EngineContext::new(LeanMdHeader::default(), PathBuf::from(".")));
        let args = DirectiveArgs::parse("context rust/src/lmd/engine.rs");
        let out = GraphBridge.execute(&ctx, &args).expect("context op must not error");
        // Either a rendered context or a graceful "no context" line — never empty.
        assert!(!out.trim().is_empty(), "got empty output");
    }
```

- [ ] **Step 2: Test laufen lassen — muss fehlschlagen**

Run: `cargo nextest run -p lean-ctx context_op_renders_for_a_real_file`
Expected: FAIL — `unknown @graph op 'context'` (Resolve-Error → `expect` panics).

- [ ] **Step 3: Dispatch-Arm + Import implementieren**

Import oben in `graph.rs` (via `insert_after_symbol` an den letzten `use`):
```rust
use crate::core::graph_context;
```

Dispatch-Arm (in `GraphBridge/execute` via `replace_symbol_body`, vor `other =>`):
```rust
            "context" => {
                let target = args.positional(1).ok_or(BridgeError::MissingArg("path"))?;
                let abs = if std::path::Path::new(target).is_absolute() {
                    target.to_string()
                } else {
                    format!("{root}/{target}")
                };
                match graph_context::build_graph_context(&abs, root, None) {
                    Some(gc) => Ok(graph_context::format_graph_context(&gc)),
                    None => Ok(format!("No graph context available for '{target}'")),
                }
            }
```

- [ ] **Step 4: Test laufen lassen — muss bestehen**

Run: `cargo nextest run -p lean-ctx context_op_renders_for_a_real_file`
Expected: PASS.

- [ ] **Step 5: Reformat + commit**

```bash
git add rust/src/lmd/bridges/graph.rs
git commit -m "feat(lmd): @graph context op (build_graph_context + format)"
```

---

## Task 7: Op `recent-neighbors` (explizite Seed-Pfade, v1)

`graph_neighbor_ranks_for_recent_files` braucht Seed-Pfade. Der `EngineContext` ist render-lokal und **session-frei** — es gibt kein `session.files_touched` hier. v1 nimmt die Seeds darum als **explizite positionale Args** (`@graph recent-neighbors <p1> <p2> …`). Die Session-Automatik (G-1) wird nachgerüstet, sobald die `ctx_md_*`-MCP-Schicht (Phase 8) den Render mit Session-Kontext speist — als Follow-up notiert.

**Files:**
- Modify: `rust/src/lmd/bridges/graph.rs`

- [ ] **Step 1: Failing test schreiben**

Via `insert_before_symbol` (vor `fn graph_is_registered`):
```rust
    #[test]
    fn recent_neighbors_requires_at_least_one_seed() {
        use crate::lmd::header::LeanMdHeader;
        use std::path::PathBuf;
        let ctx = Rc::new(EngineContext::new(LeanMdHeader::default(), PathBuf::from(".")));
        let err = GraphBridge
            .execute(&ctx, &DirectiveArgs::parse("recent-neighbors"))
            .unwrap_err();
        assert!(matches!(err, BridgeError::MissingArg(_)), "got: {err:?}");
    }

    #[test]
    fn recent_neighbors_renders_for_real_seed() {
        use crate::lmd::header::LeanMdHeader;
        use std::path::PathBuf;
        let ctx = Rc::new(EngineContext::new(LeanMdHeader::default(), PathBuf::from(".")));
        let args = DirectiveArgs::parse("recent-neighbors rust/src/lmd/engine.rs");
        let out = GraphBridge.execute(&ctx, &args).expect("must not error");
        assert!(!out.trim().is_empty(), "got empty output");
    }
```

- [ ] **Step 2: Tests laufen lassen — müssen fehlschlagen**

Run: `cargo nextest run -p lean-ctx recent_neighbors_requires_at_least_one_seed recent_neighbors_renders_for_real_seed`
Expected: FAIL — `unknown @graph op 'recent-neighbors'`.

- [ ] **Step 3: Format-Fn + Dispatch-Arm implementieren**

Format-Fn (via `insert_after_symbol` nach `fmt_callees`):
```rust
/// Render the rank map (lower rank = closer neighbor) as a sorted list.
fn fmt_recent_neighbors(root: &str, seeds: &[String]) -> String {
    match graph_context::graph_neighbor_ranks_for_recent_files(root, seeds, 10, 20) {
        Some(ranks) if !ranks.is_empty() => {
            let mut entries: Vec<(&String, &usize)> = ranks.iter().collect();
            entries.sort_by(|a, b| a.1.cmp(b.1).then_with(|| a.0.cmp(b.0)));
            let mut out = format!("{} graph neighbor(s) of {} recent seed(s):\n", entries.len(), seeds.len());
            for (path, rank) in entries {
                out.push_str(&format!("  [{rank}] {path}\n"));
            }
            out
        }
        _ => format!("No graph neighbors for {} seed(s)", seeds.len()),
    }
}
```

Dispatch-Arm (in `GraphBridge/execute` via `replace_symbol_body`, vor `other =>`). Seeds = alle positionalen Args ab Index 1, zu Repo-relativen Keys normalisiert:
```rust
            "recent-neighbors" => {
                let seeds: Vec<String> = (1..)
                    .map_while(|i| args.positional(i))
                    .map(|p| graph_index::graph_relative_key(p, root))
                    .collect();
                if seeds.is_empty() {
                    return Err(BridgeError::MissingArg("seed-path"));
                }
                Ok(fmt_recent_neighbors(root, &seeds))
            }
```

- [ ] **Step 4: Tests laufen lassen — müssen bestehen**

Run: `cargo nextest run -p lean-ctx recent_neighbors_requires_at_least_one_seed recent_neighbors_renders_for_real_seed`
Expected: PASS.

- [ ] **Step 5: Reformat + commit**

```bash
git add rust/src/lmd/bridges/graph.rs
git commit -m "feat(lmd): @graph recent-neighbors op (explicit seeds, v1)"
```

---

## Task 8: e2e-Render-Test + Phase-3-Abschluss

End-to-end via `render()` (Parser → Bridge → Output) + Voll-Suite + Spec-Status.

**Files:**
- Modify: `rust/src/lmd/engine.rs` (`#[cfg(test)]`-e2e-Test)
- Modify: `docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md` (§6 Phase 3 → bestanden)

- [ ] **Step 1: e2e-Failing-Test schreiben**

Via `mcp__serena__insert_after_symbol` (an eine bestehende Engine-Test-Fn) in `rust/src/lmd/engine.rs`:
```rust
    #[test]
    fn graph_directive_renders_dependents_e2e() {
        // Render a `@graph dependents` directive end-to-end against this repo.
        let out = render("@graph dependents rust/src/lmd/engine.rs\n");
        // Either a dependents list or the graceful "No dependents" line — the
        // directive must be dispatched (not the unknown-directive fallback).
        assert!(
            out.contains("dependent") || out.contains("No dependents"),
            "got: {out}"
        );
        assert!(!out.contains("unknown directive"), "got: {out}");
    }
```

- [ ] **Step 2: Test laufen lassen — muss bestehen (Bridge ist seit Task 3 registriert)**

Run: `cargo nextest run -p lean-ctx graph_directive_renders_dependents_e2e`
Expected: PASS. (Falls FAIL mit „unknown directive": Registrierung aus Task 3 prüfen.)

- [ ] **Step 3: Voll-Suite + fmt-Gate**

Run (via `ctx_shell`):
```
cargo nextest run -p lean-ctx
cargo fmt --check
```
Expected: alle lmd-Tests grün; `cargo fmt --check` ohne Diff. (Bei fmt-Diff: `cargo fmt` laufen lassen, erneut prüfen.)

- [ ] **Step 4: Spec-Status auf Phase 3 = bestanden**

In `docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md` die §6-Zeile **3** (via `ctx_edit`, Markdown) auf bestanden setzen — Kern (ohne Trailing-Padding) ersetzen:
- alt-Kern: `| **3** | **`@graph`** (7 statische Ops, … → CI/Golden-Parity §8.2`
- neu: Prefix `✅ **bestanden (2026-06-05):**` + Hinweis „7 Ops live; `get_forward_deps` (dependencies) als symmetrische Core-Methode ergänzt; `recent-neighbors` v1 = explizite Seeds (Session-Auto → Phase-8-Follow-up); EngineContext-Graph-Memo (§4.2a-analog); N/N lmd-Tests grün".

- [ ] **Step 5: Commit**

reformat etwaiger `*.rs`, `cargo fmt`, dann:
```bash
git add rust/src/lmd/engine.rs docs/lean-md/specs/2026-05-31-lmd-lean-ctx-native-design.mdai.md
git commit -m "feat(lmd): @graph e2e render test; mark Phase-3 gate passed"
```

---

## Self-Review Notes (für den Ausführenden)

- **Spec-Coverage:** Alle 7 Ops aus §4.5 sind je einem Task zugeordnet (T3 dependents, T4 dependencies+related, T5 callers+callees, T6 context, T7 recent-neighbors). Infrastruktur: T1 (forward-deps-Core), T2 (EngineContext-Memo). T8 = e2e + Gate.
- **Abweichungen von der Spec (bewusst, im Plan begründet):**
  1. `dependencies` braucht die **neue** Core-Methode `get_forward_deps` (kein bestehendes forward-deps-Backing) — T1.
  2. `recent-neighbors` v1 = **explizite Seeds** statt `session.files_touched` (EngineContext ist session-frei); Session-Auto = Phase-8-Follow-up. Nach Abschluss in §9 G-1 der Spec als „v1=explizite Seeds, Auto deferred" präzisieren.
- **Typ-Konsistenz:** Format-Fns heißen durchgängig `fmt_<op>`; `ctx.index()`/`ctx.call_graph()` liefern `Rc<ProjectIndex>`/`Rc<CallGraph>`; `depth_arg` einheitlich (default 2, clamp 1..=5).
- **PathJail (§7):** geerbt — `load_or_build`/`build_graph_context` traversieren nur `jail_root`; `graph_relative_key(path, root)` normalisiert das Ziel. Kein eigener Jail.
- **Offen für T8-Reviewer:** finale Testzahl (`N/N`) im Spec-Status aus dem realen `cargo nextest run`-Output einsetzen, nicht raten.
