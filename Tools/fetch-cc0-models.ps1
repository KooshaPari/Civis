<#
.SYNOPSIS
    Fetch CC0 (public-domain) .glb models into
    clients/bevy-ref/assets/models/ for the `models` cargo feature.

.DESCRIPTION
    Lands a curated, expanded set of CC0 low-poly models. The original six
    "canonical" slots that the loader (src/gltf_models.rs::asset_paths) expects
    are kept stable; everything else is *extra variety* for emergent worlds:
    multiple building types, several tree/rock variants, non-human creatures,
    and vehicles. The Rust loader is owned elsewhere — wiring the new filenames
    in is a separate task (see the filename list at the bottom of this header).

    Canonical slots (must stay named exactly):
        civilian.glb  -> KayKit Knight (CC0)        — humanoid life-form
        building.glb  -> KayKit medieval home_A (CC0)
        tree.glb      -> KayKit single tree_A (CC0)
        rock.glb      -> KayKit rock_A prop (CC0)
        road.glb      -> KayKit hex road tile (CC0)
        cart.glb      -> Khronos ToyCar (CC0)       — vehicle

    Expanded variety (new — wire into gltf_models.rs when ready):
        buildings:  building_house_B, building_tower, building_church (temple),
                    building_market, building_tavern (hut/inn), building_well
        nature:     tree_b, tree_large, rock_b, rock_c
        creatures:  creature_skeleton_minion, creature_skeleton_warrior
                    (CC0 non-human life forms for emergent fauna)
        vehicles:   cart_wheelbarrow (second cart variant), boat (best-effort)

    Two acquisition paths, both 100% CC0:

      1. DIRECT  — source already ships a single-file .glb; download as-is.
      2. PACK    — source ships multi-file glTF (.gltf + .bin + .png). We pull
                   the trio into a temp dir and pack to a single .glb with
                   `npx gltf-pipeline -b` (binary, embedded). Requires Node/npx.

    Sources (all CC0 1.0 Universal):
      - KayKit by Kay Lousberg — https://kaylousberg.com (License.txt: CC0)
        mirrored on github.com/KayKit-Game-Assets:
          * KayKit-Character-Pack-Adventures-1.0   (humans)
          * KayKit-Character-Pack-Skeletons-1.0    (skeleton creatures)
          * KayKit-Medieval-Hexagon-Pack-1.0       (buildings/nature/tiles/props)
      - Khronos glTF-Sample-Assets — ToyCar is CC0 1.0.
      - Quaternius (https://quaternius.com, CC0) — pirate kit, BEST-EFFORT boat.
        Quaternius distributes via zip/itch, not raw GitHub, so the boat entry
        is allowed to fail without aborting; the loader falls back to a
        procedural primitive and the slot is documented here for a manual drop.

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

$KayHex   = 'https://raw.githubusercontent.com/KayKit-Game-Assets/KayKit-Medieval-Hexagon-Pack-1.0/main/addons/kaykit_medieval_hexagon_pack/Assets/gltf'
$KayChar  = 'https://raw.githubusercontent.com/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0/main/addons/kaykit_character_pack_adventures/Characters/gltf'
$KaySkel  = 'https://raw.githubusercontent.com/KayKit-Game-Assets/KayKit-Character-Pack-Skeletons-1.0/main/addons/kaykit_character_pack_skeletons/Characters/gltf'
$Khronos  = 'https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models'

# BEST-EFFORT only (Quaternius ships zips, not raw GLBs). Documented so the
# script stays runnable; failure is expected and tolerated. To land a boat
# manually: download the Quaternius Pirate Kit (CC0) from
#   https://quaternius.com/packs/piratekit.html
# and drop e.g. Boat.glb here as `boat.glb`.
$QuatBoat = 'https://quaternius.com/packs/piratekit.html'

# --- Asset manifest ----------------------------------------------------------
# Mode 'direct': single Url -> .glb.
# Mode 'pack'  : Gltf/Bin/Tex relative to Base -> packed to .glb via gltf-pipeline.
# Optional 'BestEffort=$true' downgrades a failure from warning to info note.
$assets = @(
    # --- canonical (loader expects these exact names) -----------------------
    @{ File='civilian.glb'; Mode='direct'; Label='KayKit Adventures - Knight (human)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Url="$KayChar/Knight.glb" },

    @{ File='building.glb'; Mode='pack'; Label='KayKit Hexagon - building_home_A (house)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/buildings/blue";
       Gltf='building_home_A_blue.gltf'; Bin='building_home_A_blue.bin'; Tex='hexagons_medieval.png' },

    @{ File='tree.glb'; Mode='pack'; Label='KayKit Hexagon - tree_single_A';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/nature";
       Gltf='tree_single_A.gltf'; Bin='tree_single_A.bin'; Tex='hexagons_medieval.png' },

    @{ File='rock.glb'; Mode='pack'; Label='KayKit Hexagon - rock_single_A';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/nature";
       Gltf='rock_single_A.gltf'; Bin='rock_single_A.bin'; Tex='hexagons_medieval.png' },

    @{ File='road.glb'; Mode='pack'; Label='KayKit Hexagon - hex_road_A';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/tiles/roads";
       Gltf='hex_road_A.gltf'; Bin='hex_road_A.bin'; Tex='hexagons_medieval.png' },

    @{ File='cart.glb'; Mode='direct'; Label='Khronos glTF-Sample-Assets - ToyCar (vehicle)';
       License='CC0 1.0 (Public, KhronosGroup)'; Url="$Khronos/ToyCar/glTF-Binary/ToyCar.glb" },

    # --- expanded buildings (hut/house/tower/temple/market) -----------------
    @{ File='building_house_B.glb'; Mode='pack'; Label='KayKit Hexagon - building_home_B (house variant)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/buildings/blue";
       Gltf='building_home_B_blue.gltf'; Bin='building_home_B_blue.bin'; Tex='hexagons_medieval.png' },

    @{ File='building_tower.glb'; Mode='pack'; Label='KayKit Hexagon - building_tower_A (tower)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/buildings/blue";
       Gltf='building_tower_A_blue.gltf'; Bin='building_tower_A_blue.bin'; Tex='hexagons_medieval.png' },

    @{ File='building_church.glb'; Mode='pack'; Label='KayKit Hexagon - building_church (temple)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/buildings/blue";
       Gltf='building_church_blue.gltf'; Bin='building_church_blue.bin'; Tex='hexagons_medieval.png' },

    @{ File='building_market.glb'; Mode='pack'; Label='KayKit Hexagon - building_market (market)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/buildings/blue";
       Gltf='building_market_blue.gltf'; Bin='building_market_blue.bin'; Tex='hexagons_medieval.png' },

    @{ File='building_tavern.glb'; Mode='pack'; Label='KayKit Hexagon - building_tavern (inn/hut)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/buildings/blue";
       Gltf='building_tavern_blue.gltf'; Bin='building_tavern_blue.bin'; Tex='hexagons_medieval.png' },

    @{ File='building_well.glb'; Mode='pack'; Label='KayKit Hexagon - building_well (small structure)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/buildings/blue";
       Gltf='building_well_blue.gltf'; Bin='building_well_blue.bin'; Tex='hexagons_medieval.png' },

    # --- expanded nature (tree + rock variants) -----------------------------
    @{ File='tree_b.glb'; Mode='pack'; Label='KayKit Hexagon - tree_single_B';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/nature";
       Gltf='tree_single_B.gltf'; Bin='tree_single_B.bin'; Tex='hexagons_medieval.png' },

    @{ File='tree_large.glb'; Mode='pack'; Label='KayKit Hexagon - trees_A_large (cluster)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/nature";
       Gltf='trees_A_large.gltf'; Bin='trees_A_large.bin'; Tex='hexagons_medieval.png' },

    @{ File='rock_b.glb'; Mode='pack'; Label='KayKit Hexagon - rock_single_B';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/nature";
       Gltf='rock_single_B.gltf'; Bin='rock_single_B.bin'; Tex='hexagons_medieval.png' },

    @{ File='rock_c.glb'; Mode='pack'; Label='KayKit Hexagon - rock_single_C';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/nature";
       Gltf='rock_single_C.gltf'; Bin='rock_single_C.bin'; Tex='hexagons_medieval.png' },

    # --- creatures (CC0 non-human life for emergent fauna) ------------------
    @{ File='creature_skeleton_minion.glb'; Mode='direct'; Label='KayKit Skeletons - Skeleton_Minion';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Url="$KaySkel/Skeleton_Minion.glb" },

    @{ File='creature_skeleton_warrior.glb'; Mode='direct'; Label='KayKit Skeletons - Skeleton_Warrior';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Url="$KaySkel/Skeleton_Warrior.glb" },

    # --- vehicles -----------------------------------------------------------
    @{ File='cart_wheelbarrow.glb'; Mode='pack'; Label='KayKit Hexagon - wheelbarrow (cart variant)';
       License='CC0 1.0 (Kay Lousberg, kaylousberg.com)'; Base="$KayHex/decoration/props";
       Gltf='wheelbarrow.gltf'; Bin='wheelbarrow.bin'; Tex='hexagons_medieval.png' },

    @{ File='boat.glb'; Mode='direct'; BestEffort=$true;
       Label='Quaternius Pirate Kit - Boat (BEST-EFFORT: manual drop required)';
       License='CC0 1.0 (Quaternius, quaternius.com)'; Url=$QuatBoat }
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
    } elseif ($a.BestEffort) {
        Write-Host "  [note] $($a.File): best-effort source not auto-fetchable; manual drop documented above."
        $provenance += "- $($a.File): $($a.Label) — $($a.License) — BEST-EFFORT (manual drop; source: $($a.Url))"
    } else {
        Write-Warning "  [FAIL] $($a.File): could not acquire"
        $fail++
        $provenance += "- $($a.File): $($a.Label) — $($a.License) — ACQUISITION FAILED (re-run this script)"
    }
}

$manifest = Join-Path $ModelsDir 'PROVENANCE.txt'
@(
    "CC0 model provenance — generated by Tools/fetch-cc0-models.ps1",
    "All assets are CC0 1.0 / public domain (KayKit by Kay Lousberg, Khronos, Quaternius).",
    "No attribution legally required; provenance retained for auditability.",
    ""
) + $provenance | Set-Content -Path $manifest -Encoding utf8

Write-Host "[fetch-cc0-models] ok=$ok skipped=$skip failed=$fail -> $ModelsDir"
Write-Host "[fetch-cc0-models] provenance: $manifest"
exit 0
