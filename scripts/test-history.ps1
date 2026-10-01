param([string]$OutputDirectory = "$PSScriptRoot/../output/history/smoke", [switch]$Compact)
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path "$PSScriptRoot/..").Path
$directory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $directory) { throw 'Choose a fresh isolated output directory.' }
$data = Join-Path $directory 'data'
& rtk proxy node "$PSScriptRoot/seed-history-fixture.mjs" $data
if ($LASTEXITCODE) { throw 'History fixture failed.' }
$oldData = $env:DOLORES_DATA_DIR
$oldSmoke = $env:DOLORES_SMOKE_DIR
$process = $null
try {
    $env:DOLORES_DATA_DIR = $data
    $env:DOLORES_SMOKE_DIR = $directory
    $exe = Join-Path $workspace 'apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe'
    $arguments = @{FilePath=$exe;WorkingDirectory=[IO.Path]::GetDirectoryName($exe);WindowStyle='Hidden';PassThru=$true}
    if ($Compact) { $arguments.ArgumentList = '--compact' }
    $process = Start-Process @arguments
    $deadline = [DateTime]::UtcNow.AddSeconds(45)
    $report = Join-Path $directory 'report.json'
    while (!(Test-Path -LiteralPath $report)) {
        $process.Refresh()
        if ($process.HasExited) { throw 'History diagnostic exited before its report.' }
        if ([DateTime]::UtcNow -gt $deadline) { throw 'History diagnostic timed out. Build with -HistorySmoke first.' }
        Start-Sleep -Milliseconds 250
    }
    $result = Get-Content -Raw -LiteralPath $report | ConvertFrom-Json
    if (!$result.ok) { throw $result.error }
    $result | ConvertTo-Json -Depth 5
} finally {
    if ($null -ne $process -and !$process.HasExited) { Stop-Process -Id $process.Id }
    $env:DOLORES_DATA_DIR = $oldData
    $env:DOLORES_SMOKE_DIR = $oldSmoke
}
