//! `.civreplay` binary container (FR-REPLAY-001 partial).
//!
//! Wraps the engine's RON-encoded [`ReplayLog`] in a versioned envelope:
//!
//! ```text
//! [8  bytes] magic (`CIVREPL\0`)
//! [4  bytes] format version (LE u32)
//! [4  bytes] RON payload length (LE u32)
//! [N  bytes] RON payload
//! [32 bytes] footer checksum (SHA-256 of header + payload)
//! ```

use crate::replay::{ReplayError, ReplayLog};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Eight-byte file magic.
pub const MAGIC: &[u8; 8] = b"CIVREPL\0";

/// Container format version (distinct from [`ReplayLog::schema_version`]).
pub const FORMAT_VERSION: u32 = 1;

/// Footer size for the SHA-256 checksum of header + payload.
pub const FOOTER_CHECKSUM_LEN: usize = 32;

const HEADER_LEN: usize = MAGIC.len() + 4 + 4;

/// Serialize `log` into a `.civreplay` file at `path`.
pub fn save_civreplay(path: impl AsRef<Path>, log: &ReplayLog) -> Result<(), ReplayError> {
    std::fs::write(path, encode_civreplay(log)?)?;
    Ok(())
}

/// Load a [`ReplayLog`] from a `.civreplay` file at `path`.
pub fn load_civreplay(path: impl AsRef<Path>) -> Result<ReplayLog, ReplayError> {
    let data = std::fs::read(path)?;
    decode_civreplay(&data)
}

/// Serialize `log` into an in-memory `.civreplay` byte buffer.
pub fn encode_civreplay(log: &ReplayLog) -> Result<Vec<u8>, ReplayError> {
    let payload = ron::to_string(log)?;
    let payload_len = u32::try_from(payload.len()).map_err(|_| ReplayError::PayloadTooLarge)?;

    let mut out = Vec::with_capacity(HEADER_LEN + payload.len() + FOOTER_CHECKSUM_LEN);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(payload.as_bytes());
    let checksum = sha256_header_and_payload(&out);
    out.extend_from_slice(&checksum);
    Ok(out)
}

fn sha256_header_and_payload(header_and_payload: &[u8]) -> [u8; FOOTER_CHECKSUM_LEN] {
    let digest = Sha256::digest(header_and_payload);
    digest.into()
}

/// Deserialize a [`ReplayLog`] from an in-memory `.civreplay` buffer.
pub fn decode_civreplay(data: &[u8]) -> Result<ReplayLog, ReplayError> {
    let min_len = HEADER_LEN + FOOTER_CHECKSUM_LEN;
    if data.len() < min_len {
        return Err(ReplayError::Truncated);
    }

    if data.get(..MAGIC.len()) != Some(MAGIC.as_slice()) {
        return Err(ReplayError::InvalidMagic);
    }

    let version = u32::from_le_bytes(data[8..12].try_into().expect("slice len"));
    if version != FORMAT_VERSION {
        return Err(ReplayError::UnsupportedFormatVersion(version));
    }

    let payload_len = u32::from_le_bytes(data[12..16].try_into().expect("slice len")) as usize;
    let payload_end = HEADER_LEN
        .checked_add(payload_len)
        .ok_or(ReplayError::Truncated)?;
    let expected_len = payload_end
        .checked_add(FOOTER_CHECKSUM_LEN)
        .ok_or(ReplayError::Truncated)?;
    if data.len() != expected_len {
        return Err(ReplayError::Truncated);
    }

    let stored_checksum = &data[payload_end..expected_len];
    let expected_checksum = sha256_header_and_payload(&data[..payload_end]);
    if stored_checksum != expected_checksum {
        return Err(ReplayError::ChecksumMismatch);
    }

    let payload = std::str::from_utf8(&data[HEADER_LEN..payload_end])?;
    let log: ReplayLog = ron::from_str(payload)?;
    log.verify_hash_chain()?;
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::ReplayLog;
    use civ_tactics::DamageEvent;
    use civ_voxel::{MaterialId, WorldCoord};
    use tempfile::NamedTempFile;

    fn sample_log() -> ReplayLog {
        let mut log = ReplayLog {
            seed: 42,
            ..ReplayLog::default()
        };
        log.record_tick(1);
        log.record_voxel_write(1, WorldCoord { x: 1, y: 2, z: 3 }, MaterialId(7));
        log.record_damage(
            2,
            DamageEvent {
                center: WorldCoord { x: 0, y: 0, z: 0 },
                radius_voxels: 2,
                energy: 11,
            },
        );
        log.record_research(3, vec![1, 2, 3], true);
        log
    }

    #[test]
    fn civreplay_roundtrip_preserves_replay_log() {
        let log = sample_log();
        let file = NamedTempFile::new().unwrap();
        save_civreplay(file.path(), &log).unwrap();
        let loaded = load_civreplay(file.path()).unwrap();
        assert_eq!(loaded, log);
    }

    #[test]
    fn civreplay_roundtrip_preserves_running_hash() {
        let log = sample_log();
        let root = log.running_hash.expect("sample log records a tick");
        let file = NamedTempFile::new().unwrap();
        save_civreplay(file.path(), &log).unwrap();
        let loaded = load_civreplay(file.path()).unwrap();
        assert_eq!(loaded.running_hash, Some(root));
        assert_eq!(loaded.hash_chain_root(), Some(root));
        assert_eq!(loaded.recompute_running_hash(), Some(root));
    }

    #[test]
    fn civreplay_rejects_hash_chain_mismatch() {
        let mut log = sample_log();
        log.running_hash = Some([0u8; crate::hash_chain::HASH_LEN]);
        let err = decode_civreplay(&encode_civreplay(&log).unwrap()).unwrap_err();
        assert!(matches!(err, ReplayError::HashChainMismatch));
    }

    #[test]
    fn civreplay_roundtrip_empty_log() {
        let log = ReplayLog::default();
        let file = NamedTempFile::new().unwrap();
        save_civreplay(file.path(), &log).unwrap();
        let loaded = load_civreplay(file.path()).unwrap();
        assert_eq!(loaded, log);
    }

    #[test]
    fn civreplay_rejects_tampered_payload() {
        let log = sample_log();
        let file = NamedTempFile::new().unwrap();
        save_civreplay(file.path(), &log).unwrap();

        let mut bytes = std::fs::read(file.path()).unwrap();
        bytes[HEADER_LEN] ^= 0x01;

        let err = decode_civreplay(&bytes).unwrap_err();
        assert!(matches!(err, ReplayError::ChecksumMismatch));
    }

    #[test]
    fn civreplay_rejects_tampered_footer() {
        let log = sample_log();
        let file = NamedTempFile::new().unwrap();
        save_civreplay(file.path(), &log).unwrap();

        let mut bytes = std::fs::read(file.path()).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;

        let err = decode_civreplay(&bytes).unwrap_err();
        assert!(matches!(err, ReplayError::ChecksumMismatch));
    }

    #[test]
    fn civreplay_rejects_all_zero_footer() {
        let log = sample_log();
        let mut bytes = encode_civreplay(&log).unwrap();
        let footer_start = bytes.len() - FOOTER_CHECKSUM_LEN;
        bytes[footer_start..].fill(0);

        let err = decode_civreplay(&bytes).unwrap_err();
        assert!(matches!(err, ReplayError::ChecksumMismatch));
    }
}
