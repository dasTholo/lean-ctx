pub mod calibration;
#[cfg(test)]
mod calibration_tests;
pub mod confidence;
pub mod distillation;
#[cfg(test)]
mod distillation_tests;
pub mod fusion;
pub mod profile;
pub mod rules;
pub mod validation;
#[cfg(test)]
mod validation_tests;

use profile::TaskProfileLocal;
use std::{fmt, sync::Arc};

/// Analyzes task inputs into ranked profile hypotheses.
pub trait TaskAnalyzer: std::fmt::Debug + Send + Sync {
    fn analyze(&self, input: &TaskAnalysisInput) -> Result<ProfileHypothesis, TriageError>;
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Captures the signals used to classify a task.
pub struct TaskAnalysisInput {
    pub query: String,
    pub files_touched: Vec<String>,
    pub active_diagnostics: usize,
    pub session_context: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Describes one task-profile classification hypothesis.
pub struct ProfileHypothesis {
    pub profile: TaskProfileLocal,
    pub confidence_milli: u16,
    pub backend: TriageBackendLocal,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Identifies the backend that produced a triage hypothesis.
pub enum TriageBackendLocal {
    #[default]
    Rules,
    Semantic,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a task-triage failure.
pub enum TriageError {
    NoSignal,
    ModelUnavailable,
    InternalError(String),
}

#[derive(Debug, Clone)]
/// Orchestrates analyzers to select the strongest task profile.
pub struct TriageEngine {
    pub analyzers: Vec<Arc<dyn TaskAnalyzer>>,
}

impl fmt::Display for TriageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSignal => write!(f, "no triage signal available"),
            Self::ModelUnavailable => write!(f, "triage model unavailable"),
            Self::InternalError(error) => write!(f, "triage internal error: {error}"),
        }
    }
}

impl std::error::Error for TriageError {}

impl TriageEngine {
    pub fn new(analyzers: Vec<Box<dyn TaskAnalyzer>>) -> Self {
        Self {
            analyzers: analyzers.into_iter().map(Arc::from).collect(),
        }
    }

    pub fn with_rules() -> Self {
        Self::new(vec![Box::new(rules::RuleTriageBackend)])
    }

    #[allow(clippy::match_same_arms)]
    pub fn analyze(&self, input: &TaskAnalysisInput) -> Result<ProfileHypothesis, TriageError> {
        let mut best: Option<ProfileHypothesis> = None;
        let mut first_error = None;
        for analyzer in &self.analyzers {
            match analyzer.analyze(input) {
                Ok(candidate)
                    if best.as_ref().is_none_or(|current: &ProfileHypothesis| {
                        candidate.confidence_milli > current.confidence_milli
                    }) =>
                {
                    best = Some(candidate);
                }
                Ok(_) => {}
                Err(error) if first_error.is_none() => {
                    first_error = Some(error);
                }
                Err(_) => {}
            }
        }
        best.ok_or_else(|| first_error.unwrap_or(TriageError::NoSignal))
    }
}

impl Default for TriageEngine {
    fn default() -> Self {
        Self::with_rules()
    }
}
