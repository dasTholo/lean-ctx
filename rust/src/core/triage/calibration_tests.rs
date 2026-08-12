use super::calibration::{
    PredictionResult, calibration_curve, format_calibration_report, run_calibration_benchmark,
};

fn prediction(confidence_milli: u16, correct: bool) -> PredictionResult {
    PredictionResult {
        predicted_intent: "test".into(),
        gold_intent: "test".into(),
        confidence_milli,
        correct,
    }
}

#[test]
fn test_calibration_buckets() {
    let curve = calibration_curve(&[prediction(0, false), prediction(1000, true)]);
    assert_eq!(curve.buckets.len(), 5);
}

#[test]
fn test_perfect_calibration() {
    assert_eq!(
        calibration_curve(&[prediction(1000, true)]).brier_score,
        0.0
    );
}

#[test]
fn test_brier_score_bounded() {
    let curve = calibration_curve(&[prediction(0, true), prediction(1000, false)]);
    assert!((0.0..=1.0).contains(&curve.brier_score));
}

#[test]
fn test_benchmark_runs() {
    assert!(run_calibration_benchmark().total_predictions > 0);
}

#[test]
fn test_report_has_table() {
    assert!(format_calibration_report(&run_calibration_benchmark()).contains("Confidence Range"));
}
