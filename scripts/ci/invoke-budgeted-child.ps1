#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ChildPath,
    [Parameter(Mandatory)][string]$ChildArgumentsJson,
    [Parameter(Mandatory)][string]$TargetDirectory,
    [Parameter(Mandatory)][ValidateRange(1,[long]::MaxValue)][long]$ExpectedGrowthBytes,
    [Parameter(Mandatory)][ValidateRange(1,[long]::MaxValue)][long]$ReserveBytes,
    [Parameter(Mandatory)][ValidateRange(1,64)][int]$Jobs,
    [Parameter(Mandatory)][string]$ReceiptDirectory,
    [switch]$Execute
)
$ErrorActionPreference = 'Stop'
$ChildArguments = @($ChildArgumentsJson | ConvertFrom-Json | ForEach-Object { [string]$_ })

# Cooperative per-volume serialization gate.
#
# Implementation: a single Windows named System.Threading.Mutex per volume
# ("Global\civis-budget-volume-<letter>"). Held continuously across:
#   1. Get-Volume / Win32_Process observations (capacity + owner scan)
#   2. Re-sampling capacity after acquisition (so the headroom check sees the
#      state at admission time, not at process start)
#   3. Child launch + wait + log drain
# Released in a single outer finally block. Process death releases the mutex
# automatically (named mutex), so there is no reaper / Remove-Item logic.
#
# Acquisition is nonblocking (WaitOne(0)). On contention the call is rejected
# with state=rejected, rejectionReason=serialized. The mutex name uses the
# "Global\" namespace so unrelated Local\ namespaces (per-session) cannot
# shadow it.
$script:MutexNamePrefix = 'Global\civis-budget-volume-'
function New-VolumeMutex {
    param([Parameter(Mandatory)][string]$VolumeLetter)
    $name = $script:MutexNamePrefix + $VolumeLetter.ToLower()
    return [System.Threading.Mutex]::new($false, $name)
}
function Try-AcquireVolumeGate {
    param([Parameter(Mandatory)]$Mutex)
    try {
        return [pscustomobject]@{ Acquired = ($Mutex.WaitOne(0)); Abandoned = $false }
    } catch [Threading.AbandonedMutexException] {
        # Previous holder exited without releasing. The mutex IS ours now;
        # treat as acquired so the gate does not deadlock on a crashed owner.
        return [pscustomobject]@{ Acquired = $true; Abandoned = $true }
    }
}

