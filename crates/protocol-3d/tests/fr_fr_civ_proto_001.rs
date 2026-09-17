//! Tests for FR-CIV-PROTO-001
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_001 {
    use civ_protocol_3d::{SCHEMA_VERSION, FRAME3D_BINARY_MAGIC};

    #[test]
    fn verify_fr_civ_proto_001_basic() {
        assert!(SCHEMA_VERSION > 0, "schema version must be set");
    }

    #[test]
    fn binary_magic_is_f3d0() {
        assert_eq!(FRAME3D_BINARY_MAGIC, b"F3D0");
    }
}
