# audio

> Pure-Rust audio substrate with four-tier bus mixing, biome-driven ambience, and reactive SFX coalescing.

## Overview

The `audio` crate provides the entire audio mixing and routing pipeline for Civis as a pure-Rust, engine-independent layer. It has zero dependencies on game engines or platform audio APIs — all functions are pure and deterministic, making the system trivially testable and replayable.

Audio is organized into a four-tier bus hierarchy: Ambient (biome beds), Score (mood-driven stems), SFX (event-reactive sounds), and UI (interface feedback). Each tier has its own modulation rules, and cross-tier bus ducking ensures the mix stays intelligible under all conditions.

Ambient beds are driven by biome type with diurnal, seasonal, and weather modulation layers. Score stems respond to a mood vector, while SFX events are coalesced to prevent overlapping playback during rapid simulation ticks.

## Features

- Four-tier bus mix: Ambient, Score, Sfx, Ui
- Biome-driven ambient bed selection and blending
- Diurnal, seasonal, and weather modulation layers
- Mood-driven score stem crossfading
- Reactive event SFX with coalescing to prevent overlap
- UI sound language for interface feedback
- Bus ducking for intelligible mixing
- Fully pure and deterministic — no I/O or async

## Usage

```rust
use audio::*;
```

## Architecture

- **AudioMix** — Top-level mixer holding bus state and producing per-frame sample buffers
- **AmbientBlend** — Blends biome ambient beds weighted by time of day, season, and weather
- **ScoreStem** — Mood-driven music stem with crossfade envelopes
- **SfxCoalescer** — Deduplicates and queues overlapping sound events within a time window
- **MoodVector** — Continuous mood descriptor driving score selection

All types are constructed from simulation state each tick and return deterministic output, with no hidden I/O.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
