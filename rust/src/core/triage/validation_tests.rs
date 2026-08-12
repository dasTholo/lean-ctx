use super::validation::{ValidationResult, validate_against_gold_set};

const GOLD_SET: &str = include_str!("../../../data/triage_gold_set.jsonl");

fn rows() -> Vec<serde_json::Value> {
    GOLD_SET
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn test_gold_set_parseable() {
    assert!(rows().len() >= 200);
}

#[test]
fn test_gold_set_schema() {
    for row in rows() {
        for field in ["id", "query", "language", "labels", "metadata"] {
            assert!(row.get(field).is_some(), "missing {field}");
        }
        for field in [
            "intent",
            "task_class",
            "complexity",
            "scope",
            "reasoning_need",
            "risk",
        ] {
            assert!(row["labels"].get(field).is_some(), "missing labels.{field}");
        }
    }
}

#[test]
fn test_gold_set_distribution() {
    let rows = rows();
    assert!(rows.iter().filter(|row| row["language"] == "en").count() >= 150);
    assert!(rows.iter().filter(|row| row["language"] == "de").count() >= 50);
}

#[test]
fn test_rules_baseline_accuracy() {
    let result: ValidationResult = validate_against_gold_set();
    assert!(
        result.accuracy >= 0.40,
        "baseline accuracy: {}",
        result.accuracy
    );
}
