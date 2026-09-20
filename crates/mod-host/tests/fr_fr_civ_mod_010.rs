//! Tests for FR-CIV-MOD-010 — Guest state save/load
//!
//! Epic: FR-CIV-MOD
//! Verifies ModGuestStateSave roundtrip serialization.

#[cfg(test)]
mod fr_fr_civ_mod_010 {
    /// FR-CIV-MOD-010: Empty GuestStateSave roundtrips through JSON.
    #[test]
    fn empty_save_roundtrips() {
        let save = civ_mod_host::ModGuestStateSave::empty();
        let json = save.to_json().expect("serialize");
        let restored = civ_mod_host::ModGuestStateSave::from_json(&json).expect("deserialize");
        assert_eq!(restored.version, save.version);
    }

    /// FR-CIV-MOD-010: MOD_GUEST_STATE_VERSION is positive.
    #[test]
    fn guest_state_version_positive() {
        assert!(civ_mod_host::MOD_GUEST_STATE_VERSION > 0);
    }
}
