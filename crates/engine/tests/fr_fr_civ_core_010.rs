//! Tests for FR-CIV-CORE-010
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-010: Replay File Format.
//! Export runs to .civreplay format (header + event log + checksum).

#[cfg(test)]
mod fr_fr_civ_core_010 {
    /// encode_civreplay produces a buffer with correct magic, version, and checksum.
    #[test]
    fn replay_format_has_magic_version_checksum() {
        let log = civ_engine::replay::ReplayLog::default();
        let encoded = civ_engine::encode_civreplay(&log).expect("encode");
        // Magic: CIVREPL\0
        assert_eq!(&encoded[..8], b"CIVREPL\0");
        // Format version (LE u32 at offset 8)
        let version = u32::from_le_bytes(encoded[8..12].try_into().unwrap());
        assert_eq!(version, civ_engine::replay_format::FORMAT_VERSION);
        // Footer checksum is 32 bytes (SHA-256)
        // Minimum size = header (16) + checksum (32) = 48, plus RON payload
        assert!(
            encoded.len() >= 8 + 4 + 4 + civ_engine::replay_format::FOOTER_CHECKSUM_LEN,
            "encoded replay must be at least header + checksum"
        );
    }

    /// encode then decode round-trips for a non-empty log.
    #[test]
    fn replay_encode_decode_round_trip() {
        let mut log = civ_engine::replay::ReplayLog::default();
        log.record_tick(1);
        log.record_tick(2);
        let encoded = civ_engine::encode_civreplay(&log).expect("encode");
        let decoded = civ_engine::decode_civreplay(&encoded).expect("decode");
        assert_eq!(decoded.events.len(), log.events.len());
    }
}
