[CmdletBinding()]
param(
    [Parameter()]
    [string]$InstallDir,

    [Parameter()]
    [string]$ReportPath,

    [Parameter()]
    [switch]$NoPause
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Resolve-ChatProExecutable {
    param([string]$RequestedInstallDir)

    $candidates = [System.Collections.Generic.List[string]]::new()
    if (-not [string]::IsNullOrWhiteSpace($RequestedInstallDir)) {
        $candidates.Add($RequestedInstallDir)
    }
    $candidates.Add($PSScriptRoot)
    $candidates.Add((Join-Path $PSScriptRoot '..'))
    $candidates.Add((Join-Path $PSScriptRoot '..\..'))

    try {
        $running = Get-Process -Name 'A.I.Live Chat Pro' -ErrorAction SilentlyContinue |
            Select-Object -First 1 -ExpandProperty Path
        if ($running) {
            $candidates.Add((Split-Path -Parent $running))
        }
    }
    catch {
        Write-Verbose "Could not inspect the running Chat Pro path: $($_.Exception.Message)"
    }

    foreach ($candidate in $candidates) {
        if ([string]::IsNullOrWhiteSpace($candidate)) {
            continue
        }
        try {
            $resolved = [System.IO.Path]::GetFullPath($candidate)
        }
        catch {
            continue
        }
        $exe = Join-Path $resolved 'A.I.Live Chat Pro.exe'
        if (Test-Path -LiteralPath $exe -PathType Leaf) {
            return [pscustomobject]@{
                InstallDir = $resolved
                Executable = $exe
            }
        }
    }

    if (-not [string]::IsNullOrWhiteSpace($RequestedInstallDir)) {
        throw "A.I.Live Chat Pro.exe was not found in the requested folder: $RequestedInstallDir"
    }

    Add-Type -AssemblyName System.Windows.Forms
    $dialog = [System.Windows.Forms.FolderBrowserDialog]::new()
    $dialog.Description = 'Select the A.I.Live Chat Pro installation folder'
    $dialog.ShowNewFolderButton = $false
    if ($dialog.ShowDialog() -ne [System.Windows.Forms.DialogResult]::OK) {
        throw 'No installation folder was selected. Repair cancelled.'
    }
    $selected = [System.IO.Path]::GetFullPath($dialog.SelectedPath)
    $selectedExe = Join-Path $selected 'A.I.Live Chat Pro.exe'
    if (-not (Test-Path -LiteralPath $selectedExe -PathType Leaf)) {
        throw "A.I.Live Chat Pro.exe was not found in the selected folder: $selected"
    }
    return [pscustomobject]@{
        InstallDir = $selected
        Executable = $selectedExe
    }
}

try {
    $target = Resolve-ChatProExecutable -RequestedInstallDir $InstallDir
    if ([string]::IsNullOrWhiteSpace($ReportPath)) {
        $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
        $reportRoot = Join-Path ([System.IO.Path]::GetTempPath()) 'OCLive\repair'
        [System.IO.Directory]::CreateDirectory($reportRoot) | Out-Null
        $ReportPath = Join-Path $reportRoot "installation-repair-$stamp.json"
    }
    else {
        $ReportPath = [System.IO.Path]::GetFullPath($ReportPath)
        $reportParent = Split-Path -Parent $ReportPath
        if ($reportParent) {
            [System.IO.Directory]::CreateDirectory($reportParent) | Out-Null
        }
    }

    Write-Host 'A.I.Live Chat Pro Safe Repair' -ForegroundColor Cyan
    Write-Host "Install directory: $($target.InstallDir)"
    Write-Host "Report path: $ReportPath"

    $arguments = @(
        '--repair-installation',
        '--repair-resource-dir', "`"$($target.InstallDir)`"",
        '--repair-report', "`"$ReportPath`""
    )
    $process = Start-Process -FilePath $target.Executable `
        -ArgumentList $arguments `
        -Wait `
        -PassThru `
        -WindowStyle Hidden

    if (-not (Test-Path -LiteralPath $ReportPath -PathType Leaf)) {
        throw "Repair exited with code $($process.ExitCode) but did not create a report: $ReportPath"
    }

    $report = Get-Content -LiteralPath $ReportPath -Raw -Encoding UTF8 | ConvertFrom-Json
    Write-Host ''
    Write-Host "Role packs: $($report.roleCount); directory plugins: $($report.pluginCount)"
    foreach ($action in $report.actions) {
        $color = if ($action.status -eq 'failed') { 'Red' } elseif ($action.status -eq 'repaired') { 'Yellow' } else { 'Green' }
        Write-Host "[$($action.status)] $($action.code) - $($action.summary)" -ForegroundColor $color
        if ($action.detail) {
            Write-Host "  $($action.detail)" -ForegroundColor DarkGray
        }
    }
    foreach ($issue in $report.issues) {
        $color = if ($issue.severity -eq 'error') { 'Red' } else { 'Yellow' }
        Write-Host "[$($issue.scope)/$($issue.category)/$($issue.severity)] $($issue.code) - $($issue.summary)" -ForegroundColor $color
        if ($issue.detail) {
            Write-Host "  $($issue.detail)" -ForegroundColor DarkGray
        }
        if ($issue.path) {
            Write-Host "  Path: $($issue.path)" -ForegroundColor DarkGray
        }
    }

    Write-Host ''
    if ($report.success) {
        Write-Host 'Repair and verification passed. Restart Chat Pro.' -ForegroundColor Green
    }
    else {
        Write-Host 'Safe actions completed, but reinstall or manual attention is still required.' -ForegroundColor Red
    }
    Write-Host "Keep and attach this report when requesting support: $ReportPath" -ForegroundColor Cyan

    if (-not $NoPause) {
        Read-Host 'Press Enter to close'
    }
    exit $process.ExitCode
}
catch {
    Write-Host "[REPAIR_WRAPPER_FAILED] $($_.Exception.Message)" -ForegroundColor Red
    if (-not $NoPause) {
        Read-Host 'Press Enter to close'
    }
    exit 2
}
