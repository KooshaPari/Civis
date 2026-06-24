#Requires -Version 5.1
<#
.SYNOPSIS
  Compare JsonRpcMethod wire names in jsonrpc.rs against docs/api/jsonrpc-surface.md.

.EXIT CODES
  0  Catalog matches
  1  Drift detected (diff printed)
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$RustPath = Join-Path $RepoRoot 'crates\server\src\jsonrpc.rs'
$DocPath = Join-Path $RepoRoot 'docs\api\jsonrpc-surface.md'

function Get-JsonRpcMethodsFromRust {
    param([string] $Path)
    $text = Get-Content -LiteralPath $Path -Raw
    if ($text -notmatch 'impl JsonRpcMethod \{([\s\S]*?)\n\}') {
        throw "JsonRpcMethod impl block not found in $Path"
    }
    $implBody = $Matches[1]
    $names = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($m in [regex]::Matches($implBody, 'Self::\w+\s*=>\s*"([^"]+)"')) {
        [void]$names.Add($m.Groups[1].Value)
    }
    foreach ($m in [regex]::Matches($implBody, '"([^"]+)"\s*=>\s*Some\(Self::')) {
        [void]$names.Add($m.Groups[1].Value)
    }
    if ($names.Count -eq 0) {
        throw "No JsonRpcMethod wire names found in $Path"
    }
    [string[]]($names | Sort-Object)
}

function Get-JsonRpcMethodsFromDoc {
    param([string] $Path)
    $lines = Get-Content -LiteralPath $Path
    $inCatalog = $false
    $names = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($line in $lines) {
        if ($line -match '^## Method catalog') {
            $inCatalog = $true
            continue
        }
        if ($inCatalog -and $line -match '^---\s*$') {
            break
        }
        if (-not $inCatalog) { continue }
        if ($line -match '^\|\s*`([a-z][a-z0-9_.]*)`\s*\|') {
            [void]$names.Add($Matches[1])
        }
    }
    if ($names.Count -eq 0) {
        throw "No method rows found in Method catalog section of $Path"
    }
    [string[]]($names | Sort-Object)
}

$rust = Get-JsonRpcMethodsFromRust -Path $RustPath
$doc = Get-JsonRpcMethodsFromDoc -Path $DocPath

$onlyRust = @(Compare-Object -ReferenceObject $doc -DifferenceObject $rust |
    Where-Object { $_.SideIndicator -eq '=>' } |
    ForEach-Object { $_.InputObject })
$onlyDoc = @(Compare-Object -ReferenceObject $doc -DifferenceObject $rust |
    Where-Object { $_.SideIndicator -eq '<=' } |
    ForEach-Object { $_.InputObject })

if ($onlyRust.Length -eq 0 -and $onlyDoc.Length -eq 0) {
    Write-Host "jsonrpc catalog OK ($($rust.Count) methods)" -ForegroundColor Green
    exit 0
}

Write-Host 'jsonrpc catalog DRIFT' -ForegroundColor Red
Write-Host "  rust ($($rust.Count)): $($rust -join ', ')"
Write-Host "  doc  ($($doc.Count)): $($doc -join ', ')"
if ($onlyRust) {
    Write-Host '  in rust only:' -ForegroundColor Yellow
    $onlyRust | ForEach-Object { Write-Host "    + $_" }
}
if ($onlyDoc) {
    Write-Host '  in doc only:' -ForegroundColor Yellow
    $onlyDoc | ForEach-Object { Write-Host "    - $_" }
}
exit 1
