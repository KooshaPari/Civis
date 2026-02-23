# RND-007: Adaptive Music Architecture -- Kira, Howler.js, and AI Music Generation

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-delta

---

## Executive Summary

This document specifies the adaptive music system for CivLab, spanning the Bevy desktop client (Kira audio engine), the web client (Howler.js), and AI music asset generation (MusicGen). The core design is a layered mixing approach: 8 pre-generated mood tracks play simultaneously with independent volume envelopes, crossfaded by game-state transitions via smooth tweens. Kira 0.12 provides the necessary primitives (TrackHandle volume automation, ClockHandle for beat-synced transitions, Tween easing curves). Howler.js mirrors this on the web with its Web Audio API gain nodes. MusicGen (Meta AudioCraft, local, free, Apache-2.0) is recommended for generating the 8 mood tracks offline during asset pipeline, avoiding runtime API costs.

---

## Research Findings

### 1. Kira Audio Engine (Rust / Bevy Client)

**Version:** Kira 0.12.0 (latest stable, docs.rs/kira/latest)
**Bevy Integration:** `bevy_kira_audio` v0.23 (supports Bevy 0.15)
**License:** MIT OR Apache-2.0

#### Core API Surface

| Struct | Role | Key Methods |
|--------|------|-------------|
| `AudioManager<DefaultBackend>` | Top-level controller. Owns the audio thread. | `::new(settings)`, `.play(sound_data)`, `.add_sub_track(settings)`, `.add_clock(settings)` |
| `StaticSoundData` | Pre-loaded audio buffer (entire file in memory). Appropriate for music tracks <60s or looping stems. | `::from_file(path)`, `.with_settings(settings)` |
| `StreamingSoundData` | Streaming from disk. Appropriate for long ambient tracks. | `::from_file(path)` |
| `TrackHandle` | Sub-mixer channel. Controls volume, panning, effects for all sounds routed to it. | `.set_volume(value, tween)`, `.set_panning(value, tween)`, `.play(sound_data)` |
| `ClockHandle` | Musical timing source. Ticks at configurable BPM. Events can be scheduled on clock ticks. | `::new(settings)`, `.set_speed(bpm, tween)` |
| `Tween` | Smooth value transition over duration with easing curve. | `Tween { start_time, duration, easing }` |
| `Easing` | Curve shape for tweens. | `Linear`, `InPowi(i32)`, `OutPowi(i32)`, `InOutPowi(i32)` |
| `Decibels` / `Volume` | Volume representation. | `Volume::Amplitude(f64)`, `Volume::Decibels(f64)` |

#### Adaptive Music Design Pattern

The standard pattern for adaptive game music in Kira:

1. **Create 8 sub-tracks** (one per mood layer) via `AudioManager::add_sub_track()`.
2. **Load 8 looping stems** as `StaticSoundData` (or `StreamingSoundData` for longer pieces).
3. **Play all 8 simultaneously** from game start, each routed to its own sub-track. Set initial volumes according to the starting game state.
4. **On game-state change**, call `track_handle.set_volume(target, tween)` on each track to crossfade between mood layers.
5. **Use ClockHandle** for beat-quantized transitions: schedule volume changes to land on the next beat boundary so crossfades sound musical rather than abrupt.

