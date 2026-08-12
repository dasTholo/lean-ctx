//! Customer-facing A/B measurement for unoptimized baselines and lean-ctx treatment.

pub mod baseline_recorder;
pub mod comparison;
pub mod report;
pub mod treatment_recorder;

use std::{fs, io, path::PathBuf};

pub use baseline_recorder::{BaselineCall, BaselineRecorder};
pub use comparison::{ComparisonReport, QualityComparison};
pub use treatment_recorder::{TreatmentCall, TreatmentRecorder};

const MEASUREMENT_DIR: &str = "measurement";
const MODE_FILE: &str = "mode.txt";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementMode {
    Baseline,
    Treatment,
    Off,
}

impl MeasurementMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Treatment => "treatment",
            Self::Off => "off",
        }
    }

    fn parse(value: &str) -> Self {
        match value.trim() {
            "baseline" => Self::Baseline,
            "off" => Self::Off,
            _ => Self::Treatment,
        }
    }
}

/// Owns A/B persistence and provides the single tool-metrics integration point.
#[derive(Debug, Clone)]
pub struct MeasurementFramework {
    data_dir: PathBuf,
}

impl MeasurementFramework {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn from_data_dir() -> Self {
        Self::new(
            crate::core::data_dir::lean_ctx_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("lean-ctx")),
        )
    }

    pub fn directory(&self) -> PathBuf {
        self.data_dir.join(MEASUREMENT_DIR)
    }

    pub fn mode(&self) -> MeasurementMode {
        fs::read_to_string(self.directory().join(MODE_FILE))
            .map_or(MeasurementMode::Treatment, |value| {
                MeasurementMode::parse(&value)
            })
    }

    pub fn set_mode(&self, mode: MeasurementMode) -> io::Result<()> {
        fs::create_dir_all(self.directory())?;
        fs::write(
            self.directory().join(MODE_FILE),
            format!("{}\n", mode.as_str()),
        )
    }

    pub fn comparison(&self) -> io::Result<ComparisonReport> {
        comparison::compare_paths(
            &self.directory().join("baseline.jsonl"),
            &self.directory().join("treatment.jsonl"),
            &crate::core::value_gate::ValueGateStore::load_from_disk(),
        )
    }

    pub fn record_tool_call(
        &self,
        session_id: &str,
        tool_name: &str,
        input_tokens: u64,
        original_output_tokens: u64,
        savings_tokens: u64,
        model: &str,
    ) -> io::Result<()> {
        match self.mode() {
            MeasurementMode::Baseline => {
                BaselineRecorder::new(self.directory()).record(&BaselineCall::new(
                    session_id,
                    tool_name,
                    input_tokens,
                    original_output_tokens,
                    model,
                ))
            }
            MeasurementMode::Treatment => {
                TreatmentRecorder::new(self.directory()).record(&TreatmentCall::new(
                    session_id,
                    tool_name,
                    input_tokens,
                    original_output_tokens,
                    savings_tokens,
                    model,
                ))
            }
            MeasurementMode::Off => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_switching_works_correctly() {
        let dir = tempfile::tempdir().unwrap();
        let framework = MeasurementFramework::new(dir.path());
        assert_eq!(framework.mode(), MeasurementMode::Treatment);
        framework.set_mode(MeasurementMode::Baseline).unwrap();
        assert_eq!(framework.mode(), MeasurementMode::Baseline);
        framework.set_mode(MeasurementMode::Off).unwrap();
        assert_eq!(framework.mode(), MeasurementMode::Off);
    }
}
