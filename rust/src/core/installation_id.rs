//! Persistent anonymous installation identifier for default-on telemetry.
//!
//! Generates a random UUID v4 on first call and persists it as a plain-text
//! file in the data directory. The ID is never derived from hardware, OS
//! fingerprints, or user identity — it is pure randomness.

use std::path::PathBuf;

use fs2::FileExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TelemetryIdentity {
    installation_id: String,
    deletion_token: String,
}

fn id_path() -> Result<PathBuf, String> {
    crate::core::paths::data_dir().map(|d| d.join("installation_id"))
}

fn deletion_token_path() -> Result<PathBuf, String> {
    crate::core::paths::data_dir().map(|d| d.join("telemetry_deletion_token"))
}

fn identity_path() -> Result<PathBuf, String> {
    crate::core::paths::data_dir().map(|d| d.join("telemetry_identity.json"))
}

/// When the current identity was created: the file's birth time where the
/// filesystem records one, otherwise its last write (identity files are only
/// rewritten on creation, migration or `telemetry reset-id`). An identity
/// adopted from a sibling data directory keeps the sibling's age.
pub(crate) fn identity_created_at() -> Option<std::time::SystemTime> {
    let path = identity_path().ok()?;
    let own = read_identity(&path)?;
    std::iter::once(path.clone())
        .chain(sibling_identity_paths(&path))
        .filter(|candidate| {
            read_identity(candidate)
                .is_some_and(|other| other.installation_id == own.installation_id)
        })
        .filter_map(|candidate| file_created_at(&candidate))
        .min()
}

fn file_created_at(path: &std::path::Path) -> Option<std::time::SystemTime> {
    let metadata = std::fs::metadata(path).ok()?;
    metadata.created().or_else(|_| metadata.modified()).ok()
}

fn read_identity(path: &std::path::Path) -> Option<TelemetryIdentity> {
    let identity: TelemetryIdentity =
        serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    (is_valid_uuid(&identity.installation_id) && is_lower_hex_256(&identity.deletion_token))
        .then_some(identity)
}

/// The identity files of the same user's other LeanCTX data directories:
/// legacy `~/.lean-ctx`, mixed `$XDG_CONFIG_HOME/lean-ctx` and
/// `$XDG_DATA_HOME/lean-ctx`. Processes started with a different data-dir
/// resolution (an agent host with its own `XDG_*`, an upgrade across the
/// #408 layout change) would otherwise mint a second identity for one
/// installation. An explicit `LEAN_CTX_DATA_DIR` pins exactly one directory.
fn sibling_identity_paths(current: &std::path::Path) -> Vec<PathBuf> {
    if std::env::var_os("LEAN_CTX_DATA_DIR").is_some() {
        return Vec::new();
    }
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let base = |variable: &str, fallback: PathBuf| {
        std::env::var(variable)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map_or(fallback, PathBuf::from)
            .join("lean-ctx")
    };
    let canonical = |path: &std::path::Path| std::fs::canonicalize(path).ok();
    let current_dir = current.parent().and_then(canonical);
    [
        home.join(".lean-ctx"),
        base("XDG_CONFIG_HOME", home.join(".config")),
        base("XDG_DATA_HOME", home.join(".local").join("share")),
    ]
    .into_iter()
    .filter(|dir| current_dir.is_none() || canonical(dir) != current_dir)
    .map(|dir| dir.join("telemetry_identity.json"))
    .filter(|path| path.is_file())
    .collect()
}

/// The oldest valid identity among the sibling data directories, if any.
fn sibling_identity(current: &std::path::Path) -> Option<TelemetryIdentity> {
    sibling_identity_paths(current)
        .into_iter()
        .filter_map(|path| Some((file_created_at(&path)?, read_identity(&path)?)))
        .min_by_key(|(created, _)| *created)
        .map(|(_, identity)| identity)
}

fn with_deletion_lock<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let lock_path = crate::core::paths::data_dir()?.join("telemetry_deletion_token.lock");
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create data dir: {e}"))?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.create(true).read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let lock = options
        .open(lock_path)
        .map_err(|e| format!("Cannot open telemetry credential lock: {e}"))?;
    lock.lock_exclusive()
        .map_err(|e| format!("Cannot lock telemetry credential: {e}"))?;
    let result = operation();
    FileExt::unlock(&lock).map_err(|e| format!("Cannot unlock telemetry credential: {e}"))?;
    result
}

/// Return the existing installation ID or generate a fresh UUID v4.
pub(crate) fn get_or_create() -> Result<String, String> {
    Ok(get_or_create_identity()?.0)
}

