# Bevy GI Status

Date: 2026-05-26

## Summary

Bevy 0.18 includes an in-tree Solari module for raytraced lighting. The standalone plugin entrypoint is `bevy::solari::SolariPlugins`, and the Bevy crate exposes it through the `bevy_solari` feature.

For this repo, the practical integration path is:

- gate Solari behind a local `solari` feature in `clients/bevy-ref`
- forward that feature to Bevy's `bevy_solari`
- add `bevy::solari::SolariPlugins` in `standalone.rs`

## Findings

1. Solari exists for Bevy 0.18.
2. The published crate is `bevy_solari` 0.18.1.
3. The Bevy docs list `bevy_solari` as a Bevy crate feature, described as "Provides raytraced lighting (experimental)".
4. `SolariPlugins` provides raytraced direct and indirect lighting, plus BLAS / scene binding support.
5. Bevy also documents `pathtracer::PathtracingPlugin` for non-realtime validation, but it is not added by default.

## Repo Change

- `clients/bevy-ref/Cargo.toml`
  - added a `solari` feature
  - forwarded that feature to `bevy/bevy_solari`
- `clients/bevy-ref/src/bin/standalone.rs`
  - conditionally adds `bevy::solari::SolariPlugins` when `solari` is enabled
  - added a few colored cube "building" blockers so indirect lighting has visible surfaces to bounce onto

## Alternatives if Solari is not usable on the target machine

If the graphics stack or driver path blocks Solari at runtime, the next options to investigate are:

- probe-based GI / DDGI
- radiance cascades
- screen-space probes

Those are not wired into this repo yet. Solari is the only Bevy 0.18 GI path confirmed here with an official plugin surface.

## References

- https://docs.rs/bevy/latest/bevy/solari/index.html
- https://docs.rs/bevy/latest/bevy/solari/struct.SolariPlugins.html
- https://docs.rs/crate/bevy_solari/0.18.1
- https://docs.rs/bevy/latest/bevy/
