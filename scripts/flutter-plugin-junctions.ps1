param([string]$AppDirectory = "$PSScriptRoot/../apps/dolores_flutter")
$ErrorActionPreference = 'Stop'
$app = (Resolve-Path -LiteralPath $AppDirectory).Path
$metadata = Get-Content -Raw -LiteralPath (Join-Path $app '.flutter-plugins-dependencies') | ConvertFrom-Json
# Flutter reads directory junctions as links on Windows. Only create generated
# project links; never change the SDK, Pub cache or Windows developer settings.
foreach ($platform in @('windows', 'linux')) {
    $links = [IO.Path]::GetFullPath((Join-Path $app "$platform/flutter/ephemeral/.plugin_symlinks"))
    if (!$links.StartsWith($app + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Plugin link directory is outside the app.' }
    New-Item -ItemType Directory -Path $links -Force | Out-Null
    foreach ($plugin in $metadata.plugins.$platform) {
        if ($plugin.name -notmatch '^[a-z][a-z0-9_]*$') { throw 'Invalid plugin name.' }
        $target = (Resolve-Path -LiteralPath $plugin.path).Path
        $link = Join-Path $links $plugin.name
        if (Test-Path -LiteralPath $link) {
            $item = Get-Item -LiteralPath $link -Force
            if (!$item.LinkType -or [IO.Path]::GetFullPath($item.Target[0]).TrimEnd('\') -ne $target.TrimEnd('\')) { throw "Unexpected existing plugin link: $link" }
        } else {
            New-Item -ItemType Junction -Path $link -Target $target | Out-Null
        }
    }
}
