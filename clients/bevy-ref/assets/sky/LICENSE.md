# Sky / HDRI Assets — License

## `kloofendal_43d_clear_puresky_1k.hdr`

- **Source:** Poly Haven — https://polyhaven.com/a/kloofendal_43d_clear_puresky
- **Author:** Greg Zaal / Poly Haven
- **License:** CC0 1.0 Universal (Public Domain Dedication)
  - https://creativecommons.org/publicdomain/zero/1.0/
- **Downloaded from:** https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/kloofendal_43d_clear_puresky_1k.hdr
- **Resolution:** 1k equirectangular HDR (Radiance RGBE `.hdr`)
- **Use:** Clear-sky environment map / skybox + image-based lighting (IBL) for the
  Bevy reference client. CC0 = no attribution required, but credit is retained
  here as good practice.

Poly Haven publishes all of its HDRIs, textures, and models under CC0, making
them safe for any use (commercial, redistribution, modification) with no
restrictions.

## Wiring note (follow-up, NOT done here — assets only)

To use this skybox in the Bevy reference client:

1. **Skybox** — load the `.hdr` via `asset_server.load("sky/kloofendal_43d_clear_puresky_1k.hdr")`
   and attach a `Skybox { image, brightness, .. }` component to the 3D camera.
2. **IBL / ambient** — attach `EnvironmentMapLight { diffuse_map, specular_map, intensity, .. }`
   to the same camera for image-based lighting. An equirectangular `.hdr` must
   first be converted to a cubemap (e.g. via `bevy`'s `light_consts` / a
   compute pass, or pre-baked with a tool); the raw equirect HDR is the source.
3. Both `Skybox` and `EnvironmentMapLight` are in the `bevy_pbr` / `bevy_core_pipeline`
   crates. See Bevy's `examples/3d/skybox.rs` for the canonical pattern.
