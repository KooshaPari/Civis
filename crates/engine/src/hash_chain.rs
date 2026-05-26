//! Per-tick hash chain stub (FR-CORE-005 / FR-CORE-006 partial).
//!
//! Each tick extends an append-only chain: `BLAKE3(prev || tick_event_bytes)`.

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
        let recomputed =
            chain_root_from_payloads([tick_payload.to_vec(), combat]).expect("root");
        assert_eq!(after_combat, recomputed);
        assert_ne!(after_tick, after_combat);
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
