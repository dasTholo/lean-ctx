//! File-backed append-only execution ledger.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use fs2::FileExt;

use super::event::ExecutionEvent;
use super::verify::{GENESIS, hash_event, verify_events};
use super::{ExecutionLedgerError, Result};

/// Default execution-ledger location: `<data_dir>/execution/ledger.jsonl`.
pub fn default_path() -> Option<PathBuf> {
    let data_dir = crate::core::data_dir::lean_ctx_data_dir().ok()?;
    let directory = data_dir.join("execution");
    fs::create_dir_all(&directory).ok()?;
    Some(directory.join("ledger.jsonl"))
}

/// A process-independent handle to an execution ledger file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionLedgerStore {
    path: PathBuf,
}

impl ExecutionLedgerStore {
    /// Creates a store backed by `path`.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Creates a store at the configured default location.
    pub fn from_default() -> Result<Self> {
        default_path().map(Self::new).ok_or_else(|| {
            ExecutionLedgerError::InvalidRecord("data directory unavailable".to_owned())
        })
    }

    /// Returns the backing path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends one event, assigning its next sequence number and chain link.
    pub fn append(&self, mut event: ExecutionEvent) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&self.path)?;
        file.lock_exclusive()?;

        let result = (|| {
            let events = read_events_from_file(&file)?;
            if !verify_events(&events)? {
                return Err(ExecutionLedgerError::InvalidChain(
                    "cannot append to an invalid chain".to_owned(),
                ));
            }

            let previous_hash = match events.last() {
                Some(previous) => hash_event(previous)?,
                None => GENESIS.to_owned(),
            };
            let sequence_number = events
                .last()
                .map_or(1, ExecutionEvent::sequence_number)
                .checked_add(u64::from(!events.is_empty()))
                .ok_or_else(|| {
                    ExecutionLedgerError::InvalidRecord(
                        "execution ledger sequence number overflow".to_owned(),
                    )
                })?;
            event.set_chain_fields(sequence_number, previous_hash);
            let line = serde_json::to_string(&event)?;
            file.seek(SeekFrom::End(0))?;
            writeln!(file, "{line}")?;
            file.flush()?;
            Ok(())
        })();

        let _ = FileExt::unlock(&file);
        result
    }

    /// Verifies the complete on-disk chain under a shared file lock.
    pub fn verify_chain(&self) -> Result<bool> {
        let Ok(file) = File::open(&self.path) else {
            return Ok(true);
        };
        file.lock_shared()?;
        let result = read_events_from_file(&file).and_then(|events| verify_events(&events));
        let _ = FileExt::unlock(&file);
        result
    }

    /// Loads all well-formed events in file order.
    pub fn load(&self) -> Result<Vec<ExecutionEvent>> {
        let Ok(file) = File::open(&self.path) else {
            return Ok(Vec::new());
        };
        read_events_from_file(&file)
    }

    /// Returns all events associated with `task_id`.
    #[must_use]
    pub fn by_task(&self, task_id: &str) -> Vec<ExecutionEvent> {
        self.load()
            .unwrap_or_default()
            .into_iter()
            .filter(|event| event.task_id() == task_id)
            .collect()
    }

    /// Returns the last assigned sequence number, or zero for an empty/missing file.
    #[must_use]
    pub fn last_sequence(&self) -> u64 {
        self.load()
            .ok()
            .and_then(|events| events.last().map(ExecutionEvent::sequence_number))
            .unwrap_or(0)
    }
}

fn read_events_from_file(file: &File) -> Result<Vec<ExecutionEvent>> {
    let mut reader = BufReader::new(file.try_clone()?);
    reader.seek(SeekFrom::Start(0))?;
    let mut events = Vec::new();
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            return Err(ExecutionLedgerError::InvalidRecord(format!(
                "empty line at index {line_number}"
            )));
        }
        let event = serde_json::from_str(&line)?;
        events.push(event);
    }
    Ok(events)
}
