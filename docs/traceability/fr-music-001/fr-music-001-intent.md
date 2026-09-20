# Intent: FR-MUSIC-001 -- Culture-derived, drifting music-cue surfaces

> Date: 2026-09-19
> FR: FR-MUSIC-001
> Epic: FR-MUSIC

## User Intent

The product owner requires Culture-derived, drifting music-cue surfaces as part of the FR-MUSIC epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Culture-derived, drifting music-cue surfaces is properly specified, implemented, and testable within the simulation engine.

`crates/engine/src/engine/engine_tests.rs::fr_music_distinct_culture_cues_evolve_over_time`
asserts that two `ClusterCulture` profiles with distinguishable
`faction_aggression` inputs surface distinct `MusicCue` parameter
sets in the simulation snapshot (`SimulationSnapshot::music_cues`),
and that the same culture's cue drifts across consecutive ticks. The
cues are derived entirely from emergent culture state, not from
hand-tuned music.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS.
FR-MUSIC-001 contributes to the audio pipeline by guaranteeing that
the per-cluster `MusicCue` is a *function of the culture profile* —
distinct cultures never share a cue, and a culture's cue is not
static (it evolves as the cluster mutates). This is the bedrock for
the audio engine plugin to schedule adaptive music.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/engine/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] Two cultures produce two distinct cues
- [x] Same culture drifts between consecutive ticks

### How We Know This FR Is Satisfied

1. `cargo test -p civ-engine fr_music_distinct_culture_cues_evolve_over_time` passes
2. `cue_a_left != cue_a_right` (inter-culture distinction)
3. `cue_a_left != cue_b_left` (intra-culture drift on tick N+1)
4. `cue_a_right != cue_b_right` (intra-culture drift on tick N+1)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-music-001-intent.md` |
| Source | `crates/engine/src/engine/engine_tests.rs:3732` |

<!-- Covers: FR-MUSIC-001 -->
