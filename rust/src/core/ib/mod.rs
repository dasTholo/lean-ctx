//! Task-aware Information Bottleneck compression (F1).
//!
//! Extends the existing `information_bottleneck` module with intent classification:
//! instead of requiring explicit query terms, infers them from the agent's session state.

pub mod intent;
pub mod relevance;

pub use intent::TaskIntent;
pub use intent::classify_intent;
pub use relevance::{RelevanceScore, compute_relevance, intent_query_terms};
