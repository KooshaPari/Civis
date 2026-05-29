# Art / Asset Licensing Strategy

This note is a practical risk-management guide for DINOForge community packs.
It is not legal advice. The goal is to reduce infringement risk while keeping the pack pipeline usable for volunteers.

## 1) Fair-use considerations for non-commercial fan mods

Non-commercial fan work is **not automatically legal**. In U.S. law, fair use depends on a case-by-case balancing of the four statutory factors:

- Purpose and character of the use
- Nature of the copyrighted work
- Amount and substantiality used
- Effect on the market for the original

Source: U.S. Copyright Office fair use overview.  
https://www.copyright.gov/fair-use/

Practical reading for DINOForge:

- A free mod can still infringe if it copies protected assets or creates market-substitute confusion.
- “No money changed hands” helps only marginally; it is not a shield.
- Fan mod status is strongest when the work is transformative, original, and not a substitute for the source asset.
- Fair use is a defense, not a permission system. Do not treat it as the default licensing model for production packs.

Policy implication:

- Use fair use only as a narrow fallback for internal discussion, commentary, critique, parody, or documentation screenshots where necessary.
- Do not rely on fair use for ship models, character likenesses, logos, UI icons, textures, or sound assets in a distributable pack.

## 2) Prefer clearly licensed free sources

The safest reusable assets are ones with an explicit, permissive license and a clear attribution path.

### CC0 / public domain style sources

Creative Commons CC0 is a public-domain dedication:

- https://creativecommons.org/publicdomain/zero/1.0/

Use when:

- You want the lowest-friction legal posture.
- You need assets that can be redistributed, remixed, and shipped with minimal attribution overhead.

Attribution:

- CC0 generally does not require attribution, but we should still record the source in the pack manifest for auditability.

### CC BY sources

CC BY requires attribution:

- https://creativecommons.org/licenses/by/4.0/

Attribution should include:

- Creator name
- Asset title, if provided
- Source URL
- License name and version
- Whether changes were made

Recommended attribution text:

- `Asset: <title> by <creator>, <source URL>, licensed CC BY 4.0, modified by DINOForge contributors.`

### SIL Open Font License fonts

The SIL Open Font License is the right default for many free fonts:

- https://scripts.sil.org/OFL

Use when:

- You need a free typeface for UI, branding, or in-game text.

Practical notes:

- Preserve the font name and license notice.
- If you modify the font itself, follow the OFL rules for reserved font names and derivative naming.
- Keep the font license text with the pack or in the central attribution manifest.

### Kenney assets

Kenney’s game asset packs are widely used because they are free and typically released under permissive terms. Use them only after confirming the license on the specific pack or asset page.

Official site:

- https://kenney.nl/

Policy:

- Treat each Kenney pack as a distinct licensed item.
- Do not assume every Kenney asset has identical terms unless the page says so.
- Record the exact pack name, URL, and license in the manifest.

### Poly Pizza assets

Poly Pizza provides free low-poly assets that are often used in prototypes and indie projects.

Official site:

- https://poly.pizza/

Policy:

- Verify the license on the specific asset or project page before use.
- Record the asset URL, author, and license in the manifest.
- Prefer assets that are clearly marked reusable for commercial or non-commercial redistribution, depending on pack needs.

### OpenGameArt assets

OpenGameArt is useful because it aggregates assets under multiple licenses, but that also means the license must be checked per asset.

Official site:

- https://opengameart.org/

Policy:

- Never assume all OpenGameArt content is interchangeable.
- Capture the exact license, author, and any attribution text from the asset page.
- If the license is unclear, do not ship the asset.

### Sketchfab CC assets

Sketchfab has a license filter and many models are available under Creative Commons terms.

Official license pages:

- https://sketchfab.com/licenses

Policy:

- Use only models whose license page and asset page are both clear.
- Record whether the model is CC0, CC BY, or another allowed license.
- Verify whether derivatives are allowed and whether attribution is required.

## 3) Prompt-based original derivation is the safest path

For the Star Wars and Modern Warfare themes, the safest production approach is:

1. Use prompts and concept references to generate **original geometry and textures**
2. Avoid asking for “a copy of X”
3. Use aesthetic descriptors rather than source-asset identifiers
4. Manually review outputs for accidental similarity before shipping

Recommended prompt framing:

