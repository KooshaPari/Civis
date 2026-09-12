#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$TargetDirectory,
    [Parameter(Mandatory)][string]$ReceiptDirectory
)
$ErrorActionPreference = 'Stop'

function Assert-True {
    param([Parameter(Mandatory)][bool]$Condition,[Parameter(Mandatory)][string]$Message)
    if (-not $Condition) { throw "ASSERTION FAILED: $Message" }
}

$helper = (Resolve-Path (Join-Path $PSScriptRoot '..\ci\invoke-budgeted-child.ps1')).Path
$pwsh = (Get-Command pwsh -CommandType Application | Select-Object -First 1).Path
$childArgumentsJson = (@('-NoProfile','-Command',
        'Write-Output ("CARGO_TARGET_DIR=" + $env:CARGO_TARGET_DIR); Write-Output ("CARGO_BUILD_JOBS=" + $env:CARGO_BUILD_JOBS)'
    ) | ConvertTo-Json -Compress)

$beforeTarget = $env:CARGO_TARGET_DIR
$beforeJobs = $env:CARGO_BUILD_JOBS
$successParameters = @{
    ChildPath = $pwsh
    ChildArgumentsJson = $childArgumentsJson
    TargetDirectory = $TargetDirectory
    ExpectedGrowthBytes = 1
    ReserveBytes = 1
    Jobs = 7
    ReceiptDirectory = $ReceiptDirectory
    Execute = $true
}
$successOutput = @(& $pwsh -NoProfile -ExecutionPolicy Bypass -File $helper @successParameters)
$successCode = $LASTEXITCODE
$successText = $successOutput -join [Environment]::NewLine
$successReceiptPath = [regex]::Match($successText,'"receipt"\s*:\s*"([^"]+)"').Groups[1].Value
Assert-True ($successCode -eq 0) "synthetic child exit code was $successCode"
Assert-True ($successReceiptPath -and (Test-Path -LiteralPath $successReceiptPath -PathType Leaf)) 'success receipt missing'
Assert-True ($env:CARGO_TARGET_DIR -eq $beforeTarget -and $env:CARGO_BUILD_JOBS -eq $beforeJobs) 'caller environment changed'
$successReceipt = Get-Content -LiteralPath $successReceiptPath -Raw | ConvertFrom-Json
$successStdout = Get-Content -LiteralPath $successReceipt.stdoutLog -Raw
Assert-True ($successReceipt.state -eq 'completed' -and $successReceipt.exitCode -eq 0) 'synthetic child did not complete'
Assert-True ($successStdout -match [regex]::Escape("CARGO_TARGET_DIR=$TargetDirectory")) 'child target env missing'
Assert-True ($successStdout -match 'CARGO_BUILD_JOBS=7') 'child jobs env missing'

# Keep a synthetic child alive long enough to verify that the active log can
# be opened by a reader while the writer still owns it.
$readerChildJson = (@('-NoProfile','-Command',
        'Write-Output "reader-start"; Start-Sleep -Seconds 3; Write-Output "reader-end"'
    ) | ConvertTo-Json -Compress)
