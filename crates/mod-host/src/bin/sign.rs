//! Sign `mod.wasm` with Ed25519 for CivLab publish workflow (CIV-0700 §14 partial).
//!
//! Usage:
//!   civ-mod-sign <wasm-path> [sig-out]
//!   civ-mod-sign <wasm-path> [sig-out] --key <64-hex-seed>
//!
//! When `--key` is omitted, generates a new keypair and prints `author_pubkey_hex` for `manifest.toml`.

use std::{env, fs, path::PathBuf, process};

use ed25519_dalek::{Signer, SigningKey};
use rand::{rngs::OsRng, RngCore};

fn usage() -> ! {
    eprintln!(
        "Usage: civ-mod-sign <wasm-path> [sig-out] [--key <64-hex-seed>]\n\
         Default sig-out: <wasm-path>.sig"
    );
    process::exit(2);
}

fn decode_hex_seed(hex: &str) -> Result<[u8; 32], String> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(format!("expected 64 hex chars (32 bytes), got {}", hex.len()));
    }
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let pair = std::str::from_utf8(chunk).map_err(|e| e.to_string())?;
        out[i] = u8::from_str_radix(pair, 16).map_err(|e| format!("invalid hex at byte {i}: {e}"))?;
    }
    Ok(out)
}

fn pubkey_hex(pk: &[u8; 32]) -> String {
    pk.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }
    let wasm_path = PathBuf::from(&args[1]);
    let mut sig_path = wasm_path.with_extension("wasm.sig");
    let mut key_hex: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--key" {
            i += 1;
            key_hex = Some(args.get(i).cloned().unwrap_or_else(|| {
                eprintln!("--key requires a 64-character hex seed");
                process::exit(2);
            }));
            i += 1;
            continue;
        }
        if args[i].starts_with("--") {
            eprintln!("unknown flag: {}", args[i]);
            usage();
        }
        sig_path = PathBuf::from(&args[i]);
        i += 1;
    }

    let wasm = fs::read(&wasm_path).unwrap_or_else(|err| {
        eprintln!("read {}: {err}", wasm_path.display());
        process::exit(1);
    });

    let signing_key = match key_hex.as_deref() {
        Some(hex) => {
            let seed = decode_hex_seed(hex).unwrap_or_else(|err| {
                eprintln!("{err}");
                process::exit(1);
            });
            SigningKey::from_bytes(&seed)
        }
        None => {
            let mut seed = [0u8; 32];
            OsRng.fill_bytes(&mut seed);
            SigningKey::from_bytes(&seed)
        }
    };

    let signature = signing_key.sign(&wasm);
    fs::write(&sig_path, signature.to_bytes()).unwrap_or_else(|err| {
        eprintln!("write {}: {err}", sig_path.display());
        process::exit(1);
    });

    let pk_hex = pubkey_hex(signing_key.verifying_key().as_bytes());
    println!("Wrote {}", sig_path.display());
    println!("author_pubkey_hex = \"{pk_hex}\"");
    if key_hex.is_none() {
        eprintln!(
            "Save the 64-hex seed from a secure keygen if you need to re-sign; this run used an ephemeral key."
        );
    }
}