/// Read one lock-consistent installation ID and deletion credential snapshot.
pub(crate) fn get_or_create_identity() -> Result<(String, String), String> {
    with_deletion_lock(|| {
        let path = identity_path()?;
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(identity) = serde_json::from_str::<TelemetryIdentity>(&raw) {
                if is_valid_uuid(&identity.installation_id)
                    && is_lower_hex_256(&identity.deletion_token)
                {
                    tighten_secret_permissions(&path)?;
                    retire_legacy_sidecars()?;
                    return Ok((identity.installation_id, identity.deletion_token));
                }
            }
        }
        // A legacy sidecar in this directory is this directory's own identity;
        // only a directory without one adopts the user's identity from a
        // sibling data directory instead of minting a second one.
        let own_sidecar = id_path()?.is_file();
        if !own_sidecar && let Some(identity) = sibling_identity(&path) {
            persist_identity(&path, &identity)?;
            return Ok((identity.installation_id, identity.deletion_token));
        }
        let identity = TelemetryIdentity {
            installation_id: load_or_create_id()?,
            deletion_token: load_or_create_token()?,
        };
        persist_identity(&path, &identity)?;
        retire_legacy_sidecars()?;
        Ok((identity.installation_id, identity.deletion_token))
    })
}

fn load_or_create_id() -> Result<String, String> {
    let path = id_path()?;
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim().to_string();
        if is_valid_uuid(&trimmed) {
            return Ok(trimmed);
        }
    }
    let id = generate_uuid_v4();
    persist(&path, &id)?;
    Ok(id)
}

/// Regenerate the installation ID (for `lean-ctx telemetry reset-id`).
pub(crate) fn reset() -> Result<String, String> {
    with_deletion_lock(|| {
        let identity = TelemetryIdentity {
            installation_id: generate_uuid_v4(),
            deletion_token: generate_deletion_token(),
        };
        persist_identity(&identity_path()?, &identity)?;
        retire_legacy_sidecars()?;
        Ok(identity.installation_id)
    })
}

/// Return the installation-scoped 256-bit secret proving deletion ownership.
#[cfg(test)]
pub(crate) fn deletion_token() -> Result<String, String> {
    Ok(get_or_create_identity()?.1)
}

fn load_or_create_token() -> Result<String, String> {
    let path = deletion_token_path()?;
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let token = existing.trim().to_string();
        if is_lower_hex_256(&token) {
            tighten_secret_permissions(&path)?;
            return Ok(token);
        }
    }
    let token = generate_deletion_token();
    persist_secret(&path, &token)?;
    Ok(token)
}

#[cfg(test)]
pub(crate) fn deletion_token_hash() -> Result<String, String> {
    use sha2::Digest;
    Ok(hex::encode(sha2::Sha256::digest(
        deletion_token()?.as_bytes(),
    )))
}

fn persist(path: &std::path::Path, id: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create data dir: {e}"))?;
    }
    crate::core::atomic_fs::write_bytes_with_fallback(path, format!("{id}\n").as_bytes(), None)
        .map_err(|e| format!("Cannot write installation ID: {e}"))
}

fn persist_secret(path: &std::path::Path, token: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create data dir: {e}"))?;
    }
    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        Some(std::fs::Permissions::from_mode(0o600))
    };
    #[cfg(not(unix))]
    let permissions = None;
    crate::core::atomic_fs::try_atomic_write(
        path,
        format!("{token}\n").as_bytes(),
        permissions.as_ref(),
    )
    .map_err(|e| format!("Cannot write telemetry deletion credential: {e}"))?;
    tighten_secret_permissions(path)
}

fn retire_legacy_sidecars() -> Result<(), String> {
    for path in [id_path()?, deletion_token_path()?] {
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot retire legacy telemetry identity {}: {error}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

fn persist_identity(path: &std::path::Path, identity: &TelemetryIdentity) -> Result<(), String> {
    let encoded = serde_json::to_vec(identity)
        .map_err(|error| format!("Cannot encode telemetry identity: {error}"))?;
    persist_secret(path, std::str::from_utf8(&encoded).expect("JSON is UTF-8"))
}

#[cfg(unix)]
fn tighten_secret_permissions(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("Cannot secure telemetry deletion credential: {e}"))
}

#[cfg(not(unix))]
fn tighten_secret_permissions(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

fn generate_deletion_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("getrandom failed");
    hex::encode(bytes)
}

fn is_lower_hex_256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn generate_uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("getrandom failed");
    // RFC 4122 variant and version bits
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant 1
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([
            0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        ]),
    )
}

fn is_valid_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[3].len() == 4
        && parts[4].len() == 12
        && s.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
}

