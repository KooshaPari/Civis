# Emit `.ci/quality-manifest.json` after local quality gates (Windows-friendly).
$ErrorActionPreference = "Continue"
Set-Location (git rev-parse --show-toplevel)

$results = @{}
$Failed = $false

function Invoke-Gate {
    param([string]$Name, [scriptblock]$Block)
    & $Block
    if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
        $results[$Name] = @{ status = "fail"; detail = "exit $LASTEXITCODE" }
        Write-Host "  fail $Name"
        $script:Failed = $true
        return
    }
    $results[$Name] = @{ status = "pass"; detail = "" }
    Write-Host "  pass $Name"
}

Write-Host "==> civis quality manifest (local gates)"

if (Get-Command just -ErrorAction SilentlyContinue) {
    Invoke-Gate "civis_3d_verify" { just civis-3d-verify }
} else {
    Invoke-Gate "rust_fmt" { cargo fmt --check }
    Invoke-Gate "rust_clippy" { cargo clippy --workspace --all-targets -- -D warnings }
    Invoke-Gate "rust_test" { cargo test --workspace }
    Invoke-Gate "godot_test" { Push-Location clients/godot-ref/rust; cargo test; Pop-Location }
}

Invoke-Gate "web_test" { Push-Location web; npm test; Pop-Location }
Invoke-Gate "dashboard_typecheck" {
    Push-Location web/dashboard
    bun install --frozen-lockfile
    bun run typecheck
    Pop-Location
}

$gitSha = (git rev-parse HEAD).Trim()
$rust = (rustc --version 2>$null)
if (-not $rust) { $rust = "unknown" }

$attestationGates = $results.GetEnumerator() | ForEach-Object {
    @{ key = $_.Key; status = $_.Value.status }
} | Sort-Object { $_.key }

$attestation = @{
    git_sha = $gitSha
    gates = @($attestationGates)
}

$attestationJson = $attestation | ConvertTo-Json -Compress -Depth 6
$hash = python -c "import hashlib,json,sys; att=json.loads(sys.argv[1]); print(hashlib.blake2b(json.dumps(att,separators=(',',':')).encode(),digest_size=32).hexdigest())" $attestationJson

$body = [ordered]@{
    version = "1"
    repo = "Civis"
    git_sha = $gitSha
    created_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    runner = @{ host = $env:COMPUTERNAME; rust = $rust }
    gates = $results
    manifest_hash = $hash
}

New-Item -ItemType Directory -Force -Path .ci | Out-Null
$manifestPath = ".ci/quality-manifest.json"
($body | ConvertTo-Json -Depth 8) + "`n" | Set-Content -Path $manifestPath -Encoding utf8NoBOM
Write-Host "Wrote $manifestPath (git_sha=$gitSha, manifest_hash=$hash)"

if ($Failed) { exit 1 }
