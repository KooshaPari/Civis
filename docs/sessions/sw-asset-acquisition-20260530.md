# Star Wars Mesh Acquisition — Session 2026-05-30

Branch: `feat/sw-mesh-acquisition-20260530` (off `integration/v0.27.0-reconcile-20260530`)
Issue context: #964 (mesh-swap works for 14 units with real bundles); ~22 units have STUB `model.glb`.

## 1. Stub Inventory (the ~22 missing meshes)

`model.glb` is a stub if it is 896 B (cube) or 1092 B (slightly larger primitive), OR absent
entirely (dir only has `metadata.json` / `asset_manifest.json` / empty `textures/`).

### UNITS with stub `model.glb` (896/1092 B) — 10
| Unit id (raw dir)                         | Bytes | Type        | Faction | Priority |
|-------------------------------------------|-------|-------------|---------|----------|
| sw_general_grievous_sketchfab_001         | 1092  | hero        | CIS     | MARQUEE  |
| rep_jedi_knight_sketchfab_001             | 1092  | hero/elite  | REP     | MARQUEE  |
| sw_at_te_walker_sketchfab_001             | 1092  | vehicle     | REP     | MARQUEE  |
| cis_tri_fighter_sketchfab_001             | 1092  | vehicle/air | CIS     | MARQUEE  |
| sw_arc_trooper_sketchfab_001              | 1092  | elite       | REP     | HIGH     |
| cis_magnaguard_sketchfab_001              | 1092  | elite       | CIS     | HIGH     |
| rep_clone_commando_sketchfab_001          | 1092  | elite       | REP     | HIGH     |
| rep_clone_sniper_sketchfab_001            | 1092  | infantry    | REP     | MED      |
| cis_medical_droid_sketchfab_001           | 1092  | support     | CIS     | MED      |
| cis_b1_squad_sketchfab_001                | 1092  | infantry    | CIS     | LOW (dup of b1) |

Note: `rep_clone_wall_guard_sketchfab_001` (1092 B) is a clone infantry mis-named with "wall".

### UNITS / VEHICLES with NO `model.glb` at all — 9
| Unit id (raw dir)                         | Contents          | Type        | Faction | Priority |
|-------------------------------------------|-------------------|-------------|---------|----------|
| sw_aat_walker_sketchfab_001               | asset_manifest    | vehicle/tank| CIS     | MARQUEE  |
| rep_v19_torrent_sketchfab_001             | metadata+textures | vehicle/air | REP     | MARQUEE  |
| cis_laat_gunship_sketchfab_001            | metadata+textures | vehicle/air | REP/CIS | HIGH     |
| cis_dwarf_spider_droid_sketchfab_001      | metadata+textures | vehicle     | CIS     | HIGH     |
| rep_atte_walker_sketchfab_001             | metadata+textures | vehicle     | REP     | HIGH     |
| cis_stap_speeder_sketchfab_001            | metadata+textures | vehicle     | CIS     | MED      |
| rep_barc_speeder_sketchfab_001            | metadata+textures | vehicle     | REP     | MED      |
| sw_droideka_sketchfab_001                 | (has real alt: cis_droideka 4.8MB) | specialized | CIS | LOW |
| sw_clone_trooper_phase2_alt_sketchfab_001 | (has real base: phase2 8.8MB)      | infantry    | REP | LOW |

Total distinct missing-mesh units ≈ 19 unique (22 counting near-dupes). Vehicles + heroes are the
visible gaps; many infantry already have a real sibling bundle that can be reused.

## 2. Acquisition Sources & Legal Posture

### Path A — Sketchfab API (BLOCKED headless this pass)
Prior real bundles (b1, b2, droideka, bx-commando, etc.) were pulled via
`docs/research/sketchfab_downloader.py`, which requires `SKETCHFAB_API_TOKEN` (env var).
That token is NOT present in this environment, so authenticated download is blocked without the
user. Most marquee SW models (Grievous, AAT, V-19, Tri-fighter) are also flagged "restricted"
(not API-downloadable) even with a token — confirmed by `DOWNLOAD_LOG.txt` ("restricted model").
Candidate CC/free Sketchfab models identified (require manual browser download + license check):
- V-19 Torrent: sketchfab `2a9045dac44b4f3a8c1c77b905aaa844`, `67804739...`, `d10a382e...`
- Vulture droid: sketchfab `88139b67d5174829bd9942d26ce6d5af`
- AAT / Tri-fighter / Grievous: predominantly royalty-paid or restricted; verify per-model license.

### Path B — Battlefront II (Frostbite) extraction (USER-RUN, GUI tooling)
EA Battlefront II ships on the Frostbite engine. Mesh extraction requires interactive GUI tools;
none are headless-automatable in this environment. Documented path for the user to run locally:

1. Install **FrostyToolsuite** (Frosty Editor + Frosty Mod Manager), the standard Frostbite
   modding toolkit. Point it at the BF2 `Data/` directory (Origin/EA App install).
2. In Frosty Editor, browse the asset tree to the unit's mesh asset (e.g.
   `Characters/...`, `Vehicles/...`). Right-click the `MeshSet` → **Export** → glTF/OBJ
   (Frosty exports `.obj`/glTF for static, FBX via the mesh exporter plugin for skinned).
3. For skinned characters (Grievous, clones, droids) use the **MeshSetPlugin** export to FBX so
   the skeleton comes along; static vehicles (AAT, tri-fighter) export fine as OBJ/glTF.
4. Drop the exported file into `packs/warfare-starwars/assets/raw/<unit_id>/model.glb`
   (re-export/convert to glb via Blender if exported as FBX/OBJ), then run this repo's pipeline
   (`convert_real_models.py` → Unity `BuildAssetBundles`).

Units BF2 covers cleanly: clone troopers/ARC/commando/sniper, B1/B2/droideka/magnaguard/BX,
AAT, AT-TE, LAAT, tri-fighter, droidekas, Grievous, generic Jedi. Essentially ALL 22.

**Legal posture (per #946):** Extracted EA/Frostbite assets are NOT redistributable. Use is
strictly personal / non-distributed (the bundles stay local, never committed to a public release,
fair-use posture). This is why BF2 extraction is gated on the user running it locally and is NOT
performed or committed by the agent.

### Path C — Prompt-derived procedural meshes (#946 fallback — DELIVERED THIS PASS)
Fully original parametric geometry generated headlessly in Blender (4.5 present at
`C:\Program Files\Blender Foundation\Blender 4.5\blender.exe`). These are clean-room silhouettes
(no ripped IP): a 6-legged walker, a forked-wing fighter, a hovertank with turret, a quadruped
spider droid, humanoid trooper/droid forms, etc. Real geometry (thousands of verts, >40 KB glb),
so the mesh-swap renders a recognizable shape instead of a stub cube. These are committed and
bundled this pass. They are intended to be superseded by Path A/B when the user supplies a token
or runs the BF2 extractor.

## 3. Delivered This Pass
See "Build Results" section below (generated by the procedural pipeline run).

## 4. Blocked-on-User (BF2 extraction)
All 22 units can be covered at higher fidelity by Path B above. Exact steps in Path B. Requires
the user to (a) own BF2, (b) run FrostyToolsuite GUI locally, (c) drop exports into the raw dirs.

## 5. Build / Commit
See CHANGELOG and final commit SHA in session report.
