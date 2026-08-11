//! PowerShell cmdlet allowlist — verb-prefix + noun-level matching.
//!
//! PowerShell cmdlets follow a strict `Verb-Noun` naming convention.
//! This module classifies cmdlets into safe/blocked based on their verb prefix,
//! with noun-level exceptions for verbs that are safe in some contexts but
//! dangerous in others (e.g. `Out-String` is safe, `Out-File` is not).
//!
//! Additionally, it resolves dangerous PowerShell aliases that share names with
//! POSIX utilities (e.g. `rm` → `Remove-Item`, `iex` → `Invoke-Expression`)
//! so the verb gate cannot be bypassed through the POSIX allowlist (#1442).
//!
//! See GH #1442 and follow-up comments from @tr3lane.

/// Unconditionally safe verb prefixes — these are truly read-only / formatting
/// operations regardless of the noun.
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
];

/// Explicitly blocked verb prefixes — destructive or execution-capable.
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
    "Import-",
    "Export-",
    "Out-",
];

/// Noun-level safe exceptions for otherwise-blocked verb prefixes.
/// These specific cmdlets are safe despite their verb being in BLOCKED_VERB_PREFIXES.
const SAFE_CMDLET_EXCEPTIONS: &[&str] = &[
    // Set-Location = cd, the single most common PS command
    "Set-Location",
    // Pure constructors with no side effects
    "New-Guid",
    "New-TimeSpan",
    "New-Object",
    // Start-Sleep = wait, no mutation (same as the `sleep` alias)
    "Start-Sleep",
    // Clear-Host = cls, display-only (clears terminal, not data)
    "Clear-Host",
    // Out-String / Out-Null / Out-Host = formatting to stdout, no file I/O
    "Out-String",
    "Out-Null",
    "Out-Host",
    "Out-Default",
    // Import-Csv / Import-Clixml = reads data from file (doesn't execute code)
    "Import-Csv",
    "Import-Clixml",
    // Export-Csv / Export-Clixml to stdout (commonly piped, but noun is specific)
    // NOTE: Export-Csv -Path writes files, but the cmdlet is common enough in
    // agent pipelines that blocking it entirely is impractical. The file write
    // is to an explicit -Path arg, not arbitrary execution.
    "Export-Csv",
    "Export-Clixml",
    "Export-FormatData",
];

/// Dangerous PowerShell aliases that resolve to blocked cmdlets.
/// These MUST be intercepted on Windows (where the shell is PowerShell), because
/// they share names with POSIX utilities on the standard shell allowlist.
/// Without this, `iex` (= Invoke-Expression) passes the POSIX allowlist.
/// On non-Windows, these tokens are real POSIX binaries and should NOT be blocked.
const DANGEROUS_PS_ALIASES: &[(&str, &str)] = &[
    ("iex", "Invoke-Expression"),
    ("ihy", "Invoke-History"),
    ("icm", "Invoke-Command"),
    ("irm", "Invoke-RestMethod"),
    ("iwr", "Invoke-WebRequest"),
    ("rm", "Remove-Item"),
    ("rmdir", "Remove-Item"),
    ("del", "Remove-Item"),
    ("ri", "Remove-Item"),
    ("kill", "Stop-Process"),
    ("spps", "Stop-Process"),
    ("saps", "Start-Process"),
    ("sajb", "Start-Job"),
    ("ni", "New-Item"),
    ("ndr", "New-PSDrive"),
    ("mi", "Move-Item"),
    ("mv", "Move-Item"),
    ("cp", "Copy-Item"),
    ("cpi", "Copy-Item"),
    ("si", "Set-Item"),
    ("sp", "Set-ItemProperty"),
    ("sc", "Set-Content"),
    ("sl", "Set-Location"),
];

