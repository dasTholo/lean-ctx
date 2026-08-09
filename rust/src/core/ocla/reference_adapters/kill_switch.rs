//! Atomic, per-capability kill switch for reference adapters.

use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Runtime switch that disables one external capability without replacing the
/// adapter object or changing the native execution path.
pub struct KillSwitch {
    pub capability_id: String,
    pub enabled: AtomicBool,
    /// Initial/public configuration reason.  The current concurrent reason is
    /// available through [`KillSwitch::reason`].
    pub reason: Option<String>,
    current_reason: Mutex<Option<String>>,
}

impl KillSwitch {
    #[must_use]
    pub fn new(capability_id: impl Into<String>) -> Self {
        Self {
            capability_id: capability_id.into(),
            enabled: AtomicBool::new(false),
            reason: None,
            current_reason: Mutex::new(None),
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Activate before the next adapter invocation can pass the gate.
    pub fn activate(&self, reason: &str) {
        if let Ok(mut current) = self.current_reason.lock() {
            *current = Some(reason.trim().to_owned());
        }
        self.enabled.store(true, Ordering::SeqCst);
    }

    pub fn deactivate(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    /// Return the latest reason without exposing the mutex implementation.
    #[must_use]
    pub fn reason(&self) -> Option<String> {
        self.current_reason
            .lock()
            .ok()
            .and_then(|reason| reason.clone())
            .or_else(|| self.reason.clone())
    }
}

impl Default for KillSwitch {
    fn default() -> Self {
        Self::new("rtk-shell-output")
    }
}

#[cfg(test)]
mod tests {
    use super::KillSwitch;

    #[test]
    fn switch_is_atomic_and_keeps_reason() {
        let switch = KillSwitch::new("capability");
        assert!(!switch.is_active());
        switch.activate("maintenance");
        assert!(switch.is_active());
        assert_eq!(switch.reason().as_deref(), Some("maintenance"));
        switch.deactivate();
        assert!(!switch.is_active());
    }
}
