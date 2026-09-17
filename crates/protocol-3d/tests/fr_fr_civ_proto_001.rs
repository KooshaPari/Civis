//! Tests for FR-CIV-PROTO-001
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_001 {
    use civ_protocol_3d::{SCHEMA_VERSION, FRAME3D_BINARY_MAGIC};

    #[test]
    fn verify_fr_civ_proto_001_basic() {
        // 0 is the correct pre-1.0 baseline; the version is deliberately zero-based
        // and bumped only on wire-incompatible changes.
        assert_eq!(SCHEMA_VERSION, 0);
        assert_eq!(FRAME3D_BINARY_MAGIC, b"F3D0");
    }
}
