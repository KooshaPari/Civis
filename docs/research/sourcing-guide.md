# Clone Wars Sourcing Guide — Quick Start

**Full Manifest:** See `CLONE_WARS_SOURCING_MANIFEST.md` (762 lines, comprehensive)

This quick guide summarizes the key actions for scout agents and project leads.

---

## TL;DR

**warfare-starwars pack is being redirected from Original Trilogy → Clone Wars prequels.**

- **Old focus:** Stormtroopers, TIE fighters, Tatooine (Episodes IV–VI)
- **New focus:** Clone Troopers, Battle Droids, Geonosis (Episodes I–III)
- **Why:** Clone Wars duality (disciplined clones vs. mechanized droids) perfectly aligns with DINO's faction system

---

## For Scout Agents: Priority Asset Search

### Week 1–2: Scout CRITICAL Assets

Start here. These three assets define the pack's identity.

| Asset | Search Term | Target | Status |
|-------|-------------|--------|--------|
| **Clone Trooper** | `"clone trooper low poly"` or `"phase II armor"` | 350–450 tris | ⚠️ Found; needs decimation |
| **B1 Battle Droid** | `"B1 droid"` or `"spindly robot"` | 250–400 tris | ⚠️ Scout needed |
| **Geonosis Arena** | `"geonosis arena"` or `"octagonal structure"` | 400–800 tris (modular) | ⚠️ Scout needed |

**Search on:** [Sketchfab Free](https://sketchfab.com/search?features=downloadable&sort_by=-likeCount)
- Filter by: License = CC0 or CC-BY, Downloadable = Yes
- Find top 3 candidates per asset
- Evaluate in Blender (Edit Mode > Mesh Stats; try Decimate modifier if over budget)

### Week 3–4: Scout HIGH Assets

Secondary priority; still high impact.

| Asset | Search Term | Target | Status |
|-------|-------------|--------|--------|
| **General Grievous** | `"grievous"` or `"four-armed cyborg"` | 800–1200 tris | ⚠️ Scout needed |
| **AAT Tank** | `"AAT tank"` or `"armored assault tank"` | 1200–1500 tris | ⚠️ Scout needed |
| **AT-TE Walker** | `"AT-TE walker"` or `"six-legged walker"` | 1000–1400 tris | ⚠️ Scout needed |
| **Jedi Knight** | `"jedi low poly"` or `"robed warrior"` | 600–900 tris | ⚠️ Scout needed |

---

## For Project Leads: Progress Tracking

### Milestones

- **Week 1–2:** CRITICAL assets scouted (3/3 candidates found)
- **Week 3–4:** CRITICAL assets ranked; Tier A confirmed
- **Week 5–6:** HIGH assets scouted; Tier A confirmed
- **Week 7–8:** All assets integrated into `assets/manifest.yaml`
- **Week 9–10:** Addressables bundle built; tested in-engine
- **Week 11+:** Texture sourcing & polish

### Escalation Path

If Tier A (Sketchfab) fails for any asset:
1. **Activate Tier B:** Blend Swap search (`.blend` source files)
2. **If Tier B fails:** Greenlight Tier C (custom creation with Blender expert)
3. **Daily updates:** Async Slack updates; weekly sync call if blocked

---

## Quality Checklist (Per Asset)

Before an asset is considered **done**:

- [ ] License verified (CC0 or CC-BY, no NC/ND/SA clauses)
- [ ] Polycount within budget (measured in Blender Edit Mode)
- [ ] Silhouette matches Clone Wars reference
- [ ] No rigging (armature deleted in Blender)
- [ ] UV unwrapped (Smart UV Project)
- [ ] FBX export tested (no import errors)
- [ ] Texture file included (PNG, power-of-two)
- [ ] `assets/manifest.yaml` entry updated (`placeholder: false`)
- [ ] PackCompiler validation passes
- [ ] In-engine screenshot captured (Unity test project)

---

## Removed Assets (OT → Clone Wars Shift)

These assets are **explicitly removed**:

| OT Asset | Reason | Clone Wars Replacement |
|----------|--------|------------------------|
| Stormtrooper | Empire infantry | Clone Trooper Phase II |
| Darth Vader | Post-Clone Wars Sith | General Grievous |
| TIE Fighter | Imperial spacecraft | (Optional) LAAT Gunship |
| X-Wing | Rebel spacecraft | (Optional) LAAT Gunship |
| AT-AT Walker | Imperial transport | AT-TE Walker |
| Tatooine | OT home world | Geonosis Arena |
| Hoth | Rebellion ice base | (Future) Utapau/Mustafar |

**Action:** Remove OT files from `assets/raw/`, update `pack.yaml` description, purge OT references from manifests.

---

## Sketchfab Quick Links

Ready to start scouting? Use these pre-filtered searches:

- **Clone Trooper:** https://sketchfab.com/search?q=clone%20trooper%20low%20poly&license=CC0,CC-BY&downloadable=true
- **B1 Droid:** https://sketchfab.com/search?q=B1%20droid&license=CC0,CC-BY&downloadable=true
- **Geonosis:** https://sketchfab.com/search?q=geonosis%20arena&license=CC0,CC-BY&downloadable=true
- **Grievous:** https://sketchfab.com/search?q=grievous&license=CC0,CC-BY&downloadable=true
- **AAT Tank:** https://sketchfab.com/search?q=AAT%20tank&license=CC0,CC-BY&downloadable=true
- **AT-TE Walker:** https://sketchfab.com/search?q=AT-TE%20walker&license=CC0,CC-BY&downloadable=true
- **Jedi Knight:** https://sketchfab.com/search?q=jedi%20low%20poly&license=CC0,CC-BY&downloadable=true

---

## Blender Quick Reference

```bash
# Import FBX
File > Import > FBX Import

# Check polycount
Edit Mode > Overlays > Statistics (check "Faces" count)

# Reduce polycount
Object Mode > Modifiers > Add > Decimate
  Type: Collapse
  Ratio: 0.5 (adjust as needed)
  Apply

# Remove rig (if present)
Outliner: select Armature > Delete (X key)

# UV unwrap
Edit Mode > Select All (A) > U > Smart UV Project

# Export
File > Export > FBX Export
```

---

## For Contributors

**Want to contribute assets?**

1. Follow "Free Asset Sources" in the full manifest (Kenney, PolyPizza, Sketchfab CC0/CC-BY)
2. Check license carefully — **no NC, ND, or SA clauses**
3. If CC-BY: add author credit to `packs/warfare-starwars/assets/ATTRIBUTION.md`
4. Test in Blender; verify polycount budget
5. Update `assets/manifest.yaml` entry (set `placeholder: false`)
6. Run validator: `dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars`
7. Submit PR with screenshot in description

---

## Questions?

Refer to **CLONE_WARS_SOURCING_MANIFEST.md** for:
- Full scope shift rationale (section 1)
- Detailed asset specs (section 5)
- Week-by-week workstream (section 7)
- Risk mitigation (section 10)

**Project Lead:** @kooshapari
**Last Updated:** 2026-03-11
