//! Semantic triage hook for the Phase 5 ONNX classifier.

use std::path::{Path, PathBuf};

use super::{
    ProfileHypothesis, TaskAnalysisInput, TaskAnalyzer, TriageBackendLocal, TriageError,
    model_loader::ModelLoader, profile::TaskProfileLocal,
};

#[derive(Debug)]
pub struct SemanticAnalyzer {
    model_path: PathBuf,
    model: Option<ModelLoader>,
}

impl SemanticAnalyzer {
    /// Load the managed model from `<data_dir>/models/triage-v1.onnx`.
    pub fn from_data_dir(data_dir: &Path) -> Self {
        Self::from_model_path(super::model_loader::model_path(data_dir))
    }

    pub fn from_model_path(model_path: impl Into<PathBuf>) -> Self {
        let model_path = model_path.into();
        Self {
            model: ModelLoader::from_model_path(&model_path).ok().flatten(),
            model_path,
        }
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn model_available(&self) -> bool {
        self.model.is_some()
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

impl TaskAnalyzer for SemanticAnalyzer {
    fn analyze(&self, input: &TaskAnalysisInput) -> Result<ProfileHypothesis, TriageError> {
        Ok(self
            .model
            .as_ref()
            .and_then(|model| model.infer(input).ok())
            .unwrap_or_else(|| self.fallback()))
    }

    fn name(&self) -> &'static str {
        "semantic"
    }

    fn shadow_enabled(&self) -> bool {
        self.model_available()
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
        assert_eq!(result.confidence_milli, 0);
        assert_eq!(result.profile.intent, "needs_semantic_model");
    }

    #[test]
    fn data_dir_constructor_uses_managed_model_path() {
        let data = tempfile::tempdir().unwrap();
        let analyzer = SemanticAnalyzer::from_data_dir(data.path());
        assert_eq!(
            analyzer.model_path(),
            data.path().join("models/triage-v1.onnx")
        );
        assert!(!analyzer.model_available());
    }
}