/// Safe PowerShell aliases and builtins that agents use frequently.
/// These bypass the allowlist like POSIX SHELL_BUILTINS.
const POWERSHELL_BUILTINS: &[&str] = &[
    // Aliases for Get-* (read-only)
    "gci", "gi", "gp", "gpv", "gc", "gl", "gm", "gps", "gsv", "gwmi",
    // Aliases for formatting/output
    "fl", "ft", "fw", "oh", "ogv",
    // Aliases for filtering/selection
    "sls", // Select-String
    // Measure-Object
    "measure", // ForEach-Object / Where-Object
    "%", "?", // Clear-Host (display only)
    "cls", // Get-ChildItem
    "dir", // Get-Content
    "type", // pagination
    "more", // Tee-Object (read pipe, no mutation)
    "tee", // Compare-Object
    "diff", // Start-Sleep (wait, no mutation)
    "sleep", // Set-Location (= cd, navigation only)
    "sl", "cd", "chdir", // Write-Output
    "echo", // Get-History
    "h", "history", // Get-Location
    "pwd", // Sort-Object
    "sort", // Where-Object
    "where", // ForEach-Object
    "foreach",
];

/// Check a command token against the PowerShell allowlist.
///
/// Returns:
/// - `Some(true)`: safe PowerShell cmdlet / alias / builtin
/// - `Some(false)`: destructive cmdlet (blocked)
/// - `None`: not recognized as PowerShell syntax; fall through to standard allowlist
pub(super) fn check_powershell_cmdlet(base: &str) -> Option<bool> {
    // Safe builtins/aliases (fast path)
    if POWERSHELL_BUILTINS
        .iter()
        .any(|&b| b.eq_ignore_ascii_case(base))
    {
        return Some(true);
    }

    // Dangerous PS aliases that overlap with POSIX names — resolve to target cmdlet.
    // Only active on Windows where these tokens ARE PowerShell aliases.
    // On Unix, `rm`/`kill` are real POSIX binaries handled by the standard allowlist.
    if cfg!(windows) {
        if let Some(&(_, target)) = DANGEROUS_PS_ALIASES
            .iter()
            .find(|&&(alias, _)| alias.eq_ignore_ascii_case(base))
        {
            if is_safe_exception(target) {
                return Some(true);
            }
            return Some(false);
        }
    }

    // Not a Verb-Noun cmdlet? Let the standard allowlist handle it.
    if !looks_like_cmdlet(base) {
        return None;
    }

    // Exact safe exception takes priority over verb-prefix block
    if is_safe_exception(base) {
        return Some(true);
    }

    if is_safe_verb(base) {
        return Some(true);
    }

    if is_blocked_verb(base) {
        return Some(false);
    }

    // Unknown verb with Verb-Noun shape — fall through to standard allowlist
    None
}

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

fn is_safe_verb(base: &str) -> bool {
    SAFE_VERB_PREFIXES
        .iter()
        .any(|prefix| starts_with_ignore_case(base, prefix))
}

fn is_blocked_verb(base: &str) -> bool {
    BLOCKED_VERB_PREFIXES
        .iter()
        .any(|prefix| starts_with_ignore_case(base, prefix))
}

