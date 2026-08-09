//! Deterministic, frozen circuit-breaker reference behavior.
//!
//! This implementation isolates a task class/provider pair after repeated
//! failures or quality regressions. It does not learn, rank candidates, or
//! aggregate customer data; production intelligence remains Class D in
//! `lean-ctx-enterprise`.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// Static thresholds for the reference circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub half_open_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(30),
            half_open_requests: 1,
        }
    }
}

impl CircuitBreakerConfig {
    /// Validate thresholds needed for a terminating state machine.
    pub fn validate(&self) -> Result<(), String> {
        if self.failure_threshold == 0 {
            return Err("failure_threshold must be greater than zero".to_owned());
        }
        if self.recovery_timeout.is_zero() {
            return Err("recovery_timeout must be greater than zero".to_owned());
        }
        if self.half_open_requests == 0 {
            return Err("half_open_requests must be greater than zero".to_owned());
        }
        Ok(())
    }
}

/// Operational state for one task-class/provider circuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Result of applying a circuit breaker to a candidate route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CircuitBreakerDecision {
    pub allowed: bool,
    pub selected_model: String,
    pub selected_provider: String,
    pub used_fallback: bool,
}

/// Alias using routing terminology for callers that prefer it.
pub type CircuitBreakerRoute = CircuitBreakerDecision;

#[derive(Debug, Clone)]
struct CircuitEntry {
    state: CircuitBreakerState,
    failures: u32,
    half_open_in_flight: u32,
    half_open_successes: u32,
    opened_at: Option<Instant>,
}

impl Default for CircuitEntry {
    fn default() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failures: 0,
            half_open_in_flight: 0,
            half_open_successes: 0,
            opened_at: None,
        }
    }
}

/// Per-task-class and per-provider deterministic circuit-breaker manager.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    entries: HashMap<(String, String), CircuitEntry>,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

impl CircuitBreaker {
    /// Create a breaker. Call [`Self::try_new`] when invalid configuration
    /// should be rejected at construction time.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            entries: HashMap::new(),
        }
    }

    /// Create a breaker after validating its static configuration.
    pub fn try_new(config: CircuitBreakerConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self::new(config))
    }

    /// Return the immutable configuration used by this breaker.
    pub fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }

    /// Return the state for a pair; unseen pairs start closed.
    pub fn state(&self, task_class: &str, provider: &str) -> CircuitBreakerState {
        self.entries
            .get(&key(task_class, provider))
            .map_or(CircuitBreakerState::Closed, |entry| entry.state)
    }

    /// Return the number of recorded consecutive failures for a pair.
    pub fn failure_count(&self, task_class: &str, provider: &str) -> u32 {
        self.entries
            .get(&key(task_class, provider))
            .map_or(0, |entry| entry.failures)
    }

    /// Test whether a candidate request is allowed using the current clock.
    pub fn allow_request(&mut self, task_class: &str, provider: &str) -> bool {
        self.allow_request_at(task_class, provider, Instant::now())
    }

    /// Test whether a candidate request is allowed at a supplied instant.
    pub fn allow_request_at(&mut self, task_class: &str, provider: &str, now: Instant) -> bool {
        let recovery_timeout = self.config.recovery_timeout;
        let half_open_requests = self.config.half_open_requests;
        let entry = self.entries.entry(key(task_class, provider)).or_default();
        match entry.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                let recovered = entry.opened_at.is_some_and(|opened_at| {
                    now >= opened_at && now.duration_since(opened_at) >= recovery_timeout
                });
                if recovered {
                    entry.state = CircuitBreakerState::HalfOpen;
                    entry.half_open_in_flight = 0;
                    entry.half_open_successes = 0;
                    entry.opened_at = None;
                    reserve_half_open_request(entry, half_open_requests)
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => reserve_half_open_request(entry, half_open_requests),
        }
    }

    /// Record a successful candidate request using the current clock.
    pub fn record_success(&mut self, task_class: &str, provider: &str) {
        self.record_success_at(task_class, provider, Instant::now());
    }

    /// Record a successful candidate request at a supplied instant.
    pub fn record_success_at(&mut self, task_class: &str, provider: &str, _now: Instant) {
        let half_open_requests = self.config.half_open_requests;
        let entry = self.entries.entry(key(task_class, provider)).or_default();
        match entry.state {
            CircuitBreakerState::Closed => entry.failures = 0,
            CircuitBreakerState::Open => {}
            CircuitBreakerState::HalfOpen => {
                entry.half_open_in_flight = entry.half_open_in_flight.saturating_sub(1);
                entry.half_open_successes = entry.half_open_successes.saturating_add(1);
                if entry.half_open_successes >= half_open_requests {
                    close(entry);
                }
            }
        }
    }

    /// Record a failed request using the current clock.
    pub fn record_failure(&mut self, task_class: &str, provider: &str) {
        self.record_failure_at(task_class, provider, Instant::now());
    }

    /// Record a failed request at a supplied instant.
    pub fn record_failure_at(&mut self, task_class: &str, provider: &str, now: Instant) {
        let failure_threshold = self.config.failure_threshold;
        let entry = self.entries.entry(key(task_class, provider)).or_default();
        match entry.state {
            CircuitBreakerState::Closed => {
                entry.failures = entry.failures.saturating_add(1);
                if entry.failures >= failure_threshold {
                    open(entry, now);
                }
            }
            CircuitBreakerState::Open => {}
            CircuitBreakerState::HalfOpen => open(entry, now),
        }
    }

    /// Quality regression is a circuit failure in the frozen reference model.
    pub fn record_quality_regression(&mut self, task_class: &str, provider: &str) {
        self.record_failure(task_class, provider);
    }

    /// Record a quality regression at a supplied instant, useful for
    /// deterministic callers and state-machine verification.
    pub fn record_quality_regression_at(&mut self, task_class: &str, provider: &str, now: Instant) {
        self.record_failure_at(task_class, provider, now);
    }

    /// Record either a successful quality outcome or a quality regression.
    pub fn record_quality_outcome(
        &mut self,
        task_class: &str,
        provider: &str,
        quality_regressed: bool,
    ) {
        if quality_regressed {
            self.record_quality_regression(task_class, provider);
        } else {
            self.record_success(task_class, provider);
        }
    }

    /// Reset one pair to the initial closed state.
    pub fn reset(&mut self, task_class: &str, provider: &str) {
        self.entries.remove(&key(task_class, provider));
    }

    /// Select the candidate when its circuit is available, otherwise use the
    /// supplied baseline model/provider. The baseline is always allowed.
    pub fn route(
        &mut self,
        task_class: &str,
        candidate_provider: &str,
        candidate_model: &str,
        baseline_provider: &str,
        baseline_model: &str,
    ) -> CircuitBreakerDecision {
        self.route_at(
            task_class,
            candidate_provider,
            candidate_model,
            baseline_provider,
            baseline_model,
            Instant::now(),
        )
    }

    /// Deterministic route selection at a supplied instant, useful for tests.
    pub fn route_at(
        &mut self,
        task_class: &str,
        candidate_provider: &str,
        candidate_model: &str,
        baseline_provider: &str,
        baseline_model: &str,
        now: Instant,
    ) -> CircuitBreakerDecision {
        if self.allow_request_at(task_class, candidate_provider, now) {
            CircuitBreakerDecision {
                allowed: true,
                selected_model: candidate_model.to_owned(),
                selected_provider: candidate_provider.to_owned(),
                used_fallback: false,
            }
        } else {
            CircuitBreakerDecision {
                allowed: true,
                selected_model: baseline_model.to_owned(),
                selected_provider: baseline_provider.to_owned(),
                used_fallback: true,
            }
        }
    }
}

