//! Causal outcome attribution (OSS stub).
//!
//! Enterprise tracks which context chunks contributed to successful outcomes.
//! OSS: no-op recording, empty chunk queries.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A record of a context chunk delivered to the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChunkRecord {
    pub content_hash: String,
    pub source: String,
    pub token_cost: usize,
    pub turn_provided: u64,
}

impl ContextChunkRecord {
    pub fn new(
        content: &str,
        source: impl Into<String>,
        token_cost: usize,
        turn_provided: u64,
    ) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);
        Self {
            content_hash: format!("{:016x}", hasher.finish()),
            source: source.into(),
            token_cost,
            turn_provided,
        }
    }
}

/// Outcome classification for attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    Failure,
}

/// Signal emitted when a value gate outcome is recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeSignal {
    pub session_id: String,
    pub outcome: Outcome,
    pub evidence: String,
}

/// Records a context chunk for later attribution (OSS: no-op).
pub fn record_chunk(_session_id: &str, _chunk: ContextChunkRecord) -> Result<(), String> {
    Ok(())
}

/// Returns chunks recorded for the session (OSS: empty).
pub fn chunks_for_session(_session_id: &str) -> Vec<ContextChunkRecord> {
    Vec::new()
}

/// Records a value gate outcome (OSS: no-op).
pub fn record_outcome(_task_id: &str, _signal: OutcomeSignal) -> Result<(), String> {
    Ok(())
}

/// Suggests context removals based on attribution data (OSS: empty).
pub fn suggest_removals() -> Vec<String> {
    Vec::new()
}

/// Records proxy context for attribution (OSS: no-op).
pub fn record_proxy_context<R>(
    _session_id: &str,
    _request: &R,
    _turn_provided: u64,
) -> Result<(), String> {
    Ok(())
}

/// Storage path resolver.
pub struct CausalAttributor;

impl CausalAttributor {
    pub fn default_path() -> Result<PathBuf, String> {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("lean-ctx");
        Ok(dir.join("causal_attribution.jsonl"))
    }
}
