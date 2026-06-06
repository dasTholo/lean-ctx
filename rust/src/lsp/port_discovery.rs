//! Discovery of the in-IDE JetBrains backend via a per-project port file.
//!
//! The plugin writes `~/.lean-ctx/jetbrains-<projecthash>.port` (JSON, 0600).
//! `projecthash = sha256(canonical(project_root))[..16]` — Rust and Kotlin MUST
//! canonicalize identically (symlink / trailing-slash trap, spec §5.5).

use std::time::Duration;

use serde::Deserialize;

/// Contents of the per-project port file (subset Rust needs).
#[derive(Debug, Clone, Deserialize)]
pub struct PortFile {
    pub port: u16,
    pub token: String,
    pub pid: u32,
    #[serde(default)]
    pub project_root: String,
    #[serde(default)]
    pub ide_version: String,
}

/// `sha256(canonical(project_root))[..16]` as lowercase hex (first 8 bytes → 16 chars).
pub fn project_hash(project_root: &str) -> String {
    use std::fmt::Write as _;

    use sha2::{Digest, Sha256};
    let canonical = std::fs::canonicalize(project_root).map_or_else(
        |_| project_root.to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    let digest = Sha256::digest(canonical.as_bytes());
    let mut hex = String::with_capacity(16);
    for b in digest.iter().take(8) {
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// `~/.lean-ctx/jetbrains-<projecthash>.port`.
pub fn port_file_path(project_root: &str) -> Option<std::path::PathBuf> {
    let home = dirs::home_dir()?;
    Some(
        home.join(".lean-ctx")
            .join(format!("jetbrains-{}.port", project_hash(project_root))),
    )
}

/// Reads + parses the port file, or `None` if absent/unreadable/malformed.
pub fn read_port_file(project_root: &str) -> Option<PortFile> {
    let path = port_file_path(project_root)?;
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// Liveness check for the IDE process. Linux: `/proc/<pid>`. Other OSes:
/// optimistic `true` (the `/health` ping is the authoritative reachability gate).
pub fn pid_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        true
    }
}

/// `GET /health` with token header and a tight timeout (~300ms, spec §4.3).
/// ureq 3.x: per-request timeout via `.config().timeout_global(..).build()`.
pub fn health_ok(pf: &PortFile) -> bool {
    let url = format!("http://127.0.0.1:{}/health", pf.port);
    ureq::get(&url)
        .config()
        .timeout_global(Some(Duration::from_millis(300)))
        .build()
        .header("X-LeanCtx-Token", &pf.token)
        .call()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_hash_is_stable_and_16_hex() {
        let h1 = project_hash("/some/project");
        let h2 = project_hash("/some/project");
        assert_eq!(h1, h2, "hash must be deterministic");
        assert_eq!(h1.len(), 16, "expected 16 hex chars (8 bytes)");
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn port_file_absent_for_unlikely_root() {
        // A path that has no port file → None (never panics).
        assert!(read_port_file("/nonexistent/lean-ctx/project/xyz").is_none());
    }
}
