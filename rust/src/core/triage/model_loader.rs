//! Optional local ONNX classifier for task triage.
//!
//! A missing model is normal: callers receive `None` and rule-based triage
//! continues without any download or runtime dependency.

use std::path::{Path, PathBuf};

use super::{ProfileHypothesis, TaskAnalysisInput};
#[cfg(feature = "onnx")]
use super::{
    TriageBackendLocal,
    profile::{TaskProfileLocal, TaskScopeLocal},
};

pub const TRIAGE_MODEL_FILE: &str = "triage-v1.onnx";
#[cfg(feature = "onnx")]
const MAX_SEQUENCE_LENGTH: usize = 64;

pub fn model_path(data_dir: &Path) -> PathBuf {
    data_dir.join("models").join(TRIAGE_MODEL_FILE)
}

pub fn is_managed_model_path(data_dir: &Path, candidate: &Path) -> bool {
    candidate == model_path(data_dir)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriageLogits {
    pub intent: f32,
    pub complexity: f32,
    pub scope: f32,
    pub reasoning_need: f32,
    pub risk: f32,
}

/// Minimal whitespace/subword-compatible BERT input encoder.
/// Unknown pieces use the standard BERT `[UNK]` ID; this avoids inventing IDs
/// for a model vocabulary that is not part of the managed model artifact.
#[cfg(feature = "onnx")]
#[derive(Debug, Clone, Copy)]
struct BertTinyTokenizer;

#[cfg(feature = "onnx")]
impl BertTinyTokenizer {
    fn encode(self, text: &str) -> (Vec<i64>, Vec<i64>, Vec<i64>) {
        const CLS: i64 = 101;
        const SEP: i64 = 102;
        const PAD: i64 = 0;
        const UNK: i64 = 100;
        let mut ids = vec![CLS];
        for token in text
            .split(|character: char| character.is_whitespace() || character.is_ascii_punctuation())
            .filter(|token| !token.is_empty())
        {
            if ids.len() + 1 >= MAX_SEQUENCE_LENGTH {
                break;
            }
            let id = match token.to_ascii_lowercase().as_str() {
                "fix" => 8081,
                "bug" => 11829,
                "test" => 3231,
                "refactor" => 10788,
                "config" => 6149,
                "deploy" => 21296,
                "review" => 3319,
                "debug" => 8567,
                _ => UNK,
            };
            ids.push(id);
        }
        ids.push(SEP);
        let mut mask = vec![1; ids.len()];
        let mut types = vec![0; ids.len()];
        ids.resize(MAX_SEQUENCE_LENGTH, PAD);
        mask.resize(MAX_SEQUENCE_LENGTH, 0);
        types.resize(MAX_SEQUENCE_LENGTH, 0);
        (ids, mask, types)
    }
}

#[derive(Debug)]
pub struct ModelLoader {
    model_path: PathBuf,
    #[cfg(feature = "onnx")]
    session: std::sync::Mutex<ort::session::Session>,
    #[cfg(feature = "onnx")]
    input_names: Vec<String>,
    #[cfg(feature = "onnx")]
    output_name: String,
}

impl ModelLoader {
    pub fn load(data_dir: &Path) -> anyhow::Result<Option<Self>> {
        Self::from_model_path(model_path(data_dir))
    }

    pub fn from_model_path(model_path: impl Into<PathBuf>) -> anyhow::Result<Option<Self>> {
        let model_path = model_path.into();
        if !model_path.is_file() {
            return Ok(None);
        }
        #[cfg(feature = "onnx")]
        {
            let execution_providers = crate::core::ort_execution_providers::execution_providers();
            crate::core::ort_environment::ensure_ort_env(&execution_providers)?;
            let session = ort::session::Session::builder()
                .map_err(|error| anyhow::anyhow!("ORT builder: {error}"))?
                .commit_from_file(&model_path)
                .map_err(|error| anyhow::anyhow!("ORT load triage model: {error}"))?;
            let input_names: Vec<_> = session
                .inputs()
                .iter()
                .map(|input| input.name().to_owned())
                .collect();
            let output_name = session
                .outputs()
                .first()
                .map(|output| output.name().to_owned())
                .ok_or_else(|| anyhow::anyhow!("triage model has no outputs"))?;
            if input_names.len() < 2 {
                anyhow::bail!("triage model has an unsupported graph signature");
            }
            Ok(Some(Self {
                model_path,
                session: std::sync::Mutex::new(session),
                input_names,
                output_name,
            }))
        }
        #[cfg(not(feature = "onnx"))]
        {
            let _ = model_path;
            Ok(None)
        }
    }

    pub fn path(&self) -> &Path {
        &self.model_path
    }

    pub fn infer(&self, input: &TaskAnalysisInput) -> anyhow::Result<ProfileHypothesis> {
        #[cfg(feature = "onnx")]
        {
            let text = if input.query.is_empty() {
                input.session_context.as_deref().unwrap_or_default()
            } else {
                &input.query
            };
            let (ids, mask, types) = BertTinyTokenizer.encode(text);
            let ids = ort::value::Tensor::from_array(ndarray::Array2::from_shape_vec(
                (1, MAX_SEQUENCE_LENGTH),
                ids,
            )?)?;
            let mask = ort::value::Tensor::from_array(ndarray::Array2::from_shape_vec(
                (1, MAX_SEQUENCE_LENGTH),
                mask,
            )?)?;
            let types = ort::value::Tensor::from_array(ndarray::Array2::from_shape_vec(
                (1, MAX_SEQUENCE_LENGTH),
                types,
            )?)?;
            let mut session = self
                .session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let output = if self.input_names.len() >= 3 {
                session.run(ort::inputs![
                    self.input_names[0].as_str() => ids,
                    self.input_names[1].as_str() => mask,
                    self.input_names[2].as_str() => types,
                ])?
            } else {
                session.run(ort::inputs![
                    self.input_names[0].as_str() => ids,
                    self.input_names[1].as_str() => mask,
                ])?
            };
            let (_, values) = output[self.output_name.as_str()].try_extract_tensor::<f32>()?;
            let logits = TriageLogits {
                intent: *values
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("triage model returned no logits"))?,
                complexity: *values
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("triage model returned fewer than 5 logits"))?,
                scope: *values
                    .get(2)
                    .ok_or_else(|| anyhow::anyhow!("triage model returned fewer than 5 logits"))?,
                reasoning_need: *values
                    .get(3)
                    .ok_or_else(|| anyhow::anyhow!("triage model returned fewer than 5 logits"))?,
                risk: *values
                    .get(4)
                    .ok_or_else(|| anyhow::anyhow!("triage model returned fewer than 5 logits"))?,
            };
            Ok(hypothesis_from_logits(logits))
        }
        #[cfg(not(feature = "onnx"))]
        {
            let _ = input;
            anyhow::bail!("triage ONNX support is disabled")
        }
    }
}

