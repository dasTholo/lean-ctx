use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LspServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LspServerInfo {
    pub language: &'static str,
    pub binary: &'static str,
    pub install_hint: &'static str,
}

pub const KNOWN_SERVERS: &[LspServerInfo] = &[
    LspServerInfo {
        language: "rust",
        binary: "rust-analyzer",
        install_hint: "rustup component add rust-analyzer",
    },
    LspServerInfo {
        language: "typescript",
        binary: "typescript-language-server",
        install_hint: "npm install -g typescript-language-server typescript",
    },
    LspServerInfo {
        language: "python",
        binary: "pylsp",
        install_hint: "pip install python-lsp-server",
    },
    LspServerInfo {
        language: "go",
        binary: "gopls",
        install_hint: "go install golang.org/x/tools/gopls@latest",
    },
];

pub fn default_servers() -> HashMap<&'static str, LspServerConfig> {
    let mut m = HashMap::new();
    m.insert(
        "rust",
        LspServerConfig {
            command: "rust-analyzer".into(),
            args: vec![],
        },
    );
    m.insert(
        "typescript",
        LspServerConfig {
            command: "typescript-language-server".into(),
            args: vec!["--stdio".into()],
        },
    );
    m.insert(
        "javascript",
        LspServerConfig {
            command: "typescript-language-server".into(),
            args: vec!["--stdio".into()],
        },
    );
    m.insert(
        "python",
        LspServerConfig {
            command: "pylsp".into(),
            args: vec![],
        },
    );
    m.insert(
        "go",
        LspServerConfig {
            command: "gopls".into(),
            args: vec!["serve".into()],
        },
    );
    m
}

pub fn language_for_extension(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("rust"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "py" | "pyi" => Some("python"),
        "go" => Some("go"),
        "java" => Some("java"),
        "kt" | "kts" => Some("kotlin"),
        "rb" => Some("ruby"),
        "c" | "h" => Some("c"),
        "cpp" | "cxx" | "cc" | "hpp" => Some("cpp"),
        "cs" => Some("csharp"),
        _ => None,
    }
}

pub fn find_binary_in_path(binary: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
        if cfg!(windows) {
            let exe = dir.join(format!("{binary}.exe"));
            if exe.is_file() {
                return Some(exe);
            }
        }
    }
    None
}

/// Like [`find_binary_in_path`], but only returns servers that can actually
/// run. A rustup proxy (`~/.cargo/bin/rust-analyzer` → `rustup`) exists even
/// when the component is not installed and then fails on start; such a proxy
/// counts only if `--version` succeeds. Plain binaries are not executed.
pub fn find_runnable_server(binary: &str) -> Option<PathBuf> {
    find_binary_in_path(binary).and_then(runnable)
}

fn runnable(path: PathBuf) -> Option<PathBuf> {
    let is_rustup_proxy = std::fs::read_link(&path).is_ok_and(|target| {
        target
            .file_stem()
            .is_some_and(|stem| stem.eq_ignore_ascii_case("rustup"))
    });
    if !is_rustup_proxy {
        return Some(path);
    }
    // Never let the probe install a toolchain (a `rust-toolchain.toml` in the
    // working directory could otherwise trigger a download).
    std::process::Command::new(&path)
        .arg("--version")
        .env("RUSTUP_AUTO_INSTALL", "0")
        .current_dir(std::env::temp_dir())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
        .then_some(path)
}

pub fn install_hint_for_language(language: &str) -> &'static str {
    for info in KNOWN_SERVERS {
        if info.language == language {
            return info.install_hint;
        }
    }
    "No install instructions available for this language server."
}

pub fn binary_for_language(language: &str) -> Option<&'static str> {
    for info in KNOWN_SERVERS {
        if info.language == language {
            return Some(info.binary);
        }
    }
    None
}

pub fn check_server_available(language: &str) -> Result<PathBuf, String> {
    let servers = default_servers();
    let config = servers
        .get(language)
        .ok_or_else(|| format!("No LSP server configured for '{language}'"))?;

    find_runnable_server(&config.command).ok_or_else(|| {
        let hint = install_hint_for_language(language);
        format!(
            "Language server '{}' not found in PATH (or not installed behind its rustup proxy).\n\
             \n\
             ctx_refactor requires an external language server for '{}' files.\n\
             Install it with:\n\
             \n\
             \x20   {}\n\
             \n\
             Then retry. This is optional — ctx_search and ctx_graph work without it.",
            config.command, language, hint
        )
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::runnable;
    use std::os::unix::fs::{PermissionsExt, symlink};

    /// A rustup proxy without the installed component must not count as an
    /// available server (doctor showed ✓ and ctx_refactor failed on start).
    #[test]
    fn rustup_proxy_counts_only_when_it_runs() {
        let dir = tempfile::tempdir().unwrap();
        let rustup = dir.path().join("rustup");
        let proxy = dir.path().join("fake-analyzer");
        symlink(&rustup, &proxy).unwrap();

        let mut verdicts = Vec::new();
        for exit_code in [1, 0] {
            std::fs::write(&rustup, format!("#!/bin/sh\nexit {exit_code}\n")).unwrap();
            std::fs::set_permissions(&rustup, std::fs::Permissions::from_mode(0o755)).unwrap();
            verdicts.push(runnable(proxy.clone()).is_some());
        }
        assert_eq!(verdicts, vec![false, true]);
    }
}
