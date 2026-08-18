//! Terminal-output compression for Cursor terminal poll files.
//!
//! Cursor polls `.cursor/projects/*/terminals/*.txt` every ~3s via `ctx_read`.
//! These files are typically 100-500KB of raw terminal output (ANSI escapes,
//! build logs, test output) — most of which is irrelevant to the agent.
//!
//! This module provides:
//! 1. **ANSI stripping** (~30-50% savings on colored output)
//! 2. **Repeat collapse** (identical consecutive lines → `[× N collapsed]`)
//! 3. **Tail priority** (only the last N lines where actionable output is)
//! 4. **Hash dedup** (unchanged content within 10s → ~15-token stub)

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use super::{ReadOutput, SessionCache, count_tokens};

/// Maximum tail lines to keep in compressed output.
const TAIL_LINES: usize = 150;
/// Maximum header lines (metadata block at the top of terminal files).
const MAX_HEADER_LINES: usize = 15;
/// Minimum file lines before tail-truncation activates.
const TAIL_THRESHOLD: usize = TAIL_LINES + MAX_HEADER_LINES + 20;
/// Dedup window: re-reads within this duration return a stub if hash matches.
const DEDUP_WINDOW_SECS: u64 = 10;

/// Per-path state for hash-based dedup of rapid terminal polls.
struct PollState {
    hash: u64,
    last_seen: Instant,
    compressed_tokens: usize,
}

static POLL_CACHE: Mutex<Option<HashMap<String, PollState>>> = Mutex::new(None);

fn content_hash(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut h);
    h.finish()
}

/// Entry point called from `core_logic.rs` for terminal poll files.
///
/// Handles hash-dedup (unchanged → stub) and compression in one call.
pub(super) fn handle_terminal_read(
    _cache: &mut SessionCache,
    path: &str,
    content: &str,
    file_ref: &str,
    short: &str,
    original_tokens: usize,
) -> ReadOutput {
    let hash = content_hash(content);

    // Check dedup cache: if same hash within window, return stub.
    {
        let mut guard = POLL_CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cache = guard.get_or_insert_with(HashMap::new);
        if let Some(state) = cache.get(path) {
            if state.hash == hash && state.last_seen.elapsed().as_secs() < DEDUP_WINDOW_SECS {
                let stub = format!(
                    "{file_ref}={short} [terminal unchanged · {tok}→stub]",
                    tok = state.compressed_tokens,
                );
                let stub_tokens = count_tokens(&stub);
                crate::core::stats::record("ctx_read", original_tokens, stub_tokens);
                return ReadOutput {
                    content: stub,
                    resolved_mode: "terminal".into(),
                    output_tokens: stub_tokens,
                    is_cache_hit: true,
                };
            }
        }
    }

    // Try compression; fall back to full content if no savings.
    let (output, sent) = match compress_terminal(content, file_ref, short, original_tokens) {
        Some(pair) => pair,
        None => (content.to_string(), original_tokens),
    };

    // Update dedup cache.
    {
        let mut guard = POLL_CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(
            path.to_string(),
            PollState {
                hash,
                last_seen: Instant::now(),
                compressed_tokens: sent,
            },
        );
        // Evict stale entries (>60s) to avoid memory growth.
        cache.retain(|_, v| v.last_seen.elapsed().as_secs() < 60);
    }

    crate::core::stats::record("ctx_read", original_tokens, sent);
    ReadOutput {
        content: output,
        resolved_mode: "terminal".into(),
        output_tokens: sent,
        is_cache_hit: false,
    }
}

