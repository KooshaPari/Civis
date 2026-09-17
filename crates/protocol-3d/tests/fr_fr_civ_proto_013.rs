//! Tests for FR-CIV-PROTO-013
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_013 {
    use civ_protocol_3d::{encode_frame3d_bundle, decode_frame3d_bundle, is_frame3d_bundle, Frame3dBundleEncodeOptions, DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL};

    #[test]
    fn verify_fr_civ_proto_013_basic() {
        let opts = Frame3dBundleEncodeOptions { compress: true, compress_level: DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL };
        let compressed = encode_frame3d_bundle(&[], &opts).unwrap();
        let opts2 = Frame3dBundleEncodeOptions { compress: false, compress_level: DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL };
        let uncompressed = encode_frame3d_bundle(&[], &opts2).unwrap();
        assert!(is_frame3d_bundle(&compressed));
        assert!(is_frame3d_bundle(&uncompressed));
    }
}
