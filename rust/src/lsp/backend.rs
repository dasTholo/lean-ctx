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
    fn references(
        &mut self,
        uri: &Uri,
        position: Position,
        scope: &str,
    ) -> Result<Vec<Location>, String>;
    fn definition(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<GotoDefinitionResponse, String>;
    fn implementations(
        &mut self,
        uri: &Uri,
        position: Position,
        scope: &str,
    ) -> Result<Vec<Location>, String>;
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
