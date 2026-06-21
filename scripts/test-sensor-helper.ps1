param(
  [string]$HelperPath = "src-tauri/binaries/pc-health-sensor-helper-x86_64-pc-windows-msvc.exe"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $HelperPath)) {
  throw "Sensor helper not found: $HelperPath"
}

$startInfo = [System.Diagnostics.ProcessStartInfo]::new()
$startInfo.FileName = (Resolve-Path $HelperPath).Path
$startInfo.UseShellExecute = $false
$startInfo.CreateNoWindow = $true
$startInfo.RedirectStandardInput = $true
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true

$process = [System.Diagnostics.Process]::new()
$process.StartInfo = $startInfo
$started = $false

function Read-Response([System.Diagnostics.Process]$Process) {
  $readTask = $Process.StandardOutput.ReadLineAsync()
  if (-not $readTask.Wait(5000)) {
    throw "Timed out waiting for sensor helper response."
  }

  $line = $readTask.Result
  if ([string]::IsNullOrWhiteSpace($line)) {
    $stderr = $Process.StandardError.ReadToEnd()
    throw "Sensor helper closed stdout. $stderr"
  }

  return $line | ConvertFrom-Json
}

try {
  $started = $process.Start()
  if (-not $started) {
    throw "Sensor helper failed to start."
  }

  $process.StandardInput.WriteLine('{"id":1,"command":"sample"}')
  $process.StandardInput.Flush()
  $first = Read-Response $process

  $process.StandardInput.WriteLine('{"id":2,"command":"sample"}')
  $process.StandardInput.Flush()
  $second = Read-Response $process

  if (-not $first.ok) {
    throw "First sample failed: $($first.errors -join '; ')"
  }
  if (-not $second.ok) {
    throw "Second sample failed: $($second.errors -join '; ')"
  }
  if ($first.id -ne 1 -or $second.id -ne 2) {
    throw "Sensor helper request id mismatch."
  }
  if ($first.helperPid -ne $second.helperPid) {
    throw "Sensor helper process was not reused."
  }
  if ($second.sampleIndex -ne ($first.sampleIndex + 1)) {
    throw "Sensor helper sample index did not increase."
  }

  $process.StandardInput.WriteLine('{"id":3,"command":"shutdown"}')
  $process.StandardInput.Flush()
  $shutdown = Read-Response $process
  $process.StandardInput.Close()

  if (-not $shutdown.ok -or $shutdown.command -ne "shutdown") {
    throw "Sensor helper did not acknowledge shutdown."
  }
  if (-not $process.WaitForExit(5000)) {
    throw "Sensor helper did not exit after shutdown."
  }
  if ($process.ExitCode -ne 0) {
    throw "Sensor helper exited with code $($process.ExitCode)."
  }
}
finally {
  if ($started -and -not $process.HasExited) {
    $process.Kill($true)
    $process.WaitForExit()
  }
  $process.Dispose()
}
