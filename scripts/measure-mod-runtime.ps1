$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path "$PSScriptRoot/..").Path
foreach ($name in @('wasm_probe','rhai_probe')) {
  $exe = Join-Path $repo "target/release/examples/$name.exe"
  $samples = @()
  for ($i=0; $i -lt 3; $i++) {
    $process = Start-Process -FilePath $exe -WindowStyle Hidden -PassThru
    Start-Sleep -Milliseconds 200
    $process.Refresh()
    $samples += [math]::Round($process.WorkingSet64 / 1MB,2)
    $process.WaitForExit()
    if ($process.ExitCode) { throw "$name failed" }
  }
  [pscustomobject]@{runtime=$name;exeBytes=(Get-Item -LiteralPath $exe).Length;workingSetMiB=$samples;ownedProcesses=1;idleAfterExit=0} | ConvertTo-Json -Compress
}
# A plain worker inherits account authority. Demonstrate only with our own file.
$fixture = Join-Path $repo 'output/mod-worker-boundary.txt'
[IO.File]::WriteAllText($fixture,'worker-can-read')
try {
  & python (Join-Path $PSScriptRoot 'mod-worker-probe.py') $fixture
  if ($LASTEXITCODE) { throw 'Worker probe failed' }
} finally { Remove-Item -LiteralPath $fixture }