- Good: “Create a rugged, imperial sci-fi infantry helmet with layered plates, asymmetric vents, and worn matte paint.”
- Risky: “Create a stormtrooper helmet.”
- Good: “Design a modern special-operations rifle silhouette with modular rails and a compact suppressor.”
- Risky: “Make an M4 from Call of Duty.”

Why this is safest:

- The output is newly authored rather than copied.
- You reduce the chance of reproducing protectable character design, product trade dress, or iconic silhouettes.
- You can build a coherent visual language for DINOForge without inheriting source-license baggage.

Implementation guidance:

- Keep prompt logs in the asset manifest for provenance.
- Treat AI output as a draft, not a final clearance guarantee.
- Run a human “substantial similarity” review before acceptance.

## 4) What is risky, and how to avoid it

### Risky categories

- Trademarked logos, insignia, faction marks, and studio marks
- Proprietary or branded fonts
- Ripped game assets, extracted models, textures, animations, audio, or UI elements
- Recognizable copyrighted models or character designs
- “Close enough” reproductions of iconic silhouettes or costumes
- Assets scraped from mod archives, rip packs, or fan repositories with unclear provenance

### Why these are risky

- Logos and insignia can trigger trademark issues and consumer confusion.
- Fonts may be copyrighted software or carry name restrictions.
- Ripped assets are almost never defensible for distribution.
- Recognizable character or vehicle designs can be infringing even if remodeled from scratch.

### Avoidance checklist

- Do not import assets from game files, decompilers, or extraction tools unless the rights holder has explicitly licensed them for reuse.
- Do not reuse trademarked symbols, clone uniforms, or faction marks, even if recolored.
- Do not ask modelers or generators to “match” a protected design.
- Do not ship any asset unless the source, license, and attribution path are documented.
- When in doubt, replace the asset with a fully original design language.

## 5) Recommended DINOForge community pack policy

### Default rule

Community packs must ship only:

- Original assets created for DINOForge
- Assets under verified permissive licenses
- Assets with documented attribution and redistribution rights

### Forbidden by default

- Extracted or ripped assets from commercial games
- Trademarked logos or insignia
- Unverified AI outputs that intentionally imitate a protected source
- Any asset with missing or contradictory provenance

### Required pack manifest fields

Each asset should have a manifest entry with:

- `id`
- `type`
- `source`
- `creator`
- `license`
- `license_url`
- `attribution`
- `modifications`
- `provenance`
- `review_status`

### Suggested manifest format

Use YAML in pack content, with a machine-readable attribution block.

```yaml
assets:
  - id: imperial_infantry_helmet_01
    type: model
    source: prompt-derived
    creator: DINOForge Community
    license: CC-BY-4.0-or-compatible
    license_url: https://creativecommons.org/licenses/by/4.0/
    attribution: "Original model generated from prompt and modified by DINOForge contributors."
    modifications:
      - retopology
      - texture repaint
      - rig cleanup
    provenance:
      prompt: "Create a rugged imperial sci-fi infantry helmet with layered plates and worn matte paint."
      source_assets: []
      review_notes: "Human review approved; no obvious similarity to a known protected model."
    review_status: approved
```

For externally sourced assets:

```yaml
assets:
  - id: crate_metal_01
    type: prop
    source: https://opengameart.org/content/example-crate
    creator: "Jane Doe"
    license: CC-BY-4.0
    license_url: https://creativecommons.org/licenses/by/4.0/
    attribution: "Crate Metal by Jane Doe, https://opengameart.org/content/example-crate, licensed CC BY 4.0, modified by DINOForge contributors."
    modifications:
      - resized
      - texture tuned
    provenance:
      downloaded_from: https://opengameart.org/content/example-crate
      source_assets: []
      review_notes: "License verified on asset page."
    review_status: approved
```

### Attribution bundle format

At release time, publish a single `ATTRIBUTION.md` or `credits.md` generated from the manifest with:

- Asset name
- Author or source
- License
- Required attribution string
- Modifications

If the pack contains many assets, the manifest should be the source of truth and the credit file should be generated from it.

## Operational recommendation

For DINOForge community packs, the safest production strategy is:

1. Default to original, prompt-derived art direction
2. Use clearly licensed free assets only when they materially improve quality or save time
3. Track provenance and license text in a machine-readable manifest
4. Reject anything with trademark, rip, or similarity risk
5. Require final human review before pack publication

This gives the project the best balance of creative freedom, community contribution, and defensible legal posture.
