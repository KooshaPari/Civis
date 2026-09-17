//! Tests for FR-CIV-PROTO-015
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_015 {
    use civ_protocol_3d::Frame3dBundleEncodeOptions;

    #[test]
    fn verify_fr_civ_proto_015_basic() {
        let opts = Frame3dBundleEncodeOptions::default();
        assert!(!opts.compress, "default should be uncompressed");
    }
}
