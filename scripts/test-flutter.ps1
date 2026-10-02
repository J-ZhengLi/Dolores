param([string]$OutputDirectory = "$PSScriptRoot/../output/flutter-comparison/smoke", [int]$TimeoutSeconds = 40, [switch]$Compact, [string]$FixtureUrl = 'http://127.0.0.1:19421/v1')
$ErrorActionPreference = 'Stop'
if ($TimeoutSeconds -lt 10 -or $TimeoutSeconds -gt 120) { throw 'Use a timeout from 10 to 120 seconds.' }
$workspace = (Resolve-Path "$PSScriptRoot/..").Path
$directory = [IO.Path]::GetFullPath($OutputDirectory)
$data = Join-Path $directory 'data'
$report = Join-Path $directory 'report.json'
if ((Test-Path $report) -or (Test-Path "$data/dolores.db")) { throw 'Use a fresh output directory so existing data is not modified.' }
$exe = Join-Path $workspace 'apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe'
$oldData = $env:DOLORES_DATA_DIR
$oldSmoke = $env:DOLORES_SMOKE_DIR
$oldProvider = $env:DOLORES_SMOKE_PROVIDER
$process = $null
try {
    $env:DOLORES_DATA_DIR = $data
    $env:DOLORES_SMOKE_DIR = $directory
    $env:DOLORES_SMOKE_PROVIDER = $FixtureUrl
    $startArguments = @{ FilePath = $exe; WorkingDirectory = [IO.Path]::GetDirectoryName($exe); WindowStyle = 'Hidden'; PassThru = $true }
    if ($Compact) { $startArguments.ArgumentList = '--compact' }
    $process = Start-Process @startArguments
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while (!(Test-Path $report)) {
        if ($process.HasExited) { throw "Flutter exited with code $($process.ExitCode) before producing its report." }
        if ([DateTime]::UtcNow -gt $deadline) { throw 'Flutter smoke test timed out. Build with -Smoke first.' }
        Start-Sleep -Milliseconds 250
        $process.Refresh()
    }
    $result = Get-Content -Raw -LiteralPath $report | ConvertFrom-Json
    if (!$result.ok) { throw $result.error }
    $expectedWidth = if ($Compact) { 620 } else { 1120 }
    if ([math]::Abs($result.logicalWidth - $expectedWidth) -gt 1) { throw "Unexpected client width: $($result.logicalWidth)." }
    $result | ConvertTo-Json -Depth 5
} finally {
    if ($null -ne $process -and !$process.HasExited) { Stop-Process -Id $process.Id }
    $env:DOLORES_DATA_DIR = $oldData
    $env:DOLORES_SMOKE_DIR = $oldSmoke
    $env:DOLORES_SMOKE_PROVIDER = $oldProvider
}
