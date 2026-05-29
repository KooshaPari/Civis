# Contributing Packs to DINOForge

Thank you for contributing new packs to DINOForge! This guide explains how to submit packs and what to expect during the review process.

## Quick Start

1. **Fork the repo** and create a feature branch: `feature/pack-mymodname`
2. **Create your pack** in the `packs/` directory with a valid `pack.yaml` manifest
3. **Test locally** with `dotnet run --project src/Tools/PackCompiler -- validate packs/mypack --strict`
4. **Push and open a PR** — validation runs automatically
5. **Address any feedback** from validation or code review
6. **Merge** — your pack is live on the next release

---

## Pack Structure

Every pack requires this minimal structure:

```
packs/my-pack/
├── pack.yaml                 # Manifest (required)
├── README.md                 # Documentation (recommended)
├── <content-type>/          # content directory (units, factions, buildings, etc.)
└── assets/                  # (optional) visual assets, bundles, etc.
    ├── bundles/
    └── ...
```

### Minimal pack.yaml

```yaml
id: my-pack
name: My Awesome Pack
version: 0.1.0
framework_version: ">=0.23.0"
author: Your Name <you@example.com>
description: A brief description of what this pack does
type: content  # or balance, ruleset, scenario, total_conversion, utility
license: MIT   # or GPL-3.0, Apache-2.0, CC-BY-4.0, etc.

# Optional: if your pack depends on others
depends_on:
  - warfare-modern

# Optional: if your pack conflicts with others
conflicts_with: []

# What this pack registers
loads:
  factions: []
  units: []
  buildings: []
  weapons: []
  # ... other registries
```

See `schemas/pack-manifest.schema.json` for the complete schema.

---

## Pack Types

Choose the type that best describes your pack:

| Type | Purpose | Example |
|------|---------|---------|
| **content** | New gameplay entities (units, buildings, factions) | `warfare-starwars`, `economy-balanced` |
| **balance** | Numerical adjustments to existing content | Economy rebalance, unit stat tweaks |
| **ruleset** | Game rule modifications | Wave timing, victory conditions |
| **scenario** | Scripted maps, campaigns, or challenges | Tutorial scenarios, campaign packs |
| **total_conversion** | Overhauls core gameplay (maps, factions, etc.) | Full alternate universe mod |
| **utility** | Tools, QoL, UI enhancements | Debug overlays, mod menu integrations |
| **theme** | Visual customization (colors, fonts, audio) | Star Wars skins, faction color palettes |

---

## Required Fields in pack.yaml

Every pack **must** include:

- **id** — Unique identifier (kebab-case, no spaces or special chars)
  - Example: `my-awesome-pack`, `economy-2x-rates`
  - Used in manifests, file names, and pack selection
  
- **name** — Human-readable title (any casing)
  - Example: `My Awesome Pack`, `Economy 2x Rates`

- **version** — Semantic versioning (MAJOR.MINOR.PATCH)
  - Example: `0.1.0`, `1.0.0`
  - Bump when publishing updates

- **framework_version** — DINOForge version compatibility
  - Example: `">=0.23.0"`, `"^0.25.0 <1.0.0"`
  - Ensure your pack targets a released DINOForge version

- **author** — Your name or team
  - Example: `John Doe`, `Modding Squad <team@example.com>`

- **type** — Pack classification (see table above)

- **license** — Open-source license for community packs
  - Required for all submissions (enforced at review time)
  - Common choices: `MIT`, `GPL-3.0`, `Apache-2.0`, `CC-BY-4.0`

---

## Content Guidelines

### License Requirements

**All community packs must use an open-source license.** This ensures DINOForge users can modify, fork, and remix packs.

- ✅ Acceptable: MIT, GPL-3.0, Apache-2.0, CC-BY-4.0, Unlicense
- ❌ Not allowed: Proprietary, "All Rights Reserved", closed-source

When you open a PR, declare the license in your pack.yaml and in the pack's README.

### Code of Conduct

- **No hate speech, harassment, or discrimination** in pack names, descriptions, or content
- **No spam packs** — don't submit packs that are duplicates or placeholders
- **Respect intellectual property** — don't include copyrighted assets without permission
- **Be respectful in code review** — maintainers and contributors are volunteers

Violations may result in PR rejection or pack removal.

### Testing Your Pack

Before submitting:

```bash
# Install .NET 11 preview (if you haven't already)
# See https://dotnet.microsoft.com/download/dotnet/11.0

# Validate locally
dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack --strict

# Build and package
dotnet run --project src/Tools/PackCompiler -- build packs/my-pack
```

Fix any validation errors before opening the PR. The CI workflow will validate again automatically.

---

## Pull Request Checklist

Before opening a PR, ensure:

- [ ] Pack has a valid `pack.yaml` with all required fields
- [ ] `pack.yaml` specifies an open-source license
- [ ] `pack.yaml` lists correct `framework_version` (should be a released DINOForge version)
- [ ] Pack has a `README.md` explaining what it does
- [ ] Pack validates locally: `dotnet run --project src/Tools/PackCompiler -- validate packs/<your-pack> --strict`
- [ ] All pack.yaml IDs are unique (don't conflict with existing packs)
- [ ] If your pack has dependencies, they are listed in `depends_on`
- [ ] Branch is up to date with `main` (no merge conflicts)
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)

### PR Template

