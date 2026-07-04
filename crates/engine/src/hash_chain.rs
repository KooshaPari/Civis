//! Per-tick hash chain stub (FR-CORE-005 / FR-CORE-006 partial).
//!
//! Each tick extends an append-only chain: `BLAKE3(prev || tick_event_bytes)`.
//!
//! FR-CIV-PLANET-060 extends the chain to fold in Climate + WeatherGrid +
//! GeologyMap via [`climate_event_bytes`].

/// Length of a chain link (BLAKE3 digest).
pub const HASH_LEN: usize = 32;

/// Genesis link before any ticks are recorded.
pub const GENESIS: [u8; HASH_LEN] = [0u8; HASH_LEN];

/// Canonical tick-event payload for hashing (stub: little-endian tick counter).
#[must_use]
pub fn tick_event_bytes(tick: u64) -> [u8; 8] {
    tick.to_le_bytes()
}

/// Lowercase hex encoding of a chain link (64 characters for BLAKE3).
#[must_use]
pub fn hash_hex(bytes: &[u8; HASH_LEN]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Compute the next chain link from the prior hash and tick event bytes.
#[must_use]
pub fn tick_hash(prev: &[u8; HASH_LEN], tick_event_bytes: &[u8]) -> [u8; HASH_LEN] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prev);
    hasher.update(tick_event_bytes);
    *hasher.finalize().as_bytes()
}

/// Running hash-chain state for a simulation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HashChainState {
    pub running_hash: [u8; HASH_LEN],
}

impl HashChainState {
    /// Create a chain rooted at [`GENESIS`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            running_hash: GENESIS,
        }
    }

    /// Advance the chain with the given tick-event bytes and return the new root.
    pub fn advance(&mut self, tick_event_bytes: &[u8]) -> [u8; HASH_LEN] {
        self.running_hash = tick_hash(&self.running_hash, tick_event_bytes);
        self.running_hash
    }
}

/// Recompute the chain root from an ordered tick sequence (empty → `None`).
#[must_use]
pub fn chain_root_from_ticks(ticks: impl IntoIterator<Item = u64>) -> Option<[u8; HASH_LEN]> {
    let mut prev = GENESIS;
    let mut count = 0u64;
    for tick in ticks {
        prev = tick_hash(&prev, &tick_event_bytes(tick));
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(prev)
    }
}

/// Advance the chain with an arbitrary canonical payload.
#[must_use]
pub fn chain_advance(prev: &[u8; HASH_LEN], payload: &[u8]) -> [u8; HASH_LEN] {
    tick_hash(prev, payload)
}

/// Recompute the chain root from ordered tick + combat payloads (FR-CIV-TACTICS-041).
#[must_use]
pub fn chain_root_from_payloads(
    payloads: impl IntoIterator<Item = impl AsRef<[u8]>>,
) -> Option<[u8; HASH_LEN]> {
    let mut prev = GENESIS;
    let mut count = 0u64;
    for payload in payloads {
        prev = tick_hash(&prev, payload.as_ref());
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(prev)
    }
}

/// Canonical combat-event payload for the replay hash chain (FR-CIV-TACTICS-041).
///
/// The 9-field layout (all little-endian) encodes the full engagement context:
/// `"combat"(6) | tick(8) | shooter_id(8) | target_id(8) | cx(8) | cy(8) | cz(8) |
///  radius(1) | energy(4) | strength_damage(4)` = 69 bytes.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn combat_event_bytes(
    tick: u64,
    shooter_id: u64,
    target_id: u64,
    center_x: i64,
    center_y: i64,
    center_z: i64,
    radius_voxels: u8,
    energy: u32,
    strength_damage: u32,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(69);
    out.extend_from_slice(b"combat");
    out.extend_from_slice(&tick.to_le_bytes());
    out.extend_from_slice(&shooter_id.to_le_bytes());
    out.extend_from_slice(&target_id.to_le_bytes());
    out.extend_from_slice(&center_x.to_le_bytes());
    out.extend_from_slice(&center_y.to_le_bytes());
    out.extend_from_slice(&center_z.to_le_bytes());
    out.push(radius_voxels);
    out.extend_from_slice(&energy.to_le_bytes());
    out.extend_from_slice(&strength_damage.to_le_bytes());
    out
}

