# Clone Wars Asset Sourcing Manifest

**warfare-starwars Pack | Era Shift: Original Trilogy → Clone Wars Prequels**

Date: 2026-03-11
Status: Active Sourcing
Last Updated: 2026-03-11

---

## Executive Summary

The **warfare-starwars** pack has been intentionally shifted from Original Trilogy (OT) aesthetic to **Clone Wars prequel era** (Episodes I–III). This manifest documents:

1. **Strategic rationale** for the era shift
2. **Asset priority matrix** with gameplay necessity and sourcing status
3. **Removed assets** (OT-exclusive, incompatible with Clone Wars narrative)
4. **Acquisition strategy** (Sketchfab API Tier A → Blend Swap Tier B → Custom Tier C)
5. **Polycounts, sources, and next steps** for scout agents

This shift preserves the **Republic vs. CIS** faction duality while grounding it in authentic Clone Wars fiction and visual identity.

---

## 1. Scope Shift Rationale

### Why Clone Wars (Prequels)?

**Before (OT Misdirection)**
- Stormtroopers vs. Rebellion (Galactic Civil War, Episodes IV–VI)
- Empire brainwashing vs. scrappy insurgency
- Misaligned with faction narrative: CIS ≠ Rebel Alliance
- TIE/X-Wing era incompatible with low-poly ground warfare focus
- Tatooine/Hoth environments don't suit DINO's defensive/economy gameplay

**After (Clone Wars Authenticity)**
- **Clone Troopers vs. Battle Droids** (Grand Army of the Republic vs. Confederacy of Independent Systems, Episodes I–III)
- Disciplined biological elite vs. mechanized expendable swarm — perfect duality for faction design
- CIS narrative aligns with swarm tactics, rolling thunder doctrines
- **Ground-focused warfare**: tanks (AAT), walkers (AT-TE), fortifications, siege mechanics
- **Geonosis, Utapau, Mustafar** as iconic Clone Wars battlefields with distinct tactical roles

**Gameplay Alignment**
- DINO's economy → Clone Wars resource scarcity (tibanna mining, droid factories)
- DINO's defensive/morale → Clone trooper discipline vs. droid expendability
- DINO's unit roles (heavy, support, elite) → Clone Wars arc troopers, commandos, Jedi
- DINO's doctrines (formation, attrition) → Clone Wars grand strategy (Defensive Formation, Mechanized Attrition)

### Faction Narrative Tie-In

**Galactic Republic**
- **Philosophy**: Democratic, disciplined, institutional authority
- **Army composition**: Clones (biological, genetically identical, professional)
- **Command structure**: Jedi Generals + Clone Commanders
- **Doctrine flavor**: "Elite Discipline," "Jedi Leadership," "Defensive Formation"
- **Visual language**: Rounded, clean, organized (institutional white armor)
- **Territory control**: Durasteel fortifications, shield generators, command centers

**Confederacy of Independent Systems (CIS)**
- **Philosophy**: Industrial, expendable, mechanized menace
- **Army composition**: Battle droids (manufactured, easily replaced, mass-produced)
- **Command structure**: Separatist leadership + General Grievous (cyborg supreme commander)
- **Doctrine flavor**: "Mechanized Attrition," "Rolling Thunder," "Swarm Protocol"
- **Visual language**: Angular, segmented, assembly-line (tan/brown industrial aesthetic)
- **Territory control**: Factory complexes, droid production facilities, ray shield installations

### Battle Environment Authenticity

| Environment | Clone Wars Context | Gameplay Role | Assets Needed |
|---|---|---|---|
| **Geonosis** | Arena of First Battle; primary droid factory headquarters | Mid-game push, defending/assaulting high-value facility | Octagonal arena floor, factory walls, Poggle's arena viewing platforms |
| **Utapau** | Airship repair platforms; Grievous last stand; gravity well defense | Late-game fortress assault; vertical terrain advantage | Floating platforms, wind-sculpted rock formations, Separatist hangars |
| **Mustafar** | Volcanic lava planet; Jedi Temple slaughter; final Clone Wars apocalypse | Extreme/apocalypse scenario; lava hazards, vision-limiting fog | Lava flows, ash clouds, industrial walker factories |
| **Kamino** | Clone trooper breeding planet; genetic development | Optional: Republic home base, cloner facilities, rainy atmosphere | Domed cloner facilities, breeding vats, rain particle system |

**Removed Environments:**
- ~~Tatooine~~ (desert, no Clone Wars significance; minimal garrison gameplay)
- ~~Hoth~~ (Imperial base, not Clone Wars; snow not thematically relevant)
- ~~Endor~~ (Ewok battle, Episodes VI; belongs to Rebellion era, not Clone Wars)

---

## 2. Asset Priority Matrix

This matrix organizes all required assets by **gameplay necessity**, **visual prominence**, **polycount target**, and **sourcing status**. Agents should prioritize **CRITICAL** assets first, then **HIGH**, then **MEDIUM**.

### CRITICAL Tier: Faction Backbone

| Priority | Asset | Type | Role | Polycount Target | Current Status | Notes |
|----------|-------|------|------|------------------|-----------------|-------|
| **CRITICAL** | Clone Trooper Phase II | Infantry | Republic backbone; appears in squads of 8+ | 350–450 tris | ⚠️ Found (Sketchfab 4.1k, needs decimation) | Silhouette-driven: dome helmet, T-visor, shoulder pauldrons. Most visible asset. |
| **CRITICAL** | B1 Battle Droid | Infantry | CIS swarm unit; appears in squads of 12+ | 250–400 tris | ⚠️ Scout needed | Thin spindly proportions, horizontal slit visor, tan/brown color. Defines CIS aesthetic. |
| **CRITICAL** | Geonosis Arena (Building) | Environment | Map centerpiece; primary objective | 400–800 tris (modular) | ⚠️ Scout needed | Octagonal floor, arena walls, viewing platforms. Iconic Clone Wars location. |

### HIGH Tier: Faction Leaders & Combat Vehicles