```rust
// Pseudocode: AudioPlugin setup
pub struct MusicLayer {
    track: TrackHandle,
    sound: StaticSoundHandle,
}

pub struct AdaptiveMusicPlugin;

impl Plugin for AdaptiveMusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>()
           .add_systems(Startup, setup_music_layers)
           .add_systems(Update, update_music_from_game_state);
    }
}

fn setup_music_layers(
    mut commands: Commands,
    mut audio_manager: ResMut<AudioManager<DefaultBackend>>,
) {
    let layers: Vec<MusicLayer> = MOOD_TRACKS.iter().map(|path| {
        let track = audio_manager.add_sub_track(TrackBuilder::default()).unwrap();
        let sound_data = StaticSoundData::from_file(path)
            .unwrap()
            .with_settings(StaticSoundSettings::new().loop_behavior(LoopBehavior::default()));
        let sound = track.play(sound_data).unwrap();
        MusicLayer { track, sound }
    }).collect();
    commands.insert_resource(MusicLayers(layers));
}

fn update_music_from_game_state(
    game_state: Res<GameState>,
    music_config: Res<MusicStateConfig>,
    layers: Res<MusicLayers>,
) {
    let volumes = music_config.volumes_for_state(&game_state);
    let tween = Tween {
        duration: Duration::from_secs(2),
        easing: Easing::InOutPowi(2),
        ..Default::default()
    };
    for (layer, &target_vol) in layers.0.iter().zip(volumes.iter()) {
        layer.track.set_volume(Volume::Amplitude(target_vol as f64), tween);
    }
}
```

#### Supported Audio Formats

Kira supports: OGG Vorbis, MP3, FLAC, WAV. For game music stems, **OGG Vorbis** is recommended (good compression, gapless looping, no patent issues).

### 2. Howler.js (Web Client)

**Version:** Howler.js 2.2.4 (latest, npm)
**License:** MIT
**Browser Support:** Chrome, Firefox, Safari, Edge, IE11 (Web Audio API primary, HTML5 Audio fallback)

#### Core API Surface

| Object | Role | Key Methods |
|--------|------|-------------|
| `Howl` | Sound instance. Loads and controls a single audio source. | `new Howl({src, loop, volume, html5})`, `.play()`, `.pause()`, `.stop()` |
| `Howl` (volume) | Per-sound volume control. | `.volume(val)`, `.fade(from, to, duration)` |
| `Howler` | Global controller. | `Howler.volume(val)`, `Howler.mute(bool)` |

#### Adaptive Music on Web

The web mirrors the Bevy pattern but uses Howler.js `Howl` instances instead of Kira tracks:

```javascript
// MusicManager.js
class AdaptiveMusicManager {
  constructor(trackPaths) {
    this.layers = trackPaths.map(path => ({
      howl: new Howl({
        src: [path],
        loop: true,
        volume: 0.0,
        html5: true,  // HTML5 audio for long music (lower memory)
      }),
    }));
  }

  start() {
    this.layers.forEach(layer => layer.howl.play());
  }

  transitionTo(targetVolumes, durationMs = 2000) {
    this.layers.forEach((layer, i) => {
      const currentVol = layer.howl.volume();
      layer.howl.fade(currentVol, targetVolumes[i], durationMs);
    });
  }
}
```

#### Web-Specific Considerations

- **Autoplay Policy:** Browsers block audio autoplay until user interaction. Music must start on first click/keypress. Use `Howler.ctx.resume()` after user gesture.
- **HTML5 vs Web Audio:** Use `html5: true` for music tracks (streaming, lower memory). Use Web Audio (default) for short SFX (lower latency).
- **Codec:** Use OGG Vorbis with MP3 fallback: `src: ['track.ogg', 'track.mp3']`.
- **Mobile:** Volume ducking on iOS when page is backgrounded. No programmatic volume control on iOS Safari for HTML5 Audio mode.

### 3. AI Music Generation (Asset Pipeline)

#### MusicGen (Meta AudioCraft) -- RECOMMENDED

**Repository:** `facebookresearch/audiocraft` (GitHub)
**License:** Apache-2.0 (code) + CC-BY-NC-4.0 (pretrained models for non-commercial) or MIT for Hydra II (commercial)
**Models:** `musicgen-small` (300M), `musicgen-medium` (1.5B), `musicgen-large` (3.3B)
**Hardware:** GPU required. `musicgen-small` runs on 8GB VRAM. `musicgen-large` needs 16GB+.
**Output:** 32kHz mono/stereo audio, up to 30s per generation.

