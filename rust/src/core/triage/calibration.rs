use super::{TaskAnalysisInput, TaskAnalyzer, rules::RuleTriageBackend};
use std::collections::BTreeSet;
const GOLD_SET: &str = include_str!("../../../data/triage_gold_set.jsonl");
const RANGES: [(u16, u16); 5] = [(0, 200), (200, 400), (400, 600), (600, 800), (800, 1000)];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredictionResult {
    pub predicted_intent: String,
    pub gold_intent: String,
    pub confidence_milli: u16,
    pub correct: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationBucket {
    pub confidence_range: (u16, u16),
    pub total: usize,
    pub correct: usize,
    pub accuracy: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationCurve {
    pub buckets: Vec<CalibrationBucket>,
    pub overall_accuracy: f64,
    pub brier_score: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationResult {
    pub curve: CalibrationCurve,
    pub total_predictions: usize,
    pub intents_evaluated: Vec<String>,
    pub well_calibrated: bool,
}
pub fn calibration_curve(predictions: &[PredictionResult]) -> CalibrationCurve {
    let mut buckets: Vec<_> = RANGES
        .into_iter()
        .map(|confidence_range| CalibrationBucket {
            confidence_range,
            total: 0,
            correct: 0,
            accuracy: 0.0,
        })
        .collect();
    let (mut squared_error, mut correct) = (0.0, 0);
    for prediction in predictions {
        let bucket = usize::from(prediction.confidence_milli / 200).min(4);
        buckets[bucket].total += 1;
        buckets[bucket].correct += usize::from(prediction.correct);
        correct += usize::from(prediction.correct);
        let confidence = f64::from(prediction.confidence_milli) / 1000.0;
        squared_error += (confidence - f64::from(prediction.correct as u8)).powi(2);
    }
    for bucket in &mut buckets {
        bucket.accuracy = if bucket.total == 0 {
            0.0
        } else {
            bucket.correct as f64 / bucket.total as f64
        };
    }
    let total = predictions.len() as f64;
    CalibrationCurve {
        buckets,
        overall_accuracy: if total == 0.0 {
            0.0
        } else {
            correct as f64 / total
        },
        brier_score: if total == 0.0 {
            0.0
        } else {
            squared_error / total
        },
    }
}
pub fn run_calibration_benchmark() -> CalibrationResult {
    let mut intents = BTreeSet::new();
    let predictions = GOLD_SET
        .lines()
        .map(|line| {
            let task: serde_json::Value =
                serde_json::from_str(line).expect("gold set must be valid JSONL");
            let query = task["query"].as_str().expect("gold task must have query");
            let gold_intent = task["labels"]["intent"]
                .as_str()
                .expect("gold task must have intent")
                .to_owned();
            intents.insert(gold_intent.clone());
            let hypothesis = RuleTriageBackend
                .analyze(&TaskAnalysisInput {
                    query: query.to_owned(),
                    ..Default::default()
                })
                .expect("rule backend must classify gold tasks");
            let predicted_intent = hypothesis.profile.intent;
            PredictionResult {
                correct: predicted_intent == gold_intent,
                gold_intent,
                predicted_intent,
                confidence_milli: hypothesis.confidence_milli,
            }
        })
        .collect::<Vec<_>>();
    let curve = calibration_curve(&predictions);
    let well_calibrated = curve.buckets.iter().all(|bucket| {
        bucket.total == 0
            || (bucket.accuracy - f64::from(bucket.confidence_range.0 + 100) / 1000.0).abs() < 0.20
    });
    CalibrationResult {
        curve,
        total_predictions: predictions.len(),
        intents_evaluated: intents.into_iter().collect(),
        well_calibrated,
    }
}
pub fn format_calibration_report(result: &CalibrationResult) -> String {
    let mut report = String::from(
        "| Confidence Range | Predictions | Correct | Accuracy | Expected |\n|---|---:|---:|---:|---:|\n",
    );
    for bucket in &result.curve.buckets {
        let (low, high) = bucket.confidence_range;
        report.push_str(&format!(
            "| {low}-{high} | {} | {} | {:.0}% | ~{}% |\n",
            bucket.total,
            bucket.correct,
            bucket.accuracy * 100.0,
            (low + high) / 20
        ));
    }
    report.push_str(&format!(
        "\nBrier Score: {:.4}\nWell Calibrated: {}",
        result.curve.brier_score,
        if result.well_calibrated { "YES" } else { "NO" }
    ));
    report
}
