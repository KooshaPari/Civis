# Badge Criteria

DINOForge packs can display small visual badges in the F10 mod menu detail pane.
Badges give players at-a-glance information about a pack's status, quality, or type.

## Badge Types

Badges come from three sources:

| Source | Badges |
|--------|--------|
| Author-declared (in `pack.yaml`) | `early-access`, `total-conversion` |
| Curated (DINOForge team, signed list) | `verified-author`, `editors-choice` |
| Auto-computed at runtime | `popular`, `compatibility-tested` |

---

## Individual Badges

### `early-access`
**Colour:** Blue  
**Who assigns:** Pack author (self-declared)

Marks a work-in-progress release. Content may be incomplete, mechanics may change,
and save compatibility is not guaranteed between versions. Players should expect
rough edges.

**How to earn:** Declare it yourself in `pack.yaml` while the pack is in active
development. Remove it when the pack reaches a stable 1.0 release.

---

### `total-conversion`
**Colour:** Purple  
**Who assigns:** Pack author (self-declared)

Indicates the pack replaces the game's default factions, units, and theme with
an entirely different setting. Total conversions conflict with other total
conversions by definition.

**How to earn:** Declare it in `pack.yaml`. Typically paired with `type: total_conversion`.

---

### `verified-author`
**Colour:** Green  
**Who assigns:** DINOForge team (curated, signed list)

The pack author's identity has been confirmed by the DINOForge team. This badge
cannot be self-assigned — any `verified-author` value in `pack.yaml` is silently
stripped at runtime. It is granted via the curated allowlist in `BadgeComputer.CuratedBadges`.

**How to earn:** Contact the DINOForge team and provide proof of authorship. Once
approved, the pack ID is added to the curated list.

---

### `editors-choice`
**Colour:** Gold  
**Who assigns:** DINOForge team (curated, signed list)

Recognises an outstanding pack for quality, creativity, or completeness. Like
`verified-author`, this cannot be self-assigned.

**How to earn:** By invitation from the DINOForge team. Packs are nominated
periodically from the registry.

---

### `popular`
**Colour:** Orange  
**Who assigns:** Auto-computed at runtime (future: registry download counter)

Awarded when a pack exceeds 100 downloads in the DINOForge registry. Currently
reserved for future registry integration — the badge is never emitted by the
present runtime because the download counter is not yet tracked server-side.

**How to earn:** Publish the pack to the registry and accumulate 100+ downloads.

---

### `compatibility-tested`
**Colour:** Green  
**Who assigns:** Auto-computed at runtime

Awarded automatically when the pack contains concrete loadable content (units,
factions, or buildings declared in `loads:`) and has therefore been exercised
by CI pack-validation tests. Packs with empty or missing `loads:` sections do
not receive this badge.

**How to earn:** Declare at least one unit, faction, or building in your pack's
`pack.yaml` `loads:` section and ensure it passes `dotnet test`.

---

## Declaring Badges in pack.yaml

Add a `badges:` array to your manifest. Only `early-access` and `total-conversion`
are valid author-declared values — all others are ignored:

```yaml
id: my-cool-pack
name: My Cool Pack
version: 0.1.0
badges:
  - early-access
  - total-conversion
```

Any unrecognised values are silently dropped so future badge names cannot be
self-granted before they are officially supported.

---

## Technical Details

| Component | Location |
|-----------|----------|
| Schema | `schemas/pack-manifest.schema.yaml` → `badges` property |
| Manifest field | `src/SDK/PackManifest.cs` → `Badges` |
| Badge computation | `src/Runtime/UI/BadgeComputer.cs` |
| Runtime rendering | `src/Runtime/UI/Badges/BadgeRenderer.cs` |
| UI wiring | `src/Runtime/UI/ModMenuPanel.cs` → `RefreshBadgesRow()` |
| PNG assets | `assets/badges/*.png` (deployed by `DeployBadgeAssets` MSBuild target) |
| Asset generation | `scripts/generate-badge-placeholders.ps1` |

Badge PNG files (24×24) are deployed to
`BepInEx/plugins/dinoforge-ui-assets/badges/` by the `DeployBadgeAssets` MSBuild
target when building with `-p:DeployToGame=true -p:TargetFramework=netstandard2.0`.
The renderer falls back to a flat-colour square with the badge initial if the PNG
is absent.
