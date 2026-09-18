//! FR-CIV-RENDER-001 — chunk streamer load/unload radius.
//!
//! Requirement text (docs/guides/voxel-emergent-vision-and-migration.md:151):
//!
//! > "Bevy chunk streamer loads and unloads chunks within a 3-chunk camera
//! > radius; no render-thread stalls (NFR-CIV-SCALE-002)."
//!
//! `WindowPolicy::classify` is the decision the renderer's per-frame plan makes
//! for every candidate chunk, so it is the headless-checkable half of this
//! requirement: it decides what is loaded, meshed, and unloaded for a given
//! camera anchor. The geometry pass and the "no render-thread stalls" timing
//! clause need a GPU and a frame loop, which are out of scope here.
//!
//! Replaces a placeholder (`crates/voxel/tests/fr_fr_civ_render_001.rs`) whose
//! entire body was `assert_eq!(c.x, 0); assert!(FIXED_SCALE > 0);` — a check on
//! a coordinate struct that says nothing about streaming.

use civ_voxel::window::{ring_distance, EvictionKey, PolicyError, WindowPolicy};
use civ_voxel::{ChunkCoord, ChunkState};

/// Chunk coord at an integer offset from the origin.
fn cc(cx: i32, cy: i32, cz: i32) -> ChunkCoord {
    ChunkCoord { cx, cy, cz }
}

/// The origin anchor.
fn origin() -> ChunkCoord {
    cc(0, 0, 0)
}

/// A policy with a `radius`-chunk fully-meshed window and no seam band, so the
/// resident/unloaded boundary sits exactly at `radius`.
///
/// `vy_weight = 1` makes `ring_distance` plain Chebyshev, which keeps the
/// expected ring numbers in these tests easy to reason about. The vertical
/// weighting rule itself is covered separately.
fn radius_policy(radius: u8) -> WindowPolicy {
    WindowPolicy {
        mesh_ring: radius,
        sim_ring: radius,
        coarse_ring: radius,
        seam_chunks: 0,
        vy_weight: 1,
        sim_lod_step: 1,
        prefetch_ring: 0,
        forward_cone_cos_theta: 0,
        fade_ticks: 0,
    }
}

/// Covers FR-CIV-RENDER-001.
///
/// The core load/unload contract: with a three-chunk radius, every chunk at
/// ring 0..=3 is loaded (resident with a live mesh) and everything at ring 4 or
/// beyond is unloaded.
#[test]
fn fr_civ_render_001_three_chunk_radius_loads_inside_and_unloads_outside() {
    let policy = radius_policy(3);
    let anchor = origin();

    // Inside the radius: loaded and meshed.
    for ring in 0..=3u32 {
        let coord = cc(ring as i32, 0, 0);
        assert_eq!(
            ring_distance(coord, anchor, policy.vy_weight),
            ring,
            "precondition: the coord is at the expected ring"
        );
        let state = policy.classify(coord, anchor);
        assert_eq!(
            state,
            ChunkState::Meshed,
            "ring {ring} is inside the 3-chunk radius and must be meshed"
        );
        assert!(state.is_resident(), "ring {ring} must be resident");
        assert!(state.has_mesh(), "ring {ring} must have a live mesh");
    }

    // Outside the radius: unloaded, on every axis and on the ring boundary.
    for coord in [
        cc(4, 0, 0),
        cc(-4, 0, 0),
        cc(0, 4, 0),
        cc(0, 0, 4),
        cc(4, 4, 4),
    ] {
        let state = policy.classify(coord, anchor);
        assert_eq!(
            state,
            ChunkState::Unloaded,
            "chunk {coord:?} is outside the 3-chunk radius and must be unloaded"
        );
        assert!(
            !state.is_resident(),
            "chunk {coord:?} must not occupy RAM outside the radius"
        );
    }
}

