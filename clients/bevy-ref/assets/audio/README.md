# Audio Assets

Drop CC0 `.ogg` clips here to enable audio in the Civis client.
Build with `--features audio` to activate.

## Snapshot cue map (`sim.snapshot.audio_events`)

Each wire `trigger` maps to a dedicated Bevy [`SfxKind`](../../src/audio.rs) and asset path.
Battle/Birth/Death/Tech/Disaster paths are pairwise distinct (no disaster-alias fallback).

| Wire `trigger` | `SfxKind` | File | Description |
|----------------|-----------|------|-------------|
| `birth` | `Birth` | `birth.ogg` | Agent birth |
| `death` | `Death` | `death.ogg` | Agent death |
| `tech` | `Tech` | `sfx_tech.ogg` | Technology unlock |
| `battle` | `Battle` | `sfx_battle.ogg` | Combat engagement (intensity-scaled) |
| `disaster` | `Disaster` | `sfx_disaster.ogg` | World disaster (severity-scaled) |
| `build` | `Build` | `build.ogg` | Building constructed |

## Stub notes

`sfx_battle.ogg` is a short synthetic clash stub (distinct from `sfx_disaster.ogg`).
Replace with a real CC0 combat sting when available — do **not** copy the disaster clip.

## Other clips

| File | Description | Suggested source |
|------|-------------|-----------------|
| `ambient_wind.ogg` | Looping ambient bed | freesound.org / kenney.nl nature packs (CC0) |
| `sfx_diplomatic.ogg` | Diplomatic event | kenney.nl "UI Audio" (CC0) |
| `ui_click.ogg` | UI button click | kenney.nl "UI Audio" (CC0) |

All files are optional — missing clips play silence without aborting the client.
`audio.rs` skips `LoadState::Failed` / `NotLoaded` handles so a missing asset never panics.