/// Return the masked form `abcd…wxyz` for display.
pub(crate) fn masked(id: &str) -> String {
    if id.len() < 8 {
        return "********".to_string();
    }
    let clean: String = id.chars().filter(|c| *c != '-').collect();
    format!("{}…{}", &clean[..4], &clean[clean.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = std::env::var(key).ok();
            match value {
                Some(value) => crate::test_env::set_var(key, value),
                None => crate::test_env::remove_var(key),
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(previous) => crate::test_env::set_var(self.key, previous),
                None => crate::test_env::remove_var(self.key),
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_new_data_dir_adopts_the_users_identity_from_a_sibling_dir() {
        let _lock = crate::core::data_dir::test_env_lock();
        let home = tempfile::tempdir().unwrap();
        let _home = EnvVarGuard::set("HOME", Some(home.path().to_str().unwrap()));
        let _pin = EnvVarGuard::set("LEAN_CTX_DATA_DIR", None);
        let _config = EnvVarGuard::set("XDG_CONFIG_HOME", None);
        let _data = EnvVarGuard::set("XDG_DATA_HOME", None);
        let legacy = home.path().join(".lean-ctx");
        std::fs::create_dir_all(&legacy).unwrap();
        let identity = TelemetryIdentity {
            installation_id: generate_uuid_v4(),
            deletion_token: generate_deletion_token(),
        };
        persist_identity(&legacy.join("telemetry_identity.json"), &identity).unwrap();
        let xdg = home.path().join(".local/share/lean-ctx");
        std::fs::create_dir_all(&xdg).unwrap();

        let adopted =
            sibling_identity(&xdg.join("telemetry_identity.json")).expect("sibling identity");
        assert_eq!(adopted.installation_id, identity.installation_id);
        assert_eq!(adopted.deletion_token, identity.deletion_token);
        // The directory never adopts itself, and an explicit pin adopts nothing.
        assert!(sibling_identity(&legacy.join("telemetry_identity.json")).is_none());
        let _pinned = EnvVarGuard::set("LEAN_CTX_DATA_DIR", Some(xdg.to_str().unwrap()));
        assert!(sibling_identity(&xdg.join("telemetry_identity.json")).is_none());
    }

    #[test]
    fn generated_uuid_is_valid() {
        let id = generate_uuid_v4();
        assert!(is_valid_uuid(&id), "invalid UUID: {id}");
        assert_eq!(id.len(), 36);
        assert_eq!(&id[14..15], "4", "version nibble must be 4");
        let variant_nibble = u8::from_str_radix(&id[19..20], 16).unwrap();
        assert!(
            (0x8..=0xb).contains(&variant_nibble),
            "variant must be 8-b, got {variant_nibble:x}"
        );
    }

    #[test]
    fn get_or_create_roundtrip() {
        let _iso = crate::core::data_dir::isolated_data_dir();
        let a = get_or_create().unwrap();
        let b = get_or_create().unwrap();
        assert_eq!(a, b, "must return the same ID on repeated calls");
    }

    #[test]
    fn reset_changes_id() {
        let _iso = crate::core::data_dir::isolated_data_dir();
        let a = get_or_create().unwrap();
        let old_token = deletion_token().unwrap();
        let b = reset().unwrap();
        assert_ne!(a, b, "reset must produce a new ID");
        let c = get_or_create().unwrap();
        assert_eq!(b, c, "get_or_create after reset must return the new ID");
        assert_ne!(old_token, deletion_token().unwrap());
    }

    #[test]
    fn deletion_credential_is_stable_and_only_hash_is_shareable() {
        let _iso = crate::core::data_dir::isolated_data_dir();
        let token = deletion_token().unwrap();
        assert_eq!(token, deletion_token().unwrap());
        assert!(is_lower_hex_256(&token));
        let hash = deletion_token_hash().unwrap();
        assert!(is_lower_hex_256(&hash));
        assert_ne!(token, hash);
    }

    #[test]
    fn concurrent_deletion_credential_creation_converges() {
        let _iso = crate::core::data_dir::isolated_data_dir();
        let workers: Vec<_> = (0..8).map(|_| std::thread::spawn(deletion_token)).collect();
        let tokens: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap().unwrap())
            .collect();
        assert!(tokens.iter().all(|token| token == &tokens[0]));
    }

    #[cfg(unix)]
    #[test]
    fn identity_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let _iso = crate::core::data_dir::isolated_data_dir();
        get_or_create_identity().unwrap();
        let mode = std::fs::metadata(identity_path().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn corrupt_identity_never_resurrects_retired_sidecars() {
        let _iso = crate::core::data_dir::isolated_data_dir();
        let original = get_or_create_identity().unwrap();
        let rotated = reset().unwrap();
        assert!(!id_path().unwrap().exists());
        assert!(!deletion_token_path().unwrap().exists());
        std::fs::write(identity_path().unwrap(), b"corrupt").unwrap();
        let recovered = get_or_create_identity().unwrap();
        assert_ne!(recovered.0, original.0);
        assert_ne!(recovered.0, rotated);
        assert_ne!(recovered.1, original.1);
    }

    #[test]
    fn masked_format() {
        let id = "abcdef01-2345-4678-9abc-def012345678";
        assert_eq!(masked(id), "abcd…5678");
    }
}
