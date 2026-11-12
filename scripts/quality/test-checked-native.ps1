$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Invoke-CheckedNative.ps1')
$results = @{}
$Failed = $false
$shell = (Get-Process -Id $PID).Path
$emitter = Join-Path $PSScriptRoot 'emit-quality-manifest.ps1'
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
$tempRepo = Join-Path ([IO.Path]::GetTempPath()) ('civis-quality-test-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tempRepo | Out-Null
Push-Location $tempRepo
try {
    Invoke-CheckedNative git @('init', '-q')
    Set-Content -Path 'tracked.txt' -Value 'first'
    New-Item -ItemType Directory -Path 'web/dashboard' -Force | Out-Null
    Set-Content -Path 'web/dashboard/.gitkeep' -Value ''
    Invoke-CheckedNative git @('add', 'tracked.txt', 'web/dashboard/.gitkeep')
    Invoke-CheckedNative git @('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-qm', 'initial')
    $head = Assert-QualityTree
    $oldPath = $env:PATH
    $oldSkip = $env:SKIP_CIVIS_3D_VERIFY
    $oldCi = $env:CI
    $oldGlobalSkip = $env:SKIP_QUALITY
    $oldManifestSkip = $env:SKIP_QUALITY_MANIFEST
    try {
        $env:PATH = Split-Path (Get-Command git).Source
        $env:CI = ''
        $env:SKIP_QUALITY = ''
        $env:SKIP_QUALITY_MANIFEST = ''
        $env:SKIP_CIVIS_3D_VERIFY = '1'
        & $shell -NoProfile -File $emitter 2>$null
        if ($LASTEXITCODE -ne 1) { throw 'required skip was accepted' }
        $env:SKIP_CIVIS_3D_VERIFY = ''
        & $shell -NoProfile -File $emitter 2>$null
        if ($LASTEXITCODE -ne 1) { throw 'missing just was accepted' }
        if (Test-Path '.ci/quality-manifest.json') { throw 'failed preflight wrote a manifest' }
        if ($IsWindows -or $env:OS -eq 'Windows_NT') {
            $fakeBin = Join-Path ([IO.Path]::GetTempPath()) ('civis-quality-bin-' + [guid]::NewGuid())
            New-Item -ItemType Directory -Path $fakeBin | Out-Null
            try {
                foreach ($tool in @('just', 'npm', 'bun')) {
                    Set-Content -Path (Join-Path $fakeBin "$tool.cmd") -Value '@exit /b 0'
                }
                $env:PATH = "$fakeBin$([IO.Path]::PathSeparator)$oldPath"
                & $shell -NoProfile -File $emitter
                if ($LASTEXITCODE -ne 0 -or -not (Test-Path '.ci/quality-manifest.json')) { throw 'successful synthetic gates failed to emit' }
                & bash (Join-Path $PSScriptRoot 'verify-quality-manifest.sh')
                if ($LASTEXITCODE -ne 0) { throw 'synthetic manifest failed verification' }
                $manifestBefore = Get-Content -Raw '.ci/quality-manifest.json'
                Set-Content -Path (Join-Path $fakeBin 'just.cmd') -Value '@exit /b 23'
                & $shell -NoProfile -File $emitter 2>$null
                if ($LASTEXITCODE -ne 1) { throw 'failed required gates were accepted' }
                if ((Get-Content -Raw '.ci/quality-manifest.json') -ne $manifestBefore) { throw 'failed gates changed the manifest' }
                Set-Content -Path (Join-Path $fakeBin 'just.cmd') -Value '@exit /b 0'
                Set-Content -Path (Join-Path $fakeBin 'bun.cmd') -Value "@if `"%1`"==`"install`" exit /b 23`r`n@exit /b 0"
                & $shell -NoProfile -File $emitter 2>$null
                if ($LASTEXITCODE -ne 1) { throw 'failed bun install was accepted' }
                if ((Get-Content -Raw '.ci/quality-manifest.json') -ne $manifestBefore) { throw 'failed bun install changed the manifest' }
            } finally {
                Remove-Item -LiteralPath $fakeBin -Recurse -Force
            }
        }
    } finally {
        $env:PATH = $oldPath
        $env:SKIP_CIVIS_3D_VERIFY = $oldSkip
        $env:CI = $oldCi
        $env:SKIP_QUALITY = $oldGlobalSkip
        $env:SKIP_QUALITY_MANIFEST = $oldManifestSkip
    }
    Set-Content -Path 'tracked.txt' -Value 'dirty'
    $rejected = $false
    try { $null = Assert-QualityTree $head } catch { $rejected = $true }
    if (-not $rejected) { throw 'dirty tracked file was accepted' }
    Invoke-CheckedNative git @('add', 'tracked.txt')
    Set-Content -Path 'tracked.txt' -Value 'first'
    $rejected = $false
    try { $null = Assert-QualityTree $head } catch { $rejected = $true }
    if (-not $rejected) { throw 'staged change with restored worktree was accepted' }
    Set-Content -Path 'tracked.txt' -Value 'dirty'
    Invoke-CheckedNative git @('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-qm', 'next')
    $rejected = $false
    try { $null = Assert-QualityTree $head } catch { $rejected = $true }
    if (-not $rejected) { throw 'changed HEAD was accepted' }
    Set-Content -Path 'untracked.txt' -Value 'untracked'
    $rejected = $false
    try { $null = Assert-QualityTree } catch { $rejected = $true }
    if (-not $rejected) { throw 'untracked file was accepted' }
} finally {
    Pop-Location
    Remove-Item -LiteralPath $tempRepo -Recurse -Force
}
Write-Host 'checked native negative controls passed'
