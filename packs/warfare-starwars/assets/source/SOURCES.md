# warfare-starwars Source Assets

All sourced assets are CC0 or CC-BY licensed. No NC (non-commercial) or ND (no-derivatives) assets are included.

## Summary

| Pack/File | Source | License | Size | Status | Maps To |
|-----------|--------|---------|------|--------|---------|
| kenney_space-kit | Kenney.nl | CC0 | 6.4MB | Downloaded + extracted | Spaceships, turrets, hangars, speeder craft, platforms, pipes, structures |
| kenney_blaster-kit | Kenney.nl | CC0 | 1.7MB | Downloaded + extracted | All blaster/weapon props, crates |
| kenney_mini-characters | Kenney.nl | CC0 | 2.3MB | Downloaded + extracted | Humanoid character bases (male/female a–f) |
| kenney_modular-space-kit | Kenney.nl | CC0 | 6.7MB | Downloaded + extracted | Modular sci-fi corridors/rooms/doors, CIS base buildings |
| kenney_sci-fi-rts | Kenney.nl | CC0 | 2.0MB | Downloaded + extracted | 2D sprites — 48 unit icons, 16 structure icons, tiles (UI use) |
| kenney_minigolf-kit | Kenney.nl | CC0 | 3.1MB | Downloaded + extracted | Modular minigolf course pieces (ramps, corners, windmills) — terrain tile proxy |
| sci-fi-soldiers.zip | OpenGameArt.org | CC0 | 57MB | Downloaded | Clone/CIS soldier bodies, skin variants |
| Shuttle.obj + texture.png | OpenGameArt.org | CC0 | 3.4MB | Downloaded | LAAT gunship base shape |
| walker_rigged.zip | OpenGameArt.org | CC-BY 3.0 | 22MB | Downloaded | AT-TE / AAT walker body |
| sci-fi-hover-tank.zip | OpenGameArt.org | CC-BY 3.0 | 149KB | Downloaded | AAT tank / BARC speeder base |
| xStarFighter.zip | OpenGameArt.org | CC-BY 4.0 | 239KB | Downloaded | Republic starfighter / V-19 Torrent base |

---

## Kenney.nl Assets (CC0 — No attribution required)

### Space Kit
- **File**: `kenney/space-kit/kenney_space-kit.zip`
- **Source URL**: https://kenney.nl/assets/space-kit
- **Direct download**: https://kenney.nl/media/pages/assets/space-kit/cceeafbd0c-1677698978/kenney_space-kit.zip
- **License**: CC0 (Public Domain)
- **Contents** (FBX + GLTF + OBJ): astronautA.fbx, astronautB.fbx, alien.fbx, craft_speederA-D.fbx, craft_cargoA-B.fbx, craft_miner.fbx, craft_racer.fbx, rover.fbx, turret_single.fbx, turret_double.fbx, weapon_gun.fbx, weapon_rifle.fbx, hangar_largeA-B.fbx, hangar_smallA-B.fbx, platform_*.fbx, structure*.fbx, monorail_*.fbx, pipe_*.fbx, terrain_*.fbx, rocket_*.fbx, satelliteDish*.fbx

**Unit/building mappings:**
- `astronautA.fbx` / `astronautB.fbx` → Base for all humanoid clone/droid characters (needs armor geometry added in Blender)
- `craft_speederA.fbx` - `craft_speederD.fbx` → BARC Speeder base (white/blue repaint)
- `turret_single.fbx` → CIS Weapon Emplacement / Republic static turret
- `turret_double.fbx` → Republic Turret Tower (top-mounted double cannon)
- `hangar_largeA.fbx` / `hangar_largeB.fbx` → CIS Droid Factory main building shell
- `hangar_smallA.fbx` / `hangar_smallB.fbx` → Republic Forward Base / CIS Forward Post structures
- `rocket_*.fbx` (6 parts) → Droid Rocket Droid body segments
- `platform_*.fbx` (15 parts) → Republic Landing Platform / CIS Platform tiles
- `structure.fbx` + variants → Republic Tech Bunker base shell
- `monorail_trainFront.fbx` → Droid MTT (Multi Troop Transport) body base
- `terrain_*.fbx` → Geonosis terrain tiles / Ryloth canyon ground

