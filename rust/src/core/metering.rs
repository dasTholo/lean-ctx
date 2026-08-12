//! Local, append-only metering for the free product.
//!
//! Metering is deliberately observational: failures are logged and never affect
//! an MCP tool response.  The JSONL file contains token counts only.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{OpenOptions, create_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use fs2::FileExt;

const METERING_FILE: &str = "metering.jsonl";

/// One completed tool call, recorded locally as a JSONL line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeterEntry {
    pub timestamp: DateTime<Utc>,
    pub tool_name: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub savings_tokens: u64,
}

impl MeterEntry {
    #[must_use]
    pub fn new(
        tool_name: impl Into<String>,
        input_tokens: u64,
        output_tokens: u64,
        savings_tokens: u64,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            tool_name: tool_name.into(),
            input_tokens,
            output_tokens,
            savings_tokens,
        }
    }
}

/// Aggregate used by local value displays such as "Pro would have saved you $X".
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MeterAggregate {
    pub total_savings_tokens: u64,
    pub total_calls: u64,
    /// Weighted output/input ratio. `0.0` means no metered input exists yet.
    pub avg_compression_ratio: f64,
}

/// Append-only local JSONL store.
#[derive(Debug, Clone)]
pub struct MeterStore {
    path: PathBuf,
}

impl MeterStore {
    #[must_use]
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            path: data_dir.as_ref().join(METERING_FILE),
        }
    }

    pub fn from_data_dir() -> Result<Self, String> {
        Ok(Self::new(crate::core::data_dir::lean_ctx_data_dir()?))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends one complete JSON object and newline. Call this only off the hot path.
    pub fn append(&self, entry: &MeterEntry) -> Result<(), String> {
        let Some(parent) = self.path.parent() else {
            return Err("metering path has no parent".to_string());
        };
        create_dir_all(parent).map_err(|error| error.to_string())?;
        let line = serde_json::to_string(entry).map_err(|error| error.to_string())?;
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| error.to_string())?;
        file.lock_exclusive().map_err(|error| error.to_string())?;
        let write_result = writeln!(file, "{line}").map_err(|error| error.to_string());
        let unlock_result = FileExt::unlock(&file).map_err(|error| error.to_string());
        write_result.and(unlock_result)
    }

    /// Schedules persistence and returns immediately; metering cannot delay a tool call.
    pub fn append_best_effort(entry: MeterEntry) {
        tokio::task::spawn_blocking(move || {
            let result = Self::from_data_dir().and_then(|store| store.append(&entry));
            if let Err(error) = result {
                tracing::warn!(%error, "lean-ctx: failed to append local metering entry");
            }
        });
    }

    #[must_use]
    pub fn aggregate(&self) -> MeterAggregate {
        let Ok(file) = std::fs::File::open(&self.path) else {
            return MeterAggregate::default();
        };
        let mut aggregate = MeterAggregate::default();
        let mut total_input = 0_u64;
        let mut total_output = 0_u64;
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            let Ok(entry) = serde_json::from_str::<MeterEntry>(&line) else {
                continue;
            };
            aggregate.total_calls = aggregate.total_calls.saturating_add(1);
            aggregate.total_savings_tokens = aggregate
                .total_savings_tokens
                .saturating_add(entry.savings_tokens);
            total_input = total_input.saturating_add(entry.input_tokens);
            total_output = total_output.saturating_add(entry.output_tokens);
        }
        if total_input > 0 {
            aggregate.avg_compression_ratio = total_output as f64 / total_input as f64;
        }
        aggregate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_jsonl_entries() {
        let dir = tempfile::tempdir().unwrap();
        let store = MeterStore::new(dir.path());
        store
            .append(&MeterEntry::new("ctx_read", 100, 20, 80))
            .unwrap();
        store
            .append(&MeterEntry::new("ctx_search", 200, 100, 100))
            .unwrap();

        let aggregate = store.aggregate();
        assert_eq!(aggregate.total_calls, 2);
        assert_eq!(aggregate.total_savings_tokens, 180);
        assert!((aggregate.avg_compression_ratio - 0.4).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn best_effort_append_does_not_block_calling_task() {
        let entry = MeterEntry::new("ctx_read", 10, 2, 8);
        let task = tokio::spawn(async move {
            MeterStore::append_best_effort(entry);
            42_u8
        });
        assert_eq!(task.await.unwrap(), 42);
    }
}
