use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineCall {
    pub session_id: String,
    pub tool_name: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model: String,
    pub timestamp: String,
    pub raw_cost: u64,
}

impl BaselineCall {
    pub fn new(
        session_id: &str,
        tool_name: &str,
        input_tokens: u64,
        output_tokens: u64,
        model: &str,
    ) -> Self {
        Self {
            session_id: session_id.to_owned(),
            tool_name: tool_name.to_owned(),
            input_tokens,
            output_tokens,
            model: model.to_owned(),
            timestamp: Utc::now().to_rfc3339(),
            raw_cost: crate::core::value_gate::cost_tracker::calculate_cost(
                input_tokens,
                output_tokens,
                0,
                model,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaselineRecorder {
    directory: PathBuf,
}

impl BaselineRecorder {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }
    pub fn path(&self) -> PathBuf {
        self.directory.join("baseline.jsonl")
    }
    pub fn record(&self, call: &BaselineCall) -> io::Result<()> {
        append_jsonl(&self.path(), call)
    }
    pub fn read(&self) -> io::Result<Vec<BaselineCall>> {
        read_jsonl(&self.path())
    }
}

pub(crate) fn append_jsonl<T: Serialize>(path: &std::path::Path, entry: &T) -> io::Result<()> {
    fs::create_dir_all(path.parent().unwrap_or_else(|| std::path::Path::new(".")))?;
    let json = serde_json::to_string(entry).map_err(io::Error::other)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{json}")
}

pub(crate) fn read_jsonl<T: for<'de> Deserialize<'de>>(
    path: &std::path::Path,
) -> io::Result<Vec<T>> {
    match fs::read_to_string(path) {
        Ok(body) => Ok(body
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_recorder_appends_valid_jsonl_entries() {
        let dir = tempfile::tempdir().unwrap();
        let recorder = BaselineRecorder::new(dir.path());
        recorder
            .record(&BaselineCall::new("s1", "ctx_read", 10, 100, "gpt-4o"))
            .unwrap();
        recorder
            .record(&BaselineCall::new("s1", "ctx_search", 5, 40, "gpt-4o"))
            .unwrap();
        let body = fs::read_to_string(recorder.path()).unwrap();
        assert_eq!(body.lines().count(), 2);
        assert!(
            body.lines()
                .all(|line| serde_json::from_str::<BaselineCall>(line).is_ok())
        );
    }
}
