//! Tests for FR-CIV-CORE-018
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-018: Binary Frame Format.
//! Support zstd-compressed binary frames in addition to JSON-RPC.

#[cfg(test)]
mod fr_fr_civ_core_018 {
    /// Zstd compression/decompression round-trips for serialized replay data.
    #[test]
    fn zstd_round_trip() {
        let log = civ_engine::replay::ReplayLog::default();
        let encoded = civ_engine::encode_civreplay(&log).expect("encode");
        // Compress with zstd
        let compressed = zstd::encode_all(&encoded[..], 3).expect("zstd compress");
        // Decompress
        let decompressed = zstd::decode_all(&compressed[..]).expect("zstd decompress");
        assert_eq!(encoded, decompressed.as_slice(), "zstd round-trip must be lossless");
    }

    /// Compressed data is smaller than raw for non-trivial payloads.
    #[test]
    fn zstd_compression_reduces_size() {
        let mut log = civ_engine::replay::ReplayLog::default();
        for t in 1..=100 {
            log.record_tick(t);
        }
        let encoded = civ_engine::encode_civreplay(&log).expect("encode");
        let compressed = zstd::encode_all(&encoded[..], 3).expect("zstd compress");
        assert!(
            compressed.len() < encoded.len(),
            "compressed ({}) should be smaller than raw ({})",
            compressed.len(),
            encoded.len()
        );
    }
}
