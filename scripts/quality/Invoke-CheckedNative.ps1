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
