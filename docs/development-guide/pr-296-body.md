## Summary

Civis **3D foundation** branch: 11 workspace crates, three reference clients (Bevy, Godot, Unreal), web L2 spectator, and **P-U1 WorldBox UX** (spawn palette, drag-place, convoy, civ-server attach).

### P-U1 (complete)

- Spawn palette: civilian, vehicle, airport, port, hangar
- FR-CIV-UX-004 drag + convoy on web and Godot
- FR-CIV-GODOT-ATTACH civ-server WS + civ-watch terrain
- Era timelapse + buildings/military pins

### Recent waves

- **Wave 15:** trade routes, save/load, keyboard shortcuts
- **Wave 16:** terrain paint, spawn feedback, hangar
- **Wave 17:** building models, humanoid civilians, military arrowheads, Unreal scaffold

### Web (ADR-009)

- FR-CIV-WEB-000..008 including L2 authoring and Babylon (`?renderer=babylon`)

## CI / merge

**Local-first:** run `lefthook run pre-push` and commit `.ci/quality-manifest.json`.

Cloud PR job **quality-manifest (cloud verify)** only checks the manifest (see `docs/development-guide/pr-296-merge-readiness.md`).

Other workflows may show red when GitHub Actions **spending limits** block runners; they are not required for manifest-gated merge.

## Test plan

- [ ] `bash scripts/quality/verify-quality-manifest.sh`
- [ ] `just civis-3d-verify`
- [ ] `cd web && npm test`
- [ ] Godot F5 with `spectator_mode=false`, spawn hangar convoy
- [ ] Optional: `clients/unreal-show/scripts/build.ps1` on UE 5.4 machine

## After merge

Follow `docs/development-guide/p-w1-kickoff.md` for tactical warfare integration PRs.