/// Compress terminal output for token-efficient delivery.
///
/// Returns `(compressed_content, output_tokens)` or `None` if the content
/// is too small to benefit from compression.
fn compress_terminal(
    content: &str,
    file_ref: &str,
    short: &str,
    original_tokens: usize,
) -> Option<(String, usize)> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < TAIL_THRESHOLD {
        let cleaned = strip_ansi_and_collapse(content);
        let tokens = count_tokens(&cleaned);
        if tokens >= original_tokens {
            return None;
        }
        let header = format!("{file_ref}={short} [terminal {n}L]", n = lines.len());
        let body = format!("{header}\n{cleaned}");
        let sent = count_tokens(&body);
        return Some((body, sent));
    }

    let (meta_end, meta_block) = extract_metadata_header(&lines);

    let stripped_lines: Vec<String> = lines.iter().map(|l| strip_ansi(l)).collect();

    let tail_start = stripped_lines.len().saturating_sub(TAIL_LINES);
    let tail_start = tail_start.max(meta_end);

    let tail_slice = &stripped_lines[tail_start..];
    let tail_collapsed = collapse_repeats(tail_slice);

    let elided_count = tail_start - meta_end;
    let elided_bytes: usize = stripped_lines[meta_end..tail_start]
        .iter()
        .map(|l| l.len() + 1)
        .sum();

    let mut out = String::with_capacity(meta_block.len() + tail_collapsed.len() + 200);
    out.push_str(&format!(
        "{file_ref}={short} [terminal {total}L]\n",
        total = lines.len()
    ));
    out.push_str(&meta_block);

    if elided_count > 0 {
        out.push_str(&format!(
            "\n[lean-ctx: {elided_count} lines elided ({kb:.1}KB) — showing last {tail}]\n\n",
            kb = elided_bytes as f64 / 1024.0,
            tail = tail_collapsed.lines().count(),
        ));
    }

    out.push_str(&tail_collapsed);

    if let Some(exit_info) = extract_exit_info(&stripped_lines) {
        if !out.contains("exit_code") {
            out.push('\n');
            out.push_str(&exit_info);
        }
    }

    let sent = count_tokens(&out);
    if sent >= original_tokens {
        return None;
    }
    Some((out, sent))
}

/// Extracts the `---` delimited metadata block at the top of a terminal file.
fn extract_metadata_header(lines: &[&str]) -> (usize, String) {
    let mut meta = String::new();
    let mut in_meta = false;
    let mut end = 0;

    for (i, line) in lines.iter().enumerate().take(MAX_HEADER_LINES) {
        if line.trim() == "---" {
            meta.push_str(line);
            meta.push('\n');
            if in_meta {
                end = i + 1;
                break;
            }
            in_meta = true;
            continue;
        }
        if in_meta {
            meta.push_str(line);
            meta.push('\n');
        }
        if !in_meta && i > 2 {
            break;
        }
    }

    if end == 0 {
        end = if in_meta {
            lines.len().min(MAX_HEADER_LINES)
        } else {
            0
        };
    }

    (end, meta)
}