$readerParameters = @(
    '-NoProfile','-ExecutionPolicy','Bypass','-File',$helper,
    '-ChildPath',$pwsh,'-ChildArgumentsJson',$readerChildJson,
    '-TargetDirectory',$TargetDirectory,'-ExpectedGrowthBytes','1',
    '-ReserveBytes','1','-Jobs','2','-ReceiptDirectory',$ReceiptDirectory,'-Execute'
)
$readerBefore = @(Get-ChildItem -LiteralPath $ReceiptDirectory -Filter 'agent-smoke-budget-*.stdout.log' -File -ErrorAction SilentlyContinue | ForEach-Object FullName)
$readerStart = [Diagnostics.ProcessStartInfo]::new($pwsh)
$readerStart.UseShellExecute = $false
$readerStart.CreateNoWindow = $true
$readerStart.RedirectStandardOutput = $true
$readerStart.RedirectStandardError = $true
foreach ($argument in $readerParameters) { $readerStart.ArgumentList.Add([string]$argument) }
$readerProcess = [Diagnostics.Process]::Start($readerStart)
$readerLog = $null
$readerOpened = $false
for ($attempt = 0; $attempt -lt 50 -and -not $readerLog; $attempt++) {
    Start-Sleep -Milliseconds 100
    $readerLog = Get-ChildItem -LiteralPath $ReceiptDirectory -Filter 'agent-smoke-budget-*.stdout.log' -File -ErrorAction SilentlyContinue |
        Where-Object { $readerBefore -notcontains $_.FullName } | Select-Object -First 1
    if ($readerLog) {
        try { $null = Get-Content -LiteralPath $readerLog.FullName -Raw; $readerOpened = $true } catch { $readerOpened = $false }
    }
}
$readerProcess.WaitForExit()
$readerProcessCode = $readerProcess.ExitCode
$null = $readerProcess.StandardOutput.ReadToEnd()
$null = $readerProcess.StandardError.ReadToEnd()
Assert-True ($readerProcessCode -eq 0) "concurrent-reader child exit code was $readerProcessCode"
Assert-True $readerOpened 'active stdout log could not be opened by a reader'

$rejectParameters = $successParameters.Clone()
$rejectParameters.ExpectedGrowthBytes = 9000000000000000000
$rejectOutput = @(& $pwsh -NoProfile -ExecutionPolicy Bypass -File $helper @rejectParameters)
$rejectCode = $LASTEXITCODE
$rejectText = $rejectOutput -join [Environment]::NewLine
$rejectReceiptPath = [regex]::Match($rejectText,'"receipt"\s*:\s*"([^"]+)"').Groups[1].Value
Assert-True ($rejectCode -eq 2) "rejection exit code was $rejectCode"
Assert-True ($rejectReceiptPath -and (Test-Path -LiteralPath $rejectReceiptPath -PathType Leaf)) 'rejection receipt missing'
$rejectReceipt = Get-Content -LiteralPath $rejectReceiptPath -Raw | ConvertFrom-Json
Assert-True ($rejectReceipt.state -eq 'rejected' -and $rejectReceipt.childPid -eq $null) 'rejected run launched a child'

# Exercise agent-smoke's native hashtable splatting into a real pwsh helper
# process. The deliberately impossible budget proves argument binding and
# rejection without entering the smoke/build body.
$agentSmoke = (Resolve-Path (Join-Path $PSScriptRoot '..\agent-smoke.ps1')).Path
$smokeOutput = @(& $pwsh -NoProfile -ExecutionPolicy Bypass -File $agentSmoke -SkipUnreal -Budgeted `
        -TargetDirectory $TargetDirectory -ExpectedGrowthBytes 9000000000000000000 `
        -ReserveBytes 1 -Jobs 2 -ReceiptDirectory $ReceiptDirectory)
$smokeCode = $LASTEXITCODE
$smokeText = $smokeOutput -join [Environment]::NewLine
$smokeReceiptPath = [regex]::Match($smokeText,'"receipt"\s*:\s*"([^"]+)"').Groups[1].Value
Assert-True ($smokeCode -eq 2) "agent-smoke rejection exit code was $smokeCode"
Assert-True ($smokeReceiptPath -and (Test-Path -LiteralPath $smokeReceiptPath -PathType Leaf)) 'agent-smoke rejection receipt missing'
$smokeReceipt = Get-Content -LiteralPath $smokeReceiptPath -Raw | ConvertFrom-Json
Assert-True ($smokeReceipt.state -eq 'rejected') 'agent-smoke did not preserve helper rejection'

Write-Output "agent-smoke budgeted synthetic tests passed; success=$successReceiptPath helperRejection=$rejectReceiptPath smokeRejection=$smokeReceiptPath"
