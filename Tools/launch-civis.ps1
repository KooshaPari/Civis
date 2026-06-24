# Launch the latest Civis build from the repo dir (assets resolve relative to cwd).
$ErrorActionPreference = 'Stop'
$repo = 'C:\Users\koosh\Dev\civis-game'
$exe  = 'E:\cargo-target\release\civ-standalone.exe'
if (-not (Test-Path $exe)) { $exe = 'C:\Users\koosh\Dev\civis-game\target\release\civ-standalone.exe' }
if (-not (Test-Path $exe)) { Write-Host 'No build found — run: cargo build -p civ-bevy-ref --features bevy,egui,voxel --bin civ-standalone --release'; pause; exit 1 }
Start-Process -FilePath $exe -WorkingDirectory $repo