#[cfg(feature = "onnx")]
fn hypothesis_from_logits(logits: TriageLogits) -> ProfileHypothesis {
    let confidence = probability(logits.intent).min(probability(logits.complexity));
    let scope = probability(logits.scope);
    let profile = TaskProfileLocal {
        task_class: "coding".into(),
        intent: if logits.intent >= 0.0 {
            "coding_change"
        } else {
            "explore"
        }
        .into(),
        complexity: if logits.complexity > 0.5 {
            "high"
        } else if logits.complexity > -0.5 {
            "medium"
        } else {
            "low"
        }
        .into(),
        scope: if scope > 0.8 {
            TaskScopeLocal::CrossProject
        } else if scope > 0.6 {
            TaskScopeLocal::CrossModule
        } else if scope > 0.4 {
            TaskScopeLocal::MultiFile
        } else {
            TaskScopeLocal::SingleFile
        },
        context_need_milli: (scope * 1000.0).round() as u16,
        reasoning_need_milli: (probability(logits.reasoning_need) * 1000.0).round() as u16,
        risk_signal_milli: (probability(logits.risk) * 1000.0).round() as u16,
        confidence_milli: (confidence * 1000.0).round() as u16,
    };
    ProfileHypothesis {
        confidence_milli: profile.confidence_milli,
        profile,
        backend: TriageBackendLocal::Semantic,
    }
}

#[cfg(feature = "onnx")]
fn probability(logit: f32) -> f32 {
    if logit.is_finite() {
        1.0 / (1.0 + (-logit).exp())
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_model_returns_none() {
        let data = tempfile::tempdir().unwrap();
        assert!(ModelLoader::load(data.path()).unwrap().is_none());
    }

    #[test]
    fn managed_path_cannot_escape_data_dir() {
        let data = tempfile::tempdir().unwrap();
        assert!(is_managed_model_path(data.path(), &model_path(data.path())));
        assert!(!is_managed_model_path(
            data.path(),
            &data.path().join("models/../other.onnx")
        ));
    }
}