**Strengths:**
- Local inference, no API costs, no rate limits.
- Text-conditioned: describe mood, tempo, instruments, and genre.
- Melody-conditioned: supply a reference melody to guide generation.
- Multi-band diffusion decoder available for higher quality output.

**Limitations:**
- 30s max per generation (can be extended with overlap-add stitching).
- Loopability not guaranteed; post-processing needed to create seamless loops (crossfade tail into head).
- Quality is good but not studio-grade. Adequate for game background music.

**Hydra II Alternative:** Rightsify's Hydra II is a MusicGen-based model trained entirely on licensed music. MIT license, commercially safe. Similar quality to `musicgen-medium`.

#### Suno API (Alternative, Paid)

**API:** REST, paid per generation.
**Quality:** Higher than MusicGen (closer to studio quality).
**License:** Commercial use allowed with paid plan.
**Cost:** ~$0.05/generation.

**Verdict:** MusicGen for development and MVP (free, local, good enough). Suno as upgrade path if higher quality music is needed for production release.

#### Track Generation Strategy

Generate 8 mood tracks:

| Track ID | Mood | Prompt Template |
|----------|------|-----------------|
| `base_calm` | Peaceful / Idle | "Calm ambient orchestral, gentle strings, 80 BPM, loopable" |
| `base_tense` | Tension rising | "Suspenseful orchestral, low brass, timpani rolls, 90 BPM" |
| `battle_low` | Minor skirmish | "Moderate battle music, snare drums, horns, 110 BPM" |
| `battle_high` | Major war | "Epic battle orchestral, full orchestra, choir, 130 BPM" |
| `prosperity` | Economic boom | "Triumphant fanfare, major key, brass and strings, 100 BPM" |
| `crisis` | Famine / collapse | "Dark ambient, minor key, cello solo, sparse percussion, 70 BPM" |
| `discovery` | New territory / tech | "Wonder and exploration, woodwinds, harp, gentle percussion, 85 BPM" |
| `diplomacy` | Negotiations | "Elegant courtly music, harpsichord, chamber strings, 95 BPM" |

Post-processing pipeline:
1. Generate 30s raw audio with MusicGen.
2. Normalize loudness to -14 LUFS (EBU R128).
3. Apply crossfade loop (2s fade at tail/head boundary).
4. Export as OGG Vorbis quality 6 (~160kbps).
5. Validate loop point with automated playback test.

---

## Decision

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Desktop audio engine | **Kira 0.12 via bevy_kira_audio** | Only mature Bevy audio plugin. Sub-track mixing, tweens, and clock handles provide all needed adaptive music primitives. |
| Web audio engine | **Howler.js 2.2** | De facto standard for browser game audio. Fade API, HTML5 streaming mode, broad browser support. |
| Music generation | **MusicGen (audiocraft)** for MVP | Free, local, Apache-2.0 code. Quality sufficient for game background music. Suno as paid upgrade path. |
| Track format | **OGG Vorbis** (primary) + MP3 (web fallback) | Good compression, gapless looping, patent-free. |
| Mixing architecture | **8 simultaneous looping layers** with volume automation | Industry-standard adaptive music pattern. Simple, predictable, easy to tune. |

---

## Implementation Contract

### AudioPlugin API (Bevy/Kira)

