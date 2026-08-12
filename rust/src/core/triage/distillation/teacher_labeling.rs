use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeacherConfig {
    pub model: String,
    pub batch_size: usize,
    pub output_path: PathBuf,
}

impl Default for TeacherConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet".into(),
            batch_size: 16,
            output_path: PathBuf::from("teacher_labels.jsonl"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabeledSample {
    pub task_id: String,
    pub query: String,
    pub language: String,
    pub teacher_labels: TeacherLabels,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeacherLabels {
    pub intent: String,
    pub complexity: String,
    pub scope: String,
    pub reasoning_need: String,
    pub risk: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeacherPrompt {
    pub task_id: String,
    pub prompt: String,
    pub gold_labels: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct GoldTask {
    id: String,
    query: String,
    labels: serde_json::Value,
}

pub fn prepare_teacher_batch(gold_set_path: &Path, _config: &TeacherConfig) -> Vec<TeacherPrompt> {
    let Ok(contents) = std::fs::read_to_string(gold_set_path) else {
        return Vec::new();
    };
    contents.lines().filter_map(|line| serde_json::from_str::<GoldTask>(line).ok()).map(|task| TeacherPrompt {
        task_id: task.id,
        prompt: format!("Classify this developer task: {}. Respond with JSON: intent, complexity, scope, reasoning_need, risk", task.query),
        gold_labels: Some(task.labels),
    }).collect()
}
