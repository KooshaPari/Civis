//! WASM determinism scan before guest instantiation (CIV-0700 §3.5 / §14.5).

use thiserror::Error;
use wasmparser::{Operator, Parser, Payload};

/// Errors from the pre-instantiation determinism scan.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DeterminismError {
    /// WASM parse failure.
    #[error("wasm parse: {0}")]
    Parse(String),
    /// Instruction rejected for cross-platform determinism.
    #[error("non-deterministic instruction: {instruction}")]
    RejectedInstruction {
        /// Human-readable opcode label.
        instruction: String,
    },
}

/// Scan a WASM module for instructions that break replay determinism.
///
/// MVP rules (CIV-0700 §3.5): reject platform-sensitive float ops and atomics.
pub fn scan_wasm_determinism(wasm_bytes: &[u8]) -> Result<(), DeterminismError> {
    for payload in Parser::new(0).parse_all(wasm_bytes) {
        let payload = payload.map_err(|e| DeterminismError::Parse(e.to_string()))?;
        if let Payload::CodeSectionEntry(body) = payload {
            let mut reader = body
                .get_operators_reader()
                .map_err(|e| DeterminismError::Parse(e.to_string()))?;
            while !reader.eof() {
                let op = reader
                    .read()
                    .map_err(|e| DeterminismError::Parse(e.to_string()))?;
                if let Some(label) = reject_operator(op) {
                    return Err(DeterminismError::RejectedInstruction {
                        instruction: label.to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn reject_operator(op: Operator<'_>) -> Option<&'static str> {
    // wasmparser renames atomic opcodes across versions; stable Debug prefix is enough for MVP.
    let opcode = format!("{op:?}");
    if opcode.contains("Atomic") {
        return Some("atomic");
    }
    match op {
        Operator::F32Nearest | Operator::F64Nearest => Some("f32.nearest/f64.nearest"),
        Operator::F32Sqrt | Operator::F64Sqrt => Some("f32.sqrt/f64.sqrt"),
        Operator::I32Clz | Operator::I64Clz => Some("i32.clz/i64.clz"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_minimal_policy_module() {
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (param i64) (result i32)
                i32.const 0)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        scan_wasm_determinism(&wasm).expect("policy module should pass scan");
    }

    #[test]
    fn rejects_f32_sqrt() {
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (param i64) (result i32)
                f32.const 1.0
                f32.sqrt
                drop
                i32.const 0)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let err = scan_wasm_determinism(&wasm).expect_err("sqrt should fail");
        assert!(matches!(err, DeterminismError::RejectedInstruction { .. }));
    }
}