/// Canonical climate-event payload for the replay hash chain (FR-CIV-PLANET-060).
///
/// Layout (all little-endian):
/// `"climate"(7) | tick(8) | day_phase_bits(4) | year_phase_bits(4) |
///  moon_phase_bits(4) | tide_offset_bits(4) |
///  [region_id(4) | temp_c_fp(4) | precip_mm_fp(4)]* |
///  [region_id(4) | biome(1)]*`
///
/// The `f32` fields from [`civ_planet::Climate`] are encoded as their raw IEEE-754
/// bit patterns (little-endian `u32`) to guarantee bit-identical serialisation on
/// every platform.
#[must_use]
pub fn climate_event_bytes(
    tick: u64,
    climate: &civ_planet::Climate,
    weather_grid: &[civ_planet::WeatherCell],
    geology_map: &civ_planet::GeologyMap,
) -> Vec<u8> {
    // 7 (tag) + 8 (tick) + 4×4 (climate f32 fields) + weather * 12 + geology * 5
    let capacity = 7 + 8 + 16 + weather_grid.len() * 12 + geology_map.regions.len() * 5;
    let mut out = Vec::with_capacity(capacity);
    out.extend_from_slice(b"climate");
    out.extend_from_slice(&tick.to_le_bytes());
    // Climate f32 fields as raw IEEE-754 bit patterns for determinism
    out.extend_from_slice(&climate.day_phase.to_bits().to_le_bytes());
    out.extend_from_slice(&climate.year_phase.to_bits().to_le_bytes());
    out.extend_from_slice(&climate.moon_phase.to_bits().to_le_bytes());
    out.extend_from_slice(&climate.tide_offset.to_bits().to_le_bytes());
    // Per-region weather cells
    for cell in weather_grid {
        out.extend_from_slice(&cell.region_id.to_le_bytes());
        out.extend_from_slice(&cell.temp_c_fp.to_le_bytes());
        out.extend_from_slice(&cell.precip_mm_fp.to_le_bytes());
    }
    // Geology map regions
    for region in &geology_map.regions {
        out.extend_from_slice(&region.region_id.to_le_bytes());
        out.push(region.biome as u8);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CORE-006 partial — identical inputs yield identical chain links.
    #[test]
    fn chain_is_deterministic() {
        let event = tick_event_bytes(7);
        let first = tick_hash(&GENESIS, &event);
        assert_eq!(first, tick_hash(&GENESIS, &event));

        let second = tick_hash(&first, &tick_event_bytes(8));
        assert_eq!(second, tick_hash(&first, &tick_event_bytes(8)));

        let mut state = HashChainState::new();
        assert_eq!(state.advance(&event), first);
        assert_eq!(state.advance(&tick_event_bytes(8)), second);
    }

    #[test]
    fn chain_root_from_ticks_matches_incremental() {
        let ticks = [1u64, 2, 3];
        let mut state = HashChainState::new();
        for tick in ticks {
            state.advance(&tick_event_bytes(tick));
        }
        assert_eq!(chain_root_from_ticks(ticks), Some(state.running_hash));
        assert_eq!(chain_root_from_ticks([]), None);
    }

    #[test]
    fn hash_hex_is_lowercase_64_chars() {
        let bytes = [0xab_u8, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89];
        let mut hash = [0u8; HASH_LEN];
        hash[..8].copy_from_slice(&bytes);
        assert_eq!(
            hash_hex(&hash),
            "abcdef0123456789000000000000000000000000000000000000000000000000"
        );
    }

    /// FR-CORE-005 partial — tampering with tick bytes changes the link.
    #[test]
    fn tamper_changes_hash() {
        let mut event = tick_event_bytes(42);
        let intact = tick_hash(&GENESIS, &event);

        event[0] ^= 0x01;
        let tampered = tick_hash(&GENESIS, &event);

        assert_ne!(intact, tampered);
    }

    #[test]
    fn combat_payload_extends_chain() {
        let tick_payload = tick_event_bytes(1);
        let after_tick = chain_advance(&GENESIS, &tick_payload);
        let combat = combat_event_bytes(1, 10, 20, 0, 0, 0, 2, 100, 0);
        let after_combat = chain_advance(&after_tick, &combat);
        let recomputed = chain_root_from_payloads([tick_payload.to_vec(), combat]).expect("root");
        assert_eq!(after_combat, recomputed);
        assert_ne!(after_tick, after_combat);
    }

    /// FR-CIV-PLANET-060 — chain digest changes on any ClimateFrame field delta.
    #[test]
    fn replay_hash_chain_differs_when_climate_changes() {
        use civ_planet::{Climate, GeologyMap, PlanetConfig, WeatherCell};
        use civ_planet::weather::{SeasonKind, WeatherKind};

        let base_climate = Climate {
            tick: 1,
            day_phase: 0.25,
            year_phase: 0.5,
            moon_phase: 0.1,
            tide_offset: 0.3,
        };
        let weather = vec![WeatherCell {
            region_id: 0,
            latitude_fp: 0,
            season: SeasonKind::Summer,
            kind: WeatherKind::Clear,
            temp_c_fp: 20_000,
            precip_mm_fp: 1_000,
            storm_intensity_fp: 0,
        }];
        let planet_cfg = PlanetConfig {
            radius_km: 6_371,
            axial_tilt_deg: 23,
            day_length_ticks: 24_000,
            year_length_ticks: 8_766_000,
        };
        let geology = GeologyMap::seed(&planet_cfg);

        // Baseline payload
        let base_payload = climate_event_bytes(1, &base_climate, &weather, &geology);
        let base_hash = chain_advance(&GENESIS, &base_payload);

        // Mutate day_phase — hash must differ
        let mut changed_climate = base_climate;
        changed_climate.day_phase = 0.75;
        let changed_hash = chain_advance(
            &GENESIS,
            &climate_event_bytes(1, &changed_climate, &weather, &geology),
        );
        assert_ne!(base_hash, changed_hash, "day_phase delta must change chain");

        // Mutate tide_offset — hash must differ
        let mut changed_tide = base_climate;
        changed_tide.tide_offset = -0.3;
        let changed_hash = chain_advance(
            &GENESIS,
            &climate_event_bytes(1, &changed_tide, &weather, &geology),
        );
        assert_ne!(
            base_hash, changed_hash,
            "tide_offset delta must change chain"
        );

        // Mutate weather cell temperature — hash must differ
        let weather_changed = vec![WeatherCell {
            region_id: 0,
            latitude_fp: 0,
            season: SeasonKind::Summer,
            kind: WeatherKind::Clear,
            temp_c_fp: 30_000,
            precip_mm_fp: 1_000,
            storm_intensity_fp: 0,
        }];
        let changed_hash = chain_advance(
            &GENESIS,
            &climate_event_bytes(1, &base_climate, &weather_changed, &geology),
        );
        assert_ne!(
            base_hash, changed_hash,
            "WeatherCell temp delta must change chain"
        );

        // Mutate geology map (different planet config changes biomes)
        let mut tweaked_planet = planet_cfg;
        tweaked_planet.radius_km = planet_cfg.radius_km + 2_000;
        let geology_changed = GeologyMap::seed(&tweaked_planet);
        let changed_hash = chain_advance(
            &GENESIS,
            &climate_event_bytes(1, &base_climate, &weather, &geology_changed),
        );
        assert_ne!(
            base_hash, changed_hash,
            "GeologyMap delta must change chain"
        );

        // Identical inputs must produce the identical hash (determinism)
        let replay_hash = chain_advance(
            &GENESIS,
            &climate_event_bytes(1, &base_climate, &weather, &geology),
        );
        assert_eq!(
            base_hash, replay_hash,
            "identical inputs must be deterministic"
        );
    }

    #[test]
    fn tick_hash_uses_blake3_not_sha256() {
        let event = tick_event_bytes(7);
        let hash = tick_hash(&GENESIS, &event);
        let sha256_first = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(GENESIS);
            hasher.update(event);
            let digest = hasher.finalize();
            let mut out = [0u8; HASH_LEN];
            out.copy_from_slice(&digest);
            out
        };
        assert_ne!(hash, sha256_first);
    }
}
