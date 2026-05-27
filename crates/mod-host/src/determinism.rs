//! WASM determinism scan before guest instantiation (CIV-0700 §3.5 / §14.5).

use crate::float_data_flow::scan_float_action_emit_contamination;
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
    /// Float instructions present while `determinism-strict` feature is enabled.
    #[error("float contamination: {count} float instructions (strict mode)")]
    FloatContamination {
        /// Number of float opcodes observed.
        count: u32,
    },
    /// Float-derived value reaches `civlab::action_emit` (CIV-0700 §3.5 data-flow trace).
    #[error("float contamination: {count} action_emit sites with float-derived args")]
    ActionEmitFloatContamination {
        /// Number of contaminated `action_emit` call sites.
        count: u32,
    },
}

/// Summary from scanning a WASM module (FR-CIV-TACTICS-057 / FR-CIV-TACTICS-061).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeterminismScanReport {
    /// Count of `f32` / `f64` opcodes (internal use may be OK; strict mode rejects).
    pub float_instruction_count: u32,
    /// `action_emit` call sites where a float-derived value is passed (CIV-0700 §3.5).
    pub float_contamination_site_count: u32,
    /// Hard-rejected opcodes (atomics, sqrt, nearest, clz).
    pub hard_rejections: Vec<String>,
    /// Float-derived values reaching `civlab::action_emit`.
    pub float_contamination_sites: Vec<crate::FloatContaminationSite>,
}

/// Scan a WASM module and return opcode statistics.
pub fn scan_wasm_determinism_report(
    wasm_bytes: &[u8],
) -> Result<DeterminismScanReport, DeterminismError> {
    let mut report = DeterminismScanReport::default();
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
                if is_float_operator(&op) {
                    report.float_instruction_count += 1;
                }
                if let Some(label) = reject_operator(op) {
                    report.hard_rejections.push(label.to_string());
                }
            }
        }
    }
    let sites = scan_float_action_emit_contamination(wasm_bytes)
        .map_err(|e| DeterminismError::Parse(e))?;
    report.float_contamination_sites = sites;
    report.float_contamination_site_count =
        u32::try_from(report.float_contamination_sites.len()).unwrap_or(u32::MAX);
    Ok(report)
}

/// Scan a WASM module for instructions that break replay determinism.
///
/// MVP rules (CIV-0700 §3.5): reject platform-sensitive float ops and atomics.
/// With feature `determinism-strict`, any float opcode fails the scan.
pub fn scan_wasm_determinism(wasm_bytes: &[u8]) -> Result<(), DeterminismError> {
    let report = scan_wasm_determinism_report(wasm_bytes)?;
    if let Some(first) = report.hard_rejections.first() {
        return Err(DeterminismError::RejectedInstruction {
            instruction: first.clone(),
        });
    }
    if cfg!(feature = "determinism-strict") && report.float_instruction_count > 0 {
        return Err(DeterminismError::FloatContamination {
            count: report.float_instruction_count,
        });
    }
    if report.float_contamination_site_count > 0 {
        return Err(DeterminismError::ActionEmitFloatContamination {
            count: report.float_contamination_site_count,
        });
    }
    Ok(())
}

fn is_float_operator(op: &Operator<'_>) -> bool {
    let opcode = format!("{op:?}");
    opcode.contains("F32") || opcode.contains("F64")
}

fn reject_operator(op: Operator<'_>) -> Option<&'static str> {
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

    #[test]
    fn report_counts_float_ops_without_hard_reject() {
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (param i64) (result i32)
                f32.const 1.0
                drop
                i32.const 0)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let report = scan_wasm_determinism_report(&wasm).expect("scan");
        assert!(report.float_instruction_count >= 1);
        assert!(report.hard_rejections.is_empty());
        scan_wasm_determinism(&wasm).expect("non-sqrt float allowed in default mode");
    }

    #[test]
    fn rejects_reinterpret_f64_before_action_emit() {
        const WAT: &str = r#"
            (module
              (import "civlab" "action_emit" (func (param i64)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                f64.const 1.0
                i64.reinterpret_f64
                call 0
                i32.const 0)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let report = scan_wasm_determinism_report(&wasm).expect("scan");
        assert_eq!(report.float_contamination_site_count, 1);
        let err = scan_wasm_determinism(&wasm).expect_err("reinterpret before action_emit");
        assert!(matches!(
            err,
            DeterminismError::ActionEmitFloatContamination { count: 1 }
        ));
    }

    #[test]
    fn trunc_before_action_emit_passes_data_flow_scan() {
        const WAT: &str = r#"
            (module
              (import "civlab" "action_emit" (func (param i64)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                f64.const 1.0
                i64.trunc_f64_s
                call 0
                i32.const 0)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let report = scan_wasm_determinism_report(&wasm).expect("scan");
        assert_eq!(report.float_contamination_site_count, 0);
        scan_wasm_determinism(&wasm).expect("truncated float arg is clean");
    }
}
