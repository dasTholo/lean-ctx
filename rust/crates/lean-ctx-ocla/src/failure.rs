//! Stable failure semantics for capability invocations.

use serde::{Deserialize, Serialize};

/// Normalized failure or fallback outcome for a capability invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityFailureKind {
    Timeout,
    Unavailable,
    RejectedByPolicy,
    InvalidOutput,
    PartialReversible,
    FallbackToNative,
}

/// Failure details correlated to a task and capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityFailure {
    pub kind: CapabilityFailureKind,
    pub capability_id: String,
    pub task_id: String,
    pub detail: Option<String>,
    pub fallback_available: bool,
}
