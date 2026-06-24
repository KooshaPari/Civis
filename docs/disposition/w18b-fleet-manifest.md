# W18b — Civis fleet manifest closeout

**Date:** 2026-06-19  
**Wave:** W18b-G (pheno fleet) + L5-114 (gfx sister-repo supersession)  
**Owner:** Civis  
**Status:** **COMPLETE**

## Summary

- **W18b pheno gate:** no `KooshaPari/pheno` or `phenoShared` git pins in `Cargo.toml` / `go.mod` on `main` (phenoShared reusable workflows remain — separate lane).
- **Gfx repoint:** `crates/voxel` git dep repointed from archived `phenotype-voxel` → `phenotype-gfx` compat crate `phenotype-voxel`.

## Applied repoint

### `crates/voxel/Cargo.toml`

| Before | After |
|--------|-------|
| `git = "https://github.com/KooshaPari/phenotype-voxel.git"` | `git = "https://github.com/KooshaPari/phenotype-gfx.git"`, `package = "phenotype-voxel"` |

## Verification

```bash
cargo check -p civ-voxel
```

## Related

- [phenotype-gfx docs/disposition/w18b-fleet-manifest.md](https://github.com/KooshaPari/phenotype-gfx/blob/main/docs/disposition/w18b-fleet-manifest.md)
- [phenotype-gfx#10](https://github.com/KooshaPari/phenotype-gfx/pull/10) — voxel kernel inlined
