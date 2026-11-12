function Invoke-CheckedNative {
    param([string]$Command, [string[]]$Arguments = @())
    $global:LASTEXITCODE = $null
    & $Command @Arguments
    if ($null -eq $LASTEXITCODE) {
        throw "$Command did not report a native exit code"
    }
    if ($LASTEXITCODE -ne 0) {
        throw "$Command failed with exit $LASTEXITCODE"
    }
}

function Invoke-Gate {
    param([string]$Name, [scriptblock]$Block)
    $global:LASTEXITCODE = $null
    try {
        $ErrorActionPreference = 'Stop'
        & $Block
        if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
            throw "gate failed with exit $LASTEXITCODE"
        }
    } catch {
        $results[$Name] = @{ status = "fail"; detail = $_.Exception.Message }
        Write-Host "  fail $Name"
        $script:Failed = $true
        return
    }
    $results[$Name] = @{ status = "pass"; detail = "" }
    Write-Host "  pass $Name"
}

function Assert-QualityTree {
    param([string]$ExpectedHead)
    $head = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $head) { throw 'cannot identify quality gate HEAD' }
    if ($ExpectedHead -and $head -ne $ExpectedHead) { throw 'quality gate HEAD changed' }
    & git diff --quiet -- . ':(exclude).ci/quality-manifest.json'
    if ($LASTEXITCODE -ne 0) { throw 'working tree is dirty or unavailable' }
    & git diff --cached --quiet HEAD -- . ':(exclude).ci/quality-manifest.json'
    if ($LASTEXITCODE -ne 0) { throw 'index is dirty or unavailable' }
    $untracked = @(git ls-files --others --exclude-standard -- . ':(exclude).ci/quality-manifest.json')
    if ($LASTEXITCODE -ne 0 -or $untracked.Count -ne 0) { throw 'quality gate tree has untracked files or is unavailable' }
    return $head
}