fn is_safe_exception(base: &str) -> bool {
    SAFE_CMDLET_EXCEPTIONS
        .iter()
        .any(|&exc| exc.eq_ignore_ascii_case(base))
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
            "Read-Host",
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
            "Enable-PSRemoting",
            "Disable-PSBreakpoint",
            "Restart-Service",
            "Import-Module",
            "Out-File",
            "Export-ModuleMember",
        ] {
            assert_eq!(
                check_powershell_cmdlet(cmd),
                Some(false),
                "{cmd} should be blocked"
            );
        }
    }

    #[test]
    fn safe_exceptions_override_blocked_verb() {
        for cmd in [
            "Set-Location",
            "New-Guid",
            "New-TimeSpan",
            "New-Object",
            "Start-Sleep",
            "Clear-Host",
            "Out-String",
            "Out-Null",
            "Out-Host",
            "Import-Csv",
            "Import-Clixml",
            "Export-Csv",
        ] {
            assert_eq!(
                check_powershell_cmdlet(cmd),
                Some(true),
                "{cmd} should be safe (exception)"
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn dangerous_aliases_blocked_on_windows() {
        for alias in ["iex", "rm", "rmdir", "kill", "del", "ri", "ni"] {
            assert_eq!(
                check_powershell_cmdlet(alias),
                Some(false),
                "{alias} is a dangerous PS alias and must be blocked on Windows"
            );
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn dangerous_aliases_fall_through_on_unix() {
        // On Unix, rm/kill/iex are real POSIX binaries — not PS aliases
        for alias in ["rm", "kill"] {
            assert_eq!(
                check_powershell_cmdlet(alias),
                None,
                "{alias} should fall through to standard allowlist on Unix"
            );
        }
    }

    #[test]
    fn alias_sl_is_safe_builtin() {
        // sl is in POWERSHELL_BUILTINS directly (Set-Location = cd)
        assert_eq!(check_powershell_cmdlet("sl"), Some(true));
    }

    #[test]
    fn non_cmdlets_return_none() {
        for cmd in ["git", "cargo", "ls", "npm", "docker", "curl"] {
            assert_eq!(check_powershell_cmdlet(cmd), None, "{cmd} is not a cmdlet");
        }
    }

    #[test]
    fn powershell_builtins_pass() {
        for cmd in [
            "gci", "gc", "dir", "type", "sls", "%", "?", "cls", "sleep", "measure", "echo", "pwd",
            "sort", "where", "foreach",
        ] {
            assert_eq!(
                check_powershell_cmdlet(cmd),
                Some(true),
                "{cmd} is a PS builtin/alias"
            );
        }
    }

    #[test]
    fn case_insensitive_cmdlet_matching() {
        assert_eq!(check_powershell_cmdlet("get-date"), Some(true));
        assert_eq!(check_powershell_cmdlet("GET-DATE"), Some(true));
        assert_eq!(check_powershell_cmdlet("Get-Date"), Some(true));
        assert_eq!(check_powershell_cmdlet("remove-item"), Some(false));
        assert_eq!(check_powershell_cmdlet("INVOKE-Expression"), Some(false));
        assert_eq!(check_powershell_cmdlet("set-location"), Some(true));
        assert_eq!(check_powershell_cmdlet("NEW-GUID"), Some(true));
    }

    #[test]
    #[cfg(windows)]
    fn case_insensitive_alias_matching_on_windows() {
        assert_eq!(check_powershell_cmdlet("IEX"), Some(false));
        assert_eq!(check_powershell_cmdlet("Iex"), Some(false));
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

    #[test]
    fn import_module_is_blocked_but_import_csv_is_safe() {
        assert_eq!(check_powershell_cmdlet("Import-Module"), Some(false));
        assert_eq!(check_powershell_cmdlet("Import-Csv"), Some(true));
        assert_eq!(check_powershell_cmdlet("Import-Clixml"), Some(true));
        assert_eq!(check_powershell_cmdlet("Import-PSSession"), Some(false));
    }

    #[test]
    fn out_file_blocked_but_out_string_safe() {
        assert_eq!(check_powershell_cmdlet("Out-File"), Some(false));
        assert_eq!(check_powershell_cmdlet("Out-String"), Some(true));
        assert_eq!(check_powershell_cmdlet("Out-Null"), Some(true));
        assert_eq!(check_powershell_cmdlet("Out-Host"), Some(true));
        assert_eq!(check_powershell_cmdlet("Out-GridView"), Some(false));
    }

    #[test]
    fn start_sleep_safe_but_start_process_blocked() {
        assert_eq!(check_powershell_cmdlet("Start-Sleep"), Some(true));
        assert_eq!(check_powershell_cmdlet("Start-Process"), Some(false));
        assert_eq!(check_powershell_cmdlet("Start-Job"), Some(false));
    }

    #[test]
    #[cfg(windows)]
    fn mv_and_cp_are_dangerous_aliases_on_windows() {
        // On PowerShell, mv → Move-Item, cp → Copy-Item
        assert_eq!(check_powershell_cmdlet("mv"), Some(false));
        assert_eq!(check_powershell_cmdlet("cp"), Some(false));
    }

    #[test]
    #[cfg(not(windows))]
    fn mv_and_cp_are_posix_binaries_on_unix() {
        // On Unix, mv/cp are real binaries — handled by standard allowlist
        assert_eq!(check_powershell_cmdlet("mv"), None);
        assert_eq!(check_powershell_cmdlet("cp"), None);
    }
}
