# DINOForge Pack Registry

This directory contains the pack registry and documentation for all available DINOForge content packs.

## Files

- **`index.md`** - Main pack browser page with filtering and search
- **`registry.json`** - Machine-readable registry for programmatic access
- **`*.md`** - Individual pack detail pages (auto-generated from pack.yaml)

## Pack Registry Features

### Browser Interface (`index.md`)

The main pack index provides a modern browsing experience with:

- **Search**: Find packs by name, author, or description
- **Type Filtering**: Filter by pack type (Total Conversion, Content, Balance, etc.)
- **Sorting**: Sort by name, version, author, or type
- **Pack Cards**: Rich visual cards showing key metadata and content counts
- **Responsive Design**: Works on desktop, tablet, and mobile

### Features per Pack Card

Each pack card displays:
- Pack icon (256x256)
- Type badge with color coding
- Name and version
- Author attribution
- Short description (100 chars)
- Content statistics (units, buildings, factions, weapons)
- "View Details" link

### Type Colors

Pack types are color-coded for quick identification:
- **Content** (Blue): Standard packs adding new units, buildings, or items
- **Total Conversion** (Purple): Complete game replacements or new themes
- **Balance** (Green): Economy, unit stats, or gameplay tweaks
- **Scenario** (Orange): Campaign scripting and custom victory conditions
- **Utility** (Cyan): Tools and helper packs
- **Ruleset** (Pink): Rule modifications and variant rulebooks

## Machine-Readable Registry

The `registry.json` file provides programmatic access to all pack metadata:

```json
{
  "packs": [
    {
      "id": "warfare-starwars",
      "name": "Star Wars - Clone Wars",
      "version": "0.1.0",
      "author": "DINOForge",
      "type": "total_conversion",
      "description": "...",
      "url": "/packs/warfare-starwars",
      "iconUrl": "/packs/warfare-starwars/icon.png",
      "factionCount": 2,
      "unitCount": 26,
      "buildingCount": 20,
      "framework_version": ">=0.5.0 <0.26.0"
    }
  ],
  "total": 9,
  "generated": "2026-05-28T..."
}
```

Use this API for:
- Package managers and mod launchers
- CLI tooling (`dinoforge pack list`)
- Third-party pack browsers
- Version compatibility checking

## Adding a New Pack

To add your pack to the registry:

1. **Create pack structure** (if not already done):
   ```
   packs/your-pack-id/
     pack.yaml          # Required: Metadata and manifest
     icon.png           # Optional: 256x256 pack icon
     units/
     buildings/
     factions/
     ...
   ```

2. **Add pack metadata** to `pack.yaml`:
   ```yaml
   id: your-pack-id
   name: Your Pack Name
   version: 0.1.0
   author: Your Name
   type: content  # content, balance, total_conversion, scenario, utility, ruleset
   description: |
     Your pack description here.
     Can span multiple lines.
   
   framework_version: ">=0.5.0 <0.26.0"
   depends_on: []
   conflicts_with: []
   
   loads:
     units: [units]
     factions: [factions]
   ```

3. **Generate registry** (from repo root):
   ```bash
   ./scripts/generate-pack-registry.ps1
   ```

4. **Verify** the registry is updated:
   - `docs/packs/registry.json` should contain your pack
   - `docs/packs/your-pack-id.md` should be generated

5. **Submit** via pull request or GitHub issue

## Pack Icons

Pack icons should be:
- **Size**: 256x256 pixels (square)
- **Format**: PNG with transparency
- **Location**: `packs/<pack-id>/icon.png`
- **Style**: Clear, representative of pack content

A default icon is used if none is provided.

## Vue Components

Two Vue components support the pack browser:

### `<PackCard />`

Renders an individual pack card with metadata.

Props:
```typescript
pack: {
  id: string
  name: string
  version: string
  author: string
  type: 'content' | 'balance' | 'total_conversion' | 'scenario' | 'utility' | 'ruleset'
  description: string
  url: string
  iconUrl?: string
  factionCount: number
  unitCount: number
  buildingCount: number
}
```

### `<PackFilter />`

Provides search, type filtering, and sorting controls.

Events:
```typescript
@filter="({ search, types, sort }) => {}"
```

## Generating the Registry

The registry is automatically generated from `pack.yaml` files:

```bash
./scripts/generate-pack-registry.ps1
```

This script:
- Scans all `packs/*/pack.yaml` files
- Extracts metadata (name, version, author, etc.)
- Counts content items (units, buildings, factions, etc.)
- Copies pack icons to the public directory
- Generates `registry.json`
- Creates/updates per-pack markdown pages

## SEO & Metadata

Each pack detail page includes:
- Open Graph metadata for social sharing
- Semantic HTML structure
- Keyword-rich descriptions
- Related links and resources

## Future Enhancements

Planned features for the pack store:

- [ ] Screenshot galleries per pack
- [ ] User ratings and reviews
- [ ] GitHub star counts (if pack repo provided)
- [ ] Weekly/monthly download statistics
- [ ] Featured pack carousel on homepage
- [ ] Pack collections/playlists
- [ ] Advanced filtering (framework version, content types)
- [ ] Donation/sponsorship links per author
- [ ] Multi-language support

## Troubleshooting

**Issue**: Registry not updating after adding a pack

**Solution**: Run `./scripts/generate-pack-registry.ps1` to regenerate.

---

**Last Updated**: 2026-05-28

For issues or questions, visit the [DINOForge GitHub repository](https://github.com/KooshaPari/Dino).
