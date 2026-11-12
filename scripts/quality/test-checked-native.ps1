$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Invoke-CheckedNative.ps1')
$results = @{}
$Failed = $false
$shell = (Get-Process -Id $PID).Path
$global:LASTEXITCODE = 17
Invoke-Gate 'success' { Invoke-CheckedNative $shell @('-NoProfile', '-Command', 'exit 0') }
if ($results.success.status -ne 'pass') { throw 'stale exit code affected success' }
$script:ranLater = $false
Invoke-Gate 'first-fails' {
    Invoke-CheckedNative $shell @('-NoProfile', '-Command', 'exit 23')
    $script:ranLater = $true
    Invoke-CheckedNative $shell @('-NoProfile', '-Command', 'exit 0')
}
if ($results['first-fails'].status -ne 'fail' -or $script:ranLater) { throw 'failed command was masked' }
Invoke-Gate 'missing' { Invoke-CheckedNative 'civis-command-that-does-not-exist' }
if ($results.missing.status -ne 'fail') { throw 'missing command was accepted' }
function Invoke-FakeNoExit {}
Invoke-Gate 'no-exit' { Invoke-CheckedNative 'Invoke-FakeNoExit' }
if ($results['no-exit'].status -ne 'fail') { throw 'missing native exit status was accepted' }
Invoke-Gate 'later-fails' {
    Invoke-CheckedNative $shell @('-NoProfile', '-Command', 'exit 0')
    Invoke-CheckedNative $shell @('-NoProfile', '-Command', 'exit 9')
}
if ($results['later-fails'].status -ne 'fail' -or -not $Failed) { throw 'later failure was accepted' }
Write-Host 'checked native negative controls passed'
