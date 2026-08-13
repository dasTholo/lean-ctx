//! Deterministic, local-only conversion signals for the free product.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SESSION_COUNT_THRESHOLD: usize = 5;
pub const DECISION_SESSION_COUNT_THRESHOLD: usize = 10;
pub const SAVINGS_USD_THRESHOLD: f64 = 10.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProTriggerReason {
    SessionCount,
    CumulativeSavings,
    MultiDevice,
}

/// The local fact that made a Pro conversion message relevant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProTriggerKind {
    SessionCount,
    DecisionCount,
    CumulativeSavings,
    MultiDevice,
}

/// A concise, evidence-backed Pro message suitable for terminal or JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct ConversionMessage {
    pub trigger: ProTriggerKind,
    pub headline: String,
    pub detail: String,
    pub session_count: u64,
    pub evidence_value: String,
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

/// Generate the vision-specified conversion messages from local facts only.
#[must_use]
pub fn generate_conversion_messages(
    sessions: &[SessionSignal],
    proven_savings_usd: f64,
    decision_count: u64,
) -> Vec<ConversionMessage> {
    let session_count = u64::try_from(sessions.len()).unwrap_or(u64::MAX);
    let mut messages = Vec::new();

    if sessions.len() >= SESSION_COUNT_THRESHOLD {
        messages.push(ConversionMessage {
            trigger: ProTriggerKind::SessionCount,
            headline: format!("Your context now spans {session_count} sessions."),
            detail: "Pro syncs this across all your machines.".to_string(),
            session_count,
            evidence_value: format!("{session_count} sessions"),
        });
    }

    if sessions.len() >= DECISION_SESSION_COUNT_THRESHOLD {
        messages.push(ConversionMessage {
            trigger: ProTriggerKind::DecisionCount,
            headline: format!("You've made {decision_count} decisions this week."),
            detail: "Pro learns which models work best for YOUR tasks.".to_string(),
            session_count,
            evidence_value: format!("{decision_count} decisions"),
        });
    }

    if proven_savings_usd.is_finite() && proven_savings_usd > SAVINGS_USD_THRESHOLD {
        messages.push(ConversionMessage {
            trigger: ProTriggerKind::CumulativeSavings,
            headline: format!("You've saved ${proven_savings_usd:.2} this week with LeanCTX."),
            detail: "Pro tracks this across all devices.".to_string(),
            session_count,
            evidence_value: format!("${proven_savings_usd:.2}"),
        });
    }

    let device_count = sessions
        .iter()
        .flat_map(|session| session.agent_ids.iter().map(String::as_str))
        .collect::<BTreeSet<_>>()
        .len();
    if device_count > 1 {
        messages.push(ConversionMessage {
            trigger: ProTriggerKind::MultiDevice,
            headline: format!("We noticed you on {device_count} devices."),
            detail: "Pro keeps your context in sync.".to_string(),
            session_count,
            evidence_value: format!("{device_count} devices"),
        });
    }

    messages
}

