//! Single source of truth for locating OpenCode's global config file.
//!
//! OpenCode accepts both `opencode.json` and `opencode.jsonc`. lean-ctx used to
//! hardcode `opencode.json` in every writer, so a user whose config was
//! `opencode.jsonc` got a *second*, competing global config containing nothing
//! but the lean-ctx MCP entry and shadow-mode denies — their providers, models
//! and foreign MCP servers appeared to vanish (#1585).
//!
//! Resolution rule, applied by `init`, `setup`, doctor, the shadow-mode
//! permission updater and uninstall alike:
//!
//! 1. exactly one of the two exists → that file
//! 2. both exist → `opencode.json` (OpenCode's own default name), reported via
//!    [`Resolved::ambiguous`] so callers can warn instead of silently picking
//! 3. neither exists → `opencode.json`, to be created

use std::path::{Path, PathBuf};

/// Where the OpenCode config lives, plus whether the choice was ambiguous.
pub struct Resolved {
    /// The file to read and write.
    pub path: PathBuf,
    /// Both `opencode.json` and `opencode.jsonc` exist on disk.
    pub ambiguous: bool,
}

impl Resolved {
    /// Display form for CLI output, e.g. `~/.config/opencode/opencode.jsonc`.
    pub fn display(&self, home: &Path) -> String {
        self.path.strip_prefix(home).map_or_else(
            |_| self.path.display().to_string(),
            |rest| format!("~/{}", rest.display()),
        )
    }
}

/// Resolve the config inside an OpenCode config directory.
pub fn resolve_in_dir(dir: &Path) -> Resolved {
    let json = dir.join("opencode.json");
    let jsonc = dir.join("opencode.jsonc");
    match (json.exists(), jsonc.exists()) {
        (false, true) => Resolved {
            path: jsonc,
            ambiguous: false,
        },
        (true, true) => Resolved {
            path: json,
            ambiguous: true,
        },
        _ => Resolved {
            path: json,
            ambiguous: false,
        },
    }
}

/// Resolve OpenCode's global config directory for this platform.
pub fn config_dir(home: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("opencode");
        }
        home.join(".config/opencode")
    }
    #[cfg(not(windows))]
    {
        home.join(".config/opencode")
    }
}

/// Resolve the global OpenCode config for a given home directory.
pub fn resolve(home: &Path) -> Resolved {
    resolve_in_dir(&config_dir(home))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(p: &Path) {
        std::fs::write(p, "{}").unwrap();
    }

    #[test]
    fn picks_the_only_file_that_exists() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("opencode.jsonc"));
        let r = resolve_in_dir(dir.path());
        assert_eq!(r.path.file_name().unwrap(), "opencode.jsonc");
        assert!(!r.ambiguous);
    }

    #[test]
    fn plain_json_is_used_when_it_is_the_one_present() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("opencode.json"));
        let r = resolve_in_dir(dir.path());
        assert_eq!(r.path.file_name().unwrap(), "opencode.json");
        assert!(!r.ambiguous);
    }

    #[test]
    fn both_present_prefers_json_and_reports_ambiguity() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("opencode.json"));
        touch(&dir.path().join("opencode.jsonc"));
        let r = resolve_in_dir(dir.path());
        assert_eq!(r.path.file_name().unwrap(), "opencode.json");
        assert!(
            r.ambiguous,
            "callers must be able to warn instead of silently choosing"
        );
    }

    #[test]
    fn neither_present_targets_json_for_creation() {
        let dir = tempfile::tempdir().unwrap();
        let r = resolve_in_dir(dir.path());
        assert_eq!(r.path.file_name().unwrap(), "opencode.json");
        assert!(!r.ambiguous);
    }

    #[test]
    fn jsonc_is_never_shadowed_by_a_file_we_would_create() {
        // The #1585 regression: a user with only opencode.jsonc must not end up
        // with lean-ctx writing a competing opencode.json.
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("opencode.jsonc"));
        assert_eq!(
            resolve_in_dir(dir.path()).path,
            dir.path().join("opencode.jsonc")
        );
    }
}
