param([Parameter(Mandatory = $true)][string]$Destination)
$ErrorActionPreference = 'Stop'
$workspace = Split-Path $PSScriptRoot -Parent
$destinationPath = [IO.Path]::GetFullPath($Destination)
$adapterSource = Join-Path $workspace 'adapters/browser'
if ($destinationPath -eq $adapterSource) { throw 'Choose a separate desktop browser-adapter directory.' }
New-Item -ItemType Directory -Path $destinationPath -Force | Out-Null
Copy-Item -LiteralPath "$adapterSource/package.json", "$adapterSource/package-lock.json" -Destination $destinationPath
Push-Location $destinationPath
try {
    & npm.cmd ci --ignore-scripts --no-audit --no-fund
    if ($LASTEXITCODE) { throw 'Optional adapter installation failed; browser remains unavailable.' }
} finally { Pop-Location }
Write-Output 'Browser adapter installed. Node and Edge/Chrome must already be installed. Restart Dolores to refresh its environment; no existing browser profile will be used.'
