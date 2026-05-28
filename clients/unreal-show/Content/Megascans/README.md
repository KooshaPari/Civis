# Megascans / Quixel (L5 art — artist-owned)

Engineering slot for Manor Lords–grade landscape materials. **Do not commit** Megascans assets to git.

## Import checklist (artist)

1. Install **Quixel Bridge** and sign in to Fab/Megascans.
2. Export landscape master material + Nanite rocks/foliage to this project.
3. Target paths (conventions):
   - `Content/Materials/Megascans/` — master materials and instances
   - `Content/Megascans/` — meshes and textures from Bridge
4. In **CivShow** map, assign landscape material on `AVoxelTerrain` / world partition landscape actor.
5. Verify in-editor: Nanite enabled on rocks; foliage painted on landscape layer.

## Agent / CI note

Presence of this folder satisfies the L5 art **engineering slot**; asset import remains artist-owned (see `docs/development-guide/fr-l5-visual-pass.md`).