function Resolve-OrdinaryDirectory {
    param([Parameter(Mandatory)][string]$Path)
    if (-not [IO.Path]::IsPathFullyQualified($Path)) { throw "Absolute directory required: $Path" }
    $item = Get-Item -LiteralPath $Path -Force
    if (-not $item.PSIsContainer) { throw "Not a directory: $Path" }
    for ($cursor = $item; $null -ne $cursor; $cursor = $cursor.Parent) {
        if ($cursor.Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw "Reparse ancestry is not allowed: $($cursor.FullName)"
        }
    }
    $item.FullName.TrimEnd('\')
}

function Save-Receipt {
    param([Parameter(Mandatory)][string]$Path,[Parameter(Mandatory)][hashtable]$Record)
    $temporary = "$Path.tmp"
    $Record | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $temporary -Encoding utf8
    [IO.File]::Move($temporary,$Path,$true)
}

function New-LogStream {
    param([Parameter(Mandatory)][string]$Path)
    [IO.FileStream]::new($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
}

$child = (Get-Item -LiteralPath $ChildPath -Force).FullName
$target = Resolve-OrdinaryDirectory $TargetDirectory
$receiptRoot = Resolve-OrdinaryDirectory $ReceiptDirectory
if ($target -notmatch '^[DG]:\\.+$' -or $target.Length -le 3) {
    throw 'Target must be an existing non-root D: or G: directory'
}
$sandboxRoot = (Join-Path ([Environment]::GetFolderPath('UserProfile')) 'agents\sandbox').TrimEnd('\')
if (-not ($receiptRoot.Equals($sandboxRoot,[StringComparison]::OrdinalIgnoreCase) -or
        $receiptRoot.StartsWith("$sandboxRoot\",[StringComparison]::OrdinalIgnoreCase))) {
    throw "ReceiptDirectory must be inside the current user's agents\sandbox: $sandboxRoot"
}
$volumeLetter = $target.Substring(0,1)
$id = [guid]::NewGuid().ToString('N')
$receiptPath = Join-Path $receiptRoot "agent-smoke-budget-$id.json"
$mutex = New-VolumeMutex -VolumeLetter $volumeLetter
$acquired = $false
$abandoned = $false
$reason = $null
$admitted = $false
$capacityOk = $false
$ownersOk = $false
$volume = $null
$owners = @()
$free = [decimal]0
$required = [decimal]$ExpectedGrowthBytes + [decimal]$ReserveBytes

# All observations (capacity, owner scan) and the resulting admission decision
# happen INSIDE the volume gate. Two concurrent invocations on the same drive
# can no longer race to consume the same headroom: the second call sees the
# mutex already held and is rejected with rejectionReason=serialized.
# Capacity is re-sampled AFTER acquisition so the headroom check reflects
# state under the gate, not state at process start.
try {
    $gate = Try-AcquireVolumeGate -Mutex $mutex
    $acquired = [bool]$gate.Acquired
    $abandoned = [bool]$gate.Abandoned
    if (-not $acquired) {
        $reason = 'serialized'
    } else {
        $volume = Get-Volume -DriveLetter $volumeLetter
        if ([string]$volume.HealthStatus -ne 'Healthy' -or [string]$volume.OperationalStatus -ne 'OK') {
            throw "Destination volume is not Healthy/OK: $($volume.HealthStatus)/$($volume.OperationalStatus)"
        }
        $free = [decimal]$volume.SizeRemaining
        # Only same-target Rust/Cargo owners are relevant. Unrelated archival
        # or copy processes may mention a parent G: path without owning
        # Cargo's target tree.
        $owners = @(Get-CimInstance Win32_Process | Where-Object {
            $_.ProcessId -ne $PID -and
            $_.Name -match '^(cargo|rustc|rustup|rust-lld|clippy-driver|link|lld-link)(\.exe)?$' -and
            $_.CommandLine -and
            $_.CommandLine.IndexOf($target,[StringComparison]::OrdinalIgnoreCase) -ge 0
        } | Select-Object ProcessId,Name,CreationDate,CommandLine)
        $capacityOk = ($free -ge $required)
        $ownersOk = ($owners.Count -eq 0)
        if (-not $capacityOk) {
            $reason = 'capacity'
        } elseif (-not $ownersOk) {
            $reason = 'owners'
        } else {
            $admitted = $true
        }
    }

    $record = [ordered]@{
        schemaVersion = 1
        runId = $id
        observedUtc = [DateTime]::UtcNow.ToString('o')
        mode = if ($Execute) { 'execute' } else { 'preflight' }
        state = if ($admitted) { 'preflight' } else { 'rejected' }
        childPath = $child
        childArguments = @($ChildArguments)
        targetDirectory = $target
        jobs = $Jobs
        receiptDirectory = $receiptRoot
        budget = [ordered]@{
            admitted = $capacityOk
            freeBytes = if ($null -ne $volume) { $volume.SizeRemaining } else { $null }
            expectedGrowthBytes = $ExpectedGrowthBytes
            reserveBytes = $ReserveBytes
            requiredFreeBytes = $required
            projectedFreeBytes = ($free - [decimal]$ExpectedGrowthBytes)
        }
        observedTargetOwners = $owners
        admitted = $admitted
        rejectionReason = $reason
        volumeGate = [ordered]@{
            acquired = $acquired
            abandoned = $abandoned
            name = ($script:MutexNamePrefix + $volumeLetter.ToLower())
        }
        globalEnvironmentChanged = $false
        ownerScan = 'Advisory command-line scan only; environment-only target ownership may be missed.'
        limitation = 'Point-in-time capacity admission, not a disk quota or atomic reservation. Unrelated growth can consume headroom.'
        serialization = 'Per-volume named System.Threading.Mutex held from observation through child completion. Nonblocking acquire; held by another run -> rejected/serialized.'
        childPid = $null
        childStartUtc = $null
        stdoutLog = $null
        stderrLog = $null
        exitCode = $null
        error = $null
    }
    Save-Receipt $receiptPath $record

    if (-not $Execute -or -not $admitted) {
        [pscustomobject]@{receipt=$receiptPath;state=$record.state;admitted=$record.admitted;rejectionReason=$reason;budget=$record.budget;ownerCount=$owners.Count;volumeGate=$record.volumeGate} | ConvertTo-Json -Depth 5
        if ($Execute -and -not $admitted) { exit 2 }
        return
    }

    $stdoutPath = Join-Path $receiptRoot "agent-smoke-budget-$id.stdout.log"
    $stderrPath = Join-Path $receiptRoot "agent-smoke-budget-$id.stderr.log"
    $record.stdoutLog = $stdoutPath
    $record.stderrLog = $stderrPath
    $start = [Diagnostics.ProcessStartInfo]::new($child)
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    foreach ($argument in $ChildArguments) { $start.ArgumentList.Add([string]$argument) }
    $start.Environment['CARGO_TARGET_DIR'] = $target
    $start.Environment['CARGO_BUILD_JOBS'] = [string]$Jobs

    $stdout = $null
    $stderr = $null
    try {
        $stdout = New-LogStream $stdoutPath
        $stderr = New-LogStream $stderrPath
        $record.state = 'launching'
        Save-Receipt $receiptPath $record
        $process = [Diagnostics.Process]::Start($start)
        $record.childPid = $process.Id
        $record.childStartUtc = $process.StartTime.ToUniversalTime().ToString('o')
        $record.state = 'running'
        Save-Receipt $receiptPath $record
        $outCopy = $process.StandardOutput.BaseStream.CopyToAsync($stdout)
        $errCopy = $process.StandardError.BaseStream.CopyToAsync($stderr)
        $process.WaitForExit()
        [void]$outCopy.GetAwaiter().GetResult()
        [void]$errCopy.GetAwaiter().GetResult()
        $record.exitCode = $process.ExitCode
        $record.state = if ($process.ExitCode -eq 0) { 'completed' } else { 'failed' }
    } catch {
        $record.state = 'observer_error'
        $record.error = $_.Exception.Message
        throw
    } finally {
        if ($stdout) { $stdout.Dispose() }
        if ($stderr) { $stderr.Dispose() }
        $record.finishedUtc = [DateTime]::UtcNow.ToString('o')
        Save-Receipt $receiptPath $record
    }
    [pscustomobject]@{receipt=$receiptPath;state=$record.state;admitted=$record.admitted;rejectionReason=$reason;childPid=$record.childPid;exitCode=$record.exitCode;stdoutLog=$stdoutPath;stderrLog=$stderrPath;volumeGate=$record.volumeGate} | ConvertTo-Json -Depth 5
    exit ([int]$record.exitCode)
} finally {
    if ($acquired) {
        try { $mutex.ReleaseMutex() } catch { }
    }
    try { $mutex.Dispose() } catch { }
}