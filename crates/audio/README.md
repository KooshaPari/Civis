# civ-audio

CIV-0800 audio substrate: four-tier bus mix, biome-driven ambient
beds, mood-driven score stems, SFX coalescing. Pure-Rust core,
no engine deps.

See [`src/lib.rs`](src/lib.rs) for the full module index and design
notes. The substrate is consumed by `clients/bevy-ref/src/audio.rs`
(the kira-bound plugin); the math here is testable in isolation.

## What's in this crate

| Module | FR ID(s) | What it provides |
|--------|----------|------------------|
| `bus` | FR-CIV-AUDIO-001 | `BusId` (4 tiers + Master), `BusLevels` |
| `mix` | FR-CIV-AUDIO-001 | `AudioMix` resource, `AudioMixPreset`, schema version |
| `ambient` | FR-CIV-AUDIO-002 | `AmbientBed`, `BiomeFootprint`, `BedWeights`, `AmbientBlend` |
| `mood` | FR-CIV-AUDIO-004 | `MoodVector`, `ScoreStem`, `StemMix`, `ScoreCadence` |
| `sfx` | FR-CIV-AUDIO-006 | `SfxKind`, `SfxCoalescer`, `SfxQueue`, per-kind cap |

## What is NOT in this crate

- kira / `bevy_kira_audio` plugin (lives in `clients/bevy-ref`)
- CC0 audio asset files + `CREDITS.md` (designer + asset lead work)
- Graceful-silence invariant tests with 0 files (engine concern)