| Priority | Asset | Type | Role | Polycount Target | Current Status | Notes |
|----------|-------|------|------|------------------|-----------------|-------|
| **HIGH** | General Grievous | Hero | CIS supreme commander; appears solo | 800–1200 tris | ⚠️ Scout needed | Four-armed cyborg, cloaked, staff-wielder. Instantly recognizable. Avoid over-articulation. |
| **HIGH** | AAT Tank (Armored Assault Tank) | Vehicle | CIS siege weapon; main battle tank | 1200–1500 tris | ⚠️ Scout needed | Rounded tan hull, rotating turret, crew compartment. Iconic CIS machinery. |
| **HIGH** | AT-TE Walker | Vehicle | Republic siege walker; six-legged | 1000–1400 tris | ⚠️ Scout needed | Chunky legs, rotating head cannon, crew hatches. Memorable silhouette. |
| **HIGH** | Jedi Knight (Generic) | Hero | Republic commander; Obi-Wan/Anakin substitute | 600–900 tris | ⚠️ Scout needed | Robed humanoid, lightsaber (non-physical), serene pose. Face not detailed (generic). |

### MEDIUM Tier: Support & Depth

| Priority | Asset | Type | Role | Polycount Target | Current Status | Notes |
|----------|-------|------|------|------------------|-----------------|-------|
| **MEDIUM** | B2 Super Battle Droid | Infantry | CIS heavy trooper; elite swarm unit | 500–750 tris | ⚠️ Scout needed | Larger than B1, blue-gray armor, stronger appearance. Slow but durable. |
| **MEDIUM** | Droideka (Shield Droid) | Infantry | CIS fast mobile unit; high-priority target | 600–900 tris | ⚠️ Scout needed | Wheeled, retractable shield bubble (transparent blue @ 20% alpha), triple rotary blasters. |
| **MEDIUM** | Clone Commander (ARC Trooper) | Infantry | Republic specialist; squad leader variant | 400–550 tris | ⚠️ Scout needed | Similar to trooper but with green armor accent, tech harness detail. |
| **MEDIUM** | Clone Heavy (Z-6 Gunner) | Infantry | Republic heavy support | 400–600 tris | ⚠️ Placeholder exists | Wider stance, oversized rotary cannon, yellow accent stripe. Reference existing placeholder. |
| **MEDIUM** | LAAT/i Gunship (Troop Transport) | Vehicle | Republic aerial support; optional | 1200–1800 tris | ⚠️ Scout later | Bulky transport with door guns, troop bay. Lower priority than ground units. |
| **MEDIUM** | Clone Commando (Special Forces) | Infantry | Republic infiltration unit | 350–500 tris | ⚠️ Scout needed | Black under-suit, tech loadout, visor-heavy helmet. Stealthy aesthetic. |

### LOW Tier: Variety & Polish

