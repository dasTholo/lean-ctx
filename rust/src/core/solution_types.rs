//! Backward-compatible Solution Intelligence type exports.
//!
//! Canonical definitions live in `core::knowledge` so decision persistence and
//! public Solution Intelligence APIs share exactly one representation.
pub use crate::core::knowledge::{SolutionDecisionKind, SolutionDecisionMeta, SolutionStatus};
