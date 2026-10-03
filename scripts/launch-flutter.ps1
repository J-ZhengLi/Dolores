param([string]$DataDirectory = '')
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path "$PSScriptRoot/..").Path
$exe = Join-Path $workspace 'apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe'
if (!(Test-Path -LiteralPath $exe)) { throw 'Build the normal Flutter release before launching.' }
$previousData = $env:DOLORES_DATA_DIR
$previousSmoke = $env:DOLORES_SMOKE_DIR
try {
    if ($DataDirectory) {
        if (![IO.Path]::IsPathRooted($DataDirectory)) { throw 'Choose an absolute application data directory.' }
        $env:DOLORES_DATA_DIR = [IO.Path]::GetFullPath($DataDirectory)
    }
    $env:DOLORES_SMOKE_DIR = $null
    $app = Start-Process -FilePath $exe -WorkingDirectory ([IO.Path]::GetDirectoryName($exe)) -WindowStyle Normal -PassThru
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    do {
        Start-Sleep -Milliseconds 200
        $app.Refresh()
        if ($app.HasExited) { throw 'Dolores exited before its window opened.' }
    } while ($app.MainWindowHandle -eq 0 -and [DateTime]::UtcNow -lt $deadline)
    if ($app.MainWindowHandle -eq 0) { throw 'Dolores did not open a desktop window in time.' }
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class DoloresVerificationWindow {
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window, int command);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
}
'@
    [DoloresVerificationWindow]::ShowWindow($app.MainWindowHandle, 9) | Out-Null
    [DoloresVerificationWindow]::SetForegroundWindow($app.MainWindowHandle) | Out-Null
    if (![DoloresVerificationWindow]::IsWindowVisible($app.MainWindowHandle)) { throw 'The app window is not visible.' }
    [pscustomobject]@{ processId = $app.Id; windowVisible = $true }
} finally {
    $env:DOLORES_DATA_DIR = $previousData
    $env:DOLORES_SMOKE_DIR = $previousSmoke
}
