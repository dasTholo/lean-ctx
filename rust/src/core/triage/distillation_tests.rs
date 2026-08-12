use std::path::PathBuf;

use super::distillation::{
    LabeledSample, TeacherConfig, TeacherLabels, TinyModelConfig, model_pack_path,
    prepare_teacher_batch, split_dataset, validate_distribution,
};

fn sample(id: &str, intent: &str, complexity: &str, language: &str) -> LabeledSample {
    LabeledSample {
        task_id: id.into(),
        query: "task".into(),
        language: language.into(),
        confidence: 0.9,
        teacher_labels: TeacherLabels {
            intent: intent.into(),
            complexity: complexity.into(),
            scope: "single_file".into(),
            reasoning_need: "low".into(),
            risk: "low".into(),
        },
    }
}

#[test]
fn test_teacher_batch_prepares_prompts() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/triage_gold_set.jsonl");
    let prompts = prepare_teacher_batch(&path, &TeacherConfig::default());
    assert_eq!(prompts.len(), 500);
    assert!(
        prompts[0]
            .prompt
            .contains("Respond with JSON: intent, complexity, scope, reasoning_need, risk")
    );
}

#[test]
fn test_split_deterministic() {
    let samples = vec![
        sample("b", "generate", "mechanical", "en"),
        sample("a", "review", "complex", "de"),
    ];
    assert_eq!(split_dataset(&samples), split_dataset(&samples));
    assert_eq!(split_dataset(&samples).0[0].task_id, "a");
}

#[test]
fn test_distribution_report() {
    let report = validate_distribution(&[
        sample("a", "generate", "mechanical", "en"),
        sample("b", "generate", "complex", "de"),
    ]);
    assert_eq!(
        (
            report.total,
            report.per_intent["generate"],
            report.per_complexity["mechanical"],
            report.per_language["de"]
        ),
        (2, 2, 1, 1)
    );
}

#[test]
fn test_model_config_defaults() {
    assert_eq!(
        (
            TinyModelConfig::default().layers,
            TinyModelConfig::default().hidden_size
        ),
        (2, 128)
    );
}

#[test]
fn test_model_pack_path() {
    assert!(model_pack_path().starts_with(dirs::home_dir().unwrap()));
}
