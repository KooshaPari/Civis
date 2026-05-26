# Star Wars Unit Textures

Complete procedural texture set for all 26 Star Wars units (13 Republic + 13 CIS).

## Quick Stats

- **Total Textures**: 26 (100% complete)
- **Dimensions**: 512x512 pixels
- **Format**: PNG (sRGB, RGBA)
- **File Size Range**: 2.1-4.7 KB per texture
- **Total Size**: ~180 KB
- **Factions**: Republic (13) + CIS (13)

## File Organization

### Republic Textures (13)
```
republic_rep_clone_militia_albedo.png         - Clone Militia
republic_rep_clone_trooper_albedo.png         - Clone Trooper
republic_rep_clone_heavy_albedo.png           - Clone Heavy Trooper
republic_rep_clone_sharpshooter_albedo.png    - Clone Sharpshooter
republic_rep_barc_speeder_albedo.png          - BARC Speeder
republic_rep_atte_crew_albedo.png             - AT-TE Crew
republic_rep_clone_medic_albedo.png           - Clone Medic
republic_rep_arf_trooper_albedo.png           - ARF Trooper
republic_rep_arc_trooper_albedo.png           - ARC Trooper
republic_rep_jedi_knight_albedo.png           - Jedi Knight
republic_rep_clone_wall_guard_albedo.png      - Clone Wall Guard
republic_rep_clone_sniper_albedo.png          - Clone Sniper
republic_rep_clone_commando_albedo.png        - Clone Commando
```

### CIS Textures (13)
```
cis_cis_b1_battle_droid_albedo.png            - B1 Battle Droid
cis_cis_b1_squad_albedo.png                   - B1 Squad
cis_cis_b2_super_battle_droid_albedo.png      - B2 Super Battle Droid
cis_cis_sniper_droid_albedo.png               - Sniper Droid
cis_cis_stap_pilot_albedo.png                 - STAP Pilot
cis_cis_aat_crew_albedo.png                   - AAT Crew
cis_cis_medical_droid_albedo.png              - Medical Droid
cis_cis_probe_droid_albedo.png                - Probe Droid
cis_cis_bx_commando_droid_albedo.png          - BX Commando Droid
cis_cis_general_grievous_albedo.png           - General Grievous
cis_cis_droideka_albedo.png                   - Droideka
cis_cis_dwarf_spider_droid_albedo.png         - DSD1 Dwarf Spider Droid
cis_cis_magnaguard_albedo.png                 - IG-100 MagnaGuard
```

## Metadata

**UNIT_TEXTURE_MANIFEST.json**: Complete inventory with unit class, tier, vehicle/infantry type, and color palette source.

## Color Schemes

### Republic Palette
- **Primary**: #F5F5F5 (pristine white)
- **Secondary**: #1A3A6B (deep navy blue)
- **Tertiary**: #64A0DC (accent blue)
- **Aesthetic**: High-tech, clean, organized

### CIS Palette
- **Primary**: #444444 (dark grey)
- **Secondary**: #B35A00 (rust orange)
- **Tertiary**: #663300 (dark brown)
- **Aesthetic**: Industrial, mechanical, utilitarian

## Tier Distribution

- **Tier 1 (Basic)**: 8 units
- **Tier 2 (Standard)**: 9 units
- **Tier 3 (Elite)**: 9 units

## Unit Types

- **Infantry**: 11 units (Republic-heavy: clone troopers, Jedi, commandos)
- **Vehicles/Droids**: 15 units (CIS-heavy: battle droids, tanks, walkers)

## Integration Notes

1. All textures are procedurally generated and optimized for rapid iteration
2. Placeholder-grade quality suitable for prototyping and testing
3. For production builds, consider upgrading to hand-crafted or photogrammetry-based textures
4. Textures focus on faction color differentiation (not unit silhouette realism)
5. 3D models will provide actual visual distinction beyond color

## Next Steps

1. Import into Unity Addressables system
2. Create material presets paired with normal maps
3. Link to 3D unit models via ContentLoader
4. Test in-game appearance and readability
5. Gather feedback for potential quality iteration

## Generation

All textures were generated using `generate_unit_textures.py` with HSV-based faction color transformation on 2026-03-12.

For regeneration or customization, see the parent directory's UNIT_TEXTURE_GENERATION_SUMMARY.md.

---

**Status**: Ready for integration testing
**Quality**: Procedural (prototype-grade)
**Last Updated**: 2026-03-12
