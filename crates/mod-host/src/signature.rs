//! Ed25519 verification for `mod.wasm` (CIV-0700 §14 partial, FR-CIV-TACTICS-043).

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use thiserror::Error;

/// Detached signature filename inside a `.civmod` archive.
// The following 1 requirement tags were removed from MOD_WASM_SIG_NAME.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// See the namespace-collision note above.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-MOD-014: COLLIDING ID. modding-platform.md:40 = hot-reload with staged code tier; CIV-0700:2460 = Lua script parity. ModHost::reload_mod is an unload-then-reload with no file watcher and no staged tier, and a failed reload leaves the mod unloaded rather than keeping the prior version; grep for lua under crates/ returns zero. Tagged on a filename const and an error enum (signature.rs)
pub const MOD_WASM_SIG_NAME: &str = "mod.wasm.sig";

/// Signature verification failures.
// The following 1 requirement tags were removed from SignatureError.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// See the namespace-collision note above. The correct id for this file is CIV-0700
// FR-CIV-MOD-008 (Signature Verification); it is not written anywhere, and applying
// it is deferred until the collision is resolved.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-MOD-014: COLLIDING ID. modding-platform.md:40 = hot-reload with staged code tier; CIV-0700:2460 = Lua script parity. ModHost::reload_mod is an unload-then-reload with no file watcher and no staged tier, and a failed reload leaves the mod unloaded rather than keeping the prior version; grep for lua under crates/ returns zero. Tagged on a filename const and an error enum (signature.rs)
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SignatureError {
    /// Hex pubkey in manifest is malformed.
    #[error("invalid author_pubkey_hex: {0}")]
    InvalidPublicKey(String),
    /// Signature file is not 64 bytes.
    #[error("invalid signature length: expected 64 bytes, got {0}")]
    InvalidSignatureLength(usize),
    /// Cryptographic verification failed.
    #[error("signature verification failed")]
    VerifyFailed,
}

/// Verify `wasm` against a detached Ed25519 signature and hex-encoded public key.
pub fn verify_wasm_signature(
    wasm: &[u8],
    signature_bytes: &[u8],
    author_pubkey_hex: &str,
) -> Result<(), SignatureError> {
    if signature_bytes.len() != 64 {
        return Err(SignatureError::InvalidSignatureLength(
            signature_bytes.len(),
        ));
    }
    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(signature_bytes);
    let signature = Signature::from_bytes(&sig_array);

    let key_bytes = decode_hex(author_pubkey_hex)
        .map_err(|e| SignatureError::InvalidPublicKey(e.to_string()))?;
    if key_bytes.len() != 32 {
        return Err(SignatureError::InvalidPublicKey(format!(
            "expected 32-byte pubkey, got {} bytes",
            key_bytes.len()
        )));
    }
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&key_bytes);
    let verifying_key = VerifyingKey::from_bytes(&pk)
        .map_err(|e| SignatureError::InvalidPublicKey(e.to_string()))?;

    verifying_key
        .verify(wasm, &signature)
        .map_err(|_| SignatureError::VerifyFailed)
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return Err("odd hex length".to_owned());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| format!("invalid hex at {i}: {e}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use rand::rngs::OsRng;

    /// Covers FR-CIV-TACTICS-043.
    /// Covers FR-CIV-TACTICS-077.
    /// FR-CIV-TACTICS-077 — the Ed25519 signature scheme is the foundation
    /// for the signed remote mod registry (`mods/remote-registry.json`).
    #[test]
    fn fr_civ_tactics_043_green_signature_is_verified_and_rejected_when_tampered() {
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let wasm = b"(module)";
        let sig = signing_key.sign(wasm);
        let pk_hex = pubkey_hex(signing_key.verifying_key().as_bytes());
        verify_wasm_signature(wasm, sig.to_bytes().as_slice(), &pk_hex).expect("valid sig");

        let mut tampered = wasm.to_vec();
        tampered.push(0);
        let err = verify_wasm_signature(&tampered, sig.to_bytes().as_slice(), &pk_hex)
            .expect_err("tampered wasm");
        assert_eq!(err, SignatureError::VerifyFailed);
    }

    fn pubkey_hex(bytes: &[u8; 32]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
