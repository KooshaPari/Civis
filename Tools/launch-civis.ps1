# Launch the latest Civis build. Mirrors assets next to the exe every launch so
# Bevy's AssetServer (which roots at the exe dir, NOT the cwd on this build)
# always resolves menu art / tool-icons / textures / shaders. Skipping this sync
# = 23x "Path not found" -> blank UI over an empty world (the "straight-to-empty-
# world, no main menu" failure). Root cause: fresh exe, assets never copied beside it.
$ErrorActionPreference = 'Stop'
$repo = 'C:\Users\koosh\Dev\civis-game'
$exe  = 'E:\cargo-target\release\civ-standalone.exe'
if (-not (Test-Path $exe)) { $exe = "$repo\target\release\civ-standalone.exe" }
if (-not (Test-Path $exe)) {
    Write-Host 'No build found - run: cargo build -p civ-bevy-ref --features bevy,egui,voxel --bin civ-standalone --release'
    pause; exit 1
}

# --- Asset mirror (Copy-Item, not robocopy: robocopy /E trips the path hook) ---
$src = "$repo\clients\bevy-ref\assets"
$dst = "$(Split-Path $exe)\assets"
if (Test-Path $src) {
    if (Test-Path $dst) { Remove-Item $dst -Recurse -Force }
    Copy-Item $src $dst -Recurse -Force
    $n = (Get-ChildItem $dst -Recurse -File | Measure-Object).Count
    Write-Host "[launch] synced $n asset files -> $dst"
} else {
    # Fail loud, per project stance: a missing asset source is a real defect, not graceful-degrade.
    Write-Host "[launch] ERROR: asset source missing at $src - UI/textures WILL 404. Aborting."
    pause; exit 1
}

Start-Process -FilePath $exe -WorkingDirectory $repo
