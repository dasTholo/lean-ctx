//! Deterministic, task-scoped grouping of typed evidence references.

use crate::core::evidence_ledger::EvidenceRef;

/// A finalized set of typed evidence references for one task.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceBundle {
    pub task_id: String,
    pub refs: Vec<EvidenceRef>,
    /// BLAKE3 of the concatenated `content_hash` values in insertion order.
    pub bundle_hash: String,
}

impl EvidenceBundle {
    /// Create an empty bundle for `task_id`.
    #[must_use]
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            refs: Vec::new(),
            bundle_hash: String::new(),
        }
    }

    /// Append one reference and invalidate a previously finalized hash.
    pub fn add_ref(&mut self, evidence: EvidenceRef) {
        self.refs.push(evidence);
        self.bundle_hash.clear();
    }

    /// Compute the content hash for the current reference list.
    pub fn finalize(&mut self) {
        self.bundle_hash = Self::calculate_hash(&self.refs);
    }

    /// Return whether the bundle is internally consistent and finalized.
    #[must_use]
    pub fn verify(&self) -> bool {
        !self.task_id.is_empty()
            && self
                .refs
                .iter()
                .all(|evidence| evidence.task_id == self.task_id)
            && self.bundle_hash == Self::calculate_hash(&self.refs)
    }

    fn calculate_hash(refs: &[EvidenceRef]) -> String {
        let content_hashes = refs
            .iter()
            .map(|evidence| evidence.content_hash.as_str())
            .collect::<String>();
        crate::core::hasher::hash_str(&content_hashes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::evidence_ledger::{DataClassification, EvidenceKind};

    fn evidence(ref_id: &str, content_hash: &str) -> EvidenceRef {
        EvidenceRef {
            ref_id: ref_id.to_owned(),
            kind: EvidenceKind::ToolExecution,
            task_id: "task-1".to_owned(),
            content_hash: content_hash.to_owned(),
            classification: DataClassification::Public,
            retention_days: Some(30),
            created_at: "2026-08-09T12:00:00Z".to_owned(),
        }
    }

    #[test]
    fn finalize_and_verify_round_trip() {
        let mut bundle = EvidenceBundle::new("task-1".to_owned());
        bundle.add_ref(evidence("ref-1", "hash-1"));
        bundle.add_ref(evidence("ref-2", "hash-2"));
        assert!(!bundle.verify());
        bundle.finalize();
        assert!(bundle.verify());
    }

    #[test]
    fn changing_a_reference_invalidates_the_bundle() {
        let mut bundle = EvidenceBundle::new("task-1".to_owned());
        bundle.add_ref(evidence("ref-1", "hash-1"));
        bundle.finalize();
        bundle.refs[0].content_hash = "tampered".to_owned();
        assert!(!bundle.verify());
    }

    #[test]
    fn mismatched_task_id_is_not_verified() {
        let mut bundle = EvidenceBundle::new("task-1".to_owned());
        bundle.add_ref(evidence("ref-1", "hash-1"));
        bundle.refs[0].task_id = "task-2".to_owned();
        bundle.finalize();
        assert!(!bundle.verify());
    }
}
