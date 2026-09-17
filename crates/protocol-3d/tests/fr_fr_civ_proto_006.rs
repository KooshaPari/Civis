//! Tests for FR-CIV-PROTO-006
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_006 {
    use civ_protocol_3d::{encode_frame3d_bundle, decode_frame3d_bundle, Frame3dBundleEncodeOptions, DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL};

    #[test]
    fn verify_fr_civ_proto_006_basic() {
        let options = Frame3dBundleEncodeOptions { compress: true, compress_level: DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL };
        let encoded = encode_frame3d_bundle(&[], &options).expect("encode compressed");
        let decoded = decode_frame3d_bundle(&encoded).expect("decode compressed");
        assert!(decoded.frames.is_empty());
    }
}