**Modifications needed:**
- astronaut → clone trooper: add dome helmet, chest plate panels, arm vambraces in Blender; scale up for TABS proportions
- craft_speeder → BARC Speeder: repaint white/blue (#F5F5F5 / #1A3A6B); add pilot figure atop
- turret_double → AT-TE top cannon: attach to walker body (see walker_rigged.zip)

---

### Blaster Kit
- **File**: `kenney/blaster-kit/kenney_blaster-kit.zip`
- **Source URL**: https://kenney.nl/assets/blaster-kit
- **Direct download**: https://kenney.nl/media/pages/assets/blaster-kit/aa06525a20-1753959510/kenney_blaster-kit_2.1.zip
- **License**: CC0 (Public Domain)
- **Version**: 2.1
- **Contents**: blaster-a through blaster-r (18 weapon variants), grenades, scopes, silencers, clips, crates, smoke, targets/fragments

**Unit/building mappings:**
- `blaster-a.fbx` → Clone DC-15A blaster rifle (long variant — closest match)
- `blaster-b.fbx` → Clone DC-15S carbine (shorter barrel variant)
- `blaster-c.fbx` → Clone DC-17 pistol (compact variant)
- `blaster-e.fbx` / `blaster-f.fbx` → CIS E-5 battle droid blaster
- `blaster-r.fbx` → CIS Droideka twin blasters
- `grenade-a.fbx` → Clone thermal detonator prop
- `scope-large-a.fbx` → DC-15x sniper scope attachment
- `crate-medium.fbx` → Supply Depot cargo crate prop
- All blasters → Weapon props for all infantry units

---

### Mini Characters 1
- **File**: `kenney/mini-characters-1/kenney_mini-characters.zip`
- **Source URL**: https://kenney.nl/assets/mini-characters-1
- **Direct download**: https://kenney.nl/media/pages/assets/mini-characters-1/a745467fe1-1721210573/kenney_mini-characters.zip
- **License**: CC0 (Public Domain)
- **Contents**: character-male-a through f (6 variants), character-female-a through f (6 variants), accessibility aids (not needed for this pack)

**Unit/building mappings:**
- `character-male-a.fbx` → Primary base mesh for all clone trooper variants (before armor shell)
- `character-male-b.fbx` → Alternate clone pose variant (heavy trooper stance)
- `character-male-c.fbx` → Jedi Knight base (add robes as flat panels in Blender)
- `character-male-d.fbx` through `f.fbx` → Additional pose variants for droid/NPC characters

**Modifications needed:**
- All characters need TABS-style exaggeration: bigger head, wider stance, shorter legs relative to torso
- Clone armor: wrap geo panels over base mesh or replace entirely with armored shell
- Battle droid: replace head with triangular CIS droid head, swap limbs for thinner mechanical tubes

---

### Modular Space Kit
- **File**: `kenney/modular-space-kit/kenney_modular-space-kit.zip`
- **Source URL**: https://kenney.nl/assets/modular-space-kit
- **Direct download**: https://kenney.nl/media/pages/assets/modular-space-kit/13f81361ae-1771146076/kenney_modular-space-kit_1.0.zip
- **License**: CC0 (Public Domain)
- **Status**: Downloaded + extracted (6.8MB, 214 files, 120 FBX models)
- **Contents**: 40 modular sci-fi station panels, corridors, connectors, windows, doors, walls with animation and multiple color variants

**Unit/building mappings (anticipated):**
- Wall panels → CIS Droid Factory exterior walls, Republic Barracks walls
- Floor tiles → Republic Command Bunker floor, CIS Base floor
- Doors/airlocks → all buildings' entrance geometry

### Minigolf Kit
- **File**: `kenney/minigolf-kit/` (extracted)
- **Source URL**: https://kenney.nl/assets/minigolf-kit
- **Direct download**: https://kenney.nl/media/pages/assets/minigolf-kit/378928a3bb-1741163874/kenney_minigolf-kit.zip
- **License**: CC0 (Public Domain)
- **Size**: 3.1MB extracted, 646 files
- **Contents**: Modular course pieces (straight, corner, incline, helix, windmill, obstacles), GLB + OBJ + FBX formats with textures

**Unit/building mappings:**
- Ramp pieces → Terrain elevation tiles for Geonosis/Ryloth maps
- Corner/curve pieces → Modular base boundary walls
- Windmill obstacle → Visual prop for environment dressing
- Straight flat tiles → Walkway/platform connectors between buildings

---

### Sci-Fi RTS
- **File**: `kenney/sci-fi-rts/kenney_sci-fi-rts.zip`
- **Source URL**: https://kenney.nl/assets/sci-fi-rts
- **Direct download**: https://kenney.nl/media/pages/assets/sci-fi-rts/d981493ccb-1677693650/kenney_sci-fi-rts.zip
- **License**: CC0 (Public Domain)
- **Size**: 2.0MB extracted
- **Contents**: 2D isometric RTS sprites — 48 unit sprites, 16 structure sprites, tile sheets, environment sheets; PNG, Vector, and Spritesheet formats

**Unit/building mappings (2D sprites for UI icons — not 3D meshes):**
- `PNG/Default size/Unit/scifiUnit_01–16.png` → Republic unit card icon references (humanoids, vehicles)
- `PNG/Default size/Unit/scifiUnit_17–32.png` → CIS unit card icon references (droids, mechs)
- `PNG/Default size/Unit/scifiUnit_33–48.png` → Additional unit icon variants
- `PNG/Default size/Structure/scifiStructure_01–08.png` → Republic building icon references
- `PNG/Default size/Structure/scifiStructure_09–16.png` → CIS building icon references
- `PNG/Default size/Tile/` → Terrain tile references for map thumbnail generation
- `Spritesheet/` → Consolidated sprite atlas for runtime UI rendering

**Usage notes**: These are 2D top-down sprites suitable for HUD minimap icons, unit selection UI, and tooltip thumbnails. Not intended as 3D mesh sources. Ready to use as-is without Blender processing.

---

## OpenGameArt.org Assets

### Sci-Fi Soldiers (CC0)
- **File**: `opengameart/sci-fi-soldier/sci-fi-soldiers.zip`
- **Source URL**: https://opengameart.org/content/sci-fi-soldier
- **Direct download**: https://opengameart.org/sites/default/files/Sci-fi%20Soldiers.zip
- **License**: CC0 (Public Domain)
- **Creator**: credited in zip README
- **Size**: 57MB (includes Blender source + multiple Unity-ready exports)
- **Contents**: Sci-fi soldier body with cannon-hand variant, 3 different skin textures, 3 head variants

**Unit/building mappings:**
- Base soldier body → Backup humanoid base for clone troopers (if Kenney mini-characters insufficient)
- Cannon-hand variant → Clone Heavy Trooper Z-6 rotary cannon arm proxy
- Skin/head variants → CIS Battle Droid base (recolor yellow/tan, replace head geometry)

**Modifications needed:**
- Swap head geometry for CIS droid triangular head
- Retexture: white (#FFFFFF) armor for clones; tan/yellow (#C8A860) for battle droids
- Remove human details, add mechanical panel lines

---

### SciFi Shuttle (CC0)
- **Files**: `opengameart/scifi-shuttle/Shuttle.obj`, `opengameart/scifi-shuttle/texture.png`
- **Source URL**: https://opengameart.org/content/scifi-shuttle
- **Direct download**: https://opengameart.org/sites/default/files/Shuttle.obj + https://opengameart.org/sites/default/files/texture_0.png
- **License**: CC0 (Public Domain) — attribution appreciated but not required
- **Creator**: Kenten Fina
- **Size**: 436-face OBJ + 2048×2048 texture PNG (~3.4MB total)

**Unit/building mappings:**
- `Shuttle.obj` → LAAT/i Gunship base shape (low poly troop transport silhouette)
- `texture.png` → Reference texture atlas; replace with Republic white/gray livery
- LAAT gunship modifications: widen fuselage, add chin gun pods, add side door ramps
- Also usable as: Republic Dropship, generic troop transport vehicle

**Modifications needed:**
- Scale and proportions adjustment to match TABS style
- Retexture to Republic white (#F5F5F5) with blue trim (#1A3A6B)
- Add chin-mounted gun pods (can use blaster-a.fbx from Blaster Kit)

---

### Walker (Rigged) (CC-BY 3.0)
- **File**: `opengameart/walker-rigged/walker_rigged.zip`
- **Source URL**: https://opengameart.org/content/walker-rigged
- **Direct download**: https://opengameart.org/sites/default/files/walker_rigged.zip
- **License**: CC-BY 3.0 (Creative Commons Attribution 3.0)
- **ATTRIBUTION REQUIRED**: Credit original creator in pack README and in-game credits
- **Creator**: See zip README (attributed in pack)
- **Size**: 22MB (Blender source + Substance Painter textures)
- **Contents**: Fully rigged biped/quadruped mech walker with Blender rig and textures

**Unit/building mappings:**
- Walker body → AT-TE six-legged walker base (needs 4 additional legs added, rectangular hull added on top)
- Walker rig → AAT battle tank treads/suspension reference (adapt for tracked vehicle)
- Rigged legs → Octuptarra Tri-Droid leg animation reference

**Modifications needed:**
- AT-TE conversion: replicate leg rig ×6 (original has 4), add rectangular boxy hull, add top-mounted mass driver cannon (use weapon_gun.fbx extended)
- Retexture: white/gray Republic colors for AT-TE; dark gray for CIS walkers

---

### Sci-Fi Hover Tank (CC-BY 3.0)
- **File**: `opengameart/sci-fi-hover-tank/sci-fi-hover-tank.zip`
- **Source URL**: https://opengameart.org/content/sci-fi-hover-tank
- **Direct download**: https://opengameart.org/sites/default/files/Player%20tank_1.zip
- **License**: CC-BY 3.0 (Creative Commons Attribution 3.0)
- **ATTRIBUTION REQUIRED**: Credit "Kingshemboo" (or as credited in zip)
- **Creator**: Kingshemboo
- **Size**: 149KB

**Unit/building mappings:**
- Hover tank body → AAT (Armored Assault Tank) base shape
- Also usable as: Republic Hover Artillery (repainted), BARC Speeder side view reference
- Modifications: Repaint dark tan (#8B7355) for CIS AAT; add front cannon array; add round forward cockpit dome

---

### X-Star Fighter (CC-BY 4.0)
- **File**: `opengameart/x-star-fighter/xStarFighter.zip`
- **Source URL**: https://opengameart.org/content/x-star-fighter
- **Direct download**: https://opengameart.org/sites/default/files/xStarFighter.zip
- **License**: CC-BY 4.0 (Creative Commons Attribution 4.0)
- **ATTRIBUTION REQUIRED**: Credit original creator
- **Contents**: xStarFighter.fbx, Texture.png, frontSprite.png, sideSprite.png, topSprite.png
- **Size**: 239KB

**Unit/building mappings:**
- `xStarFighter.fbx` → V-19 Torrent Starfighter / ARC-170 base shape (Republic air support)
- S-foil wing configuration adaptable to Star Wars X-wing silhouette
- Modifications: Adjust wing angle, add Republic white/gray paint scheme, add astromech dome on back

---

## Assets NOT Downloaded (Research Findings)

### PolyPizza — Requires JavaScript for download
PolyPizza hosts many relevant CC0/CC-BY models from Quaternius and Poly by Google. The download mechanism requires JavaScript/browser interaction and cannot be automated via WebFetch. A human should manually download these GLB files:

| Model | Creator | License | URL | Maps to |
|-------|---------|---------|-----|---------|
| Animated Robot | Quaternius | CC0 | https://poly.pizza/m/QCm7qe9uNJ | Battle Droid base |
| Robot Enemy | Quaternius | CC0 | https://poly.pizza/m/ejDr8lRglP | Battle Droid infantry |
| Robot Enemy Flying | Quaternius | CC0 | https://poly.pizza/m/lF3jeRJwiH | Droid rocket droid |
| Robot Enemy Legs Gun | Quaternius | CC0 | https://poly.pizza/m/lFZfDh2hzP | Spider Droid legs ref |
| SWAT | Quaternius | CC0 | https://poly.pizza/m/Btfn3G5Xv4 | Clone trooper body base |
| Character Soldier | Quaternius | CC0 | https://poly.pizza/m/PpLF4rt4ah | Clone trooper infantry |
| Spaceship (×7) | Quaternius | CC0 | https://poly.pizza/m/uCeLfsdmNP etc. | Republic/CIS starships |
| Squad of Speeder Troopers | Joe Scalise | CC-BY 3.0 | https://poly.pizza/m/7KijfpdhoCB | Clone/speeder trooper reference |
| Mech Assault Walker | Alimayo Arango | CC-BY 3.0 | https://poly.pizza/m/6s3_n8xzzvo | AT-TE walker reference |

### Quaternius Direct (CC0 Packs — Require browser/JS)
Quaternius hosts full packs on https://quaternius.com but download buttons require JavaScript. The following packs are CC0 and directly relevant:
- **Ultimate Space Kit** (92 models, CC0): https://quaternius.com/packs/ultimatespacekit.html — mechs, spaceships, planets, aliens
- **Animated Robot Pack** (CC0): Characters, robots with animations
- **Sci-Fi Essentials Kit** (CC0): Enemies, robots, guns, environments

### Sketchfab — Not downloaded
Sketchfab free section has some CC-BY models but requires account/login for download. Skip unless specific models are identified.

---

## Attribution Notes

Assets with required attribution (CC-BY):

### walker_rigged.zip (CC-BY 3.0)
- Attribute to: See zip README (original creator name on file inside zip)
- Where to attribute: `packs/warfare-starwars/CREDITS.md` (to be created), and any in-game credits screen

### sci-fi-hover-tank.zip (CC-BY 3.0)
- Attribute to: "Kingshemboo" (OpenGameArt username)
- OGA Profile: https://opengameart.org/users/kingshemboo
- Where to attribute: `packs/warfare-starwars/CREDITS.md`

### xStarFighter.zip (CC-BY 4.0)
- Attribute to: Creator credited in zip README
- Where to attribute: `packs/warfare-starwars/CREDITS.md`

### SciFi Shuttle (CC0, attribution appreciated but not required)
- Creator: Kenten Fina
- Appreciated but not required by license

---

## Next Steps for Human

1. **Complete PolyPizza downloads manually**: Visit the poly.pizza URLs above and download the GLB files to `source/polypizza/[creator]/[model-name].glb`
2. **Complete Quaternius pack downloads**: Visit quaternius.com and download Ultimate Space Kit and Animated Robot Pack zips to `source/quaternius/`
3. **Extract and audit**: Extract all zips, audit model poly counts against pack manifest budgets
4. **Blender assembly**: Use extracted source files as bases; apply modifications described above
5. **Create CREDITS.md**: Compile attribution from CC-BY assets into `packs/warfare-starwars/CREDITS.md`
6. **Update manifest**: Change `placeholder: false` as each mesh is assembled and committed
