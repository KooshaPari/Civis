<#
.SYNOPSIS
    Fetch CC0 (public-domain) .glb models into
    clients/bevy-ref/assets/models/ for the `models` cargo feature.

.DESCRIPTION
    Lands a curated set of CC0 low-poly models at the exact filenames the loader
    (src/gltf_models.rs::asset_paths) expects, replacing the procedural
    capsule/cuboid/cone primitives:

        civilian.glb  -> KayKit Knight (CC0)        — humanoid life-form
        building.glb  -> KayKit medieval home (CC0) — a real house
        tree.glb      -> KayKit single tree (CC0)
        rock.glb      -> KayKit rock prop (CC0)
        road.glb      -> KayKit hex road tile (CC0)
        cart.glb      -> Khronos ToyCar (CC0)       — vehicle

    Two acquisition paths, both 100% CC0:

      1. DIRECT  — source already ships a single-file .glb; download as-is.
      2. PACK    — source ships multi-file glTF (.gltf + .bin + .png). We pull
                   the trio into a temp dir and pack to a single .glb with
                   `npx gltf-pipeline -b` (binary, embedded). Requires Node/npx.

    Sources (all CC0 1.0 Universal):
      - KayKit by Kay Lousberg — https://kaylousberg.com (License.txt: CC0)
        mirrored on github.com/KayKit-Game-Assets
      - Khronos glTF-Sample-Assets — per-model CC0 (ToyCar: CC0 1.0)

    Network-robust: a failed asset is reported but does NOT abort the rest;
    the loader falls back to the procedural primitive per missing slot.
    Re-runnable: skips files already present unless -Force.

.PARAMETER ModelsDir
    Output dir. Defaults to clients/bevy-ref/assets/models.

.PARAMETER Force
    Re-download/re-pack even if the .glb already exists.

.EXAMPLE
    pwsh Tools/fetch-cc0-models.ps1
    pwsh Tools/fetch-cc0-models.ps1 -Force
#>
[CmdletBinding()]
param(
    [string]$ModelsDir,
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $ModelsDir) {
    $ModelsDir = Join-Path $repoRoot 'clients/bevy-ref/assets/models'
}
New-Item -ItemType Directory -Force -Path $ModelsDir | Out-Null

$KayHex  = 'https://raw.githubusercontent.com/KayKit-Game-Assets/KayKit-Medieval-Hexagon-Pack-1.0/main/addons/kaykit_medieval_hexagon_pack/Assets/gltf'
$KayChar = 'https://raw.githubusercontent.com/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0/main/addons/kaykit_character_pack_adventures/Characters/gltf'
$Khronos = 'https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models'

# --- Asset manifest ----------------------------------------------------------
# Mode 'direct': single Url -> .glb.
# Mode 'pack'  : Gltf/Bin/Tex relative to Base -> packed to .glb via gltf-pipeline.
$assets = @(
    @{ File='civilian.glb'; Mode='direct'; Label='KayKit Character Pack Adventures - Knight';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Url="$KayChar/Knight.glb" },

    @{ File='building.glb'; Mode='pack'; Label='KayKit Medieval Hexagon - building_home_A';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/buildings/blue";
       Gltf='building_home_A_blue.gltf'; Bin='building_home_A_blue.bin'; Tex='hexagons_medieval.png' },

    @{ File='tree.glb'; Mode='pack'; Label='KayKit Medieval Hexagon - tree_single_A';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/nature";
       Gltf='tree_single_A.gltf'; Bin='tree_single_A.bin'; Tex='hexagons_medieval.png' },

    @{ File='rock.glb'; Mode='pack'; Label='KayKit Medieval Hexagon - rock_single_A';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/nature";
       Gltf='rock_single_A.gltf'; Bin='rock_single_A.bin'; Tex='hexagons_medieval.png' },

    @{ File='road.glb'; Mode='pack'; Label='KayKit Medieval Hexagon - hex_road_A';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/tiles/roads";
       Gltf='hex_road_A.gltf'; Bin='hex_road_A.bin'; Tex='hexagons_medieval.png' },

    @{ File='cart.glb'; Mode='direct'; Label='Khronos glTF-Sample-Assets - ToyCar';
       License='CC0 1.0 (Public, KhronosGroup)'; Url="$Khronos/ToyCar/glTF-Binary/ToyCar.glb" }
)

