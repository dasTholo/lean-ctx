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
        "Out-String",
        "ConvertTo-Json",
        "ConvertFrom-Csv",
        "Sort-Object Name",
        "Where-Object Length -gt 100",
        "ForEach-Object { $_.Name }",
        "Compare-Object $a $b",
        "Join-Path C:\\Users test",
        "Split-Path C:\\Users\\test",
        "Resolve-Path .",
        "Import-Csv data.csv",
        "Export-Csv out.csv",
    ] {
        assert!(
            check_all_segments(cmd, &list).is_ok(),
            "PowerShell cmdlet should pass: {cmd}"
        );
    }
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
    for cmd in ["gci", "gc file.txt", "sls pattern file.txt", "dir", "cls"] {
        assert!(
            check_all_segments(cmd, &list).is_ok(),
            "PowerShell alias should pass: {cmd}"
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
    let result = check_all_segments("remove-item file.txt", &list);
    assert!(result.is_err());
}
