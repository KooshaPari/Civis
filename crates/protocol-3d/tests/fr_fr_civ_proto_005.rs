//! Tests for FR-CIV-PROTO-005
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_005 {
    use civ_protocol_3d::{encode_frame3d_bundle, decode_frame3d_bundle, is_frame3d_bundle, Frame3dBundleEncodeOptions, FRAME3D_BUNDLE_MAGIC};

    #[test]
    fn verify_fr_civ_proto_005_basic() {
        let options = Frame3dBundleEncodeOptions { compress: false };
        let encoded = encode_frame3d_bundle(&[], &options).expect("encode");
        assert!(is_frame3d_bundle(&encoded));
        let decoded = decode_frame3d_bundle(&encoded).expect("decode");
        assert!(decoded.is_empty());
    }

    #[test]
    fn bundle_magic_is_f3db() {
        assert_eq!(FRAME3D_BUNDLE_MAGIC, b"F3DB");
    }
}
