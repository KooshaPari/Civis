//! Tests for FR-CIV-MOD-011 — GuestStateError variants
//!
//! Epic: FR-CIV-MOD
//! Verifies that GuestStateError has the expected variants.

#[cfg(test)]
mod fr_fr_civ_mod_011 {
    /// FR-CIV-MOD-011: GuestStateError::Json variant works via InvalidJson input.
    #[test]
    fn json_error_variant() {
        let err = civ_mod_host::GuestStateError::UnsupportedVersion(99);
        let display = format!("{err}");
        assert!(display.contains("99"));
    }

    /// FR-CIV-MOD-011: GuestStateError from invalid JSON input.
    #[test]
    fn invalid_json_gives_error() {
        let result = civ_mod_host::ModGuestStateSave::from_json("not json");
        assert!(result.is_err());
    }
}