/// Covers FR-CIV-RENDER-001.
///
/// The streamer windows on the camera, so the same chunk must load and unload as
/// the anchor moves past it. That is the "unloads on move" half of the
/// requirement, and it must be reversible.
#[test]
fn fr_civ_render_001_window_follows_the_anchor() {
    let policy = radius_policy(2);
    let target = cc(10, 0, 0);

    // Anchor at the origin: the target is far outside the window.
    assert_eq!(
        policy.classify(target, origin()),
        ChunkState::Unloaded,
        "the target starts outside the window"
    );

    // Anchor moves onto the target: now loaded.
    assert_eq!(
        policy.classify(target, target),
        ChunkState::Meshed,
        "moving the camera onto the chunk must load it"
    );

    // Anchor moves one chunk past the edge of the window: unloaded again.
    let moved_away = cc(10 + 3, 0, 0);
    assert_eq!(
        policy.classify(target, moved_away),
        ChunkState::Unloaded,
        "moving the camera away must unload the chunk again"
    );
}

/// Covers FR-CIV-RENDER-001.
///
/// Residency is bounded: the number of loaded chunks for a given radius is a
/// fixed finite set, not something that grows as the camera wanders. This is the
/// property that keeps a streamer from leaking memory across a long session.
#[test]
fn fr_civ_render_001_resident_set_is_bounded_regardless_of_travel() {
    let policy = radius_policy(3);
    let radius = 3i32;

    // Count the lattice points inside the weighted-Chebyshev radius.
    let mut expected = 0usize;
    for cx in -radius..=radius {
        for cy in -radius..=radius {
            for cz in -radius..=radius {
                if ring_distance(cc(cx, cy, cz), origin(), policy.vy_weight) <= radius as u32 {
                    expected += 1;
                }
            }
        }
    }

    // Re-derive the count from a distant anchor; it must be identical, because
    // the window translates with the camera rather than accumulating.
    let far = cc(1_000, 0, -750);
    let mut actual = 0usize;
    for cx in -radius..=radius {
        for cy in -radius..=radius {
            for cz in -radius..=radius {
                let coord = cc(far.cx + cx, far.cy + cy, far.cz + cz);
                if policy.classify(coord, far).is_resident() {
                    actual += 1;
                }
            }
        }
    }

    assert_eq!(
        actual, expected,
        "the resident set must have the same size at any anchor"
    );
    assert!(
        expected > 0,
        "a 3-chunk radius must load at least one chunk"
    );
}

/// Covers FR-CIV-RENDER-001.
///
/// `classify` may be called several times per frame for the same chunk, so it
/// must be a pure function: no hidden state, same answer every call.
#[test]
fn fr_civ_render_001_classification_is_a_pure_function() {
    let policy = radius_policy(3);
    let anchor = cc(-7, 2, 5);

    for coord in [cc(0, 0, 0), cc(-7, 2, 5), cc(-4, 2, 5), cc(100, 0, 0)] {
        let first = policy.classify(coord, anchor);
        for _ in 0..8 {
            assert_eq!(
                policy.classify(coord, anchor),
                first,
                "classify({coord:?}) must not vary between calls"
            );
        }
    }
}

/// Covers FR-CIV-RENDER-001.
///
/// The vertical weight controls how much a step up or down counts toward the
/// ring, which decides whether a chunk one layer above the camera is inside the
/// radius. With `vy_weight = 2` a chunk 4 above is at ring 8, i.e. outside a
/// 3-chunk window.
#[test]
fn fr_civ_render_001_vertical_steps_are_weighted() {
    // This test is about the vertical metric, so the policy must actually use
    // the default `vy_weight = 2`; `radius_policy` pins it to 1 for the
    // Chebyshev-shaped assertions elsewhere.
    let policy = WindowPolicy {
        vy_weight: 2,
        ..radius_policy(3)
    };

    // dy of 1 costs 2 with vy_weight = 2.
    assert_eq!(
        ring_distance(cc(0, 1, 0), origin(), 2),
        2,
        "one vertical step must cost vy_weight rings"
    );
    assert_eq!(
        ring_distance(cc(0, 4, 0), origin(), 2),
        8,
        "four vertical steps must cost eight rings"
    );

    // With vy_weight = 2 a chunk one step up is at ring 2, inside a 3-chunk
    // window...
    assert_eq!(
        policy.classify(cc(0, 1, 0), origin()),
        ChunkState::Meshed,
        "one step up (ring 2) is inside a 3-chunk radius"
    );
    // ...but two steps up is at ring 4, outside it, even though it is only two
    // chunks away in world space. That is the point of the vertical weight:
    // the window does not fill the Y axis as the camera flies.
    assert_eq!(
        policy.classify(cc(0, 2, 0), origin()),
        ChunkState::Unloaded,
        "two steps up (ring 4) is outside a 3-chunk radius"
    );
}