/// Suppress all conversion messages after the user dismisses them.
#[must_use]
pub fn generate_visible_conversion_messages(
    sessions: &[SessionSignal],
    proven_savings_usd: f64,
    decision_count: u64,
    dismissed: bool,
) -> Vec<ConversionMessage> {
    if dismissed {
        Vec::new()
    } else {
        generate_conversion_messages(sessions, proven_savings_usd, decision_count)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct ConversionMessagePreferences {
    #[serde(default)]
    dismissed: bool,
}

fn preferences_path() -> Result<std::path::PathBuf, String> {
    Ok(crate::core::data_dir::lean_ctx_data_dir()?.join("pro_conversion_messages.json"))
}

/// Returns whether terminal conversion messages have been dismissed locally.
#[must_use]
pub fn conversion_messages_dismissed() -> bool {
    preferences_path()
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|json| serde_json::from_str::<ConversionMessagePreferences>(&json).ok())
        .is_some_and(|preferences| preferences.dismissed)
}

/// Persist the user's local conversion-message preference.
pub fn set_conversion_messages_dismissed(dismissed: bool) -> Result<(), String> {
    let path = preferences_path()?;
    let preferences = ConversionMessagePreferences { dismissed };
    let json = serde_json::to_string(&preferences).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| error.to_string())
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
        let events = evaluate(&[], 10.01);
        assert!(
            events
                .iter()
                .any(|event| event.reason == ProTriggerReason::CumulativeSavings)
        );
    }

    #[test]
    fn conversion_messages_do_not_fire_before_five_sessions() {
        let sessions = (0..4)
            .map(|n| signal(&format!("session-{n}"), "agent-a"))
            .collect::<Vec<_>>();

        assert!(generate_conversion_messages(&sessions, 0.0, 0).is_empty());
    }

    #[test]
    fn session_message_includes_session_count() {
        let sessions = (0..5)
            .map(|n| signal(&format!("session-{n}"), "agent-a"))
            .collect::<Vec<_>>();
        let message = generate_conversion_messages(&sessions, 0.0, 0)
            .into_iter()
            .find(|message| message.trigger == ProTriggerKind::SessionCount)
            .expect("session message");

        assert_eq!(message.headline, "Your context now spans 5 sessions.");
        assert_eq!(message.detail, "Pro syncs this across all your machines.");
        assert_eq!(message.session_count, 5);
    }

    #[test]
    fn decision_message_includes_weekly_decision_count() {
        let sessions = (0..10)
            .map(|n| signal(&format!("session-{n}"), "agent-a"))
            .collect::<Vec<_>>();
        let message = generate_conversion_messages(&sessions, 0.0, 47)
            .into_iter()
            .find(|message| message.trigger == ProTriggerKind::DecisionCount)
            .expect("decision message");

        assert_eq!(message.headline, "You've made 47 decisions this week.");
        assert_eq!(
            message.detail,
            "Pro learns which models work best for YOUR tasks."
        );
        assert_eq!(message.evidence_value, "47 decisions");
    }

    #[test]
    fn savings_message_includes_dollar_amount() {
        let message = generate_conversion_messages(&[], 42.5, 0)
            .into_iter()
            .find(|message| message.trigger == ProTriggerKind::CumulativeSavings)
            .expect("savings message");

        assert_eq!(
            message.headline,
            "You've saved $42.50 this week with LeanCTX."
        );
        assert_eq!(message.detail, "Pro tracks this across all devices.");
        assert_eq!(message.evidence_value, "$42.50");
    }

    #[test]
    fn savings_message_requires_more_than_ten_dollars() {
        assert!(
            generate_conversion_messages(&[], 10.0, 0)
                .iter()
                .all(|message| message.trigger != ProTriggerKind::CumulativeSavings)
        );
    }

    #[test]
    fn multi_device_message_includes_device_count() {
        let sessions = vec![
            signal("session-a", "agent-a"),
            signal("session-b", "agent-b"),
        ];
        let message = generate_conversion_messages(&sessions, 0.0, 0)
            .into_iter()
            .find(|message| message.trigger == ProTriggerKind::MultiDevice)
            .expect("multi-device message");

        assert_eq!(message.headline, "We noticed you on 2 devices.");
        assert_eq!(message.detail, "Pro keeps your context in sync.");
        assert_eq!(message.evidence_value, "2 devices");
    }

    #[test]
    fn dismissed_messages_are_not_repeated() {
        let _data_dir = crate::core::data_dir::isolated_data_dir();
        let sessions = (0..5)
            .map(|n| signal(&format!("session-{n}"), "agent-a"))
            .collect::<Vec<_>>();

        set_conversion_messages_dismissed(true).expect("persist dismissal");
        assert!(conversion_messages_dismissed());
        assert!(
            generate_visible_conversion_messages(
                &sessions,
                0.0,
                0,
                conversion_messages_dismissed()
            )
            .is_empty()
        );
    }

    #[test]
    fn conversion_messages_serialize_as_valid_json() {
        let messages = generate_conversion_messages(&[], 42.5, 0);
        let json = serde_json::to_string(&messages).expect("serialize messages");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

        assert_eq!(value[0]["trigger"], "cumulative_savings");
        assert_eq!(value[0]["evidence_value"], "$42.50");
    }

    #[test]
    fn multiple_conversion_messages_can_fire_together() {
        let mut sessions = (0..10)
            .map(|n| signal(&format!("session-{n}"), "agent-a"))
            .collect::<Vec<_>>();
        sessions[1].agent_ids.insert("agent-b".to_string());

        let messages = generate_conversion_messages(&sessions, 42.5, 47);
        assert_eq!(messages.len(), 4);
        assert!(
            messages
                .iter()
                .any(|message| message.trigger == ProTriggerKind::SessionCount)
        );
        assert!(
            messages
                .iter()
                .any(|message| message.trigger == ProTriggerKind::DecisionCount)
        );
        assert!(
            messages
                .iter()
                .any(|message| message.trigger == ProTriggerKind::CumulativeSavings)
        );
        assert!(
            messages
                .iter()
                .any(|message| message.trigger == ProTriggerKind::MultiDevice)
        );
    }
}