/// Strips ANSI escape sequences (CSI + OSC) from a single line.
fn strip_ansi(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            // CSI sequence: ESC [ ... <letter>
            i += 2;
            while i < bytes.len() && !bytes[i].is_ascii_alphabetic() && bytes[i] != b'm' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
        } else if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b']' {
            // OSC sequence: ESC ] ... (BEL or ST)
            i += 2;
            while i < bytes.len() && bytes[i] != 0x07 {
                if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            if i < bytes.len() && bytes[i] == 0x07 {
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }

    String::from_utf8_lossy(&out).to_string()
}

fn strip_ansi_and_collapse(content: &str) -> String {
    let lines: Vec<String> = content.lines().map(strip_ansi).collect();
    collapse_repeats(&lines)
}

/// Collapses runs of 3+ identical consecutive lines into `[× N collapsed]`.
fn collapse_repeats(lines: &[String]) -> String {
    if lines.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(lines.len() * 40);
    let mut prev = &lines[0];
    let mut count: usize = 1;

    for line in &lines[1..] {
        if line == prev {
            count += 1;
        } else {
            emit_line(&mut out, prev, count);
            prev = line;
            count = 1;
        }
    }
    emit_line(&mut out, prev, count);
    out
}

fn emit_line(out: &mut String, line: &str, count: usize) {
    if count > 2 {
        out.push_str(line);
        out.push('\n');
        out.push_str(&format!("[× {count} identical lines collapsed]\n"));
    } else {
        for _ in 0..count {
            out.push_str(line);
            out.push('\n');
        }
    }
}

/// Extracts exit_code/elapsed info from the footer metadata block.
fn extract_exit_info(lines: &[String]) -> Option<String> {
    let tail = if lines.len() > 10 {
        &lines[lines.len() - 10..]
    } else {
        lines
    };

    let mut in_footer = false;
    let mut info = String::new();
    for line in tail {
        if line.trim() == "---" {
            if in_footer {
                break;
            }
            in_footer = true;
            continue;
        }
        if in_footer
            && (line.starts_with("exit_code:")
                || line.starts_with("elapsed_ms:")
                || line.starts_with("ended_at:")
                || line.starts_with("status:"))
        {
            info.push_str(line.trim());
            info.push('\n');
        }
    }

    if info.is_empty() { None } else { Some(info) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_color_codes() {
        assert_eq!(strip_ansi("\x1b[1;32mOK\x1b[0m"), "OK");
        assert_eq!(strip_ansi("no escapes"), "no escapes");
        assert_eq!(strip_ansi("\x1b[31merror\x1b[0m: bad"), "error: bad");
    }

    #[test]
    fn collapse_repeats_groups_identical_lines() {
        let lines: Vec<String> = vec![
            "line A".into(),
            "line B".into(),
            "line B".into(),
            "line B".into(),
            "line B".into(),
            "line C".into(),
        ];
        let result = collapse_repeats(&lines);
        assert!(result.contains("[× 4 identical lines collapsed]"));
        assert!(result.contains("line A"));
        assert!(result.contains("line C"));
    }

    #[test]
    fn extract_metadata_header_parses_terminal_format() {
        let content =
            "---\npid: 1234\ncwd: /foo\nstatus: running\n---\noutput line 1\noutput line 2";
        let lines: Vec<&str> = content.lines().collect();
        let (end, meta) = extract_metadata_header(&lines);
        assert_eq!(end, 5);
        assert!(meta.contains("pid: 1234"));
        assert!(meta.contains("cwd: /foo"));
    }

    #[test]
    fn compress_small_file_strips_ansi_only() {
        let content = "\x1b[32mOK\x1b[0m\ntest passed\n";
        let result = compress_terminal(content, "F1", "test.txt", 100);
        assert!(result.is_some());
        let (body, _) = result.unwrap();
        assert!(!body.contains("\x1b["));
        assert!(body.contains("OK"));
    }

    #[test]
    fn compress_large_file_tail_truncates() {
        let mut content = String::from("---\npid: 99\ncwd: /x\nstatus: done\n---\n");
        for i in 0..500 {
            content.push_str(&format!("build output line {i}\n"));
        }
        content.push_str("---\nexit_code: 0\nelapsed_ms: 5000\n---\n");
        let tokens = count_tokens(&content);
        let result = compress_terminal(&content, "F1", "build.txt", tokens);
        assert!(result.is_some());
        let (body, sent) = result.unwrap();
        assert!(sent < tokens, "must save tokens: {sent} < {tokens}");
        assert!(body.contains("lines elided"));
        assert!(body.contains("exit_code: 0"));
        assert!(body.contains("pid: 99"));
    }

    #[test]
    fn dedup_returns_stub_on_unchanged_reread() {
        let mut cache = SessionCache::default();
        let content = "---\npid: 1\ncwd: /x\n---\nline 1\nline 2\n";
        let tokens = count_tokens(content);

        let r1 = handle_terminal_read(
            &mut cache,
            "/tmp/test_term.txt",
            content,
            "F1",
            "test_term.txt",
            tokens,
        );
        assert_eq!(r1.resolved_mode, "terminal");
        assert!(!r1.is_cache_hit);

        let r2 = handle_terminal_read(
            &mut cache,
            "/tmp/test_term.txt",
            content,
            "F1",
            "test_term.txt",
            tokens,
        );
        assert!(
            r2.is_cache_hit,
            "second read of same content must be a stub"
        );
        assert!(r2.content.contains("unchanged"));
        assert!(r2.output_tokens < r1.output_tokens);
    }

    #[test]
    fn dedup_recompresses_on_changed_content() {
        let mut cache = SessionCache::default();
        let content_a = "---\npid: 1\ncwd: /x\n---\nline 1\nline 2\n";
        let content_b = "---\npid: 1\ncwd: /x\n---\nline 1\nline 2\nline 3\n";
        let tokens_a = count_tokens(content_a);
        let tokens_b = count_tokens(content_b);

        let _r1 = handle_terminal_read(
            &mut cache,
            "/tmp/test_term2.txt",
            content_a,
            "F1",
            "test_term2.txt",
            tokens_a,
        );
        let r2 = handle_terminal_read(
            &mut cache,
            "/tmp/test_term2.txt",
            content_b,
            "F1",
            "test_term2.txt",
            tokens_b,
        );
        assert!(!r2.is_cache_hit, "changed content must not be a stub");
    }

    #[test]
    fn extract_exit_info_from_footer() {
        let lines: Vec<String> = vec![
            "output".into(),
            "---".into(),
            "exit_code: 0".into(),
            "elapsed_ms: 1234".into(),
            "---".into(),
        ];
        let info = extract_exit_info(&lines);
        assert!(info.is_some());
        let info = info.unwrap();
        assert!(info.contains("exit_code: 0"));
        assert!(info.contains("elapsed_ms: 1234"));
    }
}
