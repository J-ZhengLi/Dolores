param([Parameter(Mandatory = $true)][string]$DataDirectory, [Parameter(Mandatory = $true)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path "$PSScriptRoot/..").Path
$source = (Resolve-Path -LiteralPath $DataDirectory).Path
$destination = [IO.Path]::GetFullPath($OutputDirectory)
if (!(Test-Path "$source/dolores.db")) { throw 'Provide a completed fixture database, with its application closed.' }
if (Test-Path $destination) { throw 'Provide a fresh comparison output directory.' }
New-Item -ItemType Directory -Path $destination | Out-Null
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class DoloresComparisonWindow {
 [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out Rect rect);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
}
'@
$oldData = $env:DOLORES_DATA_DIR
$oldSmoke = $env:DOLORES_SMOKE_DIR
$oldBrowser = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$reports = @()
$oldDpiContext = [DoloresComparisonWindow]::SetThreadDpiAwarenessContext([IntPtr](-4))
if ($oldDpiContext -eq [IntPtr]::Zero) { throw 'Could not disable DPI virtualization for window measurements.' }
try {
    $env:DOLORES_SMOKE_DIR = $null
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $null
    foreach ($name in @('dolores-native', 'dolores-desktop', 'dolores_flutter')) {
        $data = Join-Path $destination "$name-data"
        New-Item -ItemType Directory -Path $data | Out-Null
        Get-ChildItem -LiteralPath $source -File -Filter 'dolores.db*' | Copy-Item -Destination $data
        $env:DOLORES_DATA_DIR = $data
        $exe = if ($name -eq 'dolores_flutter') { Join-Path $workspace 'apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe' } else { Join-Path $workspace "target/release/$name.exe" }
        # These are the interactive applications under comparison, not helpers.
        $app = Start-Process -FilePath $exe -WorkingDirectory ([IO.Path]::GetDirectoryName($exe)) -WindowStyle Normal -PassThru
        try {
            Start-Sleep -Seconds 8
            $app.Refresh()
            $handle = $app.MainWindowHandle
            $rect = New-Object DoloresComparisonWindow+Rect
            if (![DoloresComparisonWindow]::GetClientRect($handle, [ref]$rect)) { throw 'Could not inspect the application window.' }
            $dpi = [DoloresComparisonWindow]::GetDpiForWindow($handle)
            $width = [math]::Round(($rect.Right - $rect.Left) * 96 / $dpi, 2)
            $height = [math]::Round(($rect.Bottom - $rect.Top) * 96 / $dpi, 2)
            if (![DoloresComparisonWindow]::IsWindowVisible($handle) -or [DoloresComparisonWindow]::IsIconic($handle)) { throw 'Application must be visible and unminimized.' }
            if ([math]::Abs($width - 1120) -gt 1 -or [math]::Abs($height - 780) -gt 1) { throw "Expected 1120 x 780 client area; $name is $width x $height." }
            $metrics = (& "$PSScriptRoot/measure-runtime.ps1" -AppProcessId $app.Id -Samples 5) | ConvertFrom-Json
            $metrics | Add-Member NoteProperty window @{ visible = $true; minimized = $false; dpi = $dpi; clientWidth = $width; clientHeight = $height }
            $bundleBytes = if ($name -eq 'dolores_flutter') { (Get-ChildItem -LiteralPath ([IO.Path]::GetDirectoryName($exe)) -Recurse -File | Where-Object Extension -notin @('.pdb', '.lib', '.exp') | Measure-Object Length -Sum).Sum } else { (Get-Item -LiteralPath $exe).Length }
            $metrics | Add-Member NoteProperty runtimeBundleBytes $bundleBytes
            $reports += $metrics
            [IO.File]::WriteAllText((Join-Path $destination "$name.json"), ($metrics | ConvertTo-Json -Depth 8), (New-Object Text.UTF8Encoding($false)))
            $metrics | Select-Object app, meanWorkingSetMiB, meanPrivateBytesMiB, meanOneCoreCpuPercent, runtimeBundleBytes | Format-List
        } finally {
            $app.Refresh()
            if (!$app.HasExited -and $app.ProcessName -eq $name -and $app.Path -eq $exe) { Stop-Process -Id $app.Id }
        }
    }
    [IO.File]::WriteAllText((Join-Path $destination 'comparison.json'), ($reports | ConvertTo-Json -Depth 8), (New-Object Text.UTF8Encoding($false)))
} finally {
    $env:DOLORES_DATA_DIR = $oldData
    $env:DOLORES_SMOKE_DIR = $oldSmoke
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $oldBrowser
    [void][DoloresComparisonWindow]::SetThreadDpiAwarenessContext($oldDpiContext)
}
