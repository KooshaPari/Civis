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
    match op {
        Operator::F32Nearest | Operator::F64Nearest => Some("f32.nearest/f64.nearest"),
        Operator::F32Sqrt | Operator::F64Sqrt => Some("f32.sqrt/f64.sqrt"),
        Operator::I32Clz | Operator::I64Clz => Some("i32.clz/i64.clz"),
        Operator::MemoryAtomicNotify { .. }
        | Operator::MemoryAtomicWait32 { .. }
        | Operator::MemoryAtomicWait64 { .. } => Some("memory.atomic.wait/notify"),
        Operator::I32AtomicLoad { .. }
        | Operator::I64AtomicLoad { .. }
        | Operator::I32AtomicLoad16U { .. }
        | Operator::I32AtomicLoad8U { .. }
        | Operator::I64AtomicLoad8U { .. }
        | Operator::I32AtomicStore { .. }
        | Operator::I64AtomicStore { .. }
        | Operator::I32AtomicStore16 { .. }
        | Operator::I32AtomicStore8 { .. }
        | Operator::I64AtomicStore8 { .. }
        | Operator::I32AtomicRmwAdd { .. }
        | Operator::I64AtomicRmwAdd { .. }
        | Operator::I32AtomicRmw8AddU { .. }
        | Operator::I32AtomicRmw16AddU { .. }
        | Operator::I64AtomicRmw8AddU { .. }
        | Operator::I32AtomicRmwSub { .. }
        | Operator::I64AtomicRmwSub { .. }
        | Operator::I32AtomicRmw8SubU { .. }
        | Operator::I32AtomicRmw16SubU { .. }
        | Operator::I64AtomicRmw8SubU { .. }
        | Operator::I32AtomicRmwAnd { .. }
        | Operator::I64AtomicRmwAnd { .. }
        | Operator::I32AtomicRmw8AndU { .. }
        | Operator::I32AtomicRmw16AndU { .. }
        | Operator::I64AtomicRmw8AndU { .. }
        | Operator::I32AtomicRmwOr { .. }
        | Operator::I64AtomicRmwOr { .. }
        | Operator::I32AtomicRmw8OrU { .. }
        | Operator::I32AtomicRmw16OrU { .. }
        | Operator::I64AtomicRmw8OrU { .. }
        | Operator::I32AtomicRmwXor { .. }
        | Operator::I64AtomicRmwXor { .. }
        | Operator::I32AtomicRmw8XorU { .. }
        | Operator::I32AtomicRmw16XorU { .. }
        | Operator::I64AtomicRmw8XorU { .. }
        | Operator::I32AtomicRmwXchg { .. }
        | Operator::I64AtomicRmwXchg { .. }
        | Operator::I32AtomicRmw8XchgU { .. }
        | Operator::I32AtomicRmw16XchgU { .. }
        | Operator::I64AtomicRmw8XchgU { .. }
        | Operator::I32AtomicRmwCmpxchg { .. }
        | Operator::I64AtomicRmwCmpxchg { .. }
        | Operator::I32AtomicRmw8CmpxchgU { .. }
        | Operator::I32AtomicRmw16CmpxchgU { .. }
        | Operator::I64AtomicRmw8CmpxchgU { .. }
        | Operator::AtomicFence => Some("atomic.fence"),
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
        assert!(matches!(
            err,
            DeterminismError::RejectedInstruction { .. }
        ));
    }
}
