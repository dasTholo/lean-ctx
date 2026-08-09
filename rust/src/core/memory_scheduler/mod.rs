//! FSRS-based Context Memory Scheduler (F5).
//!
//! Models knowledge retention using the Free Spaced Repetition Scheduler algorithm.
//! Determines which facts need re-injection based on their retrievability score.

pub mod decay;
pub mod fsrs;

pub use fsrs::MemoryState;
#[cfg(test)]
pub use fsrs::update_stability;
pub use fsrs::{initial_state, retrievability};
