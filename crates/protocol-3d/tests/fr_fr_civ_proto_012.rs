//! Tests for FR-CIV-PROTO-012
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_012 {
    use civ_protocol_3d::{encode_frame3d_bundle, decode_frame3d_bundle, Frame3dBundleEncodeOptions};

    #[test]
    fn verify_fr_civ_proto_012_basic() {
        let opts = Frame3dBundleEncodeOptions { compress: false };
        let encoded = encode_frame3d_bundle(&[], &opts).unwrap();
        let decoded = decode_frame3d_bundle(&encoded).unwrap();
        assert_eq!(decoded.len(), 0);
    }
}
