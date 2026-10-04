$ErrorActionPreference = 'Stop'
$processes = @(Get-CimInstance Win32_Process)
$ownedIds = @($processes | Where-Object { $_.Name -eq 'node.exe' -and $_.CommandLine -match '[\\/]browser\.cjs' } | ForEach-Object ProcessId)
$roots = $ownedIds.Count
do {
    $before = $ownedIds.Count
    $ownedIds = @($ownedIds + @($processes | Where-Object { $_.ParentProcessId -in $ownedIds } | ForEach-Object ProcessId) | Select-Object -Unique)
} while ($ownedIds.Count -gt $before)
$memory = ($processes | Where-Object { $_.ProcessId -in $ownedIds } | Measure-Object WorkingSetSize -Sum).Sum
[pscustomobject]@{workers = $roots; processes = $ownedIds.Count; workingSetMiB = [Math]::Round(([double]$memory / 1MB), 2)} | ConvertTo-Json -Compress
