# Sign mods/<ModId>/mod.wasm and print author_pubkey_hex for manifest.toml.
param(
    [ValidateSet("example-policy", "example-economic")]
    [string]$ModId = "example-policy",
    [string]$RepoRoot = (Join-Path $PSScriptRoot ".."),
    [string]$KeyHex = ""
)
$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path $RepoRoot
$WasmPath = Join-Path $RepoRoot "mods\$ModId\mod.wasm"
$SigPath = Join-Path $RepoRoot "mods\$ModId\mod.wasm.sig"
if (-not (Test-Path -LiteralPath $WasmPath)) {
    Push-Location $RepoRoot
    try { & just civis-3d-mod-wasm } finally { Pop-Location }
}
Push-Location $RepoRoot
try {
    $args = @("run", "-p", "civ-mod-host", "--bin", "civ-mod-sign", "--", $WasmPath, $SigPath)
    if ($KeyHex) { $args += @("--key", $KeyHex) }
    & cargo @args
} finally { Pop-Location }
