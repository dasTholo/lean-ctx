use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use serde::Serialize;

use super::teacher_labeling::LabeledSample;

pub const TRAIN_SPLIT: f64 = 0.70;
pub const VALIDATION_SPLIT: f64 = 0.15;
pub const TEST_SPLIT: f64 = 0.15;

#[derive(Debug, Clone, PartialEq)]
pub struct TrainingConfig {
    pub train_split: f64,
    pub validation_split: f64,
    pub test_split: f64,
    pub shuffle_seed: u64,
    pub max_seq_len: usize,
    pub vocab_size: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            train_split: TRAIN_SPLIT,
            validation_split: VALIDATION_SPLIT,
            test_split: TEST_SPLIT,
            shuffle_seed: 0x5EED_5EED,
            max_seq_len: 96,
            vocab_size: 8192,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatasetSplit {
    pub train: Vec<LabeledSample>,
    pub validation: Vec<LabeledSample>,
    pub test: Vec<LabeledSample>,
}

impl DatasetSplit {
    pub fn total(&self) -> usize {
        self.train.len() + self.validation.len() + self.test.len()
    }
}

/// Deterministically shuffles samples by task ID before a 70/15/15 split.
pub fn split_dataset(samples: &[LabeledSample]) -> DatasetSplit {
    split_dataset_with_seed(samples, TrainingConfig::default().shuffle_seed)
}

pub fn split_dataset_with_seed(samples: &[LabeledSample], shuffle_seed: u64) -> DatasetSplit {
    let mut shuffled = samples.to_vec();
    shuffled.sort_by_cached_key(|sample| stable_shuffle_key(&sample.task_id, shuffle_seed));

    let train_end = samples.len() * 70 / 100;
    let validation_end = train_end + samples.len() * 15 / 100;
    DatasetSplit {
        train: shuffled[..train_end].to_vec(),
        validation: shuffled[train_end..validation_end].to_vec(),
        test: shuffled[validation_end..].to_vec(),
    }
}

fn stable_shuffle_key(task_id: &str, shuffle_seed: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&shuffle_seed.to_le_bytes());
    hasher.update(task_id.as_bytes());
    *hasher.finalize().as_bytes()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AugmentationHint {
    pub name: &'static str,
    pub instruction: &'static str,
}

pub fn augmentation_hints() -> &'static [AugmentationHint] {
    const HINTS: [AugmentationHint; 4] = [
        AugmentationHint {
            name: "case_variation",
            instruction: "Vary capitalization without changing task intent.",
        },
        AugmentationHint {
            name: "imperative_rephrase",
            instruction: "Rephrase as a direct implementation request while preserving labels.",
        },
        AugmentationHint {
            name: "question_rephrase",
            instruction: "Rephrase as a developer question while preserving labels.",
        },
        AugmentationHint {
            name: "concise_rephrase",
            instruction: "Remove incidental wording without changing task intent or scope.",
        },
    ];
    &HINTS
}

#[derive(Debug, Serialize)]
struct FineTuningRecord<'a> {
    input: &'a str,
    output: &'a super::teacher_labeling::TeacherLabels,
    language: &'a str,
    confidence: f64,
}

pub fn export_fine_tuning_jsonl(samples: &[LabeledSample], output_path: &Path) -> io::Result<()> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    for sample in samples {
        serde_json::to_writer(
            &mut writer,
            &FineTuningRecord {
                input: &sample.query,
                output: &sample.teacher_labels,
                language: &sample.language,
                confidence: sample.confidence,
            },
        )
        .map_err(io::Error::other)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributionReport {
    pub total: usize,
    pub per_intent: HashMap<String, usize>,
    pub per_complexity: HashMap<String, usize>,
    pub per_language: HashMap<String, usize>,
}

pub fn validate_distribution(samples: &[LabeledSample]) -> DistributionReport {
    let mut report = DistributionReport {
        total: samples.len(),
        per_intent: HashMap::new(),
        per_complexity: HashMap::new(),
        per_language: HashMap::new(),
    };
    for sample in samples {
        *report
            .per_intent
            .entry(sample.teacher_labels.intent.clone())
            .or_default() += 1;
        *report
            .per_complexity
            .entry(sample.teacher_labels.complexity.clone())
            .or_default() += 1;
        *report
            .per_language
            .entry(sample.language.clone())
            .or_default() += 1;
    }
    report
}
