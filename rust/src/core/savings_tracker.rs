//! Per-session compression accounting, persisted when the tracker is dropped.

use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Write,
    panic::{AssertUnwindSafe, catch_unwind},
};

fn tracker() -> &'static std::sync::Mutex<SessionSavingsTracker> {
    static TRACKER: std::sync::OnceLock<std::sync::Mutex<SessionSavingsTracker>> =
        std::sync::OnceLock::new();
    TRACKER.get_or_init(|| std::sync::Mutex::new(SessionSavingsTracker::default()))
}

pub fn record_compression(raw_tokens: u64, compressed_tokens: u64, tool: &str) {
    record_best_effort(tracker(), raw_tokens, compressed_tokens, tool);
}

pub fn session_summary() -> SessionSavings {
    tracker().try_lock().map_or_else(
        |_| SessionSavings::default(),
        |tracker| tracker.session_summary(),
    )
}

/// Persist the process-global tracker at a known session boundary.
///
/// Accounting is best-effort: lock, serialization, and filesystem errors must
/// never delay or prevent the server from shutting down.
pub fn persist_session_summary() {
    if let Ok(tracker) = tracker().try_lock() {
        let _ = catch_unwind(AssertUnwindSafe(|| tracker.persist()));
    }
}

fn record_best_effort(
    tracker: &std::sync::Mutex<SessionSavingsTracker>,
    raw_tokens: u64,
    compressed_tokens: u64,
    tool: &str,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Ok(mut tracker) = tracker.try_lock() {
            tracker.record_compression(raw_tokens, compressed_tokens, tool);
        }
    }));
}

#[rustfmt::skip]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
/// Summarizes compression savings accumulated during a session.
pub struct SessionSavings {
    pub total_raw: u64,
    pub total_compressed: u64,
    pub savings_tokens: u64,
    pub savings_percent: f64,
    pub tool_breakdown: Vec<(String, u64)>,
}

#[rustfmt::skip]
#[derive(Debug, Default)]
/// Accumulates and persists per-session compression savings.
pub struct SessionSavingsTracker {
    raw: u64,
    compressed: u64,
    tools: BTreeMap<String, u64>,
}

#[rustfmt::skip]
impl SessionSavingsTracker {
    pub fn record_compression(&mut self, raw_tokens: u64, compressed_tokens: u64, tool: &str) {
        let saved = raw_tokens.saturating_sub(compressed_tokens);
        self.raw = self.raw.saturating_add(raw_tokens);
        self.compressed = self.compressed.saturating_add(compressed_tokens);
        let entry = self.tools.entry(tool.to_owned()).or_default();
        *entry = entry.saturating_add(saved);
    }

    pub fn session_summary(&self) -> SessionSavings {
        let savings_tokens = self.raw.saturating_sub(self.compressed);
        SessionSavings {
            total_raw: self.raw,
            total_compressed: self.compressed,
            savings_tokens,
            savings_percent: percent(savings_tokens, self.raw),
            tool_breakdown: self
                .tools
                .iter()
                .map(|(tool, saved)| (tool.clone(), *saved))
                .collect(),
        }
    }

    fn persist(&self) {
        if self.raw == 0 {
            return;
        }
        let Ok(dir) = crate::core::paths::state_dir() else {
            return;
        };
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("session_savings.jsonl"))
        else {
            return;
        };
        let Ok(summary) = serde_json::to_string(&self.session_summary()) else {
            return;
        };
        let _ = writeln!(file, "{summary}");
    }
}

#[rustfmt::skip]
impl Drop for SessionSavingsTracker {
    fn drop(&mut self) {
        let _ = catch_unwind(AssertUnwindSafe(|| self.persist()));
    }
}

#[rustfmt::skip] pub fn percent(numerator: u64, denominator: u64) -> f64 { if denominator == 0 { 0.0 } else { numerator as f64 * 100.0 / denominator as f64 } }

#[cfg(test)] #[rustfmt::skip] mod tests { use super::*;

    #[test] fn test_session_tracker_accumulates() {
        let mut tracker = SessionSavingsTracker::default();
        tracker.record_compression(1_000, 600, "ctx_read");
        tracker.record_compression(500, 400, "ctx_read");
        tracker.record_compression(100, 50, "ctx_search");
        let savings = tracker.session_summary();
        assert_eq!((savings.total_raw, savings.total_compressed, savings.savings_tokens), (1_600, 1_050, 550));
        assert_eq!(savings.tool_breakdown, vec![("ctx_read".into(), 500), ("ctx_search".into(), 50)]);
    }

    #[test]
    fn test_tracker_error_ignored() {
        let tracker = std::sync::Mutex::new(SessionSavingsTracker::default());
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = tracker.lock().expect("fresh test mutex");
            panic!("poison tracker");
        }));

        let result = catch_unwind(AssertUnwindSafe(|| {
            record_best_effort(&tracker, 100, 25, "ctx_read");
        }));

        assert!(result.is_ok());
    }
}
