//! PowerShell cmdlet allowlist — verb-prefix based matching.
//!
//! PowerShell cmdlets follow a strict `Verb-Noun` naming convention. Read-only
//! verbs (`Get-`, `Test-`, `Measure-`, …) are inherently safe; destructive
//! verbs (`Remove-`, `Stop-`, `Set-`, `Invoke-`, …) remain gated.
//!
//! This module is shell-aware: it activates on Windows or when the command
//! looks like a PowerShell cmdlet (contains a `-` after a verb prefix).
//! See GH #1442.

/// Safe (read-only / formatting / output) PowerShell verb prefixes.
/// A cmdlet matching `<prefix>-<noun>` passes the allowlist automatically.
const SAFE_VERB_PREFIXES: &[&str] = &[
    "Get-",
    "Test-",
    "Measure-",
    "Select-",
    "Where-",
    "ForEach-",
    "Sort-",
    "Group-",
    "Compare-",
    "Format-",
    "Out-",
    "Write-",
    "ConvertTo-",
    "ConvertFrom-",
    "Join-",
    "Split-",
    "Resolve-",
    "Convert-",
    "Find-",
    "Search-",
    "Show-",
    "Wait-",
    "Watch-",
    "Trace-",
    "Debug-",
    "Assert-",
    "Confirm-",
    "Read-",
    "Import-",
    "Export-",
];

/// Explicitly blocked PowerShell verb prefixes — destructive or
/// execution-capable. These NEVER auto-pass, even on Windows.
const BLOCKED_VERB_PREFIXES: &[&str] = &[
    "Remove-",
    "Clear-",
    "Stop-",
    "Set-",
    "New-",
    "Start-",
    "Invoke-",
    "Enable-",
    "Disable-",
    "Uninstall-",
    "Restart-",
    "Suspend-",
    "Resume-",
    "Reset-",
    "Revoke-",
    "Deny-",
    "Block-",
    "Unblock-",
    "Disconnect-",
    "Dismount-",
    "Unregister-",
];

/// Common PowerShell aliases and builtins that agents use frequently.
/// These bypass the allowlist like POSIX `SHELL_BUILTINS`.
const POWERSHELL_BUILTINS: &[&str] = &[
    // Aliases for Get-* (read-only)
    "gci", "gi", "gp", "gpv", "gc", "gl", "gm", "gps", "gsv", "gwmi",
    // Aliases for formatting/output
    "fl", "ft", "fw", "oh", "ogv",
    // Aliases for filtering/selection
    "sls",     // Select-String
    "measure", // Aliases for ForEach/Where
    "%", "?",     // Common safe aliases
    "cls",   // Clear-Host (display only)
    "dir",   // Get-ChildItem
    "type",  // Get-Content
    "more",  // pagination
    "tee",   // Tee-Object
    "diff",  // Compare-Object
    "sleep", // Start-Sleep (wait, no mutation)
];

/// Returns `true` if `base` is a safe PowerShell cmdlet (verb-prefix match),
/// a known safe alias/builtin, or explicitly blocked (returns `false`).
///
/// Returns `None` if the command doesn't look like a PowerShell cmdlet at all
/// (no Verb-Noun pattern), meaning the caller should fall through to the
/// standard allowlist check.
pub(super) fn check_powershell_cmdlet(base: &str) -> Option<bool> {
    if POWERSHELL_BUILTINS
        .iter()
        .any(|&b| b.eq_ignore_ascii_case(base))
    {
        return Some(true);
    }

    if !looks_like_cmdlet(base) {
        return None;
    }

    if is_safe_cmdlet(base) {
        return Some(true);
    }

    if is_blocked_cmdlet(base) {
        return Some(false);
    }

    // Unknown verb with Verb-Noun shape — fall through to standard allowlist
    None
}

