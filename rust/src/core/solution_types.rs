use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SolutionDecisionKind {
    StdlibChosen,
    NativeUsed,
    Reuse,
    YagniSkip,
    OneLineSolution,
    DebtAccepted,
}

impl fmt::Display for SolutionDecisionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::StdlibChosen => "stdlib chosen",
            Self::NativeUsed => "native used",
            Self::Reuse => "reuse",
            Self::YagniSkip => "YAGNI skip",
            Self::OneLineSolution => "one-line solution",
            Self::DebtAccepted => "debt accepted",
        };
        f.write_str(label)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SolutionStatus {
    Accepted,
    Deferred,
    Resolved,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SolutionDecisionMeta {
    pub kind: SolutionDecisionKind,
    pub chosen: String,
    pub alternatives: Vec<String>,
    pub rationale: Option<String>,
    pub status: SolutionStatus,
    pub scope: Vec<String>,
    pub loc_impact: Option<i32>,
    pub upgrade_condition: Option<String>,
}
