//! Cross-agent stigmergy (OSS stub).
//!
//! Enterprise enables agents to leave "pheromone" signals for coordination.
//! OSS: no-op deposit, empty signal reads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Signal kind classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalKind {
    Exploration,
    Edit,
    Error,
    Success,
    Active,
    Complexity,
    ReviewNeeded,
    Issue,
    Completed,
}

/// A pheromone signal deposited by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PheromoneSignal {
    pub agent_id: String,
    pub kind: SignalKind,
    pub path: String,
    pub symbol: Option<String>,
    pub strength: f64,
    pub deposited_at: DateTime<Utc>,
    pub note: Option<String>,
}

/// Deposits a signal (OSS: no-op).
pub fn deposit_signal(_signal: PheromoneSignal) {}

/// Reads signals for a path (OSS: empty).
pub fn read_signals(_path: &str, _filter: Option<SignalKind>) -> Vec<PheromoneSignal> {
    Vec::new()
}

/// Resets all stored signals (OSS: no-op).
pub fn reset_signals() {}

/// Pressure field entry for a single path.
#[derive(Debug, Clone, Default)]
pub struct PressureField {
    pub total_strength: f64,
    pub agent_count: usize,
}

/// Hot file pressure map (OSS stub).
pub struct PressureMap {
    pub fields: HashMap<String, PressureField>,
}

impl PressureMap {
    /// Builds a pressure map from signals.
    pub fn from_signals(signals: &[PheromoneSignal]) -> Self {
        let mut fields: HashMap<String, (f64, HashSet<String>)> = HashMap::new();
        for signal in signals {
            let entry = fields.entry(signal.path.clone()).or_default();
            entry.0 += signal.strength;
            entry.1.insert(signal.agent_id.clone());
        }
        let fields = fields
            .into_iter()
            .map(|(path, (strength, agents))| {
                (
                    path,
                    PressureField {
                        total_strength: strength,
                        agent_count: agents.len(),
                    },
                )
            })
            .collect();
        Self { fields }
    }

    /// Returns pressure for a specific path.
    pub fn pressure_at(&self, path: &str) -> &PressureField {
        static DEFAULT: PressureField = PressureField {
            total_strength: 0.0,
            agent_count: 0,
        };
        self.fields.get(path).unwrap_or(&DEFAULT)
    }

    /// Returns hot files sorted by pressure.
    pub fn hot_files(&self, top_n: usize) -> Vec<(String, f64)> {
        let mut entries: Vec<_> = self
            .fields
            .iter()
            .map(|(path, field)| (path.clone(), field.total_strength))
            .collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(top_n);
        entries
    }
}
