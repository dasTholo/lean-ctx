use super::baseline_recorder::{append_jsonl, read_jsonl};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreatmentCall {
    pub session_id: String,
    pub tool_name: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model: String,
    pub timestamp: String,
    pub raw_cost: u64,
    pub compressed_tokens: u64,
    pub savings_tokens: u64,
    pub compression_ratio: f64,
}

impl TreatmentCall {
    pub fn new(
        session_id: &str,
        tool_name: &str,
        input_tokens: u64,
        original_output_tokens: u64,
        savings_tokens: u64,
        model: &str,
    ) -> Self {
        let compressed_tokens = original_output_tokens.saturating_sub(savings_tokens);
        Self {
            session_id: session_id.to_owned(),
            tool_name: tool_name.to_owned(),
            input_tokens,
            output_tokens: compressed_tokens,
            model: model.to_owned(),
            timestamp: Utc::now().to_rfc3339(),
            raw_cost: crate::core::value_gate::cost_tracker::calculate_cost(
                input_tokens,
                compressed_tokens,
                0,
                model,
            ),
            compressed_tokens,
            savings_tokens: original_output_tokens.saturating_sub(compressed_tokens),
            compression_ratio: if original_output_tokens == 0 {
                1.0
            } else {
                compressed_tokens as f64 / original_output_tokens as f64
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreatmentRecorder {
    directory: PathBuf,
}

impl TreatmentRecorder {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }
    pub fn path(&self) -> PathBuf {
        self.directory.join("treatment.jsonl")
    }
    pub fn record(&self, call: &TreatmentCall) -> io::Result<()> {
        append_jsonl(&self.path(), call)
    }
    pub fn read(&self) -> io::Result<Vec<TreatmentCall>> {
        read_jsonl(&self.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treatment_recorder_includes_compression_data() {
        let dir = tempfile::tempdir().unwrap();
        let recorder = TreatmentRecorder::new(dir.path());
        recorder
            .record(&TreatmentCall::new("s1", "ctx_read", 10, 100, 60, "gpt-4o"))
            .unwrap();
        let entry = recorder.read().unwrap().pop().unwrap();
        assert_eq!(
            (
                entry.output_tokens,
                entry.compressed_tokens,
                entry.savings_tokens
            ),
            (40, 40, 60)
        );
        assert!((entry.compression_ratio - 0.4).abs() < f64::EPSILON);
    }
}
