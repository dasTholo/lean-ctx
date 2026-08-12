use std::collections::HashMap;

use super::teacher_labeling::LabeledSample;

#[derive(Debug, Clone, PartialEq)]
pub struct TrainingConfig {
    pub train_split: f64,
    pub max_seq_len: usize,
    pub vocab_size: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            train_split: 0.8,
            max_seq_len: 96,
            vocab_size: 8192,
        }
    }
}

pub fn split_dataset(samples: &[LabeledSample]) -> (Vec<LabeledSample>, Vec<LabeledSample>) {
    let mut ordered = samples.to_vec();
    ordered.sort_by(|left, right| left.task_id.cmp(&right.task_id));
    let split_at = ordered.len() * 4 / 5;
    (ordered[..split_at].to_vec(), ordered[split_at..].to_vec())
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