/// Covers FR-CIV-RENDER-001.
///
/// The seam band is what stops a visible pop at the radius boundary: with
/// `seam_chunks = 1` the first ring past the mesh window is held `Resident`
/// rather than dropped straight to `Unloaded`.
#[test]
fn fr_civ_render_001_seam_band_holds_the_boundary_ring() {
    let mut policy = radius_policy(2);
    policy.seam_chunks = 1;

    // Inside the mesh window.
    assert_eq!(
        policy.classify(cc(2, 0, 0), origin()),
        ChunkState::Meshed
    );
    // The seam band (ring 3) is still resident, just not meshed.
    let seam = policy.classify(cc(3, 0, 0), origin());
    assert_eq!(
        seam,
        ChunkState::Resident,
        "the ring past the mesh window must be held resident by the seam band"
    );
    assert!(seam.is_resident() && !seam.has_mesh());
    // Past the seam: gone.
    assert_eq!(
        policy.classify(cc(4, 0, 0), origin()),
        ChunkState::Unloaded
    );

    // With a fade configured the same band reports a fade countdown instead.
    policy.fade_ticks = 4;
    assert_eq!(
        policy.classify(cc(3, 0, 0), origin()),
        ChunkState::Fading { ticks_remaining: 4 },
        "with fade_ticks set the seam band must ramp down rather than hold"
    );
}

/// Covers FR-CIV-RENDER-001.
///
/// Loading a ring is only safe if the policy itself is coherent. An incoherent
/// policy (sim window wider than the coarse window, or a zero divider) must be
/// rejected at construction rather than producing a degenerate stream.
#[test]
fn fr_civ_render_001_incoherent_policies_are_rejected() {
    // Sanity: a coherent policy is accepted.
    let ok = WindowPolicy::checked(3, 3, 3, 0, 1, 1, 0, 0, 0);
    assert!(ok.is_ok(), "a coherent 3-chunk policy must be accepted");

    // Zero vertical weight.
    assert!(matches!(
        WindowPolicy::checked(1, 1, 2, 1, 0, 2, 0, 0, 0),
        Err(PolicyError::ZeroVyWeight)
    ));

    // Zero sim cadence divisor.
    assert!(matches!(
        WindowPolicy::checked(1, 1, 2, 1, 2, 0, 0, 0, 0),
        Err(PolicyError::ZeroSimLodStep)
    ));

    // Sim window wider than the coarse window would leave a band with no cohort.
    assert!(matches!(
        WindowPolicy::checked(1, 3, 2, 1, 2, 2, 0, 0, 0),
        Err(PolicyError::SimRingAboveCoarseRing)
    ));
}

/// Covers FR-CIV-RENDER-001.
///
/// Unloading must be ordered, not arbitrary: when the budget is tight the
/// furthest chunks go first, so the streamer sheds the least useful work. This
/// is the deterministic-eviction half of "loads and unloads".
#[test]
fn fr_civ_render_001_eviction_prefers_the_furthest_chunk() {
    let anchor = origin();
    let vy = 1;

    let near = EvictionKey::new(cc(1, 0, 0), anchor, vy, 0);
    let far = EvictionKey::new(cc(9, 0, 0), anchor, vy, 0);

    // `EvictionKey` orders worst-first, so the far chunk must sort before the
    // near one.
    assert!(
        far < near,
        "the furthest chunk must be evicted before a near one"
    );

    // Two chunks at the same ring are separated by recency, so the choice stays
    // total and deterministic.
    let same_ring_a = EvictionKey::new(cc(5, 0, 0), anchor, vy, 0);
    let same_ring_b = EvictionKey::new(cc(0, 0, 5), anchor, vy, 7);
    assert!(
        same_ring_a != same_ring_b,
        "chunks at equal ring but different recency must be distinguishable"
    );
}
