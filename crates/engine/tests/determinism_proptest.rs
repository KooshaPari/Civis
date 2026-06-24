//! Property-based determinism checks for the simulation engine.

use civ_engine::{load_civreplay, save_civreplay, Simulation, FOOTER_CHECKSUM_LEN};
use proptest::prelude::*;
use tempfile::NamedTempFile;

fn run_ticks(seed: u64, n: usize) -> (u64, usize) {
    let mut sim = Simulation::with_seed(seed);
    for _ in 0..n {
        sim.tick();
    }
    (sim.state.tick, sim.replay_log().events.len())
}

fn footer_checksum(bytes: &[u8]) -> &[u8] {
    &bytes[bytes.len() - FOOTER_CHECKSUM_LEN..]
}

proptest! {
    /// Same seed + N ticks => identical final tick and replay event count.
    #[test]
    fn same_seed_and_tick_count_yields_identical_outcome(
        seed in any::<u64>(),
        n in 1usize..=50,
    ) {
        let a = run_ticks(seed, n);
        let b = run_ticks(seed, n);
        prop_assert_eq!(a, b);
        prop_assert_eq!(a.0, n as u64, "final tick should equal tick count");
    }

    /// save_civreplay → load_civreplay → save_civreplay preserves the file checksum footer.
    #[test]
    fn civreplay_roundtrip_preserves_checksum(
        seed in any::<u64>(),
        n in 1usize..=50,
    ) {
        let mut sim = Simulation::with_seed(seed);
        for _ in 0..n {
            sim.tick();
        }

        let first = NamedTempFile::new().expect("temp file");
        let second = NamedTempFile::new().expect("temp file");
        save_civreplay(first.path(), sim.replay_log()).expect("save");
        let loaded = load_civreplay(first.path()).expect("load");
        save_civreplay(second.path(), &loaded).expect("re-save");

        let bytes1 = std::fs::read(first.path()).expect("read");
        let bytes2 = std::fs::read(second.path()).expect("read");
        prop_assert_eq!(footer_checksum(&bytes1), footer_checksum(&bytes2));
        prop_assert_eq!(bytes1, bytes2);
        prop_assert_eq!(loaded, sim.replay_log().clone());
    }
}
