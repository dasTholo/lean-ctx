//! Deterministic, local-only conversion signals for the free product.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SESSION_COUNT_THRESHOLD: usize = 5;
pub const SAVINGS_USD_THRESHOLD: f64 = 5.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProTriggerReason {
    SessionCount,
    CumulativeSavings,
    MultiDevice,
}

/// A conversion signal with local, inspectable evidence. Nothing is sent remotely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProTriggerEvent {
    pub reason: ProTriggerReason,
    pub evidence: String,
}

#[derive(Debug, Clone, Default)]
pub struct SessionSignal {
    pub id: String,
    pub agent_ids: BTreeSet<String>,
}

/// Evaluate the three Class A triggers from caller-provided local facts.
#[must_use]
pub fn evaluate(sessions: &[SessionSignal], proven_savings_usd: f64) -> Vec<ProTriggerEvent> {
    let mut events = Vec::new();
    if sessions.len() >= SESSION_COUNT_THRESHOLD {
        events.push(ProTriggerEvent {
            reason: ProTriggerReason::SessionCount,
            evidence: format!("{} local sessions recorded", sessions.len()),
        });
    }
    if proven_savings_usd.is_finite() && proven_savings_usd > SAVINGS_USD_THRESHOLD {
        events.push(ProTriggerEvent {
            reason: ProTriggerReason::CumulativeSavings,
            evidence: format!("${proven_savings_usd:.2} proven local savings"),
        });
    }
    let agent_ids: BTreeSet<&str> = sessions
        .iter()
        .flat_map(|session| session.agent_ids.iter().map(String::as_str))
        .collect();
    if agent_ids.len() > 1 {
        events.push(ProTriggerEvent {
            reason: ProTriggerReason::MultiDevice,
            evidence: format!("{} distinct local agent IDs observed", agent_ids.len()),
        });
    }
    events
}

/// Reads persisted sessions and evaluates local signals, including this live session.
#[must_use]
pub fn check_local(current: SessionSignal) -> Vec<ProTriggerEvent> {
    let mut sessions = crate::core::session::SessionState::all_session_signals();
    if let Some(existing) = sessions.iter_mut().find(|session| session.id == current.id) {
        existing.agent_ids.extend(current.agent_ids);
    } else {
        sessions.push(current);
    }
    sessions.sort_by(|left, right| left.id.cmp(&right.id));
    let proven_savings_usd = if crate::core::savings_ledger::verify().valid {
        crate::core::savings_ledger::summary().saved_usd
    } else {
        0.0
    };
    evaluate(&sessions, proven_savings_usd)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(id: &str, agent: &str) -> SessionSignal {
        SessionSignal {
            id: id.to_string(),
            agent_ids: BTreeSet::from([agent.to_string()]),
        }
    }

    #[test]
    fn pro_trigger_fires_after_five_sessions() {
        let sessions = (0..5)
            .map(|n| signal(&format!("session-{n}"), "agent-a"))
            .collect::<Vec<_>>();
        let events = evaluate(&sessions, 0.0);
        assert!(
            events
                .iter()
                .any(|event| event.reason == ProTriggerReason::SessionCount)
        );
    }

    #[test]
    fn pro_trigger_fires_when_proven_savings_exceed_threshold() {
        let events = evaluate(&[], 5.01);
        assert!(
            events
                .iter()
                .any(|event| event.reason == ProTriggerReason::CumulativeSavings)
        );
    }
}
