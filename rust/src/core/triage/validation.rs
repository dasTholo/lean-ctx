//! Gold-set evaluation for the deterministic triage baseline.

use std::collections::HashMap;

use serde::Deserialize;

use super::{TaskAnalysisInput, TaskAnalyzer, rules::RuleTriageBackend};

const GOLD_SET: &str = include_str!("../../../data/triage_gold_set.jsonl");

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub total: usize,
    pub correct: usize,
    pub accuracy: f64,
    /// Per intent: `(true_positives, gold_examples)`; derive recall as `tp/gold`.
    pub per_intent: HashMap<String, (usize, usize)>,
}

#[derive(Deserialize)]
struct GoldTask {
    query: String,
    labels: GoldLabels,
}

#[derive(Deserialize)]
struct GoldLabels {
    intent: String,
}

pub fn validate_against_gold_set() -> ValidationResult {
    let tasks: Vec<GoldTask> = GOLD_SET
        .lines()
        .map(|line| serde_json::from_str(line).expect("gold set must be valid JSONL"))
        .collect();
    let mut gold_counts = HashMap::<String, usize>::new();
    let mut predicted_counts = HashMap::<String, usize>::new();
    let mut true_positives = HashMap::<String, usize>::new();
    for task in &tasks {
        *gold_counts.entry(task.labels.intent.clone()).or_default() += 1;
        let predicted = RuleTriageBackend
            .analyze(&TaskAnalysisInput {
                query: task.query.clone(),
                ..Default::default()
            })
            .expect("rule backend must classify gold tasks")
            .profile
            .intent;
        *predicted_counts.entry(predicted.clone()).or_default() += 1;
        if predicted == task.labels.intent {
            *true_positives.entry(predicted).or_default() += 1;
        }
    }
    let correct = true_positives.values().sum();
    let per_intent = gold_counts
        .iter()
        .map(|(intent, total)| {
            (
                intent.clone(),
                (*true_positives.get(intent).unwrap_or(&0), *total),
            )
        })
        .collect();
    for (intent, predicted) in predicted_counts {
        let tp = *true_positives.get(&intent).unwrap_or(&0) as f64;
        let precision = tp / predicted as f64;
        let recall = tp / *gold_counts.get(&intent).unwrap_or(&0) as f64;
        let _ = (precision, recall);
    }
    ValidationResult {
        total: tasks.len(),
        correct,
        accuracy: correct as f64 / tasks.len() as f64,
        per_intent,
    }
}
