//! Durable provenance records for file edits and git checkpoints.

mod store;
mod tracker;
mod types;

pub use store::ProvenanceStore;
pub use tracker::ProvenanceTracker;
pub use types::{
    CheckpointLinkState, CheckpointRecord, LinkState, ObservationConfidence, ProvenanceRecord,
};

/// Stable identifier for a single observed file operation.
pub type ProvenanceId = String;

/// Identifier of a `SolutionDecisionMeta` decision.
pub type DecisionId = String;

/// Stable identifier for a git-related provenance checkpoint.
pub type CheckpointId = String;

#[cfg(test)]
mod tests;
