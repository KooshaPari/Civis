# Asset Generation Pipeline — CC0 Actor Models

Reproducible headless pipeline that produces animated, rigged `.glb` actor files
for the Civis actor system. All geometry is **procedurally generated** — no
external meshes, textures, or licensed assets are bundled. Output is **CC0 1.0**
(public domain).

## Prerequisites

- **Blender 4.x LTS** — download from <https://www.blender.org/download/lts/>
  - The wrapper script (`run_asset_gen.ps1`) auto-detects Blender at the
    standard Windows install path; no manual PATH setup required.
  - Tested against Blender 4.5.5 LTS.

## How to Run

### Recommended (via `just`)

```sh
just gen-actors
```

### Direct PowerShell

```powershell
pwsh -File tools/asset-gen/run_asset_gen.ps1
```

Generate only one variant:

```powershell
pwsh -File tools/asset-gen/run_asset_gen.ps1 -Variant humanoid
pwsh -File tools/asset-gen/run_asset_gen.ps1 -Variant herd
```

Override dimensions:

```powershell
pwsh -File tools/asset-gen/run_asset_gen.ps1 -Variant humanoid -Height 2.0 -LimbScale 1.2
```

## Output Location

```
clients/bevy-ref/assets/models/
    humanoid_gen.glb    — low-poly rigged biped (idle + walk animations)
    herd_gen.glb        — low-poly rigged quadruped (idle + walk animations)
```

The files are named `*_gen.glb` to avoid clobbering the fetched CC0 reference
assets (`civilian.glb`, `herd.glb`, etc.).

## What the Pipeline Produces

| File             | Geometry                        | Bones                     | Actions          |
|------------------|---------------------------------|---------------------------|------------------|
| `humanoid_gen.glb` | 6 boxes (torso/head/4 limbs) | root/pelvis/spine/neck/4 limbs | idle (48 fr), walk (48 fr) |
| `herd_gen.glb`     | 5 boxes (body/neck/head/4 legs) | root/spine/neck/head/4 legs | idle (48 fr), walk (48 fr) |

Animations are keyframed at 24 fps. All meshes use `ARMATURE_AUTO` vertex
groups so the skinning is baked and Bevy's `GltfAssetLabel::Animation` works
out of the box.

## LLM-Agent Extension — Parameter Sweeps

The Blender script (`gen_actor.py`) accepts extra flags after `--`:

| Flag            | Type   | Default              | Description                        |
|-----------------|--------|----------------------|------------------------------------|
| `--variant`     | str    | `humanoid`           | `humanoid` or `herd`               |
| `--height`      | float  | 1.75 / 1.0           | Total height in metres             |
| `--limb-scale`  | float  | `1.0`                | Limb length multiplier             |
| `--out-dir`     | path   | `clients/bevy-ref/assets/models` | Output directory     |

To drive a sweep from an agent or script:

```python
import subprocess, os

blender = r"C:/Program Files/Blender Foundation/Blender 4.5/blender.exe"
script  = "tools/asset-gen/gen_actor.py"
configs = [
    ("humanoid", 1.6, 0.9, "dwarf"),
    ("humanoid", 2.1, 1.2, "giant"),
    ("herd",     0.8, 0.8, "pony"),
]
for (variant, height, limb, tag) in configs:
    out = f"clients/bevy-ref/assets/models/{variant}_{tag}_gen.glb"
    subprocess.run([
        blender, "-b", "-P", script, "--",
        "--variant", variant,
        "--height",  str(height),
        "--limb-scale", str(limb),
        "--out-dir", os.path.dirname(out),
    ], check=True)
```

Add more `argparse` flags to `gen_actor.py` (e.g. `--torso-scale`, `--color-r/g/b`)
and consume them in `build_humanoid` / `build_herd` for further parameterisation.

## CC0 License Note

All output `.glb` files are **public domain (CC0 1.0)**. They are generated
entirely from procedural geometry inside Blender's Python API — no external
mesh, texture, or font assets are loaded or embedded. You may use, modify, and
redistribute without attribution.

See also `clients/bevy-ref/assets/models/PROVENANCE.txt` for provenance of the
separately-fetched reference assets (`civilian.glb`, `herd.glb`, etc.).
