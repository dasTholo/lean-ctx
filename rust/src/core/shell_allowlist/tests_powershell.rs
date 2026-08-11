//! #1442: PowerShell cmdlet coverage — verb-prefix matching integration tests.

use super::{check_all_segments, tests::allow};

/// Safe PowerShell cmdlets pass through check_all_segments without being in
/// the explicit allowlist.
#[test]
fn powershell_safe_cmdlets_pass_enforcement() {
    let list = allow(&["git", "cargo"]);
    for cmd in [
        "Get-Date",
        "Get-ChildItem C:\\Temp",
        "Test-Path C:\\Temp",
        "Get-Content file.txt",
        "Write-Output hello",
        "Measure-Object",
        "Select-Object Name,Length",
        "Format-Table",
        "ConvertTo-Json",
        "ConvertFrom-Csv",
        "Sort-Object Name",
        "Where-Object Length -gt 100",
        "ForEach-Object { $_.Name }",
        "Compare-Object $a $b",
        "Join-Path C:\\Users test",
        "Split-Path C:\\Users\\test",
        "Resolve-Path .",
    ] {
        assert!(
            check_all_segments(cmd, &list).is_ok(),
            "PowerShell cmdlet should pass: {cmd}"
        );
    }
}

/// Noun-level safe exceptions pass despite blocked verb prefix.
#[test]
fn powershell_safe_exceptions_pass() {
    let list = allow(&["git"]);
    for cmd in [
        "Set-Location C:\\Users",
        "New-Guid",
        "New-TimeSpan -Hours 1",
        "New-Object System.IO.MemoryStream",
        "Start-Sleep -Seconds 5",
        "Clear-Host",
        "Out-String",
        "Out-Null",
        "Out-Host",
        "Import-Csv data.csv",
        "Import-Clixml backup.xml",
        "Export-Csv results.csv",
    ] {
        assert!(
            check_all_segments(cmd, &list).is_ok(),
            "Safe exception should pass: {cmd}"
        );
    }
}

/// Import-Module is blocked (can execute arbitrary code), but Import-Csv is safe.
#[test]
fn powershell_import_module_blocked_import_csv_safe() {
    let list = allow(&["git"]);
    assert!(check_all_segments("Import-Csv data.csv", &list).is_ok());
    assert!(check_all_segments("Import-Clixml backup.xml", &list).is_ok());
    assert!(check_all_segments("Import-Module ./evil.psm1", &list).is_err());
    assert!(check_all_segments("Import-PSSession $s", &list).is_err());
}

/// Out-File is blocked (writes to disk), but Out-String/Out-Null are safe.
#[test]
fn powershell_out_file_blocked_out_string_safe() {
    let list = allow(&["git"]);
    assert!(check_all_segments("Out-String", &list).is_ok());
    assert!(check_all_segments("Out-Null", &list).is_ok());
    assert!(check_all_segments("Out-Host", &list).is_ok());
    assert!(check_all_segments("Out-File report.txt", &list).is_err());
    assert!(check_all_segments("Out-GridView", &list).is_err());
}

/// Destructive PowerShell cmdlets are blocked.
#[test]
fn powershell_destructive_cmdlets_blocked() {
    let list = allow(&["git", "cargo"]);
    for cmd in [
        "Remove-Item file.txt",
        "Set-Content file.txt -Value hello",
        "Stop-Process -Name notepad",
        "New-Item -Path C:\\test",
        "Start-Process cmd.exe",
        "Invoke-Expression ls",
        "Invoke-Command { rm -rf / }",
        "Clear-Content file.txt",
        "Restart-Service nginx",
    ] {
        let result = check_all_segments(cmd, &list);
        assert!(
            result.is_err(),
            "PowerShell destructive cmdlet should be blocked: {cmd}"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("destructive verb"),
            "error must mention destructive verb: {err}"
        );
    }
}

/// PowerShell aliases (gci, gc, sls, etc.) pass as builtins.
#[test]
fn powershell_aliases_pass_enforcement() {
    let list = allow(&["git"]);
    for cmd in [
        "gci",
        "gc file.txt",
        "sls pattern file.txt",
        "dir",
        "cls",
        "sleep",
        "sl C:\\Users",
        "pwd",
        "echo hello",
        "sort",
        "measure",
    ] {
        assert!(
            check_all_segments(cmd, &list).is_ok(),
            "PowerShell alias should pass: {cmd}"
        );
    }
}

/// On Windows, dangerous PS aliases (iex, rm, kill) that overlap with POSIX names
/// must be blocked to prevent bypassing the verb gate.
#[test]
#[cfg(windows)]
fn powershell_dangerous_aliases_blocked_on_windows() {
    let list = allow(&["git", "cargo"]);
    for cmd in ["iex $code", "rm file.txt", "rmdir folder", "kill 1234"] {
        let result = check_all_segments(cmd, &list);
        assert!(
            result.is_err(),
            "Dangerous PS alias should be blocked on Windows: {cmd}"
        );
    }
}

/// On Unix, rm/kill/mv/cp are real POSIX binaries and pass the standard allowlist.
#[test]
#[cfg(not(windows))]
fn posix_binaries_not_blocked_on_unix() {
    let list = allow(&["git", "rm", "kill", "mv", "cp"]);
    for cmd in ["rm file.txt", "kill 1234", "mv a b", "cp a b"] {
        assert!(
            check_all_segments(cmd, &list).is_ok(),
            "POSIX binary should pass on Unix: {cmd}"
        );
    }
}

/// PowerShell cmdlets in pipelines: all segments must pass.
#[test]
fn powershell_pipeline_all_safe_passes() {
    let list = allow(&["git"]);
    assert!(
        check_all_segments(
            "Get-ChildItem | Where-Object Length -gt 100 | Sort-Object Name | Format-Table",
            &list
        )
        .is_ok()
    );
}

/// Mixed pipeline: safe cmdlet piped to destructive = blocked.
#[test]
fn powershell_pipeline_with_destructive_blocked() {
    let list = allow(&["git"]);
    let result = check_all_segments("Get-ChildItem | Remove-Item", &list);
    assert!(result.is_err());
}

/// Case insensitivity: PowerShell cmdlets are case-insensitive.
#[test]
fn powershell_case_insensitive_enforcement() {
    let list = allow(&["git"]);
    assert!(check_all_segments("get-date", &list).is_ok());
    assert!(check_all_segments("GET-CHILDITEM", &list).is_ok());
    assert!(check_all_segments("test-path C:\\Temp", &list).is_ok());
    assert!(check_all_segments("set-location C:\\Temp", &list).is_ok());
    assert!(check_all_segments("new-guid", &list).is_ok());
    let result = check_all_segments("remove-item file.txt", &list);
    assert!(result.is_err());
}

/// Start-Sleep is safe (same as `sleep` builtin), but Start-Process is blocked.
#[test]
fn powershell_start_sleep_safe_start_process_blocked() {
    let list = allow(&["git"]);
    assert!(check_all_segments("Start-Sleep -Seconds 5", &list).is_ok());
    assert!(check_all_segments("Start-Sleep 10", &list).is_ok());
    assert!(check_all_segments("Start-Process cmd.exe", &list).is_err());
    assert!(check_all_segments("Start-Job { heavy }", &list).is_err());
}
