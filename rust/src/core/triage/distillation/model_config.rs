use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct TinyModelConfig {
    pub layers: usize,
    pub hidden_size: usize,
    pub attention_heads: usize,
    pub max_seq_len: usize,
    pub vocab_size: usize,
    pub quantization: String,
    pub target_size_mb: f64,
    pub languages: Vec<String>,
}

impl Default for TinyModelConfig {
    fn default() -> Self {
        Self {
            layers: 2,
            hidden_size: 128,
            attention_heads: 2,
            max_seq_len: 96,
            vocab_size: 8192,
            quantization: "int8".into(),
            target_size_mb: 5.0,
            languages: ["en", "de", "fr", "es"].map(String::from).to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelManifest {
    pub version: String,
    pub architecture: String,
    pub training_samples: usize,
    pub accuracy_baseline: f64,
    pub created_at: String,
}

pub fn model_pack_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".lean-ctx/models/triage-tiny-v1")
}

pub fn manifest() -> ModelManifest {
    ModelManifest {
        version: "leanctx-triage-tiny-v1".into(),
        architecture: "transformer-2l-h128-a2-int8".into(),
        training_samples: 0,
        accuracy_baseline: 0.0,
        created_at: "pending-training".into(),
    }
}
