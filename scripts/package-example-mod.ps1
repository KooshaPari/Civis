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
