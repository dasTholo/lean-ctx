//! Behavioral Verbosity Learning (F10) — adaptive compression recommendations.
//!
//! Learns optimal compression levels from agent behavioral signals.

pub mod recommender;
pub mod runtime;
pub mod signals;
pub mod transcript;

pub use recommender::VerbosityProfile;
pub use recommender::recommend_level;
pub use runtime::auto_apply_happened;
pub use runtime::recommended_compression;
pub use runtime::record_tool_call;
#[cfg(test)]
pub use signals::BehaviorSignal;
pub use signals::extract_signals;
pub use transcript::TranscriptEntry;
#[cfg(test)]
pub use transcript::analyze_transcript;