```rust
/// Resource: maps game states to per-layer volume targets.
#[derive(Resource)]
pub struct MusicStateConfig {
    /// Map from GameMood enum variant to array of 8 volume amplitudes [0.0..1.0].
    pub mood_volumes: HashMap<GameMood, [f32; 8]>,
    /// Default crossfade duration in seconds.
    pub crossfade_secs: f32,
    /// Easing curve exponent for crossfades.
    pub easing_power: i32,
}

/// The 8 mood categories that drive music mixing.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum GameMood {
    Calm,
    Tense,
    BattleLow,
    BattleHigh,
    Prosperity,
    Crisis,
    Discovery,
    Diplomacy,
}

/// Resource: holds the 8 active music layer handles.
#[derive(Resource)]
pub struct MusicLayers {
    pub layers: [MusicLayer; 8],
    pub clock: ClockHandle,
}

pub struct MusicLayer {
    pub track: TrackHandle,
    pub sound: StaticSoundHandle,
}

/// System: reads GameState, computes current GameMood, applies volume targets.
/// Runs in Update schedule at 4Hz (no need for per-frame updates).
pub fn update_music_from_game_state(
    game_state: Res<GameState>,
    config: Res<MusicStateConfig>,
    layers: Res<MusicLayers>,
) {
    // 1. Derive GameMood from GameState (war intensity, economy, diplomacy flags).
    // 2. Look up mood_volumes[mood].
    // 3. For each layer, call track.set_volume(target, tween).
    // 4. Tween uses config.crossfade_secs and config.easing_power.
}
```

### WebMusicManager API (Howler.js)

```typescript
interface MusicStateConfig {
  moodVolumes: Record<GameMood, number[]>;  // 8 volumes per mood
  crossfadeMs: number;                       // default 2000
}

type GameMood =
  | 'calm' | 'tense' | 'battle_low' | 'battle_high'
  | 'prosperity' | 'crisis' | 'discovery' | 'diplomacy';

class WebMusicManager {
  private layers: Howl[];
  private config: MusicStateConfig;

  constructor(trackUrls: string[], config: MusicStateConfig);
  start(): void;                              // Play all layers (call after user gesture)
  setMood(mood: GameMood): void;              // Crossfade to target volumes
  setMasterVolume(vol: number): void;         // 0.0..1.0
  pause(): void;
  resume(): void;
}
```

### MusicStateConfig Example

```json
{
  "mood_volumes": {
    "calm":        [1.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.2, 0.0],
    "tense":       [0.3, 1.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.0],
    "battle_low":  [0.0, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    "battle_high": [0.0, 0.0, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0],
    "prosperity":  [0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.3, 0.0],
    "crisis":      [0.0, 0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    "discovery":   [0.3, 0.0, 0.0, 0.0, 0.2, 0.0, 1.0, 0.0],
    "diplomacy":   [0.3, 0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 1.0]
  },
  "crossfade_secs": 2.0,
  "easing_power": 2
}
```

### Asset Pipeline Contract

| Step | Tool | Input | Output | Deterministic? |
|------|------|-------|--------|----------------|
| Generate raw audio | MusicGen (`musicgen-medium`) | Text prompt + optional melody | 30s WAV @ 32kHz | No (generative AI) |
| Normalize loudness | FFmpeg `loudnorm` filter | Raw WAV | Normalized WAV @ -14 LUFS | Yes |
| Create loop | FFmpeg crossfade | Normalized WAV | Looped WAV | Yes |
| Encode OGG | FFmpeg `-c:a libvorbis -q:a 6` | Looped WAV | OGG Vorbis ~160kbps | Yes |
| Encode MP3 fallback | FFmpeg `-c:a libmp3lame -q:a 2` | Looped WAV | MP3 ~192kbps | Yes |
| Validate loop | Custom script (play 2x, check for click at boundary) | OGG file | Pass/fail | Yes |

### Acceptance Criteria

1. All 8 mood tracks load without error on both Bevy and web clients.
2. Crossfade from any mood to any other mood completes within `crossfade_secs` with no audible click or pop.
3. Music layers loop seamlessly with no audible gap at loop boundary.
4. Volume automation responds to game-state changes within 1 frame (16ms) of state change detection.
5. Web client handles browser autoplay policy: music starts only after first user interaction.
6. Total music asset size < 20MB (8 tracks * ~2.5MB OGG each).
7. CPU usage for audio mixing < 2% on reference hardware (M1 Mac, Chrome 120+).

---

## Detailed Technical Notes

### Kira ClockHandle for Beat-Synced Transitions

