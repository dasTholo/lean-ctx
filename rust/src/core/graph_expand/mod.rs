//! On-Demand Graph Expansion (F6) — k-hop subgraph extraction.
//!
//! Extracts local neighborhoods from the symbol graph, providing just enough
//! context to understand call chains without loading the entire graph.

pub mod hops;
pub mod partial;

#[cfg(test)]
pub use hops::expand_neighborhood;
#[cfg(test)]
pub use partial::EdgeKind;
pub use partial::PartialGraph;
