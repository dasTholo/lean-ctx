//! Codex session-history import.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags, types::ValueRef};
use serde_json::Value;

use super::{
    ImportResult, ImportSource, finish_result, home_dir, process_value, push_error, read_json_file,
    read_jsonl_files, session_id,
};

const MAX_SQLITE_ROWS_PER_TABLE: usize = 10_000;
const MAX_SCAN_DEPTH: usize = 8;

/// Imports Codex history and session files from `~/.codex`.
#[must_use]
pub fn import() -> ImportResult {
    let Some(home) = home_dir() else {
        return ImportResult::default();
    };
    import_from_root(&home.join(".codex"))
}

/// Imports Codex JSONL, JSON, and SQLite session data from a Codex directory.
#[must_use]
pub fn import_from_root(root: &Path) -> ImportResult {
    if !root.exists() {
        return ImportResult::default();
    }

    let mut jsonl = Vec::new();
    let mut json = Vec::new();
    let mut sqlite = Vec::new();
    let mut errors = Vec::new();
    collect_files(root, 0, &mut jsonl, &mut json, &mut sqlite, &mut errors);
    jsonl.sort();
    json.sort();
    sqlite.sort();

    let mut result = read_jsonl_files(ImportSource::Codex, jsonl);
    for path in json {
        result.merge(read_json_file(ImportSource::Codex, &path));
    }
    for path in sqlite {
        result.merge(read_sqlite_file(&path));
    }
    for error in errors {
        push_error(&mut result, error);
    }
    result
}

fn collect_files(
    dir: &Path,
    depth: usize,
    jsonl: &mut Vec<PathBuf>,
    json: &mut Vec<PathBuf>,
    sqlite: &mut Vec<PathBuf>,
    errors: &mut Vec<String>,
) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!("{}: {error}", dir.display()));
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(format!("{}: {error}", dir.display()));
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, depth + 1, jsonl, json, sqlite, errors);
        } else if path.is_file() {
            match path.extension().and_then(|extension| extension.to_str()) {
                Some("jsonl") => jsonl.push(path),
                Some("json") if is_named_session_file(&path) => json.push(path),
                Some("sqlite" | "sqlite3" | "db") => sqlite.push(path),
                _ => {}
            }
        }
    }
}

fn is_named_session_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let name = name.to_ascii_lowercase();
    ["history", "session", "rollout", "transcript"]
        .iter()
        .any(|marker| name.contains(marker))
}

fn read_sqlite_file(path: &Path) -> ImportResult {
    let mut result = ImportResult {
        sessions_found: 1,
        ..ImportResult::default()
    };
    let mut touched = HashSet::new();
    let mut seen = HashSet::new();
    let session = session_id(ImportSource::Codex, path);
    let connection = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(connection) => connection,
        Err(error) => {
            push_error(&mut result, format!("{}: {error}", path.display()));
            return result;
        }
    };
    let tables = match tables(&connection) {
        Ok(tables) => tables,
        Err(error) => {
            push_error(&mut result, format!("{}: {error}", path.display()));
            return result;
        }
    };

    for table in tables {
        let query = format!(
            "SELECT * FROM {} LIMIT {MAX_SQLITE_ROWS_PER_TABLE}",
            quote_identifier(&table)
        );
        let mut statement = match connection.prepare(&query) {
            Ok(statement) => statement,
            Err(error) => {
                push_error(
                    &mut result,
                    format!("{} ({table}): {error}", path.display()),
                );
                continue;
            }
        };
        let columns = statement.column_count();
        let mut rows = match statement.query([]) {
            Ok(rows) => rows,
            Err(error) => {
                push_error(
                    &mut result,
                    format!("{} ({table}): {error}", path.display()),
                );
                continue;
            }
        };
        loop {
            match rows.next() {
                Ok(Some(row)) => {
                    for column in 0..columns {
                        let Ok(ValueRef::Text(bytes)) = row.get_ref(column) else {
                            continue;
                        };
                        let text = String::from_utf8_lossy(bytes);
                        let value = serde_json::from_str::<Value>(&text)
                            .unwrap_or_else(|_| Value::String(text.into_owned()));
                        process_value(
                            ImportSource::Codex,
                            &session,
                            &value,
                            &mut result,
                            &mut touched,
                            &mut seen,
                        );
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    push_error(
                        &mut result,
                        format!("{} ({table}): {error}", path.display()),
                    );
                    break;
                }
            }
        }
    }
    finish_result(&mut result, &touched);
    result
}

fn tables(connection: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )?;
    statement
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('\"', "\"\""))
}
