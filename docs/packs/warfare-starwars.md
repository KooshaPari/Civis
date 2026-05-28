---
title: "Star Wars - Clone Wars"
layout: doc
---

# Star Wars - Clone Wars

<div class="pack-header">
  <div class="pack-meta">
    <div class="pack-info">
      <p class="pack-version">Version 0.1.0</p>
      <p class="pack-type">Total Conversion</p>
      <p class="pack-author">By DINOForge</p>
    </div>
    <div class="pack-framework">
      <span class="label">Framework:</span>
      <span class="value">>=0.5.0 &lt;0.26.0</span>
    </div>
  </div>
</div>

## Overview

A total conversion transporting DINO into the Star Wars universe during the Clone Wars. Command the clone trooper legions of the Galactic Republic against the mechanized forces of the Separatist Confederacy.

### Factions Included

- **Galactic Republic**: Disciplined elite clone army with Jedi commanders, strong morale, and specialized forces
- **Confederacy (CIS)**: Mechanized swarm doctrine with rolling thunder tactics and droid legions

## Content Summary

| Category | Count | Details |
|----------|-------|---------|
| Factions | 2 | Galactic Republic, Separatist Confederacy |
| Units | 26 per side | Clone troopers, ARF troopers, Jedi knights, Commandos, Battle droids, etc. |
| Buildings | 20 | Clone facilities, Weapons factories, Shield generators, Research labs, etc. |
| Doctrines | 6 | Elite discipline, Jedi leadership, Defensive formation, Mechanized attrition, Rolling thunder, Swarm protocol |
| Weapons | 1 | Blasters and energy weapons |
| Wave Templates | Campaign-ready | Clone Wars themed waves for multiplayer campaigns |

## Feature Highlights

### Clone Trooper Legions
Command legendary units including:
- Standard clone troopers with squad cohesion
- ARC (Advanced Recon Commando) troopers with enhanced equipment
- Heavy troops armed with anti-armor weapons
- Vehicle crews operating AT-TE walkers
- ARF troopers (Advanced Recon Force) for reconnaissance
- Speeder bike pilots for rapid deployment
- Jedi Knight units with special Force bonuses
- Clone commandos with elite training

### Separatist Forces
Build a mechanized empire with:
- Battle droids in formation tactics
- Super Battle Droids for heavy combat
- Droid commanders orchestrating offensives
- Laser cannon batteries
- Droid factories for sustained production
- Shield technologies

### Strategic Doctrines

**Republic Doctrines:**
- Elite Discipline: Bonus to clone trooper coordination
- Jedi Leadership: Enhance Force abilities, morale bonuses
- Defensive Formation: Tighter positioning, reduced casualties

**CIS Doctrines:**
- Mechanized Attrition: Sustained droid production
- Rolling Thunder: Mobile assault tactics
- Swarm Protocol: Coordinated drone tactics

## Installation

Install via the DINOForge installer or manually:

```bash
dinoforge pack install warfare-starwars
```

Or via the in-game mod manager (F10), search for "Star Wars - Clone Wars".

## Configuration

This pack provides in-game settings accessible via the F10 mod panel:

- **Difficulty Multiplier** (0.5 - 3.0): Adjust enemy unit stats for harder or easier gameplay
- **Enable Jedi Powers** (on/off): Jedi units gain Force ability bonuses
- **Clone Trooper Quality** (recruit/standard/elite/legendary): Training level for Republic forces affecting unit stats

## Dependencies

This pack is self-contained with no required dependencies.

## Compatibility

- Framework version: >=0.5.0 <0.26.0
- Minimum DINOForge: 0.5.0
- Game: Diplomacy is Not an Option (Unity 2021.3+)
- Compatible with: Vanilla DINO base

## Asset Notes

Visual assets include placeholder models. Community contributions welcome for:
- High-poly character models (clone troopers, Jedi, droids)
- Vehicle models (AT-TE, speeder bikes, droid carriers)
- Building architecture (clone bases, droid factories)
- Particle effects (blaster bolts, Force powers, explosions)

See `assets/ASSET_PIPELINE.md` for the TABS-style art guide, free CC0/CC-BY source lists (Kenney.nl, PolyPizza, Sketchfab), and Unity 2021.3 + Addressables import workflow.

## Gameplay Experience

Experience the conflict between biological discipline and mechanical coordination in a galaxy far, far away. The Republic's tactical advantage of elite training meets the CIS's numerical superiority and production capacity. Each faction plays fundamentally differently:

- **Republic**: Slower buildup, stronger individual units, doctrine-based synergies
- **CIS**: Rapid deployment, swarm tactics, economic advantages

## Support & Contribution

For issues, bug reports, feature requests, or asset contributions, visit the [DINOForge GitHub repository](https://github.com/KooshaPari/Dino).

---

<div class="pack-footer">
  <div class="pack-links">
    <a href="/packs" class="button-secondary">← Back to Registry</a>
    <a href="https://github.com/KooshaPari/Dino/issues/new" class="button-primary">Report Issue</a>
  </div>
</div>

<style scoped>
.pack-header {
  background: linear-gradient(135deg, rgba(255, 232, 31, 0.1), rgba(192, 57, 43, 0.1));
  border: 1px solid rgba(255, 232, 31, 0.3);
  border-radius: 12px;
  padding: 24px;
  margin: 24px 0 32px 0;
}

.pack-meta {
  display: flex;
  gap: 32px;
  flex-wrap: wrap;
}

.pack-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.pack-info p {
  margin: 0;
  font-size: 14px;
}

.pack-version {
  font-weight: 600;
  color: var(--vp-c-brand);
}

.pack-type {
  background: rgba(168, 85, 247, 0.2);
  color: #a855f7;
  display: inline-block;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  width: fit-content;
}

.pack-author {
  color: var(--vp-c-text-2);
  font-size: 13px;
}

.pack-framework {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--vp-c-bg);
  border-radius: 8px;
}

.pack-framework .label {
  font-weight: 500;
  color: var(--vp-c-text-2);
  font-size: 13px;
}

.pack-framework .value {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 12px;
  color: var(--vp-c-text-1);
  background: var(--vp-c-bg-soft);
  padding: 2px 8px;
  border-radius: 4px;
}

.pack-footer {
  margin-top: 48px;
  padding-top: 24px;
  border-top: 1px solid var(--vp-c-divider);
}

.pack-links {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.button-secondary,
.button-primary {
  display: inline-flex;
  align-items: center;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  text-decoration: none;
  transition: all 0.2s ease;
}

.button-secondary {
  border: 1px solid var(--vp-c-divider);
  color: var(--vp-c-text-1);
  background: var(--vp-c-bg-soft);
}

.button-secondary:hover {
  border-color: var(--vp-c-brand);
  background: var(--vp-c-bg);
}

.button-primary {
  background: var(--vp-c-brand);
  color: white;
}

.button-primary:hover {
  background: var(--vp-c-brand-dark);
  transform: translateY(-1px);
}

@media (max-width: 768px) {
  .pack-header {
    padding: 16px;
  }

  .pack-meta {
    flex-direction: column;
    gap: 16px;
  }

  .pack-framework {
    flex-direction: column;
    align-items: flex-start;
  }

  .pack-links {
    flex-direction: column;
  }

  .button-secondary,
  .button-primary {
    width: 100%;
    justify-content: center;
  }
}
</style>
