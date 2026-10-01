param([string]$OutputDirectory = "$PSScriptRoot/../output/connection-restart")
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path "$PSScriptRoot/..").Path
$directory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path $directory) { throw 'Use a fresh output directory.' }
$data = Join-Path $directory 'data'
$exe = Join-Path $workspace 'apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe'
$oldData = $env:DOLORES_DATA_DIR
$oldSmoke = $env:DOLORES_SMOKE_DIR
$oldPhase = $env:DOLORES_RESTART_PHASE
$saved = $false
$forgotten = $false
function Run-Phase([int]$phase, [string]$reportDirectory) {
    $env:DOLORES_RESTART_PHASE = "$phase"
    $env:DOLORES_SMOKE_DIR = $reportDirectory
    $report = Join-Path $reportDirectory "phase-$phase.json"
    $process = Start-Process -FilePath $exe -WorkingDirectory ([IO.Path]::GetDirectoryName($exe)) -WindowStyle Hidden -PassThru
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(20)
        while (!(Test-Path $report)) {
            $process.Refresh()
            if ($process.HasExited -or [DateTime]::UtcNow -gt $deadline) { throw "Restart phase $phase failed to report. Build with -RestartSmoke first." }
            Start-Sleep -Milliseconds 200
        }
        $result = Get-Content -Raw -LiteralPath $report | ConvertFrom-Json
        if (!$result.ok) { throw $result.error }
        return $result
    } finally {
        if (!$process.HasExited) { Stop-Process -Id $process.Id }
    }
}
try {
    $env:DOLORES_DATA_DIR = $data
    $results = @()
    # Assume a write might occur even if phase 1 fails to report; finally attempts cleanup.
    $saved = $true
    $results += Run-Phase 1 $directory
    $results += Run-Phase 2 $directory
    $results += Run-Phase 3 $directory
    $forgotten = $true
    $results += Run-Phase 4 $directory
    foreach ($file in Get-ChildItem -LiteralPath $data -File) {
        $bytes = [IO.File]::ReadAllBytes($file.FullName)
        if ([Text.Encoding]::UTF8.GetString($bytes).Contains('dolores-generated-restart-test')) { throw 'Generated test key appeared in a data file.' }
    }
    [ordered]@{ ok = $true; phases = $results; keyAbsentFromDataFiles = $true } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $directory 'report.json') -Encoding UTF8
    Get-Content -LiteralPath (Join-Path $directory 'report.json')
} finally {
    if ($saved -and !$forgotten) {
        try { Run-Phase 3 (Join-Path $directory 'cleanup') | Out-Null }
        catch { Write-Warning "Generated test credential cleanup needs attention: $_" }
    }
    $env:DOLORES_DATA_DIR = $oldData
    $env:DOLORES_SMOKE_DIR = $oldSmoke
    $env:DOLORES_RESTART_PHASE = $oldPhase
}
