use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeacherConfig {
    pub model: String,
    pub batch_size: usize,
    pub max_retries: usize,
    pub output_path: PathBuf,
}

impl Default for TeacherConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet".into(),
            batch_size: 16,
            max_retries: 3,
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

impl TeacherLabels {
    pub fn is_complete(&self) -> bool {
        [
            &self.intent,
            &self.complexity,
            &self.scope,
            &self.reasoning_need,
            &self.risk,
        ]
        .iter()
        .all(|field| !field.trim().is_empty())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeacherPrompt {
    pub task_id: String,
    pub prompt: String,
    pub gold_labels: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeacherLabelError {
    pub message: String,
}

impl std::fmt::Display for TeacherLabelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TeacherLabelError {}

#[derive(Deserialize)]
struct GoldTask {
    id: String,
    query: String,
    labels: serde_json::Value,
}

pub fn prepare_teacher_batch(gold_set_path: &Path, config: &TeacherConfig) -> Vec<TeacherPrompt> {
    let Ok(contents) = std::fs::read_to_string(gold_set_path) else {
        return Vec::new();
    };
    let batch_size = config.batch_size.max(1);
    contents
        .lines()
        .filter_map(|line| serde_json::from_str::<GoldTask>(line).ok())
        .take(batch_size)
        .map(|task| TeacherPrompt {
            task_id: task.id,
            prompt: format!(
                "Classify this developer task: {}. Respond with JSON containing exactly: intent, complexity, scope, reasoning_need, risk.",
                task.query
            ),
            gold_labels: Some(task.labels),
        })
        .collect()
}

pub fn validate_teacher_labels(labels: &TeacherLabels) -> Result<(), TeacherLabelError> {
    if labels.is_complete() {
        Ok(())
    } else {
        Err(TeacherLabelError {
            message: "teacher response is missing one or more required labels".into(),
        })
    }
}

/// Executes a teacher request with bounded retries; the caller supplies the API transport.
pub fn with_teacher_retries<T, F>(
    config: &TeacherConfig,
    mut request: F,
) -> Result<T, TeacherLabelError>
where
    F: FnMut() -> Result<T, TeacherLabelError>,
{
    let attempts = config.max_retries.saturating_add(1);
    let mut last_error = None;
    for _ in 0..attempts {
        match request() {
            Ok(response) => return Ok(response),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or(TeacherLabelError {
        message: "teacher request was not attempted".into(),
    }))
}
