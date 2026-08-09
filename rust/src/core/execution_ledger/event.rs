//! Task-aware execution events.
//!
//! The execution ledger deliberately keeps the event payloads small.  References to
//! envelopes, plans, receipts, and decisions are persisted instead of copying their
//! bodies into the ledger.  The chain fields are part of every variant so an event can
//! be serialized and verified without a second side table.

use serde::{Deserialize, Serialize};

use super::verify::hash_event;

pub use lean_ctx_protocol::ContextBalanceV1 as ContextBalance;
/// The tri-state acceptance value used by [`ExecutionEvent::OutcomeRecorded`].
///
/// This is an alias, rather than a second enum, so execution events use the same
/// wire representation as `AcceptedOutcomeV1`.
pub use lean_ctx_protocol::{AcceptanceState as TriState, ContextBalanceV1};

/// One append-only observation in the execution ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionEvent {
    TaskStarted {
        task_id: String,
        trace_id: String,
        envelope_ref: String,
        timestamp: String,
        sequence_number: u64,
        prev_hash: String,
    },
    PlanCreated {
        task_id: String,
        trace_id: String,
        plan_id: String,
        plan_ref: String,
        timestamp: String,
        sequence_number: u64,
        prev_hash: String,
    },
    ContextDelivered {
        task_id: String,
        trace_id: String,
        context_balance: ContextBalanceV1,
        timestamp: String,
        sequence_number: u64,
        prev_hash: String,
    },
    ModelInvoked {
        task_id: String,
        trace_id: String,
        plan_id: String,
        model: String,
        provider: String,
        tokens_in: u64,
        tokens_out: u64,
        latency_ms: u64,
        timestamp: String,
        sequence_number: u64,
        prev_hash: String,
    },
    ReceiptSigned {
        task_id: String,
        trace_id: String,
        receipt_id: String,
        receipt_hash: String,
        signature: String,
        timestamp: String,
        sequence_number: u64,
        prev_hash: String,
    },
    OutcomeRecorded {
        task_id: String,
        trace_id: String,
        outcome_id: String,
        accepted: TriState,
        timestamp: String,
        sequence_number: u64,
        prev_hash: String,
    },
    DecisionRecorded {
        task_id: String,
        trace_id: String,
        decision_id: String,
        kind: String,
        selected: String,
        timestamp: String,
        sequence_number: u64,
        prev_hash: String,
    },
}

impl ExecutionEvent {
    /// Returns the event's chain sequence number.
    #[must_use]
    pub const fn sequence_number(&self) -> u64 {
        match self {
            Self::TaskStarted {
                sequence_number, ..
            }
            | Self::PlanCreated {
                sequence_number, ..
            }
            | Self::ContextDelivered {
                sequence_number, ..
            }
            | Self::ModelInvoked {
                sequence_number, ..
            }
            | Self::ReceiptSigned {
                sequence_number, ..
            }
            | Self::OutcomeRecorded {
                sequence_number, ..
            }
            | Self::DecisionRecorded {
                sequence_number, ..
            } => *sequence_number,
        }
    }

    /// Returns the hash of the preceding event recorded on this event.
    #[must_use]
    pub fn prev_hash(&self) -> &str {
        match self {
            Self::TaskStarted { prev_hash, .. }
            | Self::PlanCreated { prev_hash, .. }
            | Self::ContextDelivered { prev_hash, .. }
            | Self::ModelInvoked { prev_hash, .. }
            | Self::ReceiptSigned { prev_hash, .. }
            | Self::OutcomeRecorded { prev_hash, .. }
            | Self::DecisionRecorded { prev_hash, .. } => prev_hash,
        }
    }

    /// Returns the task identity carried by this event.
    #[must_use]
    pub fn task_id(&self) -> &str {
        match self {
            Self::TaskStarted { task_id, .. }
            | Self::PlanCreated { task_id, .. }
            | Self::ContextDelivered { task_id, .. }
            | Self::ModelInvoked { task_id, .. }
            | Self::ReceiptSigned { task_id, .. }
            | Self::OutcomeRecorded { task_id, .. }
            | Self::DecisionRecorded { task_id, .. } => task_id,
        }
    }

    /// Returns the distributed trace identity carried by this event.
    #[must_use]
    pub fn trace_id(&self) -> &str {
        match self {
            Self::TaskStarted { trace_id, .. }
            | Self::PlanCreated { trace_id, .. }
            | Self::ContextDelivered { trace_id, .. }
            | Self::ModelInvoked { trace_id, .. }
            | Self::ReceiptSigned { trace_id, .. }
            | Self::OutcomeRecorded { trace_id, .. }
            | Self::DecisionRecorded { trace_id, .. } => trace_id,
        }
    }

    /// Returns the observation timestamp supplied by the caller.
    #[must_use]
    pub fn timestamp(&self) -> &str {
        match self {
            Self::TaskStarted { timestamp, .. }
            | Self::PlanCreated { timestamp, .. }
            | Self::ContextDelivered { timestamp, .. }
            | Self::ModelInvoked { timestamp, .. }
            | Self::ReceiptSigned { timestamp, .. }
            | Self::OutcomeRecorded { timestamp, .. }
            | Self::DecisionRecorded { timestamp, .. } => timestamp,
        }
    }

    /// Serializes the event in compact canonical JSON form.
    ///
    /// Struct and enum field order is fixed by the Rust declaration and
    /// `serde_json::to_string` emits no insignificant whitespace, giving the
    /// ledger a stable byte representation without relying on map iteration order.
    pub fn canonical_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Computes this event's chain hash using the Savings Ledger SHA-256 primitive.
    pub fn event_hash(&self) -> serde_json::Result<String> {
        hash_event(self)
    }

    /// Fills the chain metadata immediately before append.
    pub(crate) fn set_chain_fields(&mut self, sequence_number: u64, prev_hash: String) {
        match self {
            Self::TaskStarted {
                sequence_number: sequence,
                prev_hash: previous,
                ..
            }
            | Self::PlanCreated {
                sequence_number: sequence,
                prev_hash: previous,
                ..
            }
            | Self::ContextDelivered {
                sequence_number: sequence,
                prev_hash: previous,
                ..
            }
            | Self::ModelInvoked {
                sequence_number: sequence,
                prev_hash: previous,
                ..
            }
            | Self::ReceiptSigned {
                sequence_number: sequence,
                prev_hash: previous,
                ..
            }
            | Self::OutcomeRecorded {
                sequence_number: sequence,
                prev_hash: previous,
                ..
            }
            | Self::DecisionRecorded {
                sequence_number: sequence,
                prev_hash: previous,
                ..
            } => {
                *sequence = sequence_number;
                *previous = prev_hash;
            }
        }
    }
}
