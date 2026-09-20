//! Tests for FR-CIV-MOD-014 — Signature verification
//!
//! Epic: FR-CIV-MOD
//! Verifies the WASM signature verification surface.

#[cfg(test)]
mod fr_fr_civ_mod_014 {
    /// FR-CIV-MOD-014: SignatureError variants exist.
    #[test]
    fn signature_error_invalid_pubkey() {
        use civ_mod_host::SignatureError;
        let err = SignatureError::InvalidPublicKey("bad hex".into());
        let display = format!("{err}");
        assert!(display.contains("bad hex"));
    }

    /// FR-CIV-MOD-014: SignatureError::InvalidSignatureLength variant works.
    #[test]
    fn signature_error_invalid_length() {
        use civ_mod_host::SignatureError;
        let err = SignatureError::InvalidSignatureLength(32);
        let display = format!("{err}");
        assert!(display.contains("32"));
    }

    /// FR-CIV-MOD-014: SignatureError::VerifyFailed variant works.
    #[test]
    fn signature_error_verify_failed() {
        use civ_mod_host::SignatureError;
        let err = SignatureError::VerifyFailed;
        let display = format!("{err}");
        assert!(!display.is_empty());
    }

    /// FR-CIV-MOD-014: MOD_WASM_SIG_NAME constant is defined.
    #[test]
    fn wasm_sig_name_defined() {
        assert!(!civ_mod_host::MOD_WASM_SIG_NAME.is_empty());
    }
}
