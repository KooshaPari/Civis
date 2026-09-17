//! Tests for FR-CIV-PROTO-014
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_014 {
    use civ_protocol_3d::Frame3dBundleFlags;

    #[test]
    fn verify_fr_civ_proto_014_basic() {
        let uncompressed = Frame3dBundleFlags::uncompressed();
        assert!(!uncompressed.is_zstd());
        let zstd = Frame3dBundleFlags::zstd();
        assert!(zstd.is_zstd());
    }
}