```markdown
## Description

What does your pack do?

## Type

- [ ] Content (new units, buildings, factions)
- [ ] Balance (stat tweaks)
- [ ] Ruleset (game mechanics)
- [ ] Scenario (maps, campaigns)
- [ ] Total Conversion (major overhaul)
- [ ] Utility (tools, QoL)
- [ ] Theme (visual customization)

## Testing

How should maintainers test this pack?

- [ ] Launch game with pack enabled
- [ ] Verify [specific feature] works as expected
- [ ] No console errors in BepInEx log

## Dependencies

- Does this pack depend on other community packs? List them.
- Does this pack require a specific DINOForge version?

## License

This pack is licensed under: [MIT / GPL-3.0 / Apache-2.0 / CC-BY-4.0 / other]
```

---

## Review Process

1. **Automated Validation** (CI) — Your pack is validated against the schema
   - If validation fails, you'll see a comment with specific errors
   - Fix errors and push again — the workflow runs on every commit

2. **Code Review** — A maintainer reviews your pack for:
   - Correctness and completeness
   - License compliance
   - Potential conflicts with existing packs
   - Documentation quality
   - Community guidelines adherence

3. **Merge** — Once approved, your pack is merged to `main`

4. **Release** — Your pack ships in the next DINOForge release

---

## Common Validation Errors

### ❌ Missing required field: 'id'

**Fix:** Add `id: my-pack` to pack.yaml (kebab-case, unique)

### ❌ Invalid version format: '1.0'

**Fix:** Use semantic versioning: `version: 1.0.0`

### ❌ framework_version must be a valid semver range

**Fix:** Use valid range syntax:
- ✅ `">=0.23.0"`
- ✅ `"^0.25.0"`
- ✅ `">=0.23.0 <1.0.0"`

### ❌ Unknown pack type: 'cosmetic'

**Fix:** Use one of the valid types: `content`, `balance`, `ruleset`, `scenario`, `total_conversion`, `utility`, `theme`

### ❌ Cannot resolve dependency: 'nonexistent-pack'

**Fix:** Remove the non-existent pack from `depends_on`, or add the pack it depends on in a separate PR

---

## Examples

### Minimal Content Pack (5 units)

```yaml
id: example-troops
name: Example Troops Pack
version: 0.1.0
framework_version: ">=0.23.0"
author: Jane Doe
type: content
license: MIT

loads:
  units:
    - example_knight
    - example_archer
    - example_mage
    - example_rogue
    - example_paladin
```

### Balance Pack (Modifies Existing Content)

```yaml
id: economy-2x
name: 2x Economy Pack
version: 1.0.0
framework_version: ">=0.23.0"
author: Balance Team
type: balance
license: MIT
description: >
  Doubles all resource production and trade rates.
  Speeds up gameplay for faster matches.

loads: {}
```

### Total Conversion (Full Overhaul)

```yaml
id: scifi-conversion
name: Sci-Fi Total Conversion
version: 0.5.0
framework_version: ">=0.23.0"
author: Sci-Fi Fans
type: total_conversion
license: GPL-3.0
description: >
  Complete sci-fi overhaul: laser weapons, android units,
  floating cities, and futuristic technologies.

depends_on: []
conflicts_with:
  - warfare-medieval
  - warfare-ancient

loads:
  factions:
    - alliance
    - empire
  units: [] # populated by factions
```

---

## Troubleshooting

### My PR is stuck on validation

Check the GitHub Actions log (click the red X next to your commit). Common issues:

- **pack.yaml syntax error** — use a YAML linter to check
- **Missing pack.yaml** — ensure it's in the root of your pack directory
- **Invalid field names** — refer to the schema at `schemas/pack-manifest.schema.json`

### I want to add assets (bundles, 3D models)

See `docs/asset_pipeline_guide.md` for the full asset workflow. TL;DR:

1. Create `packs/my-pack/asset_pipeline.yaml` with asset metadata
2. Store 3D models (GLB/FBX) in `packs/my-pack/assets/source/`
3. Run `dotnet run --project src/Tools/PackCompiler -- assets build my-pack`
4. Built bundles land in `packs/my-pack/assets/bundles/`

### My pack has a dependency, but the dependency pack is under review

You can:
1. **Wait** for the dependency to merge, then update your framework_version
2. **Open both PRs together** and mention the dependency in your PR description
3. **Defer the dependency** — remove it now, add it in a follow-up PR later

---

## Getting Help

- **Docs**: Check `docs/pack_authoring_guide.md` for detailed content specs
- **Schema**: Open `schemas/pack-manifest.schema.json` in your editor for field definitions
- **Examples**: Browse `packs/` to see existing packs as references
- **Issues**: Open a GitHub issue if validation behavior seems wrong
- **Discord**: Join the community server for modding help (link in README)

---

## FAQ

**Q: Can I use 3rd-party art/music in my pack?**  
A: Only with explicit permission. Give credit in your README and declare the original author. Ensure the license allows redistribution.

**Q: What if I want to update my pack after it's merged?**  
A: Bump the version in pack.yaml, commit, and open a new PR. DINOForge will auto-detect the update.

**Q: Can I hide my pack from the mod menu?**  
A: Set `hidden: true` in pack.yaml. Useful for internal/testing packs.

**Q: How do I report a pack that violates the code of conduct?**  
A: Open a GitHub issue or contact maintainers privately at kooshapari@gmail.com.

---

## License

By contributing a pack to DINOForge, you agree that:

1. Your pack uses an open-source license (see above)
2. Your content respects third-party IP rights
3. DINOForge maintainers may provide feedback and request changes
4. Your pack will be distributed as-is in DINOForge releases

Thank you for contributing! 🎮✨
