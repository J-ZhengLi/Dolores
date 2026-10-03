param(
    [string]$FlutterSdk = '',
    [string]$CMake = '',
    [string]$Generator = 'Visual Studio 18 2026',
    [switch]$Smoke,
    [switch]$RestartSmoke,
    [switch]$HistorySmoke
)
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path "$PSScriptRoot/..").Path
if (!$FlutterSdk) { $FlutterSdk = $env:FLUTTER_SDK }
if (!$FlutterSdk -and (Test-Path "$workspace/output/toolchains/flutter/bin/flutter.bat")) {
    $FlutterSdk = "$workspace/output/toolchains/flutter"
}
if ($FlutterSdk) {
    $sdk = (Resolve-Path -LiteralPath $FlutterSdk).Path
    $flutter = Join-Path $sdk 'bin/flutter.bat'
} else {
    $flutter = (Get-Command flutter.bat -ErrorAction Stop).Source
}
$env:FLUTTER_SUPPRESS_ANALYTICS = 'true'
$env:DART_SUPPRESS_ANALYTICS = 'true'
if (@($Smoke, $RestartSmoke, $HistorySmoke).Where({ $_ }).Count -gt 1) { throw 'Choose one diagnostic entry point.' }
if (!$CMake -and (Test-Path "$workspace/output/toolchains/cmake-4.4.3-windows-x86_64/bin/cmake.exe")) {
    $CMake = "$workspace/output/toolchains/cmake-4.4.3-windows-x86_64/bin/cmake.exe"
}
$entry = if ($Smoke) { 'lib/smoke.dart' } elseif ($RestartSmoke) { 'lib/restart_smoke.dart' } elseif ($HistorySmoke) { 'lib/history_smoke.dart' } else { 'lib/main.dart' }
Push-Location $workspace
try {
    & rtk proxy cargo build -p dolores-flutter-bridge --release --locked
    if ($LASTEXITCODE) { throw 'Rust bridge build failed.' }
    Push-Location 'apps/dolores_flutter'
    try {
        try {
            $ErrorActionPreference = 'Continue'
            $dependencies = (& rtk proxy $flutter --no-version-check --suppress-analytics pub get 2>&1 | ForEach-Object { $_.ToString() } | Out-String)
            $dependenciesExit = $LASTEXITCODE
        } finally { $ErrorActionPreference = 'Stop' }
        if ($dependenciesExit -and $dependencies -match 'Building with plugins requires symlink support') {
            & "$PSScriptRoot/flutter-plugin-junctions.ps1" -AppDirectory (Get-Location).Path
            & rtk proxy $flutter --no-version-check --suppress-analytics pub get
        } else {
            Write-Output $dependencies
            if ($dependenciesExit) { throw 'Flutter dependencies failed.' }
        }
        if ($LASTEXITCODE) { throw 'Flutter dependencies failed.' }
        # A successful pub get can leave new plugin links absent, while a later
        # build's implicit pub get can remove generated junctions. Prepare links
        # after dependency resolution and avoid regenerating them during builds.
        & "$PSScriptRoot/flutter-plugin-junctions.ps1" -AppDirectory (Get-Location).Path
        # Release asset targets share build/flutter_assets across entry points.
        # A cached icon subset can otherwise survive new Dart IconData usages,
        # leaving visible controls blank. Rebuild only the generated asset stage;
        # keep code/compiler caches and icon tree shaking enabled.
        $assetCache = Join-Path (Get-Location).Path '.dart_tool/flutter_build'
        if (Test-Path -LiteralPath $assetCache) {
            Get-ChildItem -LiteralPath $assetCache -Directory | ForEach-Object {
                Get-ChildItem -LiteralPath $_.FullName -Filter 'release_bundle_windows*_assets.stamp' -File | ForEach-Object {
                    Remove-Item -LiteralPath $_.FullName
                }
            }
        }
        if ($CMake) {
            $cmakeExecutable = (Resolve-Path -LiteralPath $CMake).Path
            # Flutter writes the same generated configuration before checking
            # Visual Studio's bundled CMake. Use a portable CMake when that optional
            # installer component is absent. The SDK and engine remain unmodified.
            try {
                $ErrorActionPreference = 'Continue'
                $configuration = (& rtk proxy $flutter --no-version-check --suppress-analytics build windows --release --no-pub --config-only --target $entry 2>&1 | Out-String)
                $configurationExit = $LASTEXITCODE
            } finally { $ErrorActionPreference = 'Stop' }
            if ($configurationExit -and $configuration -notmatch 'Unable to find suitable Visual Studio toolchain') { throw $configuration }
            if (!(Test-Path 'windows/flutter/ephemeral/generated_config.cmake')) { throw 'Flutter did not generate its build configuration.' }
            & rtk proxy $cmakeExecutable -S windows -B build/windows/x64 -G $Generator -A x64 -DFLUTTER_TARGET_PLATFORM=windows-x64
            if ($LASTEXITCODE) { throw 'CMake configuration failed.' }
            & rtk proxy $cmakeExecutable --build build/windows/x64 --config Release --target INSTALL
        } else {
            & rtk proxy $flutter --no-version-check --suppress-analytics build windows --release --no-pub --target $entry
        }
        if ($LASTEXITCODE) { throw 'Flutter desktop build failed.' }
        $bundle = Join-Path $workspace 'apps/dolores_flutter/build/windows/x64/runner/Release'
        Copy-Item -LiteralPath "$workspace/target/release/dolores_flutter_bridge.dll" -Destination $bundle
        Write-Output "Built $bundle/dolores_flutter.exe (ship the entire directory)."
    } finally { Pop-Location }
} finally { Pop-Location }
