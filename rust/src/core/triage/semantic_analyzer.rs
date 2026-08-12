//! Semantic triage hook for the Phase 5 ONNX classifier.

use std::path::{Path, PathBuf};

use super::{
    ProfileHypothesis, TaskAnalysisInput, TaskAnalyzer, TriageBackendLocal, TriageError,
    profile::TaskProfileLocal,
};

#[derive(Debug)]
pub struct SemanticAnalyzer {
    model_path: PathBuf,
    model_available: bool,
    #[cfg(feature = "neural")]
    _session: Option<ort::session::Session>,
}

impl SemanticAnalyzer {
    /// Configures the future ONNX model; inference remains disabled until Phase 5 integration.
    pub fn from_model_path(model_path: impl Into<PathBuf>) -> Self {
        let model_path = model_path.into();
        #[cfg(feature = "neural")]
        let session = load_onnx_session(&model_path).ok();
        #[cfg(feature = "neural")]
        let model_available = session.is_some();
        #[cfg(not(feature = "neural"))]
        let model_available = false;
        Self {
            model_path,
            model_available,
            #[cfg(feature = "neural")]
            _session: session,
        }
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn model_available(&self) -> bool {
        self.model_available
    }

    fn fallback(&self) -> ProfileHypothesis {
        let profile = TaskProfileLocal {
            task_class: "coding".into(),
            intent: "needs_semantic_model".into(),
            confidence_milli: 0,
            ..Default::default()
        };
        ProfileHypothesis {
            profile,
            confidence_milli: 0,
            backend: TriageBackendLocal::Semantic,
        }
    }
}

#[cfg(feature = "neural")]
fn load_onnx_session(model_path: &Path) -> anyhow::Result<ort::session::Session> {
    if !model_path.is_file() {
        anyhow::bail!("semantic model does not exist: {}", model_path.display());
    }
    let execution_providers = crate::core::ort_execution_providers::execution_providers();
    crate::core::ort_environment::ensure_ort_env(&execution_providers)?;
    ort::session::Session::builder()
        .map_err(|error| anyhow::anyhow!("ORT builder: {error}"))?
        .commit_from_file(model_path)
        .map_err(|error| anyhow::anyhow!("ORT load semantic model: {error}"))
}

impl TaskAnalyzer for SemanticAnalyzer {
    fn analyze(&self, _input: &TaskAnalysisInput) -> Result<ProfileHypothesis, TriageError> {
        // ONNX session/tokenizer wiring is intentionally deferred to Phase 5.
        Ok(self.fallback())
    }

    fn name(&self) -> &'static str {
        "semantic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_model_returns_low_confidence_fallback() {
        let analyzer = SemanticAnalyzer::from_model_path("missing-triage-model.onnx");
        let result = analyzer.analyze(&TaskAnalysisInput::default()).unwrap();

        assert!(!analyzer.model_available());
        assert_eq!(result.profile.intent, "needs_semantic_model");
        assert_eq!(result.confidence_milli, 0);
        assert_eq!(result.backend, TriageBackendLocal::Semantic);
    }
}
