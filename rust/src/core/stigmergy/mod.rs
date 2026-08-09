//! Stigmergic Agent Coordination (F9) — ant-colony-inspired signals.
//!
//! Agents leave pheromone-like signals on files and symbols, enabling
//! implicit coordination between multiple agents in the same codebase.

pub mod pressure;
pub mod signal;

#[cfg(test)]
pub use pressure::PressureMap;
pub use signal::PheromoneSignal;
pub use signal::{SignalKind, deposit_signal};
#[cfg(test)]
pub use signal::{read_signals, reset_signals};