| Priority | Asset | Type | Role | Polycount Target | Current Status | Notes |
|----------|-------|------|------|------------------|-----------------|-------|
| **LOW** | Clone Trooper Hero (Captain Rex analogue) | Hero | Republic named commander; morale buff | 700–1000 tris | Consider later | Individual personality OK; tattoo markings or unique armor. Palette extension. |
| **LOW** | Clone Sharpshooter | Infantry | Republic ranged specialist; sniper | 300–450 tris | Placeholder exists | Kneeling pose, long DC-15x barrel, green camouflage accents. Reference existing. |
| **LOW** | Mustafar Volcanic Base (Building) | Environment | Alternate endgame scenario | 500–1000 tris (modular) | Research later | Lava flows, volcanic rock, industrial walker factory elements. Visual interest. |
| **LOW** | Utapau Floating Platform (Building) | Environment | Alternate fortress map | 400–800 tris (modular) | Research later | Suspended platforms, wind-sculpted rock, Separatist airship hangar. Visual variety. |
| **LOW** | Clone Militia (Phase I) | Infantry | Republic weak/conscript unit | 300–400 tris | Placeholder exists | Simpler armor than Phase II, red markings. Economic unit. Reference existing. |
| **LOW** | Blaster Bolt VFX (Republic) | Particle | Visual feedback; blue emission | 8–12 tris each | Placeholder exists | Simple quad with emissive blue (#4488FF). Spawned at fire rate. |
| **LOW** | Blaster Bolt VFX (CIS) | Particle | Visual feedback; orange-red emission | 8–12 tris each | Placeholder exists | Simple quad with emissive red-orange (#FF4400). Spawned at fire rate. |

**Key Metrics:**
- All values measured in **triangles (tris)**, not vertices
- Budgets account for armor, weapons, and child objects (helmet, limbs)
- Heroes scaled 1.3–1.5× larger than standard infantry for visual prominence
- Vehicles must read clearly at camera distance (exaggerated proportions over fine detail)

---

## 3. Removed Assets (OT, Incompatible)

The following assets are **explicitly removed** from the Clone Wars pack. They belong to the **Original Trilogy**, not the prequels, and would create narrative inconsistency.

### Removed Units

| Asset | Reason | Era | Notes |
|-------|--------|------|-------|
| ~~Stormtrooper~~ | Empire infantry, not Clone Wars; replaces clone trooper incorrectly | Episodes IV–VI (Galactic Civil War) | Belongs to hypothetical "Galactic Empire vs. Rebellion" total conversion, not this pack. |
| ~~Darth Vader~~ | Post-Clone Wars Sith; mechanically similar to Grievous but narratively wrong | Episodes IV–VI | Anakin's fall is at *end* of Clone Wars; Vader appears only after Empire formed (19 years later). |
| ~~Imperial Officer~~ | Empire bureaucracy, not Clone Wars military | Episodes IV–VI | No direct mechanical parallel in Clone Wars (no centralized Empire yet). |
| ~~Rebel Commando~~ | Rebel Alliance, not Republic or CIS | Episodes IV–VI | Belongs to Rebellion pack; incompatible with Clone Wars narrative. |

### Removed Vehicles

| Asset | Reason | Era | Notes |
|-------|--------|------|-------|
| ~~TIE Fighter~~ | Imperial fighter, not Clone Wars air unit | Episodes IV–VI | Clone Wars had clone pilots (Jedi starfighters, LAAT), not TIEs. Prioritize LAAT instead. |
| ~~X-Wing~~ | Rebel starfighter, not Clone Wars air unit | Episodes IV–VI | Rebellion-era, not prequel-era. Clone Wars focused on ground warfare anyway (DINO's strength). |
| ~~AT-AT Walker~~ | Empire transport, not Clone Wars unit | Episodes IV–VI | Belongs to Rebellion era; AT-TE is correct Clone Wars walker. |
| ~~Speeder Bike (Imperial)~~ | Empire scout vehicle | Episodes IV–VI | Clone Wars had STAP (droid speeder); use that instead. |

### Removed Environments

| Asset | Reason | Era | Notes |
|-------|--------|------|-------|
| ~~Tatooine~~ | Desert planet with minimal Clone Wars presence | Episodes I–VI (background) | Appearances: Episode I briefly, Episodes IV–VI (OT). Not a Clone Wars battlefield. |
| ~~Hoth~~ | Ice base, Imperial discovery (Episode V) | Episodes IV–VI (Rebellion) | Belongs to Rebellion vs. Empire; no Clone Wars significance. |
| ~~Endor~~ | Forest moon, Ewok rebellion (Episode VI) | Episodes IV–VI (Rebellion) | Belongs to Rebellion vs. Empire; not Clone Wars. |
| ~~Cloud City~~ | Lando's floating base (Episode V) | Episodes IV–VI (Rebellion) | Rebellion-era location; no Clone Wars connection. |

**Rationale:** Removing these assets eliminates ambiguity and prevents the pack from feeling like a "hodgepodge of Star Wars" instead of a focused **Clone Wars** experience. A player loading this pack should immediately recognize **Geonosis, not Tatooine** as the primary theater of war.

---

## 4. Asset Acquisition Strategy

### Sourcing Tier System

Assets will be acquired in **three tiers**, with fallback escalation:

#### Tier A: Sketchfab Free API + Manual Search

**Primary sourcing path** (80% of assets should come from here).

- **Platform:** https://sketchfab.com/search
- **Filters:**
  - License: **CC0 or CC-BY only** (no NC, ND, SA restrictions)
  - Downloadable: **Yes**
  - Type: **3D Models**
  - Sort: Relevance or Likes (newer models, community-validated)
- **Search keywords per asset:**
  - Clone Trooper: `"clone trooper low poly"`, `"phase II armor"`, `"republic soldier"`, `"TABS style clone"`
  - B1 Battle Droid: `"B1 droid"`, `"battle droid low poly"`, `"tan droid"`, `"spindly robot"`
  - AAT Tank: `"AAT tank low poly"`, `"armored assault tank"`, `"CIS tank"`, `"hover tank"`
  - Geonosis: `"geonosis"`, `"arena floor"`, `"octagonal structure"`, `"droid factory"`
  - General Grievous: `"grievous"`, `"four-armed cyborg"`, `"cloaked commander"`, `"cyborg warrior"`
  - AT-TE: `"AT-TE walker low poly"`, `"six-legged walker"`, `"walker tank"`, `"mech walker"`
  - Jedi Knight: `"jedi low poly"`, `"robed warrior"`, `"lightsaber user"`, `"humanoid hero"`
  - B2 Super Battle Droid: `"B2 droid"`, `"super battle droid"`, `"blue armor droid"`, `"heavy droid"`
  - Droideka: `"droideka"`, `"shield droid"`, `"wheeled robot"`, `"ball droid"`

**Evaluation criteria for each result:**
- ✅ License is CC0 or CC-BY (check in model details)
- ✅ Polycount &lt; 2× target budget (decimation acceptable via Blender Decimate modifier)
- ✅ Silhouette matches Clone Wars aesthetic (not realistic, not grimdark)
- ✅ No humanoid rigging required (static meshes only)
- ✅ Texture maps or solid colors (not relying on complex materials)
- ⚠️ If rigged: rig must be removable in Blender (delete armature, apply shape keys)

**Top 3 candidates per asset:**
Document each candidate with:
- Sketchfab URL + author name
- Download format(s) available (FBX/OBJ/GLTF priority)
- Polycount (raw + decimated estimate)
- License summary
- Strengths / weaknesses vs. budget
- Blender import notes (scale factors, cleanup required)

#### Tier B: Blend Swap + Manual Cleanup

**Secondary sourcing path** (if Tier A yields <1 candidate per asset).

- **Platform:** https://www.blendswap.com
- **Search focus:** `.blend` source files preferred (easier to modify in Blender)
- **License requirement:** CC0 or CC-BY only
- **Typical workflow:**
  1. Download `.blend` project
  2. Open in Blender, locate relevant mesh
  3. Delete non-essential geometry (rigs, helpers, extra variations)
  4. Apply Decimate modifier if over budget
  5. Perform Smart UV Project unwrap
  6. Export as FBX

**Advantages:**
- Source `.blend` files allow deeper customization (proportions, armor segments)
- Easier to reduce polycount via Decimate in the original project
- Author notes sometimes included on best practices for optimization

**Disadvantages:**
- Fewer Star Wars fan assets available (licensing risk)
- May require more Blender expertise to extract and clean up

#### Tier C: Custom Creation (Last Resort)

**Tertiary sourcing path** (only if Tiers A & B fail).

- **Justification needed:** Asset is critical, no suitable public model found, custom creation faster than waiting
- **Author:** Assigned to designated agent with Blender proficiency
- **Workflow:**
  1. Start from Kenney.nl base (Character Pack, Robot Pack, Vehicle Pack — all CC0)
  2. Modify in Blender: add Clone Wars silhouette elements (dome helmet, pauldrons, etc.)
  3. Apply solid colors + single material per unit type
  4. Check polycount budget
  5. UV unwrap (Smart UV Project)
  6. Export FBX
- **Documentation:**
  - Record time spent (estimate custom creation labor cost)
  - Document Blender modification steps in asset_manifest.json
  - Mark in provenance_index.json as `source_type: "custom_hybrid"` with Kenney base referenced
- **Quality bar:** Must match or exceed Tier A/B candidates in silhouette clarity and TABS-style proportions

### Discovery & Intake Process

For each **CRITICAL** asset, scout agents should:

1. **Week 1: Sketchfab API Sweep**
   - Execute 5–8 keyword searches per asset
   - Collect top 10 results per search
   - Filter by license (CC0/CC-BY only)
   - Estimate polycount from model viewer
   - Document in spreadsheet (see template below)

2. **Week 2: Top 3 Candidate Evaluation**
   - Download FBX/OBJ for top 3 candidates (per asset)
   - Open in Blender; check actual face count
   - Test Decimate modifier (target 50% reduction; 0.7–0.8 ratio typical)
   - Assess cleanup time (delete rigs, merge meshes, etc.)
   - Take reference screenshot of decimated result
   - **Rank by**: silhouette match → polycount achievability → source credibility

3. **Week 3: Source Confirmation**
   - For top 1 candidate: download full source (FBX + textures)
   - Create GitHub issue: `[Asset] &lt;Asset Name&gt; — Top 1 Candidate Selected`
   - Link to Sketchfab page, author, license info
   - Attach screenshot of Blender preview (before/after decimation)
   - Note any special cleanup steps (rig removal, material merging, etc.)

4. **Week 4: Intake & Integration**
   - Add source files to `packs/warfare-starwars/assets/raw/<asset_id>/`
   - Update `assets/manifest.yaml` entry: set `placeholder: false`, fill in source_file path
   - Run validation: `dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars`
   - Commit: "feat(assets): intake <asset> from Sketchfab author:name"

### Candidate Tracking Spreadsheet Template

| Asset | Tier | Search Term | URL | Author | Polys | Decimated | Format | License | Notes | Rank |
|-------|------|-------------|-----|--------|-------|-----------|--------|---------|-------|------|
| Clone Trooper | A | "clone trooper low poly" | https://sketchfab.com/... | ChrisVella | 4100 | ~2050 @ 0.5 ratio | FBX | CC-BY | Dome helmet great, needs visor cut, slight rig removal | 1 |
| Clone Trooper | A | "phase II armor" | https://sketchfab.com/... | XArtStudio | 8340 | ~4170 @ 0.5 ratio | OBJ | CC0 | Armor detailed but over budget even decimated | 3 |
| Clone Trooper | A | "TABS style clone" | https://sketchfab.com/... | LowPolyArt | 1850 | 1850 @ 1.0 ratio | GLTF | CC0 | Perfect silhouette, no decim needed, needs texture paint | 1 (alt) |
| B1 Battle Droid | A | "B1 droid" | https://sketchfab.com/... | DroidFan | 2200 | ~1100 @ 0.5 ratio | FBX | CC-BY | Spindly proportions correct, visor too detailed | 2 |
| ... | ... | ... | ... | ... | ... | ... | ... | ... | ... | ... |

---

## 5. Detailed Asset Specifications

### CRITICAL Assets — Acquisition Priority

#### Clone Trooper Phase II

| Field | Value |
|-------|-------|
| **Asset ID** | `rep_clone_trooper` |
| **Display Name** | Clone Trooper |
| **Type** | Infantry unit |
| **Role in Gameplay** | Republic backbone; primary ranged attacker |
| **Frequency** | Appears in squads of 8; visible most of the time |
| **Polycount Target** | 350–450 triangles (includes torso, limbs, helmet, rifle, pauldrons) |
| **Silhouette Signature** | Dome helmet with T-visor slot, shoulder pauldrons, slim carbine at hip (DC-15A) |
| **Color Palette** | Off-white primary (#F5F5F5), navy blue squad stripe (#1A3A6B), dark visor (#111111) |
| **Current Status** | ⚠️ Found on Sketchfab (4.1k polys, needs decimation from Sketchfab ID: unknown) |
| **Sketchfab Search Strategy** | `"clone trooper low poly"`, `"republic soldier"`, `"phase II armor"`, `"TABS style clone"` |
| **Tier A Candidates** | Top 3 required; focus on dome helmet clarity and sub-500 post-decimate polys |
| **Tier B Fallback** | Blend Swap search: `clone trooper` CC0 `.blend` files |
| **Tier C Creation** | If needed: Start from Kenney Character Pack soldier-b.fbx, add dome helmet + pauldrons in Blender |
| **Expected Lead Time** | 3–5 days (Tier A scout) or 1–2 weeks (Tier C custom) |
| **Priority Score** | 10/10 (Most visible Republic unit; faction face) |

**Blender Decimation Workflow (Tier A → Usable):**
```
1. Import FBX into Blender (File > Import > FBX Import)
2. Check Edit Mode > Mesh Stats overlay (current poly count)
3. Select all with 'A' in Object Mode
4. Add modifier: Modifiers panel > Add > Decimate
   - Type: Collapse
   - Ratio: 0.5 (start here; iterate down to 0.3–0.4 if still over budget)
5. Apply modifier
6. Check new poly count
7. If rigged: select Armature in outliner > Delete (removes rig, keeps mesh)
8. UV unwrap: Select mesh, Tab to Edit, U > Smart UV Project
9. Export as FBX (File > Export > FBX)
```

---

#### B1 Battle Droid

| Field | Value |
|-------|-------|
| **Asset ID** | `cis_b1_battle_droid` |
| **Display Name** | B1 Battle Droid |
| **Type** | Infantry unit |
| **Role in Gameplay** | CIS swarm unit; weak individually, dangerous in groups |
| **Frequency** | Appears in squads of 12–20; defines CIS aesthetic |
| **Polycount Target** | 250–400 triangles (thin limbs, segmented joints, spindly frame) |
| **Silhouette Signature** | Spindly stick-figure proportions, thin segmented limbs, horizontal slit visor, tan/brown industrial coloring |
| **Color Palette** | Sandy tan primary (#C8A87A), dark brown joint recesses (#5C3D1E), red photoreceptor (#CC2222) |
| **Current Status** | ⚠️ Scout needed |
| **Sketchfab Search Strategy** | `"B1 droid"`, `"battle droid low poly"`, `"tan droid"`, `"spindly robot"`, `"thin humanoid"` |
| **Tier A Candidates** | Top 3 required; focus on thin limb clarity and tan coloring |
| **Tier B Fallback** | Blend Swap: `battle droid` or `robot` CC0 files; modify proportions if needed |
| **Tier C Creation** | Start from Kenney Robot Pack robot_thin.fbx; elongate head, reduce arm/leg thickness, add visor slit |
| **Expected Lead Time** | 4–6 days (Tier A scout) or 2–3 weeks (Tier C custom) |
| **Priority Score** | 10/10 (Defines CIS faction; appears most frequently) |

**Critical Design Note:**
The B1 droid's **thin spindly silhouette** is essential — it conveys "expendable mass-produced unit" vs. the bulkier clone trooper. Do not default to realistic proportions; exaggerate limb thinness in TABS style.

---

#### Geonosis Arena (Building)

| Field | Value |
|-------|-------|
| **Asset ID** | `cis_geonosis_arena` (or `bldg_geonosis_arena`) |
| **Display Name** | Geonosis Arena |
| **Type** | Large static building |
| **Role in Gameplay** | Map centerpiece; primary defensive objective in Geonosis-themed scenario |
| **Frequency** | Placed 1–2 times per map; high visual prominence |
| **Polycount Target** | 400–800 triangles for modular floor + walls (can split into sub-meshes) |
| **Silhouette Signature** | Octagonal arena floor, tiered seating walls, Poggle's viewing platform overhang, industrial droid factory aesthetic |
| **Color Palette** | Tan/brown droid factory walls (#C8A87A, #5C3D1E), rust accents (#8B4513), faded red arena sand (#A0522D) |
| **Current Status** | ⚠️ Scout needed |
| **Sketchfab Search Strategy** | `"geonosis"`, `"arena floor"`, `"octagonal structure"`, `"droid factory"`, `"sci-fi arena"`, `"colosseum low poly"` |
| **Tier A Candidates** | Top 2–3; focus on clear octagonal reading and modular construction |
| **Tier B Fallback** | Blend Swap: `arena` or `colosseum` CC0 files; modify to octagon shape if needed |
| **Tier C Creation** | Build from scratch in Blender: Cylinder primitive (8 sides), scale/subdivide into arena floor, add box-modeled walls with detail |
| **Expected Lead Time** | 5–7 days (Tier A scout) or 3–4 weeks (Tier C custom) |
| **Priority Score** | 9/10 (Not as visible as units, but critical for map identity) |

**Modular Design Consideration:**
Consider splitting arena into **3 meshes**:
1. Floor (octagonal plane): 100–150 tris
2. Walls (tiered boxes): 200–300 tris
3. Poggle's platform (overhang): 100–150 tris

This allows reuse of wall segments in other CIS buildings and better render budget management.

---

### HIGH Tier Assets — Secondary Priority

#### General Grievous (Hero Unit)

| Field | Value |
|-------|-------|
| **Asset ID** | `cis_general_grievous` |
| **Display Name** | General Grievous |
| **Type** | Hero unit |
| **Role in Gameplay** | CIS supreme commander; high-impact solo unit (morale boost, combat prowess) |
| **Frequency** | 1 per match (summoned once per player, 1–2 on map) |
| **Polycount Target** | 800–1200 triangles (including 4 arms, robes, staff) |
| **Silhouette Signature** | Four-armed cyborg, flowing robes/cape, electro-staff, hunched combat posture |
| **Color Palette** | Near-black armor (#1A1A1A), tan/tan robes (#C8A87A), orange/gold staff (#FF6600 emissive for charged state) |
| **Current Status** | ⚠️ Scout needed |
| **Sketchfab Search Strategy** | `"grievous"`, `"four-armed cyborg"`, `"cloaked commander"`, `"cyborg warrior"`, `"villain hero"` |
| **Tier A Candidates** | Top 2 required; must have clear 4-arm silhouette; avoid overly realistic cyborg designs |
| **Tier B Fallback** | Blend Swap: `grievous` or `cyborg` CC0; modify if necessary for TABS proportions |
| **Tier C Creation** | Start from Kenney humanoid base; duplicate 2 extra arms (attach via boolean/parenting), add robes, sculpt menacing posture |
| **Expected Lead Time** | 6–8 days (Tier A) or 3–4 weeks (Tier C) |
| **Priority Score** | 9/10 (Hero unit; memorable, but not as common as standard infantry) |

**Critical Design Note:**
Avoid over-articulating limbs. Grievous's four arms should be clearly separate but not hyper-realistic; exaggerated proportions fit TABS style better than movie-accurate detail.

---

#### AAT Tank (Armored Assault Tank)

| Field | Value |
|-------|-------|
| **Asset ID** | `cis_aat_tank` |
| **Display Name** | AAT Tank |
| **Type** | Vehicle (siege unit) |
| **Role in Gameplay** | CIS main battle tank; heavy firepower, slow movement |
| **Frequency** | 2–4 per match (CIS late-game composition) |
| **Polycount Target** | 1200–1500 triangles (rounded tan hull + rotating turret + crew compartment) |
| **Silhouette Signature** | Rounded/bulbous tan hull, rotating turret on top, crew hatches, 4 wheel pods or hover skirts, industrial proportions |
| **Color Palette** | Sandy tan hull (#C8A87A), dark brown turret base (#5C3D1E), olive green panels/details (#4A5A2A), window dark metal (#333333) |
| **Current Status** | ⚠️ Scout needed |
| **Sketchfab Search Strategy** | `"AAT tank"`, `"armored assault tank"`, `"CIS tank"`, `"hover tank low poly"`, `"battle tank sci-fi"` |
| **Tier A Candidates** | Top 2 required; must read as "tank" (chunky wheels or skirt) not "walker"; turret rotation capability |
| **Tier B Fallback** | Generic sci-fi tank models; reskin to AAT aesthetics (tan → color, add turret detail) |
| **Tier C Creation** | Start from Kenney Vehicle Pack tank-like base; modify hull shape, add rotating turret rig (optional, can be static), retexture |
| **Expected Lead Time** | 5–7 days (Tier A) or 2–3 weeks (Tier C) |
| **Priority Score** | 8/10 (Critical for CIS late-game, but less visible than swarm units) |

---

#### AT-TE Walker (All Terrain Tactical Enforcer)

| Field | Value |
|-------|-------|
| **Asset ID** | `rep_atte_walker` |
| **Display Name** | AT-TE Walker |
| **Type** | Vehicle (siege unit) |
| **Role in Gameplay** | Republic mobile fortress; heavy armor, high HP, area denial |
| **Frequency** | 1–3 per match (Republic late-game composition) |
| **Polycount Target** | 1000–1400 triangles (6 legs, chunky body, rotating head cannon, crew hatches) |
| **Silhouette Signature** | Thick-bodied quadruped with **6 legs** (not 4), rotating turret head on top, crew compartments, industrial durasteel coloring |
| **Color Palette** | Off-white primary (#F5F5F5), blue trim (#1A3A6B), cannon barrel dark metal (#333333), rust/wear accents |
| **Current Status** | ⚠️ Scout needed |
| **Sketchfab Search Strategy** | `"AT-TE walker low poly"`, `"six-legged walker"`, `"walker tank"`, `"mech walker"`, `"armored walker"` |
| **Tier A Candidates** | Top 2 required; **must have 6 legs** (not 4); turret on head critical |
| **Tier B Fallback** | Multi-leg walker models; verify 6-leg count; retexture to white/blue Republic colors |
| **Tier C Creation** | Start from Kenney or generic walker base; add extra leg pair (often comes as 4-leg, need to insert 2 more), attach rotating turret rig, texture white/blue |
| **Expected Lead Time** | 6–8 days (Tier A) or 3–4 weeks (Tier C) |
| **Priority Score** | 8/10 (Iconic Republic vehicle; less common than B1 droids, but memorable) |

**Critical Design Note:**
The **6-leg configuration** is essential to AT-TE identity. Many walker models come as 4-leg or 2-leg; verify candidate has or can be modified to have 6 legs before selecting.

---

#### Jedi Knight (Generic)

| Field | Value |
|-------|-------|
| **Asset ID** | `rep_jedi_knight` |
| **Display Name** | Jedi Knight |
| **Type** | Hero unit |
| **Role in Gameplay** | Republic supreme commander; high-impact solo unit (force powers, lightsaber, morale) |
| **Frequency** | 1 per match (summoned once per player, 1–2 on map) |
| **Polycount Target** | 600–900 triangles (robed humanoid, lightsaber, non-physical) |
| **Silhouette Signature** | Humanoid robed figure, standing/serene pose, non-detailed face (generic Jedi), blue lightsaber (non-physical particle effect, not modeled) |
| **Color Palette** | Tan/sand robes (#C8A87A), dark brown under-tunic (#5C3D1E), face neutral tones, lightsaber blue emissive (#4488FF) handled by shader not mesh |
| **Current Status** | ⚠️ Scout needed |
| **Sketchfab Search Strategy** | `"jedi low poly"`, `"robed warrior"`, `"lightsaber user"`, `"humanoid hero"`, `"force user"` |
| **Tier A Candidates** | Top 2 required; robed humanoid; **no detailed face** (should look generic, not like a specific actor); no rigged lightsaber (saber is VFX) |
| **Tier B Fallback** | Generic robed warrior/wizard models; modify proportions, remove detailed facial features if present |
| **Tier C Creation** | Start from Kenney humanoid base; add tan robes (model as draping cloth geometry or simple cube scaled), omit face detail (solid colored sphere sufficient), leave off-hand empty for lightsaber particle effect attachment point |
| **Expected Lead Time** | 5–7 days (Tier A) or 2–3 weeks (Tier C) |
| **Priority Score** | 8/10 (Hero unit for Republic; memorable but not as visible as Grievous) |

**Critical Design Note:**
**Do not model the lightsaber as geometry**. The blade is a VFX particle effect, not a physical mesh. The unit's hand should have an attachment point or subtle grip pose, and the shader/VFX system will render the glowing blade at runtime. This saves polycount and allows dynamic blade colors (blue, green, purple for variant Jedi if needed).

---

## 6. Removed Assets & Replacement Strategy

### OT Assets Being Actively Removed

| OT Asset | Reason | Clone Wars Replacement |
|----------|--------|------------------------|
| Stormtrooper | Empire soldier, not Clone Wars | Clone Trooper Phase II |
| Darth Vader | Post-Clone Wars Sith | General Grievous (CIS commander) |
| TIE Fighter | Imperial spacecraft | (Optional) LAAT Gunship or focus on ground warfare |
| X-Wing | Rebel spacecraft | (Optional) LAAT Gunship or focus on ground warfare |
| AT-AT Walker | Imperial transport | AT-TE Walker (Republic) |
| Tatooine Desert | OT home world | Geonosis Arena (Clone Wars primary battleground) |
| Hoth Ice Base | Rebellion setting | (Future) Utapau or Mustafar (Clone Wars locations) |

**Action Items:**
1. Remove OT asset files from `packs/warfare-starwars/assets/raw/` directory
2. Update `pack.yaml` description to emphasize "Clone Wars era" and remove OT references
3. Update `assets/manifest.yaml` to remove OT placeholders
4. Search codebase for hardcoded references (unlikely, but grep for "stormtrooper", "vader", "empire" in YAML/JSON)

---

## 7. Next Steps & Workstream

### Immediate (This Week)

- [ ] **Squad Formation**: Assign scout agents to Tier A search for CRITICAL assets:
  - Agent 1: Clone Trooper Phase II (Sketchfab sweep)
  - Agent 2: B1 Battle Droid (Sketchfab sweep)
  - Agent 3: Geonosis Arena (Sketchfab sweep)

- [ ] **Search Template**: Create shared Sketchfab results tracker (spreadsheet or Notion doc)

- [ ] **Cleanup**: Remove OT asset files and update documentation
  - Delete `/assets/raw/sw_stormtrooper_sketchfab_001/`
  - Delete `/assets/raw/sw_vader_hero_sketchfab_001/` (if exists)
  - Update `pack.yaml` description to remove OT references
  - Update `assets/manifest.yaml` to remove stormtrooper/Vader placeholders

### Short Term (Weeks 2–3)

- [ ] **Tier A Evaluation**: Top 3 candidates per CRITICAL asset
  - Download FBX, open in Blender, test Decimate modifier
  - Take reference screenshots (before/after decimation)
  - Rank by silhouette match → polycount achievability → license credibility

- [ ] **Tier A Integration**: Confirm top 1 candidate per CRITICAL asset
  - Create GitHub issues (e.g., `[Asset] Clone Trooper Phase II — Tier A Confirmed`)
  - Add source files to `packs/warfare-starwars/assets/raw/<asset_id>/`
  - Update `assets/manifest.yaml` entries (set `placeholder: false`, fill source paths)

- [ ] **Validation**: Run PackCompiler validator
  ```bash
  dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars
  ```

### Medium Term (Weeks 4–6)

- [ ] **Tier A for HIGH Assets**: Begin scout for General Grievous, AAT Tank, AT-TE, Jedi Knight

- [ ] **Tier B Fallback Prep**: If any Tier A search fails, activate Tier B (Blend Swap) for that asset

- [ ] **Unit Testing**: For each integrated asset:
  - Import into test Unity project
  - Verify mesh imports correctly (no scale/rotation artifacts)
  - Check polycount in-engine (Editor > Profiler > Rendering > Mesh Vertices/Triangles)
  - Take in-engine screenshot against placeholder for comparison

- [ ] **Texture Sourcing**: For each asset, acquire or generate textures
  - Prefer solid colors (cheaper than 2k/4k PBR maps)
  - Target 256x256 or 512x512 texture resolution (power-of-two)
  - Use GIMP/Aseprite to paint TABS-style solid colors + minimal detail

### Long Term (Weeks 7–12)

- [ ] **Tier C Creation** (if needed): For any asset remaining with no Tier A/B candidate
  - Assign to Blender expert; establish iteration cycle
  - Create in-engine mockup; gather feedback
  - Polish and optimize

- [ ] **Addressables Bundle Build**: Once all assets sourced
  - Set up Addressables group in test Unity project
  - Build bundle (generates `.bundle` + `catalog_*.json`)
  - Copy to `packs/warfare-starwars/assets/`

- [ ] **Integration Testing**:
  - Load pack with real assets in DINO
  - Verify meshes render without errors
  - Check frame rate; profile for bottlenecks

- [ ] **Documentation Updates**:
  - Update `ASSET_PIPELINE.md` with Tier A sourcing results
  - Add author/attribution to `ATTRIBUTION.md` for all CC-BY assets
  - Update `pack.yaml` to remove `placeholder: true` flags

---

## 8. Sketchfab API & Automation (Optional)

If scout agents find repeated success with Sketchfab, consider semi-automating the search process:

### Sketchfab API Integration (Future Enhancement)

**Endpoint:** `https://api.sketchfab.com/v3/search`

**Example Query:**
```bash
curl -X GET "https://api.sketchfab.com/v3/search?query=clone+trooper+low+poly&license=CC0,CC-BY&downloadable=true&type=model&sort_by=-likeCount&cursor=cD0xMg%3D%3D"
```

**Parameters:**
- `query` – Search terms (URL encoded)
- `license` – Comma-separated: `CC0`, `CC-BY`
- `downloadable` – `true` (must have DL button)
- `type` – `model` (3D models only)
- `sort_by` – `-likeCount` (most liked first)
- `limit` – 20 (max per page)

**Automation Idea:**
Create a `.yml` job in GitHub Actions:
1. Run weekly Sketchfab sweeps for CRITICAL assets
2. Parse JSON response, filter by polycount (&lt; 2000 tris estimate)
3. Generate markdown report with top 5 per asset
4. Post as GitHub issue comment (notify team)

This reduces manual search time by 70–80% but still requires human evaluation for silhouette match and license verification.

---

## 9. Quality Gates & Acceptance Criteria

### Per-Asset Acceptance Checklist

Before an asset is considered **complete**, verify:

- [ ] **License Verified** – CC0 or CC-BY confirmed on Sketchfab/Blend Swap page
- [ ] **Attribution Added** – If CC-BY: entry in `packs/warfare-starwars/assets/ATTRIBUTION.md` with author name, source URL, license text
- [ ] **Polycount Budget Met** – Triangle count ≤ target (measured in Blender Edit Mode > Mesh Stats)
- [ ] **Silhouette Correct** – Visual match to Clone Wars reference (dome helmet for clone, spindly limbs for B1, etc.)
- [ ] **No Rigging** – Armature/bones removed (delete in Blender outliner)
- [ ] **UV Unwrapped** – Smart UV Project applied; no overlaps (check in Blender UV Editor)
- [ ] **FBX Export Clean** – File imports into Blender without errors; mesh intact
- [ ] **Texture File Included** – Albedo map (PNG, power-of-two resolution) or solid color note documented
- [ ] **manifest.yaml Updated** – Entry set to `placeholder: false`, source_file path filled, source_hint updated
- [ ] **Validation Passes** – `dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars` returns 0 errors
- [ ] **In-Engine Test** – Imported to test Unity project; renders without errors; screenshot captured

### Pack-Level Acceptance Criteria

Before `warfare-starwars` is considered **asset-complete**:

- [ ] All CRITICAL assets sourced (Clone Trooper, B1 Droid, Geonosis)
- [ ] All HIGH assets sourced (Grievous, AAT, AT-TE, Jedi)
- [ ] Addressables bundle built and placed at `packs/warfare-starwars/assets/warfare-starwars-assets.bundle`
- [ ] Catalog JSON placed at `packs/warfare-starwars/assets/catalog_warfare-starwars.json`
- [ ] `pack.yaml` assets section updated with real bundle reference (no longer placeholder)
- [ ] `assets/manifest.yaml` has 0 `placeholder: true` entries for CRITICAL/HIGH assets
- [ ] ATTRIBUTION.md complete with all CC-BY author credits
- [ ] `dotnet test src/DINOForge.sln` passes (pack validation tests included)
- [ ] In-engine integration test: load pack in DINO, verify all units render in Free Combat

---

## 10. Risk Mitigation

### Potential Blockers & Mitigation

| Risk | Severity | Mitigation |
|------|----------|-----------|
| No suitable Sketchfab Clone Trooper found | Medium | Activate Tier B (Blend Swap) immediately; if that fails, greenlight Tier C custom creation (Kenney base + Blender modifications) |
| B1 Droid polycount consistently over budget | Medium | Increase Decimate ratio (0.3–0.4 instead of 0.5); accept simpler silhouette; consider splitting into 2 meshes (head + body) |
| Licensing ambiguity (asset marked CC-BY but no author name) | High | **Do not use**; move to next candidate. Err on side of caution (no licenses with NC, ND, SA clauses). |
| Geonosis arena no Sketchfab match | Medium | Blend Swap fallback (generic arena/colosseum models); Tier C: build octagonal floor + walls from scratch in Blender (est. 4–6 hours labor) |
| Grievous 4-arm silhouette breaks in decimation | Medium | Rig 4 arms via parenting in Blender (non-destructive); decimate body separately; re-parent after. Or: find asset with integral 4-arm mesh. |
| Texture file in source not freely licensed | High | Ignore texture; use solid color in Blender (apply white/tan material); note in asset_manifest.json |

### Contingency Plan

**If Tier A fails to yield ≥2 candidates for CRITICAL asset:**
1. Immediately escalate to Tier B (Blend Swap search)
2. If Tier B yields result: proceed with intake
3. If Tier B yields nothing: flag for Tier C (custom creation) & request Blender expert assignment
4. Establish weekly sync with assigned agent; daily Slack updates
5. Do NOT wait idle; activate next asset in queue while Tier C is in progress

---

## 11. Conclusion & Call to Action

This **Clone Wars Asset Sourcing Manifest** establishes:

✅ **Clear era focus**: Clone Wars prequels (Episodes I–III), not Original Trilogy
✅ **Gameplay alignment**: Republic vs. CIS perfectly maps to DINO's faction/doctrine system
✅ **Prioritized asset list**: CRITICAL → HIGH → MEDIUM → LOW; clear polycount budgets
✅ **Three-tier sourcing strategy**: Sketchfab API first, Blend Swap second, custom creation last resort
✅ **Actionable next steps**: Week-by-week workstream with specific agent assignments
✅ **Quality gates**: Checklist ensures no asset slips through without proper vetting

**For Scout Agents:**
- Start with Sketchfab searches for **Clone Trooper**, **B1 Droid**, **Geonosis Arena**
- Document top 3 candidates per asset in shared spreadsheet
- Evaluate Decimate feasibility in Blender; rank by silhouette match
- Open GitHub issues for Tier A confirmations; prepare intake by Week 2

**For Project Leads:**
- Monitor agent progress against weekly milestones
- Activate Tier B/C escalations if Tier A searches stall
- Schedule integration testing once first 3 assets are confirmed

**For Contributors:**
- Follow the "Free Asset Sources" section (Kenney, PolyPizza, Sketchfab CC0/CC-BY only)
- Reference the "Contributing Assets" checklist before submitting PRs
- Coordinate with team to avoid duplicate asset sourcing

The Clone Wars vision is **clear**. Execution is **parallelizable**. Quality is **gated**. Let's build this.

---

## Appendix A: Sketchfab Free Search Quick Links

Ready to scout? Start here:

- **Clone Trooper:** https://sketchfab.com/search?q=clone%20trooper%20low%20poly&license=CC0,CC-BY&downloadable=true
- **B1 Droid:** https://sketchfab.com/search?q=B1%20droid&license=CC0,CC-BY&downloadable=true
- **AAT Tank:** https://sketchfab.com/search?q=AAT%20tank&license=CC0,CC-BY&downloadable=true
- **AT-TE Walker:** https://sketchfab.com/search?q=AT-TE%20walker&license=CC0,CC-BY&downloadable=true
- **Geonosis:** https://sketchfab.com/search?q=geonosis%20arena&license=CC0,CC-BY&downloadable=true
- **General Grievous:** https://sketchfab.com/search?q=grievous&license=CC0,CC-BY&downloadable=true
- **Jedi Knight:** https://sketchfab.com/search?q=jedi%20low%20poly&license=CC0,CC-BY&downloadable=true

---

## Appendix B: Blender Reference Checklist

**Import FBX → Evaluate → Decimate → Cleanup → Export**

```markdown
### Blender Workflow (Fast Reference)

1. **Import**: File > Import > FBX Import (.fbx)
2. **Check Polycount**: Edit Mode → Mesh Stats overlay (check "Faces" count)
3. **Add Decimate**:
   - Object Mode
   - Modifiers panel > Add > Decimate
   - Type: Collapse
   - Ratio: 0.5 (adjust as needed)
   - Apply
4. **Remove Rig** (if present):
   - Outliner: select Armature
   - Delete (X key)
5. **Unwrap UV**:
   - Edit Mode > Select All (A)
   - U > Smart UV Project
6. **Export FBX**:
   - File > Export > FBX Export
   - Name: [asset_id].fbx
   - Check "Selected Objects" if needed
   - Check "Animation" OFF (for static meshes)
```

---

**Document Version:** 1.0
**Last Updated:** 2026-03-11
**Next Review:** 2026-04-08 (1 month; assess agent progress, update blockers)
**Owner:** DINOForge Project Lead
**Stakeholders:** Scout Agents, Blender Experts, Integration Testers, Pack Maintainers
