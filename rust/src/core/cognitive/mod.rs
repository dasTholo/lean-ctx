//! Cognitive Budget Allocator (F3) — Miller's Law for LLM context.
//!
//! Limits context to 7±2 semantic chunks, prioritized by task relevance.
//! Uses tree-sitter signature boundaries for chunk detection.

pub mod chunker;

pub use chunker::{ChunkKind, SemanticChunk};
pub use chunker::{budget_select, detect_chunks, render_budget_output};