fn key(task_class: &str, provider: &str) -> (String, String) {
    (task_class.to_owned(), provider.to_owned())
}

fn reserve_half_open_request(entry: &mut CircuitEntry, limit: u32) -> bool {
    if entry.half_open_in_flight < limit {
        entry.half_open_in_flight += 1;
        true
    } else {
        false
    }
}

fn open(entry: &mut CircuitEntry, now: Instant) {
    entry.state = CircuitBreakerState::Open;
    entry.failures = 0;
    entry.half_open_in_flight = 0;
    entry.half_open_successes = 0;
    entry.opened_at = Some(now);
}

fn close(entry: &mut CircuitEntry) {
    entry.state = CircuitBreakerState::Closed;
    entry.failures = 0;
    entry.half_open_in_flight = 0;
    entry.half_open_successes = 0;
    entry.opened_at = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_secs(10),
            half_open_requests: 2,
        }
    }

    #[test]
    fn config_and_state_round_trip() {
        let config = config();
        let state = CircuitBreakerState::HalfOpen;
        let config_json = serde_json::to_string(&config).expect("config serializes");
        let state_json = serde_json::to_string(&state).expect("state serializes");
        assert_eq!(
            serde_json::from_str::<CircuitBreakerConfig>(&config_json).unwrap(),
            config
        );
        assert_eq!(
            serde_json::from_str::<CircuitBreakerState>(&state_json).unwrap(),
            state
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn state_machine_trips_recovers_and_closes_after_half_open_successes() {
        let start = Instant::now();
        let mut breaker = CircuitBreaker::try_new(config()).expect("valid config");

        breaker.record_quality_regression_at("coding", "provider-a", start);
        assert_eq!(
            breaker.state("coding", "provider-a"),
            CircuitBreakerState::Closed
        );
        breaker.record_failure_at("coding", "provider-a", start);
        assert_eq!(
            breaker.state("coding", "provider-a"),
            CircuitBreakerState::Open
        );
        assert!(!breaker.allow_request_at("coding", "provider-a", start));

        let recovered_at = start + Duration::from_secs(10);
        assert!(breaker.allow_request_at("coding", "provider-a", recovered_at));
        assert!(breaker.allow_request_at("coding", "provider-a", recovered_at));
        assert!(!breaker.allow_request_at("coding", "provider-a", recovered_at));
        assert_eq!(
            breaker.state("coding", "provider-a"),
            CircuitBreakerState::HalfOpen
        );

        breaker.record_success_at("coding", "provider-a", recovered_at);
        assert_eq!(
            breaker.state("coding", "provider-a"),
            CircuitBreakerState::HalfOpen
        );
        breaker.record_success_at("coding", "provider-a", recovered_at);
        assert_eq!(
            breaker.state("coding", "provider-a"),
            CircuitBreakerState::Closed
        );
    }

    #[test]
    fn state_is_isolated_by_task_class_and_provider_and_falls_back() {
        let start = Instant::now();
        let mut breaker = CircuitBreaker::try_new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_secs(30),
            half_open_requests: 1,
        })
        .expect("valid config");
        breaker.record_quality_regression_at("coding", "provider-a", start);

        let fallback = breaker.route_at(
            "coding",
            "provider-a",
            "candidate",
            "provider-baseline",
            "baseline",
            start,
        );
        assert!(fallback.allowed);
        assert!(fallback.used_fallback);
        assert_eq!(fallback.selected_model, "baseline");

        let independent = breaker.route_at(
            "review",
            "provider-a",
            "candidate",
            "provider-baseline",
            "baseline",
            start,
        );
        assert!(!independent.used_fallback);
    }
}