Kira's `ClockHandle` provides musical timing independent of wall-clock time. A clock ticks at a configurable BPM, and sounds/tweens can be scheduled relative to clock ticks rather than absolute time.

**Usage pattern for beat-quantized crossfades:**

```rust
// Create a clock at 120 BPM.
let clock_settings = ClockSettings::new().speed(ClockSpeed::TicksPerMinute(120.0));
let clock = audio_manager.add_clock(clock_settings).unwrap();

// Schedule a volume change to start on the next beat (next clock tick).
let next_tick = clock.time() + ClockTime::Ticks(1);
let tween = Tween {
    start_time: StartTime::ClockTime(next_tick),
    duration: Duration::from_secs(2),
    easing: Easing::InOutPowi(2),
};
layer.track.set_volume(Volume::Amplitude(0.8), tween);
```

This ensures crossfades align with musical beats, preventing the jarring effect of mid-phrase volume changes. The clock BPM should match the generated music BPM (stored in track metadata).

**Clock synchronization across layers:** All 8 layers should reference the same `ClockHandle` to ensure beat-aligned transitions. When transitioning from a 90 BPM track to a 130 BPM track, the clock speed itself can be tweened: `clock.set_speed(ClockSpeed::TicksPerMinute(130.0), speed_tween)`.

### Howler.js Web Audio API Gain Node Architecture

Howler.js exposes the underlying Web Audio API context via `Howler.ctx`. For advanced mixing beyond simple per-sound volume, you can tap into the gain node graph:

```javascript
// Access the master gain node.
const masterGain = Howler.masterGain;

// Create per-layer gain nodes for independent volume control.
const layerGains = trackPaths.map(() => {
  const gain = Howler.ctx.createGain();
  gain.connect(masterGain);
  return gain;
});

// Route each Howl to its corresponding gain node.
layers.forEach((layer, i) => {
  // Howler.js doesn't expose per-sound routing natively,
  // so use the Web Audio API directly for the connection.
  // This requires creating sounds with { html5: false } to use Web Audio.
});
```

For CivLab's web client, the simpler `Howl.fade()` API is sufficient for MVP. The gain node approach is documented here for future enhancement if sub-frame precision or per-layer effects (e.g., reverb on the ambient layer) are needed.

### MusicGen Generation Pipeline Details

**Model selection guidance:**

| Model | VRAM | Quality | Speed (30s generation) | Use Case |
|-------|------|---------|----------------------|----------|
| `musicgen-small` (300M) | 4GB | Acceptable | ~15s on A100 | Prototyping, iteration |
| `musicgen-medium` (1.5B) | 8GB | Good | ~30s on A100 | Production MVP |
| `musicgen-large` (3.3B) | 16GB | Best | ~60s on A100 | Final production tracks |
| Hydra II (1.5B) | 8GB | Good (commercially safe) | ~30s on A100 | Commercial release |

**Looping technique:**

MusicGen does not natively produce seamless loops. The post-processing pipeline must handle this:

```python
import numpy as np
import soundfile as sf

def create_seamless_loop(audio_path: str, output_path: str, crossfade_secs: float = 2.0):
    """Create a seamless loop from a MusicGen output."""
    audio, sr = sf.read(audio_path)
    crossfade_samples = int(crossfade_secs * sr)

    # Extract head and tail segments.
    tail = audio[-crossfade_samples:]
    head = audio[:crossfade_samples]

    # Create crossfade envelope.
    fade_out = np.linspace(1.0, 0.0, crossfade_samples)
    fade_in = np.linspace(0.0, 1.0, crossfade_samples)

    if audio.ndim == 2:  # stereo
        fade_out = fade_out[:, np.newaxis]
        fade_in = fade_in[:, np.newaxis]

    # Blend tail and head.
    crossfaded = tail * fade_out + head * fade_in

    # Construct loopable audio: crossfaded region + middle section.
    middle = audio[crossfade_samples:-crossfade_samples]
    looped = np.concatenate([crossfaded, middle])

    sf.write(output_path, looped, sr)
```