/// A string looks like a PowerShell cmdlet if it matches `Word-Word` pattern
/// (at least one hyphen with alphabetic chars on both sides).
fn looks_like_cmdlet(s: &str) -> bool {
    let Some(hyphen_pos) = s.find('-') else {
        return false;
    };
    if hyphen_pos == 0 || hyphen_pos == s.len() - 1 {
        return false;
    }
    let prefix = &s[..hyphen_pos];
    let suffix = &s[hyphen_pos + 1..];
    prefix.chars().all(|c| c.is_ascii_alphabetic())
        && suffix
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
}

fn is_safe_cmdlet(base: &str) -> bool {
    SAFE_VERB_PREFIXES
        .iter()
        .any(|prefix| base.starts_with(prefix) || starts_with_ignore_case(base, prefix))
}

fn is_blocked_cmdlet(base: &str) -> bool {
    BLOCKED_VERB_PREFIXES
        .iter()
        .any(|prefix| base.starts_with(prefix) || starts_with_ignore_case(base, prefix))
}

fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_cmdlets_pass() {
        for cmd in [
            "Get-Date",
            "Get-ChildItem",
            "Test-Path",
            "Measure-Object",
            "Select-Object",
            "Format-Table",
            "Out-String",
            "Write-Output",
            "ConvertTo-Json",
            "ConvertFrom-Csv",
            "Compare-Object",
            "Sort-Object",
            "Where-Object",
            "ForEach-Object",
            "Join-Path",
            "Split-Path",
            "Resolve-Path",
            "Find-Module",
            "Import-Csv",
            "Export-Csv",
        ] {
            assert_eq!(
                check_powershell_cmdlet(cmd),
                Some(true),
                "{cmd} should be safe"
            );
        }
    }

    #[test]
    fn blocked_cmdlets_rejected() {
        for cmd in [
            "Remove-Item",
            "Clear-Content",
            "Stop-Process",
            "Set-Content",
            "New-Item",
            "Start-Process",
            "Invoke-Expression",
            "Invoke-Command",
            "Invoke-WebRequest",
            "Enable-PSRemoting",
            "Disable-PSBreakpoint",
            "Restart-Service",
        ] {
            assert_eq!(
                check_powershell_cmdlet(cmd),
                Some(false),
                "{cmd} should be blocked"
            );
        }
    }

    #[test]
    fn non_cmdlets_return_none() {
        for cmd in ["git", "cargo", "ls", "npm", "docker", "curl"] {
            assert_eq!(check_powershell_cmdlet(cmd), None, "{cmd} is not a cmdlet");
        }
    }

    #[test]
    fn powershell_builtins_pass() {
        for cmd in ["gci", "gc", "dir", "type", "sls", "%", "?", "cls"] {
            assert_eq!(
                check_powershell_cmdlet(cmd),
                Some(true),
                "{cmd} is a PS builtin/alias"
            );
        }
    }

    #[test]
    fn case_insensitive_matching() {
        assert_eq!(check_powershell_cmdlet("get-date"), Some(true));
        assert_eq!(check_powershell_cmdlet("GET-DATE"), Some(true));
        assert_eq!(check_powershell_cmdlet("Get-Date"), Some(true));
        assert_eq!(check_powershell_cmdlet("remove-item"), Some(false));
        assert_eq!(check_powershell_cmdlet("INVOKE-Expression"), Some(false));
    }

    #[test]
    fn edge_cases() {
        assert_eq!(check_powershell_cmdlet("-"), None);
        assert_eq!(check_powershell_cmdlet("Get-"), None);
        assert_eq!(check_powershell_cmdlet("-Item"), None);
        assert_eq!(check_powershell_cmdlet(""), None);
        assert_eq!(check_powershell_cmdlet("Get"), None);
    }

    #[test]
    fn unknown_verb_returns_none() {
        assert_eq!(check_powershell_cmdlet("Custom-Thing"), None);
        assert_eq!(check_powershell_cmdlet("Foo-Bar"), None);
    }
}