function Test-Glb([string]$path) {
    if (-not (Test-Path $path)) { return $false }
    if ((Get-Item $path).Length -lt 200) { return $false }
    $b = [System.IO.File]::ReadAllBytes($path)[0..3]
    # glTF magic: 'glTF' = 0x67 0x6C 0x54 0x46
    return ($b[0] -eq 0x67 -and $b[1] -eq 0x6C -and $b[2] -eq 0x54 -and $b[3] -eq 0x46)
}

function Get-Direct([string]$url, [string]$out) {
    try {
        Write-Host "    GET $url"
        Invoke-WebRequest -Uri $url -OutFile $out -UseBasicParsing -TimeoutSec 90
        if (-not (Test-Glb $out)) { throw 'downloaded file is not a valid .glb' }
        return $true
    } catch {
        Write-Warning "    direct download failed: $_"
        if (Test-Path $out) { Remove-Item $out -Force }
        return $false
    }
}

function Get-Packed([hashtable]$a, [string]$out) {
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("cc0_" + [System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    try {
        foreach ($f in @($a.Gltf, $a.Bin, $a.Tex)) {
            $u = "$($a.Base)/$f"
            Write-Host "    GET $u"
            Invoke-WebRequest -Uri $u -OutFile (Join-Path $tmp $f) -UseBasicParsing -TimeoutSec 90
        }
        Write-Host "    PACK $($a.Gltf) -> $($a.File) (gltf-pipeline)"
        & npx -y gltf-pipeline -i (Join-Path $tmp $a.Gltf) -o $out -b 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0 -or -not (Test-Glb $out)) { throw "gltf-pipeline pack failed (exit $LASTEXITCODE)" }
        return $true
    } catch {
        Write-Warning "    pack failed: $_"
        if (Test-Path $out) { Remove-Item $out -Force }
        return $false
    } finally {
        Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

$ok = 0; $skip = 0; $fail = 0
$provenance = @()

foreach ($a in $assets) {
    $out = Join-Path $ModelsDir $a.File
    if ((Test-Glb $out) -and (-not $Force)) {
        Write-Host "[skip] $($a.File) already present"
        $skip++
        $provenance += "- $($a.File): $($a.Label) — $($a.License) [present]"
        continue
    }
    Write-Host "[fetch] $($a.File)  <- $($a.Label)  [$($a.Mode)]"
    $got = if ($a.Mode -eq 'direct') { Get-Direct $a.Url $out } else { Get-Packed $a $out }
    if ($got) {
        $size = (Get-Item $out).Length
        Write-Host "  [ok] $($a.File) ($size bytes)"
        $ok++
        $provenance += "- $($a.File): $($a.Label) — $($a.License) — $size bytes"
    } else {
        Write-Warning "  [FAIL] $($a.File): could not acquire"
        $fail++
        $provenance += "- $($a.File): $($a.Label) — $($a.License) — ACQUISITION FAILED (re-run this script)"
    }
}

$manifest = Join-Path $ModelsDir 'PROVENANCE.txt'
@(
    "CC0 model provenance — generated by Tools/fetch-cc0-models.ps1",
    "All assets are CC0 1.0 / public domain (KayKit by Kay Lousberg + Khronos).",
    "No attribution legally required; provenance retained for auditability.",
    ""
) + $provenance | Set-Content -Path $manifest -Encoding utf8

Write-Host "[fetch-cc0-models] ok=$ok skipped=$skip failed=$fail -> $ModelsDir"
Write-Host "[fetch-cc0-models] provenance: $manifest"
exit 0
