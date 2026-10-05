param([Parameter(Mandatory=$true)][string]$Directory)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$target = [IO.Path]::GetFullPath($Directory)
$outputRoot = [IO.Path]::GetFullPath((Join-Path $repo 'output')) + [IO.Path]::DirectorySeparatorChar
if (-not [IO.Path]::IsPathRooted($Directory) -or -not $target.StartsWith($outputRoot, [StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $target)) {
    throw 'A fresh absolute directory inside output is required.'
}
$binary = Join-Path $repo 'apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe'
if (-not (Test-Path -LiteralPath $binary)) { throw 'Build the normal release bundle first.' }
New-Item -ItemType Directory -Path $target | Out-Null
$previous = $env:DOLORES_DATA_DIR
$process = $null
try {
    $env:DOLORES_DATA_DIR = Join-Path $target 'data'
    $clock = [Diagnostics.Stopwatch]::StartNew()
    # The caller explicitly requests a visible normal desktop qualification run.
    $process = Start-Process -FilePath $binary -WindowStyle Normal -PassThru
    do {
        Start-Sleep -Milliseconds 50
        $process.Refresh()
        if ($process.HasExited) { throw 'The normal release exited before exposing a window.' }
    } while ($process.MainWindowHandle -eq 0 -and $clock.ElapsedMilliseconds -lt 15000)
    if ($process.MainWindowHandle -eq 0) { throw 'No desktop window was detected within 15 seconds.' }
    $windowMs = $clock.ElapsedMilliseconds
    Start-Sleep -Seconds 3
    $process.Refresh()
    $result = [ordered]@{
        windowPresenceMs = $windowMs
        idleWorkingBytes = $process.WorkingSet64
        idlePrivateBytes = $process.PrivateMemorySize64
        limitation = 'One isolated normal release launch; window presence is not first usable frame. Idle memory excludes active capture. Not a low-end qualification.'
    }
    $result | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $target 'public-result.json') -Encoding UTF8
    $process.Id | Set-Content -LiteralPath (Join-Path $target 'owned.pid')
    $result | ConvertTo-Json
    # Leave the owned normal window visible for subsequent visual inspection.
} catch {
    if ($null -ne $process -and -not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
    throw
} finally { $env:DOLORES_DATA_DIR = $previous }
