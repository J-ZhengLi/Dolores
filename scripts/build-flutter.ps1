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
    # Keep caller flags, but remove machine-specific paths from released code.
    # Encoded arguments also support user/workspace paths containing spaces.
    $savedEncodedFlags = $env:CARGO_ENCODED_RUSTFLAGS
    $buildFlags = @()
    if ($savedEncodedFlags) { $buildFlags += $savedEncodedFlags.Split([char]31) }
    elseif ($env:RUSTFLAGS) { $buildFlags += $env:RUSTFLAGS.Split(' ', [StringSplitOptions]::RemoveEmptyEntries) }
    foreach ($mapping in @(
        @{ Path = $workspace; Replacement = '/dolores' },
        @{ Path = [Environment]::GetFolderPath('UserProfile'); Replacement = '/build-user' },
        @{ Path = $env:CARGO_HOME; Replacement = '/cargo' },
        @{ Path = $env:RUSTUP_HOME; Replacement = '/rustup' }
    )) {
        if ($mapping.Path) {
            foreach ($prefix in @($mapping.Path, $mapping.Path.Replace('\', '/')) | Select-Object -Unique) {
                $buildFlags += '--remap-path-prefix'
                $buildFlags += "$prefix=$($mapping.Replacement)"
            }
        }
    }
    try {
        $env:CARGO_ENCODED_RUSTFLAGS = $buildFlags -join [char]31
        & cargo build -p dolores-flutter-bridge -p dolores-desktop-helper --release --locked
        if ($LASTEXITCODE) { throw 'Rust bridge build failed.' }
    } finally { $env:CARGO_ENCODED_RUSTFLAGS = $savedEncodedFlags }
    # Keep matching private debug symbols per build, outside the runtime bundle.
    $symbols = Join-Path $workspace ('output/release-symbols/' + [guid]::NewGuid().ToString('N'))
    $releaseArguments = @("--split-debug-info=$symbols")
    Push-Location 'apps/dolores_flutter'
    $savedPackageConfig = $null
    $packageConfigPath = Join-Path (Get-Location).Path '.dart_tool/package_config.json'
    try {
        try {
            $ErrorActionPreference = 'Continue'
            $dependencies = (& $flutter --no-version-check --suppress-analytics pub get 2>&1 | ForEach-Object { $_.ToString() } | Out-String)
            $dependenciesExit = $LASTEXITCODE
        } finally { $ErrorActionPreference = 'Stop' }
        if ($dependenciesExit -and $dependencies -match 'Building with plugins requires symlink support') {
            & "$PSScriptRoot/flutter-plugin-junctions.ps1" -AppDirectory (Get-Location).Path
            & $flutter --no-version-check --suppress-analytics pub get
        } else {
            Write-Output $dependencies
            if ($dependenciesExit) { throw 'Flutter dependencies failed.' }
        }
        if ($LASTEXITCODE) { throw 'Flutter dependencies failed.' }
        # A successful pub get can leave new plugin links absent, while a later
        # build's implicit pub get can remove generated junctions. Prepare links
        # after dependency resolution and avoid regenerating them during builds.
        & "$PSScriptRoot/flutter-plugin-junctions.ps1" -AppDirectory (Get-Location).Path
        # Flutter embeds its generated registrant URI as an engine entry point.
        # Give only that generated directory a stable package URI during compilation;
        # otherwise it falls outside lib/ and embeds an absolute source-machine path.
        # This adds no dependency and leaves the SDK/generated source unmodified.
        $savedPackageConfig = [IO.File]::ReadAllBytes($packageConfigPath)
        $packageConfig = Get-Content -LiteralPath $packageConfigPath -Raw | ConvertFrom-Json
        if (@($packageConfig.packages | Where-Object name -eq 'dolores_build_registrant').Count) {
            throw 'Generated registrant package alias is already present; review the package configuration.'
        }
        $languageVersion = ($packageConfig.packages | Where-Object name -eq 'dolores_flutter').languageVersion
        $packageConfig.packages = @($packageConfig.packages) + @([pscustomobject]@{
            name = 'dolores_build_registrant'; rootUri = 'flutter_build/'; packageUri = './'; languageVersion = $languageVersion
        })
        [IO.File]::WriteAllText($packageConfigPath, ($packageConfig | ConvertTo-Json -Depth 100), [Text.UTF8Encoding]::new($false))
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
                $configuration = (& $flutter --no-version-check --suppress-analytics build windows --release --no-pub @releaseArguments --config-only --target $entry 2>&1 | Out-String)
                $configurationExit = $LASTEXITCODE
            } finally { $ErrorActionPreference = 'Stop' }
            if ($configurationExit -and $configuration -notmatch 'Unable to find suitable Visual Studio toolchain') { throw $configuration }
            if (!(Test-Path 'windows/flutter/ephemeral/generated_config.cmake')) { throw 'Flutter did not generate its build configuration.' }
            & $cmakeExecutable -S windows -B build/windows/x64 -G $Generator -A x64 -DFLUTTER_TARGET_PLATFORM=windows-x64
            if ($LASTEXITCODE) { throw 'CMake configuration failed.' }
            & $cmakeExecutable --build build/windows/x64 --config Release --target INSTALL
        } else {
            & $flutter --no-version-check --suppress-analytics build windows --release --no-pub @releaseArguments --target $entry
        }
        if ($LASTEXITCODE) { throw 'Flutter desktop build failed.' }
        $bundle = Join-Path $workspace 'apps/dolores_flutter/build/windows/x64/runner/Release'
        Copy-Item -LiteralPath "$workspace/target/release/dolores_flutter_bridge.dll" -Destination $bundle
        Copy-Item -LiteralPath "$workspace/target/release/dolores-desktop-helper.exe" -Destination $bundle
        Write-Output "Built $bundle/dolores_flutter.exe (ship the entire directory)."
    } finally {
        if ($null -ne $savedPackageConfig) { [IO.File]::WriteAllBytes($packageConfigPath, $savedPackageConfig) }
        Pop-Location
    }
} finally { Pop-Location }
