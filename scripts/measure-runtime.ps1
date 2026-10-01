param(
    [Parameter(Mandatory = $true)][int]$AppProcessId,
    [int]$Samples = 5
)
$ErrorActionPreference = 'Stop'
if ($Samples -lt 1 -or $Samples -gt 30) { throw 'Choose 1 to 30 samples.' }
$appProcess = Get-Process -Id $AppProcessId
if ($appProcess.ProcessName -ne 'dolores-desktop') { throw 'The process must be the Dolores desktop executable.' }
$measurements = @()
for ($sample = 0; $sample -lt $Samples; $sample++) {
    $allProcesses = @(Get-CimInstance Win32_Process)
    $processIds = @($AppProcessId)
    do {
        $children = @($allProcesses | Where-Object { $_.ParentProcessId -in $processIds -and $_.ProcessId -notin $processIds } | ForEach-Object { [int]$_.ProcessId })
        $processIds += $children
    } while ($children.Count -gt 0)
    $processes = @(Get-Process -Id $processIds -ErrorAction SilentlyContinue)
    $measurements += [pscustomobject]@{
        processCount = $processes.Count
        workingSetMiB = [math]::Round(($processes | Measure-Object -Property WorkingSet64 -Sum).Sum / 1MB, 2)
        privateBytesMiB = [math]::Round(($processes | Measure-Object -Property PrivateMemorySize64 -Sum).Sum / 1MB, 2)
    }
    if ($sample -lt $Samples - 1) { Start-Sleep -Seconds 1 }
}
$system = Get-CimInstance Win32_OperatingSystem
$processor = Get-CimInstance Win32_Processor | Select-Object -First 1
[pscustomobject]@{
    os = $system.Caption
    cpu = $processor.Name
    logicalProcessors = $processor.NumberOfLogicalProcessors
    systemMemoryGiB = [math]::Round($system.TotalVisibleMemorySize / 1MB, 2)
    executableBytes = (Get-Item -LiteralPath $appProcess.Path).Length
    processStartUnixMs = ([DateTimeOffset]$appProcess.StartTime.ToUniversalTime()).ToUnixTimeMilliseconds()
    samples = $measurements
    meanWorkingSetMiB = [math]::Round(($measurements | Measure-Object -Property workingSetMiB -Average).Average, 2)
    note = 'Summed process-tree working sets may double-count shared pages. Browser/model/dev-server processes outside this tree are excluded.'
} | ConvertTo-Json -Depth 4
