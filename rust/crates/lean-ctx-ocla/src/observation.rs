//! Per-invocation capability observations.

use crate::failure::CapabilityFailure;
use serde::{Deserialize, Serialize};

/// A single capability invocation observation.
///
/// Aggregation and ranking deliberately live outside this wire record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityObservationV1 {
    pub task_id: String,
    pub capability_id: String,
    pub capability_version: String,
    pub invocation_start: String,
    pub invocation_end: String,
    pub latency_ms: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub success: bool,
    pub failure: Option<CapabilityFailure>,
    pub evidence_ref: Option<String>,
}
