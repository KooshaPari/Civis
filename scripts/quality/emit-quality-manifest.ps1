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

# Ensure Rust toolchain is on PATH (cargo may not be visible when invoked via pwsh hook).
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
    $env:PATH = "$cargoBin$([IO.Path]::PathSeparator)$env:PATH"
}

Write-Host "==> civis quality manifest (local gates)"
$skipCivisVerify = $env:SKIP_CIVIS_3D_VERIFY -eq "1" -or $env:SKIP_QUALITY_MANIFEST -eq "1" -or $env:SKIP_QUALITY -eq "1"

if (Get-Command just -ErrorAction SilentlyContinue) {
    if ($skipCivisVerify) {
        $results["civis_3d_verify"] = @{ status = "skip"; detail = "SKIP_CIVIS_3D_VERIFY/SKIP_QUALITY_MANIFEST set" }
        Write-Host "  skip civis_3d_verify"
    } else {
        Invoke-Gate "civis_3d_verify" { just civis-3d-verify }
    }
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

$repoRoot = (git rev-parse --show-toplevel).Trim()
$optionalUnreal = & (Join-Path $PSScriptRoot 'Invoke-OptionalUnrealGates.ps1') -RepoRoot $repoRoot
# Guard against null return when Invoke-OptionalUnrealGates.ps1 exits early
# (no Unreal install detected and CIVIS_QUALITY_UNREAL is unset).
if ($null -eq $optionalUnreal) { $optionalUnreal = @{} }
foreach ($entry in $optionalUnreal.GetEnumerator()) {
    # Unreal gates are optional: record result but never block the push.
    $results[$entry.Key] = $entry.Value
    $label = if ($entry.Value -is [hashtable]) { $entry.Value.status } else { "recorded" }
    Write-Host "  $label $($entry.Key)"
}

# Optional Extras tier (opt-in via $env:CIVIS_QUALITY_EXTRAS = "1"). Mirrors
# the bash emitter: default is `skip` so we don't add minutes to the default
# lefthook pre-push.
$extrasOptIn = $env:CIVIS_QUALITY_EXTRAS -eq "1"
$extras = @(
    @{ key = "extra_cargo_audit";   cmd = { cargo audit --quiet };                          tool = "cargo-audit" },
    @{ key = "extra_cargo_deny";     cmd = { cargo deny check };                             tool = "cargo-deny" },
    @{ key = "extra_cargo_machete";  cmd = { cargo machete };                                tool = "cargo-machete" },
    @{ key = "extra_cargo_semver";   cmd = { cargo semver-checks };                          tool = "cargo-semver-checks" },
    @{ key = "extra_trufflehog";     cmd = { trufflehog filesystem . --no-update --only-verified }; tool = "trufflehog" },
    @{ key = "extra_fr_coverage";    cmd = { & (Join-Path $repoRoot "scripts/fr-coverage/run-fr-coverage.sh") }; tool = "scripts/fr-coverage/run-fr-coverage.sh" },
    @{ key = "extra_docs_check";     cmd = { Push-Location (Join-Path $repoRoot "docs"); bun run docs:check; Pop-Location }; tool = "bun + docs:check" },
    @{ key = "extra_security_guard"; cmd = { & (Join-Path $repoRoot ".github/hooks/security-guard.sh") }; tool = ".github/hooks/security-guard.sh" }
)
foreach ($e in $extras) {
    if (-not $extrasOptIn) {
        $results[$e.key] = @{ status = "skip"; detail = "CIVIS_QUALITY_EXTRAS not set (opt-in)" }
        Write-Host "  skip $($e.key)"
        continue
    }
    $tool = $e.tool
    if ($tool -and -not (Get-Command (($tool -split '\s+')[0]) -ErrorAction SilentlyContinue)) {
        $results[$e.key] = @{ status = "skip"; detail = "$tool not installed" }
        Write-Host "  skip $($e.key) ($tool not installed)"
        continue
    }
    Invoke-Gate $e.key $e.cmd
}

$gatesJson = @{}
foreach ($entry in $results.GetEnumerator()) {
    $gatesJson[$entry.Key] = $entry.Value
}
$env:QUALITY_GATES_JSON = ($gatesJson | ConvertTo-Json -Compress -Depth 5)
$env:MANIFEST_PATH = ".ci/quality-manifest.json"
New-Item -ItemType Directory -Force -Path .ci | Out-Null
python (Join-Path $PSScriptRoot "write-quality-manifest.py")
$writeExit = $LASTEXITCODE
if ($Failed -or $writeExit -ne 0) { exit 1 }
