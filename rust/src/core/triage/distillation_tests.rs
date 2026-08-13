use std::path::PathBuf;

use super::distillation::{
    LabeledSample, TeacherConfig, TeacherLabels, TinyModelConfig, augmentation_hints,
    model_pack_path, prepare_teacher_batch, split_dataset, validate_distribution,
    validate_teacher_labels,
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
    let config = TeacherConfig {
        batch_size: 8,
        ..Default::default()
    };
    let prompts = prepare_teacher_batch(&path, &config);
    assert_eq!(prompts.len(), 8);
    assert!(
        prompts[0]
            .prompt
            .contains("JSON containing exactly: intent, complexity, scope, reasoning_need, risk")
    );
    assert!(prompts[0].gold_labels.is_some());
}

#[test]
fn test_split_is_deterministic_and_uses_70_15_15_proportions() {
    let samples: Vec<_> = (0..100)
        .map(|index| sample(&format!("task-{index:03}"), "generate", "mechanical", "en"))
        .collect();
    let split = split_dataset(&samples);
    assert_eq!(split, split_dataset(&samples));
    assert_eq!(
        (split.train.len(), split.validation.len(), split.test.len()),
        (70, 15, 15)
    );
    assert_eq!(split.total(), samples.len());
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
    let p = model_pack_path();
    assert!(
        p.ends_with("models/triage-tiny-v1"),
        "model_pack_path must end with models/triage-tiny-v1, got: {p:?}"
    );
}

#[test]
fn test_augmentation_and_required_teacher_labels() {
    assert!(
        augmentation_hints()
            .iter()
            .any(|hint| hint.name == "case_variation")
    );
    assert!(
        validate_teacher_labels(&TeacherLabels {
            intent: "generate".into(),
            complexity: "low".into(),
            scope: "single_file".into(),
            reasoning_need: "low".into(),
            risk: "low".into(),
        })
        .is_ok()
    );
}
