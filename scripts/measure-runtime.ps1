param(
    [Parameter(Mandatory = $true)][int]$AppProcessId,
    [int]$Samples = 5
)
$ErrorActionPreference = 'Stop'
if ($Samples -lt 1 -or $Samples -gt 30) { throw 'Choose 1 to 30 samples.' }
$appProcess = Get-Process -Id $AppProcessId
if ($appProcess.ProcessName -notin @('dolores-desktop', 'dolores-native', 'dolores_flutter')) { throw 'The process must be a Dolores desktop executable.' }
$initialStart = $appProcess.StartTime
$previousCpu = $null
$previousTime = $null
$measurements = @()
for ($sample = 0; $sample -lt $Samples; $sample++) {
    $root = Get-Process -Id $AppProcessId
    if ($root.StartTime -ne $initialStart) { throw 'The application process changed during measurement.' }
    $allProcesses = @(Get-CimInstance Win32_Process)
    $processIds = @($AppProcessId)
    do {
        $children = @($allProcesses | Where-Object { $_.ParentProcessId -in $processIds -and $_.ProcessId -notin $processIds } | ForEach-Object { [int]$_.ProcessId })
        $processIds += $children
    } while ($children.Count -gt 0)
    $processes = @(Get-Process -Id $processIds -ErrorAction SilentlyContinue)
    $cpuSeconds = ($processes | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
    $timestamp = [DateTimeOffset]::UtcNow
    $oneCoreCpuPercent = $null
    if ($null -ne $previousCpu) {
        $oneCoreCpuPercent = [math]::Round([math]::Max([double]0, [double]($cpuSeconds - $previousCpu)) / ($timestamp - $previousTime).TotalSeconds * 100, 2)
    }
    $measurements += [pscustomobject]@{
        processCount = $processes.Count
        workingSetMiB = [math]::Round(($processes | Measure-Object -Property WorkingSet64 -Sum).Sum / 1MB, 2)
        privateBytesMiB = [math]::Round(($processes | Measure-Object -Property PrivateMemorySize64 -Sum).Sum / 1MB, 2)
        oneCoreCpuPercent = $oneCoreCpuPercent
        cumulativeCpuSeconds = [math]::Round($cpuSeconds, 5)
        timestampUnixMs = $timestamp.ToUnixTimeMilliseconds()
    }
    $previousCpu = $cpuSeconds
    $previousTime = $timestamp
    if ($sample -lt $Samples - 1) { Start-Sleep -Seconds 1 }
}
$system = Get-CimInstance Win32_OperatingSystem
$processor = Get-CimInstance Win32_Processor | Select-Object -First 1
$cpuIntervals = @($measurements | Where-Object { $null -ne $_.oneCoreCpuPercent })
[pscustomobject]@{
    os = $system.Caption
    app = $appProcess.ProcessName
    cpu = $processor.Name
    logicalProcessors = $processor.NumberOfLogicalProcessors
    systemMemoryGiB = [math]::Round($system.TotalVisibleMemorySize / 1MB, 2)
    executableBytes = (Get-Item -LiteralPath $appProcess.Path).Length
    processStartUnixMs = ([DateTimeOffset]$appProcess.StartTime.ToUniversalTime()).ToUnixTimeMilliseconds()
    samples = $measurements
    meanWorkingSetMiB = [math]::Round(($measurements | Measure-Object -Property workingSetMiB -Average).Average, 2)
    meanPrivateBytesMiB = [math]::Round(($measurements | Measure-Object -Property privateBytesMiB -Average).Average, 2)
    meanOneCoreCpuPercent = if ($cpuIntervals.Count) { [math]::Round(($cpuIntervals | Measure-Object -Property oneCoreCpuPercent -Average).Average, 2) } else { $null }
    note = 'Summed process-tree working sets may double-count shared pages. Private bytes are committed memory, not resident RAM. CPU is percent of one logical core across sample intervals; exiting children can undercount. Browser/model/dev-server processes outside this tree are excluded.'
} | ConvertTo-Json -Depth 4
