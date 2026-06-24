# Civis 3D Asset Sources (CC0)

Canonical source URLs for all PBR textures and low-poly models consumed by
`clients/bevy-ref`. **All assets must be CC0 / public domain.** Verify the
license on the download page before fetching.

> Status: **Phase 1 scaffold** — directories and code wired, downloads pending.
> Use this doc as the manifest when the asset-fetch script lands.

## PBR Texture Sources

Primary sources, ordered by preference:

1. **Poly Haven Textures** — <https://polyhaven.com/textures> — CC0, 4K PBR sets
   with albedo + normal + ORM + displacement.
2. **ambientCG** — <https://ambientcg.com> — CC0, broad catalogue, consistent
   PBR map naming.
3. **CC0 Textures (legacy ambientCG mirror)** — <https://cc0textures.com> — CC0.

### Biome → source mapping

| Biome (asset dir)                        | Source        | Asset slug                | URL |
|------------------------------------------|---------------|---------------------------|-----|
| `assets/textures/grass_field/`           | Poly Haven    | `aerial_grass_rock`       | <https://polyhaven.com/a/aerial_grass_rock> |
| `assets/textures/sand_beach/`            | Poly Haven    | `aerial_beach_01`         | <https://polyhaven.com/a/aerial_beach_01> |
| `assets/textures/rock_cliff/`            | Poly Haven    | `rock_face_03`            | <https://polyhaven.com/a/rock_face_03> |
| `assets/textures/snow_pure/`             | Poly Haven    | `snow_02`                 | <https://polyhaven.com/a/snow_02> |
| `assets/textures/forest_floor/`          | Poly Haven    | `forest_ground_01`        | <https://polyhaven.com/a/forest_ground_01> |
| `assets/textures/dirt_ground/`           | ambientCG     | `Ground054`               | <https://ambientcg.com/view?id=Ground054> |

**Fallback mirrors** (use if primary slug is renamed):

- Grass → ambientCG `Grass004` — <https://ambientcg.com/view?id=Grass004>
- Sand → ambientCG `Ground080` — <https://ambientcg.com/view?id=Ground080>
- Rock cliff → ambientCG `Rock030` — <https://ambientcg.com/view?id=Rock030>
- Snow → ambientCG `Snow006` — <https://ambientcg.com/view?id=Snow006>
- Forest floor → ambientCG `Ground037` — <https://ambientcg.com/view?id=Ground037>

### Download conventions

- **Resolution:** 2K (2048²) source → resized to 1024² for runtime in Phase 1
  to stay under 64 MB total VRAM budget.
- **Channel packing:** combine `AmbientOcclusion`, `Roughness`, `Metalness`
  PNGs into a single `orm.ktx2` (R=AO, G=Rough, B=Metal). A `ktx2-pack.ps1`
  helper script will land alongside Phase 2.
- **Normal map convention:** OpenGL (+Y up). Poly Haven exports GL by default;
  ambientCG offers both — pick `*_NormalGL.png`.

## Low-Poly Model Sources (Phase 2+)

For trees, rocks, and decoration props on top of the heightmap terrain.

| Asset class      | Source       | URL |
|------------------|--------------|-----|
| Stylized trees   | Quaternius   | <https://quaternius.com/packs/ultimatestylizednaturepack.html> |
| Pine / fir trees | Quaternius   | <https://quaternius.com/packs/ultimatenaturepack.html> |
| Rock formations  | Quaternius   | <https://quaternius.com/packs/stylizedrocks.html> |
| Mushrooms / shrubs | Kenney     | <https://kenney.nl/assets/nature-kit> (CC0) |
| Generic CC0 mesh | Sketchfab    | <https://sketchfab.com/3d-models?features=downloadable&licenses=322a749bcfa841b29dff1e8a1bb74b0b> (filter: CC0) |

All Quaternius and Kenney packs ship as **GLTF / GLB CC0** — drop into
`assets/models/trees/` and `assets/models/rocks/` respectively.

## License compliance

- Each `assets/textures/<biome>/LICENSE.txt` must record: author, source URL,
  original asset slug, retrieval date, and the verbatim CC0 statement.
- `assets/models/<class>/LICENSE.txt` same conventions per pack.
- Audit script (TBD) walks the asset tree and fails CI when a `LICENSE.txt`
  is missing.

## Related docs

- Integration phasing: [`docs/guides/pbr-materials-plan.md`](./pbr-materials-plan.md)
- Bevy renderer entry point: `clients/bevy-ref/src/bevy_render.rs`
- Materials module (Phase 1 scaffold): `clients/bevy-ref/src/materials.rs`