**Prompt engineering for game music:**

Effective MusicGen prompts for game music follow this structure:
```
[mood adjective] [genre] music, [instruments], [tempo] BPM, [key/tonality], loopable, game soundtrack
```

Examples:
- "Calm ambient orchestral music, gentle strings and harp, 80 BPM, C major, loopable, game soundtrack"
- "Epic battle orchestral music, full orchestra with choir and war drums, 130 BPM, D minor, loopable, game soundtrack"

The "loopable" keyword has no guaranteed effect on MusicGen's output but empirically produces outputs with more consistent endings that are easier to crossfade.

### Memory and Performance Budget

**Bevy/Kira client:**
- 8 OGG tracks at ~2.5MB each = 20MB compressed on disk.
- Decoded to PCM in memory: 8 tracks * 30s * 44.1kHz * 2ch * 4 bytes = ~42MB RAM.
- Using `StaticSoundData` (all in memory): total ~42MB for music.
- Using `StreamingSoundData` (streaming from disk): ~1MB buffer per track = ~8MB total.
- Recommendation: Use `StaticSoundData` for tracks under 30s (gapless looping guarantee). Use `StreamingSoundData` for ambient layers over 60s.

**Web client (Howler.js):**
- HTML5 Audio mode: streams from server, minimal memory footprint (~1-2MB per track buffer).
- Web Audio mode: decodes entire track into AudioBuffer, ~42MB total (same as Bevy).
- Recommendation: Use `html5: true` for music tracks on web (streaming, lower memory).

**CPU budget:**
- Kira audio mixing: <1% CPU on M1 Mac for 8 simultaneous tracks.
- Howler.js: delegated to browser's audio thread, negligible JS main thread cost.
- Volume tween calculations: ~100 float operations per frame, negligible.

## Open Questions Remaining

1. **Beat-quantized transitions:** Should crossfades snap to beat boundaries via ClockHandle, or is smooth time-based crossfading sufficient for the first implementation? Recommend starting with time-based and adding beat-sync as a polish pass.

2. **Stinger system:** Short one-shot audio stingers (e.g., "battle started" brass hit) layered on top of the adaptive mix. Not covered here; should be a separate SFX system using `StaticSoundData` on a dedicated effects track.

3. **Dynamic tempo:** Should battle intensity also affect music tempo (via `PlaybackRate`), or only volume layers? Tempo changes risk making music sound unnatural. Recommend volume-only for MVP.

4. **Per-biome ambient layers:** Forest/desert/ocean ambient soundscapes as additional non-music layers. Architectural extension of the same system (additional tracks), but needs separate mood mapping.

5. **MusicGen model licensing:** The pretrained models are CC-BY-NC-4.0 (non-commercial). For commercial release, either use Hydra II (MIT) or train on licensed music. This is a production-release blocker, not an MVP blocker.

6. **Spatial audio:** Kira supports spatial audio (3D positioning of sounds relative to a listener). This is relevant for SFX (e.g., battle sounds from the direction of conflict) but not for background music. Document spatial audio API for the SFX system research.

7. **Audio format fallback chain:** OGG is not supported in all Safari versions. The fallback chain should be: OGG -> MP3 -> WAV. Howler.js handles this natively via the `src` array. Verify Safari 17+ OGG support status before launch.

---

## Sources

- Kira 0.12 API documentation: https://docs.rs/kira/latest/kira/
- bevy_kira_audio: https://github.com/NiklasEi/bevy_kira_audio
- Howler.js: https://howlerjs.com/
- Meta AudioCraft / MusicGen: https://github.com/facebookresearch/audiocraft
- MusicGen model card: https://huggingface.co/facebook/musicgen-large
- Hydra II (Rightsify): commercially-licensed MusicGen variant
- Web Audio API autoplay policy: https://developer.chrome.com/blog/autoplay
