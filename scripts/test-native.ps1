param(
    [string]$DataDirectory,
    [string]$OutputDirectory,
    [int]$TimeoutSeconds = 60
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$executable = Join-Path $root 'target/release/dolores-native.exe'
if (!(Test-Path -LiteralPath $executable)) { throw 'First build dolores-native in release mode with the smoke feature.' }
if ($TimeoutSeconds -lt 1 -or $TimeoutSeconds -gt 60) { throw 'TimeoutSeconds must be 1 to 60.' }
if (!$DataDirectory) { $DataDirectory = Join-Path $root ("output/native-smoke-" + [guid]::NewGuid().ToString('N')) }
if (!$OutputDirectory) { $OutputDirectory = Join-Path $DataDirectory 'report' }
foreach ($directory in @($DataDirectory, $OutputDirectory)) {
    if (![IO.Path]::IsPathRooted($directory)) { throw 'Use absolute directories.' }
}
if (Test-Path -LiteralPath (Join-Path $DataDirectory 'dolores.db')) { throw 'Use a fresh data directory; this runner creates fixture conversations.' }
$reportPath = Join-Path $OutputDirectory 'report.json'
if (Test-Path -LiteralPath $reportPath) { throw 'Use a fresh report directory.' }
$oldData = $env:DOLORES_DATA_DIR
$oldSmoke = $env:DOLORES_SMOKE_DIR
$app = $null
try {
    $env:DOLORES_DATA_DIR = $DataDirectory
    $env:DOLORES_SMOKE_DIR = $OutputDirectory
    $app = Start-Process -FilePath $executable -WindowStyle Hidden -PassThru
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while (!(Test-Path -LiteralPath $reportPath)) {
        $app.Refresh()
        if ($app.HasExited) { throw 'Native runner exited before completing. Check the fixture server and use a smoke-feature build.' }
        if ([DateTime]::UtcNow -ge $deadline) { throw 'Native runner timed out. Check the fixture server and use a smoke-feature build.' }
        Start-Sleep -Milliseconds 500
    }
    Get-Content -LiteralPath $reportPath
    Write-Output "Renderer screenshots: $OutputDirectory"
}
finally {
    if ($app) {
        $app.Refresh()
        if (!$app.HasExited -and $app.ProcessName -eq 'dolores-native' -and $app.Path -eq $executable) { Stop-Process -Id $app.Id }
    }
    $env:DOLORES_DATA_DIR = $oldData
    $env:DOLORES_SMOKE_DIR = $oldSmoke
}
