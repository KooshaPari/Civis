//! Tests for FR-CIV-PROTO-012
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED
//!
//! Covers the `F3DB` frame-bundle envelope: round-trip, tick coherence, and the
//! zstd-compression option.

#[cfg(test)]
mod fr_fr_civ_proto_012 {
    use civ_protocol_3d::{
        decode_frame3d_bundle, encode_frame3d_bundle, Frame3d, Frame3dBundleEncodeOptions,
        Frame3dBundleError, VoxelDeltaFrame, DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL,
    };

    fn voxel(tick: u64) -> Frame3d {
        Frame3d::VoxelDelta(VoxelDeltaFrame {
            tick,
            deltas: vec![],
        })
    }

    #[test]
    fn verify_fr_civ_proto_012_basic() {
        // The default options are the uncompressed wire layout.
        assert!(!Frame3dBundleEncodeOptions::default().compress);

        // An empty bundle encodes to a valid envelope and decodes back to zero
        // inner frames.
        let opts = Frame3dBundleEncodeOptions {
            compress: false,
            compress_level: DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL,
        };
        let encoded = encode_frame3d_bundle(&[], &opts).expect("empty bundle encodes");
        let decoded = decode_frame3d_bundle(&encoded).expect("empty bundle decodes");
        assert_eq!(decoded.frames.len(), 0);
        assert_eq!(decoded.tick, 0);

        // A populated bundle round-trips with its shared tick preserved.
        let frames = [voxel(12), voxel(12)];
        let encoded = encode_frame3d_bundle(&frames, &opts).expect("encode");
        let decoded = decode_frame3d_bundle(&encoded).expect("decode");
        assert_eq!(decoded.tick, 12);
        assert_eq!(decoded.frames.len(), 2);
        assert_eq!(decoded.frames, frames);

        // Inner frames that disagree on tick are rejected at encode time.
        let mixed = [voxel(1), voxel(2)];
        assert!(
            matches!(
                encode_frame3d_bundle(&mixed, &opts),
                Err(Frame3dBundleError::TickMismatch { .. })
            ),
            "a bundle must carry one shared tick"
        );

        // The zstd option round-trips the same frames.
        let zstd = Frame3dBundleEncodeOptions {
            compress: true,
            compress_level: DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL,
        };
        let bytes = encode_frame3d_bundle(&frames, &zstd).expect("encode zstd");
        let back = decode_frame3d_bundle(&bytes).expect("decode zstd");
        assert_eq!(back.frames, frames);
        assert_eq!(back.tick, decoded.tick, "compression must not alter the tick");
    }
}
