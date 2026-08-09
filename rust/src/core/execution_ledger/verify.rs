//! Deterministic execution-ledger hashing and chain verification.

use super::event::ExecutionEvent;
use super::{ExecutionLedgerError, Result};

/// Hash-chain anchor shared with the Savings Ledger.
pub const GENESIS: &str = "genesis";

/// Hashes one event using the Savings Ledger's SHA-256 primitive.
///
/// The canonical event bytes are compact JSON emitted in declaration order.  The
/// preceding hash is also supplied as the hash primitive's domain input, matching
/// `SavingsEvent` chaining semantics.
pub fn hash_event(event: &ExecutionEvent) -> serde_json::Result<String> {
    let canonical = event.canonical_json()?;
    Ok(crate::core::savings_ledger::event::compute_hash(
        event.prev_hash(),
        &canonical,
    ))
}

/// Verifies an ordered in-memory event sequence.
pub fn verify_events(events: &[ExecutionEvent]) -> Result<bool> {
    let mut expected_previous = GENESIS.to_string();
    let mut expected_sequence = 1_u64;

    for (index, event) in events.iter().enumerate() {
        if event.sequence_number() != expected_sequence {
            return Ok(false);
        }
        if event.prev_hash() != expected_previous {
            return Ok(false);
        }

        let actual_hash = hash_event(event)?;
        if index > 0 && actual_hash.is_empty() {
            return Err(ExecutionLedgerError::InvalidChain(
                "event hash unexpectedly empty".to_owned(),
            ));
        }
        expected_previous = actual_hash;
        expected_sequence = expected_sequence.saturating_add(1);
    }

    Ok(true)
}
