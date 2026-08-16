//! Compression leaderboard (OSS stub).
//!
//! Enterprise provides fleet-wide competitive rankings. OSS returns None.

/// Returns a rank header value if enough data is available (OSS: always None).
pub fn rank_header_if_due() -> Option<String> {
    None
}

/// Computes the current session's rank (OSS: placeholder).
pub fn compute_current_rank() -> RankInfo {
    RankInfo::default()
}

/// Formats the rank into a human-readable message.
pub fn format_rank_message(_rank: &RankInfo) -> String {
    String::new()
}

/// Rank metadata for the current session.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct RankInfo {
    pub percentile: Option<f64>,
    pub rank: Option<u32>,
    pub total_participants: Option<u32>,
}
