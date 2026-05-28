<<<<<<< HEAD
# Package a CivLab example mod directory into a .civmod archive.
param(
    [ValidateSet("example-policy", "example-economic")]
    [string]$ModId = "example-policy",
    [string]$RepoRoot = (Join-Path $PSScriptRoot "..")
)
$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path $RepoRoot
$ModDir = Join-Path $RepoRoot "mods\$ModId"
$Manifest = Join-Path $ModDir "manifest.toml"
$WasmPath = Join-Path $ModDir "mod.wasm"
$OutPath = Join-Path $ModDir "$ModId.civmod"
if (-not (Test-Path -LiteralPath $Manifest)) { throw "Missing $Manifest" }
if (-not (Test-Path -LiteralPath $WasmPath)) {
    Push-Location $RepoRoot
    try { & just civis-3d-mod-wasm } finally { Pop-Location }
=======
# Package mods/example-policy into example-policy.civmod.
param(
    [string]$ModDir = (Join-Path $PSScriptRoot "..\mods\example-policy"),
    [string]$WasmPath = (Join-Path $PSScriptRoot "..\mods\example-policy\mod.wasm"),
    [string]$OutPath = (Join-Path $PSScriptRoot "..\mods\example-policy\example-policy.civmod")
)
$ErrorActionPreference = "Stop"
$ModDir = Resolve-Path $ModDir
$Manifest = Join-Path $ModDir "manifest.toml"
if (-not (Test-Path -LiteralPath $Manifest)) { throw "Missing $Manifest" }
if (-not (Test-Path -LiteralPath $WasmPath)) {
    & just civis-3d-mod-wasm
>>>>>>> origin/main
}
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
if (Test-Path -LiteralPath $OutPath) { Remove-Item -LiteralPath $OutPath -Force }
$Mode = [System.IO.Compression.ZipArchiveMode]::Create
$Zip = [System.IO.Compression.ZipFile]::Open($OutPath, $Mode)
try {
    [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($Zip, $Manifest, "manifest.toml")
    [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($Zip, $WasmPath, "mod.wasm")
} finally { $Zip.Dispose() }
Write-Host "Wrote $OutPath"
