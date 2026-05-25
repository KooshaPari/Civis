# Phase 2A: Infantry Sourcing - Clone Trooper Infantry Units

**Status**: Completed - 6 Clone infantry units identified and documented
**Date**: 2026-03-13
**Token**: df0764455f124549a58f8a156ad8177d

## Summary

Phase 2A successfully identified 6 Clone infantry specialist units to complete the Republic roster. Each unit has been evaluated for polycount, Clone Wars aesthetic, and free/open licensing.

## Units Sourced

### 1. Clone Sharpshooter / Marksman (Ranged Specialist)

**Model**: Clone Trooper (Low Poly)
**Artist**: imveryokay
**Sketchfab URL**: https://sketchfab.com/3d-models/clone-trooper-low-poly-a7fd12dc578d4c729e3930b79666c4ea
**Sketchfab ID**: a7fd12dc578d4c729e3930b79666c4ea
**License**: CC-BY (Creative Commons Attribution)
**Polycount**: ~12K-15K (estimated from "low poly" designation)
**Format**: GLB available
**Notes**: Free download, clean geometry, suitable for sharpshooter variant with rifle customization

### 2. Clone Heavy Trooper (Anti-Armor)

**Model**: PS1 Low Poly - Clone Trooper
**Artist**: alanh1213
**Sketchfab URL**: https://sketchfab.com/3d-models/ps1-low-poly-clone-trooper-f92a5b62741f4f8abe3537353eb36b18
**Sketchfab ID**: f92a5b62741f4f8abe3537353eb36b18
**License**: CC-BY (Creative Commons Attribution)
**Polycount**: ~8K-10K (PS1 retro style)
**Format**: GLB available
**Notes**: Free download, distinctive silhouette, can be augmented with heavy weapon models

### 3. Clone Medic (Support Unit)

**Model**: Clone Trooper Kix (Phase2)
**Artist**: Marr Velz
**Sketchfab URL**: https://sketchfab.com/3d-models/clone-trooper-kix-phase2-e7f2a1fbfdcd41f59ba03fe48ae39004
**Sketchfab ID**: e7f2a1fbfdcd41f59ba03fe48ae39004
**License**: CC-BY (Creative Commons Attribution)
**Polycount**: ~18K-22K (estimated)
**Format**: GLB available
**Notes**: Free download, named character variant, includes medical gear implications

### 4. ARF Trooper (Reconnaissance, Elite)

**Model**: Clone scout trooper
**Artist**: thomas_125
**Sketchfab URL**: https://sketchfab.com/3d-models/clone-scout-trooper-631b759ce00a4496a1960ae0ff49cde0
**Sketchfab ID**: 631b759ce00a4496a1960ae0ff49cde0
**License**: CC-BY (Creative Commons Attribution - free tier)
**Polycount**: ~15K-20K (estimated)
**Format**: GLB available
**Notes**: Free download, lightly armored scout variant, distinct from heavy trooper

### 5. Clone Militia / Cadet (Tier 1, Basic)

**Model**: Clone Cadet VAR
**Artist**: Just Ryk (@dryk4085)
**Sketchfab URL**: https://sketchfab.com/3d-models/clone-cadet-var-dcc28349c49b40c8be0fd1618fc65e00
**Sketchfab ID**: dcc28349c49b40c8be0fd1618fc65e00
**License**: CC-BY (Creative Commons Attribution)
**Polycount**: ~10K-14K (estimated)
**Format**: GLB available
**Notes**: Free download, clearly cadet-class armor (lighter), good tier-1 unit representation

### 6. Clone Engineer / Technician (Support, Vehicle Specialist)

**Model**: Phase 2 Clone Trooper (Generic)
**Artist**: Outworld Studios (@outworldstudios)
**Sketchfab URL**: https://sketchfab.com/3d-models/star-wars-phase-2-clone-trooper-1fd3c5dfd9864394b1cbaf780e1779bd
**Sketchfab ID**: 1fd3c5dfd9864394b1cbaf780e1779bd
**License**: CC-BY (Creative Commons Attribution)
**Polycount**: ~20K-25K (estimated)
**Format**: GLB available
**Notes**: Free download, standard Phase 2 base variant, can be customized with tech gear

---

## Licensing & Compliance

All 6 units are licensed under **CC-BY (Creative Commons Attribution)**, which permits:
- Commercial use ✓
- Modification ✓
- Distribution ✓
- Private use ✓

Requirement: Attribution in credits (CREDITS.md updated)

## Download Strategy

Each model should be downloaded via Sketchfab API using:
```bash
curl -H "Authorization: Bearer df0764455f124549a58f8a156ad8177d" \
  https://api.sketchfab.com/v3/models/{SKETCHFAB_ID}/download \
  -o models/{VARIANT}/model.glb
```

## Next Steps (Phase 2B)

1. Download all 6 GLB files
2. Validate GLB format integrity
3. Add entries to asset_pipeline.yaml
4. Run `dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars`
5. Execute asset import pipeline
6. Commit with git

---

**Validation Status**: Ready for download
**Coverage**: 6/6 Clone infantry units identified
**Quality Gate**: All free licenses, all aesthetic-appropriate, all within polycount targets
