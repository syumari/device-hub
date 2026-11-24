param(
    [Parameter(Mandatory)][string]$Label,
    [Parameter(Mandatory)][string]$OutputPath,
    [string[]]$Names = @('lghub', 'lghub_agent', 'lghub_system_tray', 'lghub_updater', 'device-hub'),
    [ValidateRange(10, 3600)][int]$Seconds = 60
)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $OutputPath) { throw 'Refusing to overwrite an existing measurement.' }
$logicalProcessors = [Environment]::ProcessorCount
$samples = [Collections.Generic.List[object]]::new()
$previous = @{}
$clock = [Diagnostics.Stopwatch]::StartNew()
while ($clock.Elapsed.TotalSeconds -lt $Seconds) {
    $now = $clock.Elapsed.TotalSeconds
    foreach ($process in @(Get-Process -Name $Names -ErrorAction SilentlyContinue)) {
        $key = [string]$process.Id
        $cpu = $null
        $interval = $null
        $cpuSeconds = $null
        try { $cpuSeconds = $process.TotalProcessorTime.TotalSeconds } catch { }
        if ($previous.ContainsKey($key) -and $null -ne $cpuSeconds -and $null -ne $previous[$key].CPU) {
            $interval = $now - $previous[$key].Time
            if ($interval -gt 0) { $cpu = 100 * ($cpuSeconds - $previous[$key].CPU) / $interval / $logicalProcessors }
        }
        $previous[$key] = @{ CPU = $cpuSeconds; Time = $now }
        $samples.Add([pscustomobject]@{
            TimestampUTC = [DateTime]::UtcNow.ToString('o'); ElapsedSeconds = $now
            Name = $process.ProcessName; PID = $process.Id
            CPUPercentMachine = $cpu; CPUIntervalSeconds = $interval
            WorkingSetBytes = $process.WorkingSet64; PrivateBytes = $process.PrivateMemorySize64
        })
    }
    Start-Sleep -Seconds 1
}
if ($samples.Count -eq 0) { throw 'No matching processes visible; do not interpret as zero resource usage.' }
[pscustomobject]@{
    Label = $Label; DurationSeconds = $clock.Elapsed.TotalSeconds
    LogicalProcessors = $logicalProcessors; RequestedNames = $Names
    Note = 'CPU null means inaccessible or first sample. PrivateBytes is committed private memory, not resident private memory. Sum working sets can count shared pages more than once.'
    Samples = $samples
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding utf8
Write-Output "Saved $($samples.Count) process samples."
